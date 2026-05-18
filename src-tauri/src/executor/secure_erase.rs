use std::fs::{self, OpenOptions};
use std::io::{self, Seek, Write};
use std::path::Path;

use rand::rngs::OsRng;
use rand::RngCore;

use crate::error::BackendError;

const BUFFER_SIZE: usize = 64 * 1024; // 64KB

/// 安全擦除接口
pub trait SecureEraser: Send + Sync {
    fn erase_file(&self, path: &Path) -> Result<(), BackendError>;
    fn erase_directory(&self, path: &Path) -> Result<(), BackendError>;
}

/// DoD 5220.22-M 标准安全擦除实现
pub struct DoDEraser {
    pub passes: u8,
}

impl Default for DoDEraser {
    fn default() -> Self {
        Self { passes: 3 }
    }
}

impl DoDEraser {
    pub fn new(passes: u8) -> Self {
        Self { passes }
    }

    /// 仅执行覆写（不含重命名和删除），供测试验证覆写效果
    fn overwrite_only(&self, path: &Path) -> Result<(), BackendError> {
        if !path.exists() {
            return Err(BackendError::EraseError("文件不存在".to_string()));
        }

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(map_io_error)?;

        let size = file
            .metadata()
            .map_err(map_io_error)?
            .len();

        let mut buffer = vec![0u8; BUFFER_SIZE];

        for pass in 0..self.passes {
            file.seek(io::SeekFrom::Start(0))
                .map_err(map_io_error)?;

            // 填充缓冲区
            match pass {
                0 => buffer.fill(0x00),
                1 => buffer.fill(0xFF),
                _ => OsRng.fill_bytes(&mut buffer),
            }

            let mut written: u64 = 0;
            while written < size {
                let to_write = std::cmp::min(BUFFER_SIZE as u64, size - written) as usize;
                file.write_all(&buffer[..to_write])
                    .map_err(map_io_error)?;
                written += to_write as u64;
            }

            file.sync_all().map_err(map_io_error)?;
        }

        Ok(())
    }
}

impl SecureEraser for DoDEraser {
    fn erase_file(&self, path: &Path) -> Result<(), BackendError> {
        self.overwrite_only(path)?;

        // 生成 16 字符随机十六进制文件名（8 字节 → 16 个 hex 字符）
        let mut random_bytes = [0u8; 8];
        OsRng.fill_bytes(&mut random_bytes);
        let hex_name: String = random_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        let parent = path.parent().unwrap_or(Path::new("."));
        let new_path = parent.join(&hex_name);

        fs::rename(path, &new_path).map_err(map_io_error)?;
        fs::remove_file(&new_path).map_err(map_io_error)?;

        Ok(())
    }

    fn erase_directory(&self, path: &Path) -> Result<(), BackendError> {
        if !path.exists() {
            return Err(BackendError::EraseError("目录不存在".to_string()));
        }

        let mut errors: Vec<String> = Vec::new();

        // 先收集所有条目，避免在遍历过程中修改目录结构导致的问题
        let mut files: Vec<std::path::PathBuf> = Vec::new();
        let mut dirs: Vec<std::path::PathBuf> = Vec::new();

        fn collect_entries(
            dir: &Path,
            files: &mut Vec<std::path::PathBuf>,
            dirs: &mut Vec<std::path::PathBuf>,
        ) -> io::Result<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() || path.is_symlink() {
                    files.push(path);
                } else if path.is_dir() {
                    dirs.push(path.clone());
                    collect_entries(&path, files, dirs)?;
                }
            }
            Ok(())
        }

        if let Err(e) = collect_entries(path, &mut files, &mut dirs) {
            errors.push(format!("遍历目录失败: {}", e));
        }

        // 先擦除文件
        for file_path in &files {
            if let Err(e) = self.erase_file(file_path) {
                errors.push(format!("擦除文件 '{}' 失败: {}", file_path.display(), e));
            }
        }

        // 按深度从大到小排序目录，确保先删除子目录
        dirs.sort_by(|a, b| b.components().count().cmp(&a.components().count()));

        // 删除空目录（包括根目录）
        for dir_path in &dirs {
            if let Err(e) = fs::remove_dir(dir_path) {
                errors.push(format!("删除目录 '{}' 失败: {}", dir_path.display(), e));
            }
        }

        // 最后删除根目录
        if let Err(e) = fs::remove_dir(path) {
            errors.push(format!("删除根目录 '{}' 失败: {}", path.display(), e));
        }

        if !errors.is_empty() {
            return Err(BackendError::EraseError(errors.join("; ")));
        }

        Ok(())
    }
}

