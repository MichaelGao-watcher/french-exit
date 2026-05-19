use std::sync::Mutex;
use std::time::Instant;
use crate::error::BackendError;
use crate::types::ResourceConfig;
use windows::Win32::System::JobObjects::*;
use windows::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};
use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE};

/// 资源使用率快照
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_mb: u64,
}

/// CPU 时间采样点（用于计算 CPU%）
#[derive(Debug, Clone, Copy)]
struct CpuSample {
    process_time: u64,
    timestamp: Instant,
}

/// 资源控制器，通过 Windows Job Object 限制当前进程的 CPU 使用率。
#[derive(Clone, Copy)]
struct JobHandle(HANDLE);
unsafe impl Send for JobHandle {}
unsafe impl Sync for JobHandle {}

pub struct ResourceController {
    job_handle: Mutex<Option<JobHandle>>,
    config: Mutex<ResourceConfig>,
    last_cpu_sample: Mutex<Option<CpuSample>>,
}

impl ResourceController {
    /// 创建新的 ResourceController，使用默认配置（RULE-05）。
    pub fn new() -> Self {
        Self {
            job_handle: Mutex::new(None),
            config: Mutex::new(Self::default_config()),
            last_cpu_sample: Mutex::new(None),
        }
    }

    /// 返回默认资源配置：cpu_limit_percent = 30，unlimited = false。
    pub fn default_config() -> ResourceConfig {
        ResourceConfig {
            cpu_limit_percent: 30,
            unlimited: false,
        }
    }

    /// 应用资源限制。
    /// - 若 `config.unlimited = true`，直接返回 Ok，不创建 Job Object。
    /// - 否则创建 Windows Job Object，设置 CPU 速率限制，并将当前进程加入该 Job Object。
    pub fn apply_limits(&self, config: ResourceConfig) -> Result<(), BackendError> {
        // 关闭已有的 Job Object，避免重复限制。
        self.remove_limits()?;

        let mut cfg_guard = self.config.lock().map_err(|e| {
            BackendError::ResourceError(format!("Config mutex poisoned: {}", e))
        })?;
        *cfg_guard = config.clone();
        drop(cfg_guard);

        if config.unlimited {
            return Ok(());
        }

        unsafe {
            let job = CreateJobObjectW(None, None)
                .map_err(|e| BackendError::ResourceError(format!("CreateJobObjectW failed: {}", e)))?;

            let mut cpu_info = JOBOBJECT_CPU_RATE_CONTROL_INFORMATION::default();
            cpu_info.ControlFlags = JOB_OBJECT_CPU_RATE_CONTROL_ENABLE | JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP;
            cpu_info.Anonymous.CpuRate = (config.cpu_limit_percent as u32) * 100;

            SetInformationJobObject(
                job,
                JobObjectCpuRateControlInformation,
                &cpu_info as *const _ as *const std::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_CPU_RATE_CONTROL_INFORMATION>() as u32,
            ).map_err(|e| BackendError::ResourceError(format!("SetInformationJobObject failed: {}", e)))?;

            let current_process = GetCurrentProcess();
            AssignProcessToJobObject(job, current_process)
                .map_err(|e| BackendError::ResourceError(format!("AssignProcessToJobObject failed: {}", e)))?;

            let mut handle_guard = self.job_handle.lock().map_err(|e| {
                BackendError::ResourceError(format!("Job handle mutex poisoned: {}", e))
            })?;
            *handle_guard = Some(JobHandle(job));
        }

        Ok(())
    }

    /// 解除资源限制：关闭 Job Object Handle。
    /// 当最后一个 Handle 被关闭后，Job Object 的限制即失效。
    pub fn remove_limits(&self) -> Result<(), BackendError> {
        let mut handle_guard = self.job_handle.lock().map_err(|e| {
            BackendError::ResourceError(format!("Job handle mutex poisoned: {}", e))
        })?;

        if let Some(JobHandle(handle)) = handle_guard.take() {
            unsafe {
                CloseHandle(handle)
                    .map_err(|e| BackendError::ResourceError(format!("CloseHandle failed: {}", e)))?;
            }
        }

        Ok(())
    }

