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

    /// 预迁移检查（增强版）
    async fn pre_migration_check(&self, source: &Path, target: &Path) -> Result<(), String> {
        // 1. 路径安全性检查
        match self.validate_path_security(source, target) {
            Ok(_) => {},
            Err(e) => return Err(format!("路径安全检查失败: {}", e)),
        }

        // 2. 验证路径
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

        // 3. 权限检查
        match self.check_permissions(source, target).await {
            Ok(_) => {},
            Err(e) => return Err(format!("权限检查失败: {}", e)),
        }

        // 4. 磁盘空间检查
        match self.check_disk_space(source, target).await {
            Ok(_) => {},
            Err(e) => return Err(format!("磁盘空间检查失败: {}", e)),
        }

        // 5. 系统保护检查
        match self.check_system_protection(source, target) {
            Ok(_) => {},
            Err(e) => return Err(format!("系统保护检查失败: {}", e)),
        }

        info!("预迁移检查通过: {} -> {}", source.display(), target.display());
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


    /// 估计所需空间
    async fn estimate_required_space(&self, path: &Path) -> Result<u64, String> {
        use crate::disk_analyzer::DiskAnalyzer;
        
        let analyzer = DiskAnalyzer::new();
        match analyzer.get_directory_info(path).await {
            Ok(info) => Ok(info.size),
            Err(e) => Err(format!("无法估计目录大小: {}", e)),
        }
    }

    /// 获取可用空间（简化版）
    fn get_available_space(&self, path: &Path) -> Option<u64> {
        // 简化实现：使用粗略估计
        // 在实际应用中，这里应该使用系统API获取真实的磁盘空间
        let path_str = path.to_string_lossy();
        
        // 根据路径所在磁盘返回估计值
        if path_str.starts_with("C:\\") {
            Some(50 * 1024 * 1024 * 1024) // 假设C盘有50GB可用
        } else if path_str.starts_with("D:\\") {
            Some(100 * 1024 * 1024 * 1024) // 假设D盘有100GB可用
        } else {
            Some(20 * 1024 * 1024 * 1024) // 默认20GB
        }
    }

    /// 路径安全性检查
    fn validate_path_security(&self, source: &Path, target: &Path) -> Result<(), String> {
        // 检查路径遍历攻击
        let source_str = source.to_string_lossy();
        let target_str = target.to_string_lossy();
        
        // 检查是否包含路径遍历字符
        if source_str.contains("..") || target_str.contains("..") {
            return Err("路径包含非法的父目录引用".to_string());
        }
        
        // 检查是否包含特殊字符
        let invalid_chars = ['<', '>', ':', '*', '?', '|'];
        for char in &invalid_chars {
            if source_str.contains(*char) || target_str.contains(*char) {
                return Err(format!("路径包含非法字符: {}", char));
            }
        }
        
        // 检查Windows保留名称
        #[cfg(target_os = "windows")]
        {
            let reserved_names = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5",
                                 "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4",
                                 "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
            
            let source_name = source.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_uppercase();
                
            let target_name = target.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_uppercase();
            
            for reserved in &reserved_names {
                if source_name == *reserved || target_name == *reserved {
                    return Err(format!("路径使用了Windows保留名称: {}", reserved));
                }
            }
        }
        
        // 检查路径长度
        if source_str.len() > 260 || target_str.len() > 260 {
            return Err("路径过长（超过260字符）".to_string());
        }
        
        info!("路径安全检查通过");
        Ok(())
    }

    /// 权限检查
    async fn check_permissions(&self, source: &Path, target: &Path) -> Result<(), String> {
        // 检查源路径读取权限
        if !self.has_read_permission(source) {
            return Err(format!("没有源路径的读取权限: {}", source.display()));
        }
        
        // 检查目标父目录的写入权限
        let target_parent = target.parent().unwrap_or(target);
        if !self.has_write_permission(target_parent) {
            return Err(format!("没有目标父目录的写入权限: {}", target_parent.display()));
        }
        
        // 检查是否需要管理员权限
        if self.requires_admin_permission(source, target) {
            info!("迁移操作可能需要管理员权限");
            // 在实际应用中，这里可以触发UAC提示
        }
        
        info!("权限检查通过");
        Ok(())
    }

    /// 磁盘空间检查
    async fn check_disk_space(&self, source: &Path, target: &Path) -> Result<(), String> {
        // 获取源目录大小
        let required_space = match self.estimate_required_space(source).await {
            Ok(size) => size,
            Err(e) => {
                warn!("无法准确估计源目录大小: {}", e);
                // 使用粗略估计：如果无法准确估计，使用1GB作为默认值
                1024 * 1024 * 1024
            }
        };
        
        // 获取目标磁盘可用空间
        let available_space = match self.get_available_space(target) {
            Some(space) => space,
            None => {
                // 如果无法获取可用空间，检查目标父目录是否存在
                let target_parent = target.parent().unwrap_or(target);
                if !target_parent.exists() {
                    return Err(format!("目标父目录不存在: {}", target_parent.display()));
                }
                // 使用粗略估计：假设至少有10GB可用空间
                10 * 1024 * 1024 * 1024
            }
        };
        
        // 计算所需空间（源目录大小 + 20%缓冲）
        let required_with_buffer = required_space + (required_space / 5);
        
        if required_with_buffer > available_space {
            return Err(format!(
                "磁盘空间不足。需要: {} (含20%缓冲)，可用: {}", 
                crate::disk_analyzer::format_file_size(required_with_buffer),
                crate::disk_analyzer::format_file_size(available_space)
            ));
        }
        
        // 检查目标磁盘是否即将满
        let usage_percentage = (required_with_buffer as f64 / (available_space + required_with_buffer) as f64) * 100.0;
        if usage_percentage > 90.0 {
            warn!("迁移后磁盘使用率将超过90%: {:.1}%", usage_percentage);
        }
        
        info!("磁盘空间检查通过: 需要 {}, 可用 {}", 
              crate::disk_analyzer::format_file_size(required_with_buffer), 
              crate::disk_analyzer::format_file_size(available_space));
        Ok(())
    }

    /// 系统保护检查
    fn check_system_protection(&self, source: &Path, target: &Path) -> Result<(), String> {
        // 检查是否是系统关键目录
        let system_paths = [
            "C:\\Windows",
            "C:\\Program Files",
            "C:\\Program Files (x86)",
            "C:\\Users\\Default",
            "C:\\Recovery",
            "C:\\System Volume Information",
            "C:\\$Recycle.Bin",
        ];
        
        let source_str = source.to_string_lossy().to_uppercase();
        let target_str = target.to_string_lossy().to_uppercase();
        
        for system_path in &system_paths {
            if source_str.starts_with(system_path) || target_str.starts_with(system_path) {
                return Err(format!("不能操作系统保护目录: {}", system_path));
            }
        }
        
        // 检查是否是程序安装目录
        if self.is_program_installation_directory(source) || self.is_program_installation_directory(target) {
            return Err("不能迁移程序安装目录，可能导致程序无法运行".to_string());
        }
        
        // 检查是否包含系统文件
        let system_files = ["pagefile.sys", "hiberfil.sys", "swapfile.sys"];
        for system_file in &system_files {
            if source_str.contains(system_file) || target_str.contains(system_file) {
                return Err(format!("不能操作系统文件: {}", system_file));
            }
        }
        
        info!("系统保护检查通过");
        Ok(())
    }

    /// 检查是否有读取权限
    fn has_read_permission(&self, path: &Path) -> bool {
        if !path.exists() {
            return false;
        }
        
        // 尝试读取目录内容
        match fs::read_dir(path) {
            Ok(_) => true,
            Err(e) => {
                warn!("读取权限检查失败 {}: {}", path.display(), e);
                false
            }
        }
    }

    /// 检查是否需要管理员权限
    fn requires_admin_permission(&self, source: &Path, target: &Path) -> bool {
        // 检查是否涉及系统目录
        let system_paths = ["C:\\Windows", "C:\\Program Files", "C:\\Program Files (x86)"];
        let source_str = source.to_string_lossy().to_uppercase();
        let target_str = target.to_string_lossy().to_uppercase();
        
        for system_path in &system_paths {
            if source_str.starts_with(system_path) || target_str.starts_with(system_path) {
                return true;
            }
        }
        
        false
    }

    /// 检查是否是程序安装目录
    fn is_program_installation_directory(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_uppercase();
        
        // 检查常见的程序安装路径
        let program_paths = [
            "C:\\PROGRAM FILES",
            "C:\\PROGRAM FILES (X86)",
            "C:\\USERS\\",
        ];
        
        for program_path in &program_paths {
            if path_str.starts_with(program_path) {
                // 进一步检查是否包含可执行文件
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(".exe") || name.ends_with(".dll") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        
        false
    }

    /// 获取根目录
    fn get_root_directory(&self, path: &Path) -> PathBuf {
        let path_str = path.to_string_lossy();
        
        #[cfg(target_os = "windows")]
        {
            // Windows系统：提取盘符
            if let Some(drive) = path_str.chars().next() {
                return PathBuf::from(format!("{}:\\", drive));
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // Unix系统：返回根目录
            return PathBuf::from("/");
        }
        
        PathBuf::from(path_str.to_string())
    }

    /// 估计可用空间（备用方法）
    fn estimate_available_space(&self, path: &Path) -> Result<u64, String> {
        // 创建一个临时文件来测试写入
        let temp_file = path.join(".space_test.tmp");
        
        // 尝试写入不同大小的数据
        let test_sizes = [1024 * 1024, 10 * 1024 * 1024, 100 * 1024 * 1024]; // 1MB, 10MB, 100MB
        
        for size in &test_sizes {
            match self.test_write_space(&temp_file, *size) {
                Ok(_) => {
                    // 删除临时文件
                    let _ = fs::remove_file(&temp_file);
                    return Ok(*size * 10); // 估计可用空间为测试大小的10倍
                },
                Err(_) => {
                    // 删除临时文件
                    let _ = fs::remove_file(&temp_file);
                    continue;
                }
            }
        }
        
        Err("无法估计可用磁盘空间".to_string())
    }

    /// 测试写入空间
    fn test_write_space(&self, temp_file: &Path, size: u64) -> Result<(), String> {
        // 创建指定大小的临时文件
        let data = vec![0u8; size as usize];
        
        match fs::write(temp_file, &data) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("写入测试失败: {}", e)),
        }
    }

    /// 检查是否有写入权限（增强版）
    fn has_write_permission(&self, path: &Path) -> bool {
        if !path.exists() {
            // 如果路径不存在，检查父目录
            if let Some(parent) = path.parent() {
                return self.has_write_permission(parent);
            }
            return false;
        }
        
        // 尝试创建一个临时文件来测试写入权限
        let temp_file = path.join(".write_test.tmp");
        match fs::write(&temp_file, b"test") {
            Ok(_) => {
                // 删除临时文件
                let _ = fs::remove_file(&temp_file);
                true
            },
            Err(_) => false
        }
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