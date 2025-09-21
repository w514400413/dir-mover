use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};
use log::{info, error, warn};
use crate::file_operations::{FileOperator, FileOperationResult, FileOperationError};
use crate::disk_analyzer::DirectoryInfo;

/// 迁移选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationOptions {
    pub source_path: String,
    pub target_path: String,
    pub create_symlink: bool,
    pub delete_source: bool,
}

/// 迁移结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    pub success: bool,
    pub message: String,
    pub source_path: String,
    pub target_path: String,
    pub symlink_path: Option<String>,
}

/// 迁移服务
pub struct MigrationService {
    file_operator: FileOperator,
}

impl MigrationService {
    /// 创建新的迁移服务
    pub fn new() -> Self {
        Self {
            file_operator: FileOperator::new(),
        }
    }

    /// 获取文件操作器
    pub fn file_operator(&self) -> &FileOperator {
        &self.file_operator
    }

    /// 执行文件夹迁移
    pub async fn migrate_folder(&self, options: MigrationOptions) -> Result<MigrationResult, String> {
        let source = Path::new(&options.source_path);
        let target = Path::new(&options.target_path);

        info!("开始迁移: {} -> {}", source.display(), target.display());

        // 1. 预检查
        match self.pre_migration_check(source, target).await {
            Ok(_) => {},
            Err(e) => {
                return Ok(MigrationResult {
                    success: false,
                    message: format!("预检查失败: {}", e),
                    source_path: options.source_path,
                    target_path: options.target_path,
                    symlink_path: None,
                });
            }
        }

        // 2. 复制文件夹
        let copy_result = match self.file_operator.copy_path(source, target) {
            Ok(result) => result,
            Err(e) => {
                return Ok(MigrationResult {
                    success: false,
                    message: format!("复制失败: {}", e),
                    source_path: options.source_path,
                    target_path: options.target_path,
                    symlink_path: None,
                });
            }
        };

        if !copy_result.success {
            return Ok(MigrationResult {
                success: false,
                message: format!("复制失败: {}", copy_result.message),
                source_path: options.source_path,
                target_path: options.target_path,
                symlink_path: None,
            });
        }

        info!("复制完成: {}", copy_result.message);

        // 3. 验证复制结果
        match self.verify_copy_result(source, target).await {
            Ok(_) => {
                info!("复制验证通过");
            },
            Err(e) => {
                // 复制验证失败，尝试清理目标目录
                let _ = self.file_operator.delete_path(target);
                
                return Ok(MigrationResult {
                    success: false,
                    message: format!("复制验证失败: {}", e),
                    source_path: options.source_path,
                    target_path: options.target_path,
                    symlink_path: None,
                });
            }
        }

        let mut symlink_path = None;

        // 4. 创建符号链接（如果启用）
        if options.create_symlink {
            match self.create_symlink_after_migration(source, target).await {
                Ok(symlink_result) => {
                    info!("符号链接创建成功: {}", symlink_result.message);
                    symlink_path = Some(source.display().to_string());
                },
                Err(e) => {
                    error!("符号链接创建失败: {}", e);
                    // 符号链接创建失败，但不影响整体迁移结果
                }
            }
        }

        // 5. 删除源目录（如果启用）
        if options.delete_source {
            match self.file_operator.delete_path(source) {
                Ok(delete_result) => {
                    info!("源目录删除成功: {}", delete_result.message);
                },
                Err(e) => {
                    error!("源目录删除失败: {}", e);
                    // 源目录删除失败，但不影响整体迁移结果
                }
            }
        }

        Ok(MigrationResult {
            success: true,
            message: "迁移成功完成".to_string(),
            source_path: options.source_path,
            target_path: options.target_path,
            symlink_path,
        })
    }