    /// 获取当前进程的资源使用率。
    ///
    /// CPU 百分比基于 `GetProcessTimes` 的前后两次采样计算，首次调用返回 0.0。
    pub fn current_usage(&self) -> Result<ResourceUsage, BackendError> {
        let memory_mb = unsafe {
            let process = GetCurrentProcess();
            let mut mem_counters = windows::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS::default();
            let result = windows::Win32::System::ProcessStatus::GetProcessMemoryInfo(
                process,
                &mut mem_counters,
                std::mem::size_of::<windows::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS>() as u32,
            );

            match result {
                Ok(()) => (mem_counters.WorkingSetSize as u64) / (1024 * 1024),
                Err(_) => 0,
            }
        };

        let cpu_percent = unsafe {
            let process = GetCurrentProcess();
            let mut creation = FILETIME::default();
            let mut exit = FILETIME::default();
            let mut kernel = FILETIME::default();
            let mut user = FILETIME::default();

            match GetProcessTimes(process, &mut creation, &mut exit, &mut kernel, &mut user) {
                Ok(()) => {
                    let process_time = Self::filetime_to_u64(kernel) + Self::filetime_to_u64(user);
                    let now = Instant::now();

                    let mut sample_guard = self.last_cpu_sample.lock().map_err(|e| {
                        BackendError::ResourceError(format!("CPU sample mutex poisoned: {}", e))
                    })?;

                    let cpu = if let Some(last) = *sample_guard {
                        let proc_delta = process_time.saturating_sub(last.process_time);
                        let elapsed_ns = now.duration_since(last.timestamp).as_nanos() as u64;
                        let elapsed_100ns = elapsed_ns / 100;

                        if elapsed_100ns > 0 {
                            let num_cpus = std::thread::available_parallelism()
                                .map(|n| n.get() as f64)
                                .unwrap_or(1.0);
                            let raw = (proc_delta as f64 / elapsed_100ns as f64) * 100.0 / num_cpus;
                            raw.clamp(0.0, 100.0) as f32
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };

                    *sample_guard = Some(CpuSample {
                        process_time,
                        timestamp: now,
                    });

                    cpu
                }
                Err(_) => 0.0,
            }
        };

        Ok(ResourceUsage {
            cpu_percent,
            memory_mb,
        })
    }

    /// 将 Windows FILETIME 转换为 u64（100 纳秒单位）
    fn filetime_to_u64(ft: FILETIME) -> u64 {
        ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64)
    }

    /// 返回当前生效的配置副本。
    pub fn get_config(&self) -> ResourceConfig {
        self.config.lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| Self::default_config())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RC-08: 验证默认值符合 RULE-05（unlimited = false，cpu_limit_percent = 30）。
    #[test]
    fn rc_08_default_config_enforces_rule_05() {
        let default = ResourceController::default_config();
        assert!(!default.unlimited, "RULE-05: default unlimited must be false");
        assert_eq!(default.cpu_limit_percent, 30, "RULE-05: default cpu_limit_percent must be 30");

        let controller = ResourceController::new();
        let config = controller.get_config();
        assert!(!config.unlimited);
        assert_eq!(config.cpu_limit_percent, 30);
    }

    /// RC-06: 验证 `apply_limits` 在 unlimited = true 时跳过限制，不创建 Job Object。
    #[test]
    fn rc_06_apply_limits_unlimited_skips_job_object() {
        let controller = ResourceController::new();
        let config = ResourceConfig {
            cpu_limit_percent: 30,
            unlimited: true,
        };
        controller.apply_limits(config).expect("apply_limits should succeed for unlimited=true");

        let handle = controller.job_handle.lock().unwrap();
        assert!(handle.is_none(), "unlimited=true should not create a Job Object");
    }

    /// RC-07: 验证 `remove_limits` 后 Job Object Handle 被关闭（设为 None）。
    #[test]
    fn rc_07_remove_limits_clears_handle() {
        let controller = ResourceController::new();
        let config = ResourceConfig {
            cpu_limit_percent: 30,
            unlimited: false,
        };
        controller.apply_limits(config).expect("apply_limits should succeed");

        {
            let handle = controller.job_handle.lock().unwrap();
            assert!(handle.is_some(), "Job Object handle should exist after apply_limits");
        }

        controller.remove_limits().expect("remove_limits should succeed");

        let handle = controller.job_handle.lock().unwrap();
        assert!(handle.is_none(), "Job Object handle should be None after remove_limits");
    }

    /// 额外测试：验证 unlimited = false 时能成功创建 Job Object（API 参数正确）。
    #[test]
    fn apply_limits_creates_job_object_when_limited() {
        let controller = ResourceController::new();
        let config = ResourceConfig {
            cpu_limit_percent: 50,
            unlimited: false,
        };
        controller.apply_limits(config).expect("apply_limits should create Job Object");

        let handle = controller.job_handle.lock().unwrap();
        assert!(handle.is_some(), "Job Object handle should be created when unlimited=false");

        // 清理
        drop(handle);
        controller.remove_limits().unwrap();
    }
}
