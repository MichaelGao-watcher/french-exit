use std::sync::Mutex;
use crate::error::BackendError;
use crate::types::ResourceConfig;
use windows::Win32::System::JobObjects::*;
use windows::Win32::System::Threading::GetCurrentProcess;
use windows::Win32::Foundation::{CloseHandle, HANDLE};

/// 资源使用率快照
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_mb: u64,
}

/// 资源控制器，通过 Windows Job Object 限制当前进程的 CPU 使用率。
pub struct ResourceController {
    job_handle: Mutex<Option<HANDLE>>,
    config: Mutex<ResourceConfig>,
}

impl ResourceController {
    /// 创建新的 ResourceController，使用默认配置（RULE-05）。
    pub fn new() -> Self {
        Self {
            job_handle: Mutex::new(None),
            config: Mutex::new(Self::default_config()),
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
            *handle_guard = Some(job);
        }

        Ok(())
    }

    /// 解除资源限制：关闭 Job Object Handle。
    /// 当最后一个 Handle 被关闭后，Job Object 的限制即失效。
    pub fn remove_limits(&self) -> Result<(), BackendError> {
        let mut handle_guard = self.job_handle.lock().map_err(|e| {
            BackendError::ResourceError(format!("Job handle mutex poisoned: {}", e))
        })?;

        if let Some(handle) = handle_guard.take() {
            unsafe {
                CloseHandle(handle)
                    .map_err(|e| BackendError::ResourceError(format!("CloseHandle failed: {}", e)))?;
            }
        }

        Ok(())
    }

    /// 获取当前进程的资源使用率。
    /// 
    /// TODO: CPU 百分比计算需要前后两次采样或结合系统时间才能得出精确值，
    /// 当前版本返回 0.0 作为占位值，后续可基于 `GetProcessTimes` 完善。
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

        Ok(ResourceUsage {
            cpu_percent: 0.0,
            memory_mb,
        })
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