    /// 预迁移检查
    async fn pre_migration_check(&self, source: &Path, target: &Path) -> Result<(), String> {
        // 验证路径
        match self.file_operator.validate_migration_path(source, target) {
            Ok((valid, message)) => {
                if !valid {
                    return Err(message);
                }
            },
            Err(e) => {
                return Err(format!("路径验证失败: {}", e));
            }
        }

        // 检查权限
        if !self.has_write_permission(target.parent().unwrap_or(target)) {
            return Err("没有目标目录的写入权限".to_string());
        }

        // 检查磁盘空间（粗略估计）
        match self.estimate_required_space(source).await {
            Ok(required_space) => {
                if let Some(available_space) = self.get_available_space(target) {
                    if required_space > available_space {
                        return Err(format!("磁盘空间不足，需要 {}，可用 {}", 
                            self.format_size(required_space), 
                            self.format_size(available_space)));
                    }
                }
            },
            Err(e) => {
                warn!("无法估计所需空间: {}", e);
                // 空间检查失败，但不阻止迁移
            }
        }

        Ok(())
    }

    /// 验证复制结果
    async fn verify_copy_result(&self, source: &Path, target: &Path) -> Result<(), String> {
        // 检查目标是否存在
        if !target.exists() {
            return Err("目标目录不存在".to_string());
        }

        // 比较文件数量（粗略验证）
        let source_info = self.get_directory_info(source).await
            .map_err(|e| format!("无法获取源目录信息: {}", e))?;
        
        let target_info = self.get_directory_info(target).await
            .map_err(|e| format!("无法获取目标目录信息: {}", e))?;

        // 允许10%的差异（考虑到系统文件、临时文件等）
        let size_diff = if source_info.size > target_info.size {
            source_info.size - target_info.size
        } else {
            target_info.size - source_info.size
        };

        let size_diff_percentage = (size_diff as f64 / source_info.size as f64) * 100.0;
        
        if size_diff_percentage > 10.0 {
            return Err(format!("大小差异过大: {:.1}%", size_diff_percentage));
        }

        // 检查关键文件是否存在
        self.verify_key_files(source, target)?;

        Ok(())
    }

    /// 获取目录信息
    async fn get_directory_info(&self, path: &Path) -> Result<DirectoryInfo, String> {
        use crate::disk_analyzer::DiskAnalyzer;
        
        let analyzer = DiskAnalyzer::new();
        analyzer.get_directory_info(path).await
            .map_err(|e| format!("获取目录信息失败: {}", e))
    }

    /// 验证关键文件
    fn verify_key_files(&self, source: &Path, target: &Path) -> Result<(), String> {
        // 获取源目录中的前10个文件/目录
        let source_entries = std::fs::read_dir(source)
            .map_err(|e| format!("读取源目录失败: {}", e))?
            .filter_map(|entry| entry.ok())
            .take(10)
            .collect::<Vec<_>>();

        for source_entry in source_entries {
            let entry_name = source_entry.file_name();
            let target_entry_path = target.join(&entry_name);

            if !target_entry_path.exists() {
                return Err(format!("关键文件缺失: {}", entry_name.to_string_lossy()));
            }

            // 如果是目录，递归验证
            if source_entry.path().is_dir() {
                self.verify_key_files(&source_entry.path(), &target_entry_path)?;
            }
        }

        Ok(())
    }

    /// 创建符号链接
    async fn create_symlink_after_migration(&self, source: &Path, target: &Path) -> Result<FileOperationResult, FileOperationError> {
        // 在源目录的父目录中创建符号链接
        if let Some(parent) = source.parent() {
            let link_name = source.file_name()
                .ok_or_else(|| FileOperationError::InvalidPath("无法获取源目录名称".to_string()))?;
            
            let link_path = parent.join(link_name);
            
            // 如果源目录还存在，先重命名它
            if source.exists() {
                let backup_name = format!("{}.backup", link_name.to_string_lossy());
                let backup_path = parent.join(backup_name);
                std::fs::rename(source, &backup_path)
                    .map_err(|e| FileOperationError::IoError(e))?;
            }

            // 创建符号链接
            self.file_operator.create_symlink(target, &link_path)
        } else {
            Err(FileOperationError::InvalidPath("无法确定源目录父路径".to_string()))
        }
    }

