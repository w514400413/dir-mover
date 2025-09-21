use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use serde::{Serialize, Deserialize};
use std::time::SystemTime;
use log::{info, error};

/// 文件操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationResult {
    pub success: bool,
    pub message: String,
    pub source_path: String,
    pub target_path: Option<String>,
}

/// 文件操作错误
#[derive(Debug)]
pub enum FileOperationError {
    IoError(io::Error),
    PermissionDenied(String),
    PathNotFound(String),
    PathAlreadyExists(String),
    InvalidPath(String),
    OperationCancelled(String),
}

impl From<io::Error> for FileOperationError {
    fn from(error: io::Error) -> Self {
        FileOperationError::IoError(error)
    }
}

impl std::fmt::Display for FileOperationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOperationError::IoError(e) => write!(f, "IO错误: {}", e),
            FileOperationError::PermissionDenied(path) => write!(f, "权限被拒绝: {}", path),
            FileOperationError::PathNotFound(path) => write!(f, "路径不存在: {}", path),
            FileOperationError::PathAlreadyExists(path) => write!(f, "路径已存在: {}", path),
            FileOperationError::InvalidPath(path) => write!(f, "无效路径: {}", path),
            FileOperationError::OperationCancelled(msg) => write!(f, "操作已取消: {}", msg),
        }
    }
}