fn map_io_error(e: io::Error) -> BackendError {
    match e.kind() {
        io::ErrorKind::NotFound => BackendError::EraseError("文件不存在".to_string()),
        io::ErrorKind::PermissionDenied => BackendError::EraseError("权限不足".to_string()),
        _ => BackendError::IoError(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use tempfile::TempDir;

    /// SE-06: 创建 1MB 测试文件，覆写后读取验证内容非原始数据
    #[test]
    fn test_overwrite_changes_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_1mb.bin");

        // 写入 1MB 可识别原始数据
        let original_data = vec![0xABu8; 1024 * 1024];
        fs::write(&file_path, &original_data).unwrap();

        let eraser = DoDEraser::default();
        eraser.overwrite_only(&file_path).unwrap();

        // 读取覆写后的内容
        let mut overwritten = Vec::new();
        File::open(&file_path)
            .unwrap()
            .read_to_end(&mut overwritten)
            .unwrap();

        // 验证内容已改变（不再是原始数据）
        assert_ne!(
            overwritten, original_data,
            "覆写后文件内容应当与原始数据不同"
        );

        // 额外验证：前几个字节不应全为 0xAB
        assert!(
            overwritten.iter().take(1024).any(|&b| b != 0xAB),
            "覆写后文件前 1KB 不应仍全为原始字节 0xAB"
        );
    }

    /// SE-07: 验证 erase_file 后文件不存在
    #[test]
    fn test_erase_file_removes_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("to_erase.txt");

        fs::write(&file_path, "secret data here").unwrap();
        assert!(file_path.exists());

        let eraser = DoDEraser::default();
        eraser.erase_file(&file_path).unwrap();

        assert!(!file_path.exists(), "擦除后原路径不应存在");
    }

    /// SE-08: 验证嵌套目录（3 层深，每层 2~3 个文件）被完全擦除
    #[test]
    fn test_erase_nested_directory() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("nested");

        // 构建 3 层嵌套目录，每层 2~3 个文件
        // 第 1 层
        fs::create_dir(&root).unwrap();
        fs::write(root.join("l1_a.txt"), "level1 file a").unwrap();
        fs::write(root.join("l1_b.txt"), "level1 file b").unwrap();

        // 第 2 层
        let l2 = root.join("level2");
        fs::create_dir(&l2).unwrap();
        fs::write(l2.join("l2_a.txt"), "level2 file a").unwrap();
        fs::write(l2.join("l2_b.txt"), "level2 file b").unwrap();
        fs::write(l2.join("l2_c.txt"), "level2 file c").unwrap();

        // 第 3 层
        let l3 = l2.join("level3");
        fs::create_dir(&l3).unwrap();
        fs::write(l3.join("l3_a.txt"), "level3 file a").unwrap();
        fs::write(l3.join("l3_b.txt"), "level3 file b").unwrap();

        assert!(root.exists());

        let eraser = DoDEraser::default();
        eraser.erase_directory(&root).unwrap();

        assert!(!root.exists(), "擦除后根目录不应存在");
        assert!(!l2.exists(), "擦除后第 2 层目录不应存在");
        assert!(!l3.exists(), "擦除后第 3 层目录不应存在");
    }

    /// SE-09: 验证 100MB 大文件覆写不 OOM（流式 64KB 缓冲工作正常）
    #[test]
    fn test_large_file_erase_no_oom() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large_100mb.bin");

        // 创建 100MB 文件
        let size_100mb: u64 = 100 * 1024 * 1024;
        {
            let mut file = File::create(&file_path).unwrap();
            let chunk = vec![0xCDu8; BUFFER_SIZE];
            let mut written: u64 = 0;
            while written < size_100mb {
                let to_write = std::cmp::min(BUFFER_SIZE as u64, size_100mb - written) as usize;
                file.write_all(&chunk[..to_write]).unwrap();
                written += to_write as u64;
            }
        }

        assert_eq!(file_path.metadata().unwrap().len(), size_100mb);

        let eraser = DoDEraser::default();
        // 由于 erase_file 会重命名+删除，我们直接测试 overwrite_only 来验证大文件覆写成功
        // 但为了完整验证 erase_file 也能处理大文件，直接调用 erase_file
        eraser.erase_file(&file_path).unwrap();

        assert!(!file_path.exists(), "100MB 文件擦除后不应存在");
    }

    #[test]
    fn test_erase_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("no_such_file.txt");

        let eraser = DoDEraser::default();
        let result = eraser.erase_file(&nonexistent);

        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("文件不存在"), "应当返回文件不存在错误: {}", err_msg);
    }

    #[test]
    fn test_erase_nonexistent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("no_such_dir");

        let eraser = DoDEraser::default();
        let result = eraser.erase_directory(&nonexistent);

        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("目录不存在"), "应当返回目录不存在错误: {}", err_msg);
    }
}