    /// 检查是否有写入权限
    fn has_write_permission(&self, path: &Path) -> bool {
        match std::fs::metadata(path) {
            Ok(metadata) => {
                // 简化的权限检查
                // 在实际应用中，可能需要更详细的权限检查
                true
            },
            Err(_) => false,
        }
    }

    /// 估计所需空间
    async fn estimate_required_space(&self, path: &Path) -> Result<u64, String> {
        use crate::disk_analyzer::DiskAnalyzer;
        
        let analyzer = DiskAnalyzer::new();
        match analyzer.get_directory_info(path).await {
            Ok(info) => Ok(info.size),
            Err(e) => Err(format!("无法估计目录大小: {}", e)),
        }
    }

    /// 获取可用空间
    fn get_available_space(&self, path: &Path) -> Option<u64> {
        // 这里应该使用系统特定的API来获取磁盘可用空间
        // 暂时返回None，表示无法获取
        None
    }

    /// 格式化文件大小
    fn format_size(&self, size: u64) -> String {
        use crate::disk_analyzer::format_file_size;
        format_file_size(size)
    }
}

impl Default for MigrationService {
    fn default() -> Self {
        Self::new()
    }
}

/// 验证迁移选项
pub fn validate_migration_options(options: &MigrationOptions) -> Result<(), String> {
    let source = Path::new(&options.source_path);
    let target = Path::new(&options.target_path);

    // 检查源路径
    if options.source_path.is_empty() {
        return Err("源路径不能为空".to_string());
    }

    // 检查目标路径
    if options.target_path.is_empty() {
        return Err("目标路径不能为空".to_string());
    }

    // 检查路径是否相同
    if source == target {
        return Err("源路径和目标路径不能相同".to_string());
    }

    // 检查路径是否有效
    if let Ok(source_canonical) = std::fs::canonicalize(source) {
        if let Ok(target_canonical) = std::fs::canonicalize(target) {
            if source_canonical == target_canonical {
                return Err("源路径和目标路径指向同一位置".to_string());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::{self, File};
    use std::io::Write;

    #[tokio::test]
    async fn test_migrate_folder() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let target_dir = temp_dir.path().join("target");

        // 创建测试目录结构
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(source_dir.join("subdir")).unwrap();
        
        let mut file1 = File::create(source_dir.join("file1.txt")).unwrap();
        writeln!(file1, "测试内容1").unwrap();
        
        let mut file2 = File::create(source_dir.join("subdir").join("file2.txt")).unwrap();
        writeln!(file2, "测试内容2").unwrap();

        let service = MigrationService::new();
        let options = MigrationOptions {
            source_path: source_dir.display().to_string(),
            target_path: target_dir.display().to_string(),
            create_symlink: true,
            delete_source: false, // 测试中不删除源目录
        };

        let result = service.migrate_folder(options).await.unwrap();

        assert!(result.success);
        assert!(target_dir.exists());
        assert!(target_dir.join("file1.txt").exists());
        assert!(target_dir.join("subdir").exists());
        assert!(target_dir.join("subdir").join("file2.txt").exists());
    }

    #[test]
    fn test_validate_migration_options() {
        let options = MigrationOptions {
            source_path: "/path/to/source".to_string(),
            target_path: "/path/to/target".to_string(),
            create_symlink: true,
            delete_source: true,
        };

        assert!(validate_migration_options(&options).is_ok());

        let invalid_options = MigrationOptions {
            source_path: "".to_string(),
            target_path: "/path/to/target".to_string(),
            create_symlink: true,
            delete_source: true,
        };

        assert!(validate_migration_options(&invalid_options).is_err());
    }
}