/// 文件操作器
pub struct FileOperator {
    cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl FileOperator {
    /// 创建新的文件操作器
    pub fn new() -> Self {
        Self {
            cancel_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// 取消当前操作
    pub fn cancel_operation(&self) {
        self.cancel_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// 检查是否取消
    fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 验证路径
    fn validate_path(&self, path: &Path) -> Result<(), FileOperationError> {
        if path.to_string_lossy().is_empty() {
            return Err(FileOperationError::InvalidPath("路径不能为空".to_string()));
        }

        // 检查路径是否包含非法字符
        let path_str = path.to_string_lossy();
        if path_str.contains("..") && path_str.contains("...") {
            return Err(FileOperationError::InvalidPath("路径包含非法字符".to_string()));
        }

        Ok(())
    }

    /// 复制文件或目录
    pub fn copy_path(&self, source: &Path, target: &Path) -> Result<FileOperationResult, FileOperationError> {
        self.validate_path(source)?;
        self.validate_path(target)?;

        if !source.exists() {
            return Err(FileOperationError::PathNotFound(source.display().to_string()));
        }

        if target.exists() {
            return Err(FileOperationError::PathAlreadyExists(target.display().to_string()));
        }

        if self.is_cancelled() {
            return Err(FileOperationError::OperationCancelled("复制操作已取消".to_string()));
        }

        let start_time = SystemTime::now();
        
        let result = if source.is_dir() {
            self.copy_directory(source, target)
        } else {
            self.copy_file(source, target)
        };

        match &result {
            Ok(_) => {
                let duration = start_time.elapsed().unwrap_or_default();
                info!("复制完成: {} -> {} (耗时: {:?})", source.display(), target.display(), duration);
            }
            Err(e) => {
                error!("复制失败: {} -> {} (错误: {})", source.display(), target.display(), e);
            }
        }

        result
    }

    /// 复制文件
    fn copy_file(&self, source: &Path, target: &Path) -> Result<FileOperationResult, FileOperationError> {
        // 确保目标目录存在
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        // 复制文件
        fs::copy(source, target)?;

        Ok(FileOperationResult {
            success: true,
            message: "文件复制成功".to_string(),
            source_path: source.display().to_string(),
            target_path: Some(target.display().to_string()),
        })
    }

    /// 复制目录
    fn copy_directory(&self, source: &Path, target: &Path) -> Result<FileOperationResult, FileOperationError> {
        // 创建目标目录
        fs::create_dir_all(target)?;

        // 读取源目录
        let entries = fs::read_dir(source)
            .map_err(|e| FileOperationError::IoError(e))?;

        let mut copied_files = 0;
        let mut copied_dirs = 0;

        for entry in entries {
            if self.is_cancelled() {
                return Err(FileOperationError::OperationCancelled("复制操作已取消".to_string()));
            }

            let entry = entry.map_err(|e| FileOperationError::IoError(e))?;
            let entry_path = entry.path();
            let entry_name = entry.file_name();
            let target_entry_path = target.join(&entry_name);

            if entry_path.is_dir() {
                // 递归复制子目录
                self.copy_directory(&entry_path, &target_entry_path)?;
                copied_dirs += 1;
            } else {
                // 复制文件
                self.copy_file(&entry_path, &target_entry_path)?;
                copied_files += 1;
            }
        }

        Ok(FileOperationResult {
            success: true,
            message: format!("目录复制成功 (文件: {}, 目录: {})", copied_files, copied_dirs),
            source_path: source.display().to_string(),
            target_path: Some(target.display().to_string()),
        })
    }

    /// 删除文件或目录
    pub fn delete_path(&self, path: &Path) -> Result<FileOperationResult, FileOperationError> {
        self.validate_path(path)?;

        if !path.exists() {
            return Err(FileOperationError::PathNotFound(path.display().to_string()));
        }

        if self.is_cancelled() {
            return Err(FileOperationError::OperationCancelled("删除操作已取消".to_string()));
        }

        let start_time = SystemTime::now();

        let result = if path.is_dir() {
            self.delete_directory(path)
        } else {
            self.delete_file(path)
        };

        match &result {
            Ok(_) => {
                let duration = start_time.elapsed().unwrap_or_default();
                info!("删除完成: {} (耗时: {:?})", path.display(), duration);
            }
            Err(e) => {
                error!("删除失败: {} (错误: {})", path.display(), e);
            }
        }

        result
    }

    /// 删除文件
    fn delete_file(&self, path: &Path) -> Result<FileOperationResult, FileOperationError> {
        fs::remove_file(path)?;
        
        Ok(FileOperationResult {
            success: true,
            message: "文件删除成功".to_string(),
            source_path: path.display().to_string(),
            target_path: None,
        })
    }

    /// 删除目录
    fn delete_directory(&self, path: &Path) -> Result<FileOperationResult, FileOperationError> {
        let mut deleted_files = 0;
        let mut deleted_dirs = 0;

        // 递归删除目录内容
        self.delete_directory_contents(path, &mut deleted_files, &mut deleted_dirs)?;

        // 删除目录本身
        fs::remove_dir(path)?;

        deleted_dirs += 1;

        Ok(FileOperationResult {
            success: true,
            message: format!("目录删除成功 (文件: {}, 目录: {})", deleted_files, deleted_dirs),
            source_path: path.display().to_string(),
            target_path: None,
        })
    }

    /// 递归删除目录内容
    fn delete_directory_contents(
        &self,
        path: &Path,
        deleted_files: &mut u64,
        deleted_dirs: &mut u64
    ) -> Result<(), FileOperationError> {
        let entries = fs::read_dir(path)
            .map_err(|e| FileOperationError::IoError(e))?;

        for entry in entries {
            if self.is_cancelled() {
                return Err(FileOperationError::OperationCancelled("删除操作已取消".to_string()));
            }

            let entry = entry.map_err(|e| FileOperationError::IoError(e))?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                // 递归删除子目录
                self.delete_directory_contents(&entry_path, deleted_files, deleted_dirs)?;
                fs::remove_dir(&entry_path)?;
                *deleted_dirs += 1;
            } else {
                fs::remove_file(&entry_path)?;
                *deleted_files += 1;
            }
        }

        Ok(())
    }

    /// 创建符号链接（Windows）
    #[cfg(target_os = "windows")]
    pub fn create_symlink(&self, target: &Path, link_path: &Path) -> Result<FileOperationResult, FileOperationError> {
        self.validate_path(target)?;
        self.validate_path(link_path)?;

        if !target.exists() {
            return Err(FileOperationError::PathNotFound(target.display().to_string()));
        }

        if link_path.exists() {
            return Err(FileOperationError::PathAlreadyExists(link_path.display().to_string()));
        }

        if self.is_cancelled() {
            return Err(FileOperationError::OperationCancelled("创建链接操作已取消".to_string()));
        }

        // 确保链接路径的父目录存在
        if let Some(parent) = link_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 使用 Windows API 创建符号链接
        use std::os::windows::fs::symlink_dir;
        
        let result = if target.is_dir() {
            symlink_dir(target, link_path)
        } else {
            std::os::windows::fs::symlink_file(target, link_path)
        };

        result.map_err(|e| FileOperationError::IoError(e))?;

        Ok(FileOperationResult {
            success: true,
            message: "符号链接创建成功".to_string(),
            source_path: target.display().to_string(),
            target_path: Some(link_path.display().to_string()),
        })
    }

    /// 创建符号链接（Unix/Linux/macOS）
    #[cfg(not(target_os = "windows"))]
    pub fn create_symlink(&self, target: &Path, link_path: &Path) -> Result<FileOperationResult, FileOperationError> {
        self.validate_path(target)?;
        self.validate_path(link_path)?;

        if !target.exists() {
            return Err(FileOperationError::PathNotFound(target.display().to_string()));
        }

        if link_path.exists() {
            return Err(FileOperationError::PathAlreadyExists(link_path.display().to_string()));
        }

        if self.is_cancelled() {
            return Err(FileOperationError::OperationCancelled("创建链接操作已取消".to_string()));
        }

        // 确保链接路径的父目录存在
        if let Some(parent) = link_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 使用标准库创建符号链接
        std::os::unix::fs::symlink(target, link_path)?;

        Ok(FileOperationResult {
            success: true,
            message: "符号链接创建成功".to_string(),
            source_path: target.display().to_string(),
            target_path: Some(link_path.display().to_string()),
        })
    }

    /// 验证路径是否可以进行迁移
    pub fn validate_migration_path(&self, source: &Path, target: &Path) -> Result<(bool, String), FileOperationError> {
        self.validate_path(source)?;
        self.validate_path(target)?;

        // 检查源路径是否存在
        if !source.exists() {
            return Ok((false, "源路径不存在".to_string()));
        }

        // 检查目标路径是否已存在
        if target.exists() {
            return Ok((false, "目标路径已存在".to_string()));
        }

        // 检查源路径是否是目标路径的子路径（避免循环复制）
        if let Ok(source_canonical) = fs::canonicalize(source) {
            if let Ok(target_canonical) = fs::canonicalize(target.parent().unwrap_or(target)) {
                if source_canonical.starts_with(&target_canonical) {
                    return Ok((false, "不能将目录迁移到其子目录中".to_string()));
                }
            }
        }

        // 检查磁盘空间（粗略估计）
        if let Ok(source_size) = self.estimate_directory_size(source) {
            if let Some(target_parent) = target.parent() {
                if let Ok(target_parent_info) = fs::metadata(target_parent) {
                    // 这里应该检查可用空间，但fs::metadata不提供这些信息
                    // 在实际应用中，可能需要使用系统特定的API
                    if source_size > 1024 * 1024 * 1024 * 100 { // 100GB
                        return Ok((false, "源目录过大，可能需要更多磁盘空间".to_string()));
                    }
                }
            }
        }

        Ok((true, "路径验证通过".to_string()))
    }
    /// 估计目录大小
    fn estimate_directory_size(&self, path: &Path) -> Result<u64, FileOperationError> {
        if !path.exists() {
            return Err(FileOperationError::PathNotFound(path.display().to_string()));
        }

        if path.is_file() {
            return Ok(fs::metadata(path)?.len());
        }

        let mut total_size = 0u64;
        let entries = fs::read_dir(path)?;

        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                total_size += self.estimate_directory_size(&entry_path)?;
            } else {
                total_size += entry.metadata()?.len();
            }
        }

        Ok(total_size)
    }
}

impl Default for FileOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_copy_file() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("source.txt");
        let target_file = temp_dir.path().join("target.txt");

        // 创建源文件
        let mut file = File::create(&source_file).unwrap();
        writeln!(file, "测试内容").unwrap();

        let operator = FileOperator::new();
        let result = operator.copy_file(&source_file, &target_file).unwrap();

        assert!(result.success);
        assert!(target_file.exists());
        assert_eq!(result.source_path, source_file.display().to_string());
        assert_eq!(result.target_path, Some(target_file.display().to_string()));
    }

    #[tokio::test]
    async fn test_copy_directory() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let target_dir = temp_dir.path().join("target");

        // 创建源目录结构
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(source_dir.join("subdir")).unwrap();
        
        let mut file1 = File::create(source_dir.join("file1.txt")).unwrap();
        writeln!(file1, "内容1").unwrap();
        
        let mut file2 = File::create(source_dir.join("subdir").join("file2.txt")).unwrap();
        writeln!(file2, "内容2").unwrap();

        let operator = FileOperator::new();
        let result = operator.copy_directory(&source_dir, &target_dir).unwrap();

        assert!(result.success);
        assert!(target_dir.exists());
        assert!(target_dir.join("file1.txt").exists());
        assert!(target_dir.join("subdir").exists());
        assert!(target_dir.join("subdir").join("file2.txt").exists());
    }

    #[tokio::test]
    async fn test_validate_migration_path() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source");
        let target = temp_dir.path().join("target");

        fs::create_dir_all(&source).unwrap();

        let operator = FileOperator::new();
        let (valid, message) = operator.validate_migration_path(&source, &target).unwrap();

        assert!(valid);
        assert_eq!(message, "路径验证通过");
    }
}