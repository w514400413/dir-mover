use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;
use log::{info, error, warn};
use serde::{Serialize, Deserialize};

/// 目录信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub file_count: u64,
    pub subdirectories: Vec<DirectoryInfo>,
    pub last_modified: Option<SystemTime>,
    pub is_large_folder: bool,      // 新增：标识大文件夹
    pub size_percentage: f64,       // 新增：相对父目录的占比
}

/// 扫描进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub current_path: String,
    pub processed_files: u64,
    pub total_files: u64,
    pub progress: f64,
    pub processed_directories: u64, // 新增：已处理目录数
    pub total_directories: u64,     // 新增：总目录数
    pub current_directory: String,  // 新增：当前处理的目录
    pub estimated_time_remaining: u64, // 新增：预计剩余时间（秒）
    pub scan_speed: f64,            // 新增：扫描速度（文件/秒）
    pub start_time: Option<SystemTime>, // 新增：扫描开始时间
    pub large_folders_found: u64,   // 新增：发现的大文件夹数量
}

/// 磁盘分析器
#[derive(Clone)]
pub struct DiskAnalyzer {
    max_depth: usize,
    cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    progress_info: std::sync::Arc<std::sync::Mutex<ScanProgress>>,
    large_folder_threshold: u64, // 新增：大文件夹阈值（默认1GB）
    is_c_drive_mode: bool,       // 新增：是否为C盘专项扫描模式
}

impl Default for DiskAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl DiskAnalyzer {
    /// 创建新的磁盘分析器
    pub fn new() -> Self {
        Self {
            max_depth: 3, // 默认最大深度为3层
            cancel_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            large_folder_threshold: 1024 * 1024 * 1024, // 默认1GB
            is_c_drive_mode: false,
            progress_info: std::sync::Arc::new(std::sync::Mutex::new(ScanProgress {
                current_path: String::new(),
                processed_files: 0,
                total_files: 0,
                progress: 0.0,
                processed_directories: 0,
                total_directories: 0,
                current_directory: String::new(),
                estimated_time_remaining: 0,
                scan_speed: 0.0,
                start_time: None,
                large_folders_found: 0,
            })),
        }
    }

    /// 设置最大扫描深度
    pub fn set_max_depth(&mut self, depth: usize) {
        self.max_depth = depth;
    }

    /// 设置大文件夹阈值
    pub fn set_large_folder_threshold(&mut self, threshold: u64) {
        self.large_folder_threshold = threshold;
    }

    /// 设置C盘专项扫描模式
    pub fn set_c_drive_mode(&mut self, enabled: bool) {
        self.is_c_drive_mode = enabled;
    }

    /// 获取C盘路径
    pub fn get_c_drive_path() -> PathBuf {
        PathBuf::from("C:\\")
    }

    /// 检查是否为C盘路径
    pub fn is_c_drive_path(path: &Path) -> bool {
        path.to_string_lossy().to_uppercase().starts_with("C:\\")
    }

    /// 取消当前扫描
    pub fn cancel_scan(&self) {
        self.cancel_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// 重置取消标志
    pub fn reset_cancel_flag(&self) {
        self.cancel_flag.store(false, std::sync::atomic::Ordering::Relaxed);
        // 重置进度信息
        let mut progress = self.progress_info.lock().unwrap();
        *progress = ScanProgress {
            current_path: String::new(),
            processed_files: 0,
            total_files: 0,
            progress: 0.0,
            processed_directories: 0,
            total_directories: 0,
            current_directory: String::new(),
            estimated_time_remaining: 0,
            scan_speed: 0.0,
            start_time: None,
            large_folders_found: 0,
        };
    }

    /// 检查是否已取消
    fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 获取当前扫描进度
    pub fn get_scan_progress(&self) -> ScanProgress {
        let progress = self.progress_info.lock().unwrap();
        progress.clone()
    }

    /// 更新扫描进度
    fn update_progress(&self, current_path: &str, processed_files: u64, total_files: u64) {
        let mut progress = self.progress_info.lock().unwrap();
        progress.current_path = current_path.to_string();
        progress.processed_files = processed_files;
        progress.total_files = total_files;
        progress.progress = if total_files > 0 {
            (processed_files as f64 / total_files as f64) * 100.0
        } else {
            0.0
        };

        // 计算扫描速度和预计剩余时间
        if let Some(start_time) = progress.start_time {
            if let Ok(elapsed) = start_time.elapsed() {
                let elapsed_secs = elapsed.as_secs();
                if elapsed_secs > 0 {
                    progress.scan_speed = processed_files as f64 / elapsed_secs as f64;
                    if progress.scan_speed > 0.0 && total_files > processed_files {
                        let remaining_files = total_files - processed_files;
                        progress.estimated_time_remaining = (remaining_files as f64 / progress.scan_speed) as u64;
                    }
                }
            }
        }
    }

    /// 更新目录进度
    fn update_directory_progress(&self, current_directory: &str, processed_dirs: u64, total_dirs: u64) {
        let mut progress = self.progress_info.lock().unwrap();
        progress.current_directory = current_directory.to_string();
        progress.processed_directories = processed_dirs;
        progress.total_directories = total_dirs;
    }

    /// 发现大文件夹
    fn increment_large_folders_found(&self) {
        let mut progress = self.progress_info.lock().unwrap();
        progress.large_folders_found += 1;
    }

    /// 扫描目录（异步版本）
    pub async fn scan_directory_async(&self, path: &Path) -> Result<DirectoryInfo, String> {
        info!("磁盘分析器: 开始异步扫描目录 {}", path.display());
        
        // 在阻塞任务中执行同步扫描
        let path_clone = path.to_path_buf();
        let path_display = path.display().to_string();
        let analyzer = self.clone();
        
        let result = tokio::task::spawn_blocking(move || {
            analyzer.scan_directory(&path_clone)
        }).await.map_err(|e| format!("扫描任务失败: {}", e))?;
        
        match &result {
            Ok(info) => {
                info!("磁盘分析器: 异步扫描完成 {} (文件数: {}, 大小: {})",
                      path_display, info.file_count, info.size);
            }
            Err(e) => {
                error!("磁盘分析器: 异步扫描失败 {}: {}", path_display, e);
            }
        }
        
        result
    }

    /// 扫描目录（同步版本）
    pub fn scan_directory(&self, path: &Path) -> Result<DirectoryInfo, String> {
        info!("磁盘分析器: 开始同步扫描目录 {}", path.display());
        
        // 初始化进度信息
        {
            let mut progress = self.progress_info.lock().unwrap();
            progress.start_time = Some(SystemTime::now());
            progress.current_path = path.display().to_string();
        }
        
        // C盘专项扫描模式处理
        if self.is_c_drive_mode || Self::is_c_drive_path(path) {
            info!("启用C盘专项扫描模式");
            return self.scan_c_drive_directory(path);
        }
        
        let result = self.scan_directory_recursive(path, 0);
        match &result {
            Ok(info) => {
                info!("磁盘分析器: 扫描完成 {} (文件数: {}, 大小: {})",
                      path.display(), info.file_count, info.size);
                // 扫描完成时更新最终进度
                self.update_progress(&path.display().to_string(), info.file_count, info.file_count);
            }
            Err(e) => {
                error!("磁盘分析器: 扫描失败 {}: {}", path.display(), e);
            }
        }
        result
    }

    /// C盘专项扫描模式
    fn scan_c_drive_directory(&self, path: &Path) -> Result<DirectoryInfo, String> {
        info!("C盘专项扫描: 开始扫描 {}", path.display());
        
        // C盘特殊目录列表（通常需要权限或应该跳过）
        let system_dirs = [
            "C:\\Windows\\System32",
            "C:\\Windows\\SysWOW64",
            "C:\\Program Files\\Windows Defender",
            "C:\\$Recycle.Bin",
            "C:\\System Volume Information",
            "C:\\Recovery",
            "C:\\pagefile.sys",
            "C:\\hiberfil.sys",
        ];
        
        let path_str = path.to_string_lossy().to_string();
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path_str)
            .to_string();
        
        // 检查是否为系统关键目录
        let is_system_dir = system_dirs.iter().any(|dir| {
            path_str.to_uppercase().starts_with(&dir.to_uppercase())
        });
        
        if is_system_dir {
            warn!("跳过系统关键目录: {}", path.display());
            return Ok(DirectoryInfo {
                path: path_str,
                name,
                size: 0,
                file_count: 0,
                subdirectories: Vec::new(),
                last_modified: None,
                is_large_folder: false,
                size_percentage: 0.0,
            });
        }
        
        // 使用优化的扫描策略
        self.scan_directory_optimized(path, 0)
    }

    /// 优化的目录扫描（适用于大目录）
    fn scan_directory_optimized(&self, path: &Path, depth: usize) -> Result<DirectoryInfo, String> {
        if self.is_cancelled() {
            return Err("扫描已取消".to_string());
        }

        if depth > self.max_depth {
            return Ok(DirectoryInfo {
                path: path.to_string_lossy().to_string(),
                name: path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
                size: 0,
                file_count: 0,
                subdirectories: Vec::new(),
                last_modified: None,
                is_large_folder: false,
                size_percentage: 0.0,
            });
        }

        let path_str = path.to_string_lossy().to_string();
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path_str)
            .to_string();

        let mut total_size: u64 = 0;
        let mut file_count: u64 = 0;
        let mut subdirectories: Vec<DirectoryInfo> = Vec::new();

        // 优化的读取策略
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                // 权限不足时的处理
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    warn!("权限不足，跳过目录: {}", path.display());
                    return Ok(DirectoryInfo {
                        path: path_str,
                        name,
                        size: 0,
                        file_count: 0,
                        subdirectories: Vec::new(),
                        last_modified: None,
                        is_large_folder: false,
                        size_percentage: 0.0,
                    });
                }
                error!("读取目录失败 {}: {}", path.display(), e);
                return Err(format!("读取目录失败: {}", e));
            }
        };

        // C盘模式下的优化处理
        let max_entries = if self.is_c_drive_mode && depth == 0 {
            100 // C盘根目录只处理前100个条目
        } else {
            1000
        };

        let mut entry_count = 0;
        for entry in entries {
            if self.is_cancelled() {
                return Err("扫描已取消".to_string());
            }

            if entry_count >= max_entries {
                warn!("目录 {} 条目过多，只处理前 {} 个条目", path.display(), max_entries);
                break;
            }
            entry_count += 1;

            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        warn!("权限不足，跳过条目: {}", e);
                        continue;
                    }
                    error!("读取条目失败: {}", e);
                    continue;
                }
            };

            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                match self.scan_directory_optimized(&entry_path, depth + 1) {
                    Ok(subdir_info) => {
                        total_size += subdir_info.size;
                        file_count += subdir_info.file_count + 1;
                        subdirectories.push(subdir_info);
                    }
                    Err(e) => {
                        if e == "扫描已取消" {
                            return Err(e);
                        }
                        error!("扫描子目录失败 {}: {}", entry_path.display(), e);
                        continue;
                    }
                }
            } else {
                match entry.metadata() {
                    Ok(metadata) => {
                        total_size += metadata.len();
                        file_count += 1;
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::PermissionDenied {
                            warn!("权限不足，跳过文件: {}", e);
                            continue;
                        }
                        error!("获取文件元数据失败 {}: {}", entry_path.display(), e);
                        continue;
                    }
                }
            }
        }

        // 获取目录修改时间
        let last_modified = fs::metadata(path)
            .and_then(|m| m.modified().or_else(|_| m.created()))
            .ok();

        // 按大小排序子目录
        subdirectories.sort_by(|a, b| b.size.cmp(&a.size));

        // 计算大文件夹标识和占比
        let is_large_folder = total_size >= self.large_folder_threshold;
        let mut subdirs_with_percentage = subdirectories;
        if total_size > 0 {
            for subdir in &mut subdirs_with_percentage {
                subdir.size_percentage = (subdir.size as f64 / total_size as f64) * 100.0;
            }
        }

        Ok(DirectoryInfo {
            path: path_str,
            name,
            size: total_size,
            file_count,
            subdirectories: subdirs_with_percentage,
            last_modified,
            is_large_folder,
            size_percentage: 0.0,
        })
    }

    /// 递归扫描目录
    fn scan_directory_recursive(&self, path: &Path, depth: usize) -> Result<DirectoryInfo, String> {
        // 检查是否已取消
        if self.is_cancelled() {
            return Err("扫描已取消".to_string());
        }

        // 更新当前处理的目录
        self.update_directory_progress(&path.display().to_string(), 0, 0);

        // 检查深度限制
        if depth > self.max_depth {
            return Ok(DirectoryInfo {
                path: path.to_string_lossy().to_string(),
                name: path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
                size: 0,
                file_count: 0,
                subdirectories: Vec::new(),
                last_modified: None,
                is_large_folder: false,
                size_percentage: 0.0,
            });
        }

        let path_str = path.to_string_lossy().to_string();
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path_str)
            .to_string();

        info!("扫描目录: {} (深度: {})", path.display(), depth);

        let mut total_size: u64 = 0;
        let mut file_count: u64 = 0;
        let mut subdirectories: Vec<DirectoryInfo> = Vec::new();
        let mut processed_entries: u64 = 0;

        // 读取目录条目
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                error!("读取目录失败 {}: {}", path.display(), e);
                return Ok(DirectoryInfo {
                    path: path_str,
                    name,
                    size: 0,
                    file_count: 0,
                    subdirectories: Vec::new(),
                    last_modified: None,
                    is_large_folder: false,
                    size_percentage: 0.0,
                });
            }
        };

        // 限制处理的条目数量，避免扫描过多文件
        let mut entry_count = 0;
        const MAX_ENTRIES_PER_DIR: usize = 1000;

        // 处理每个条目
        for entry in entries {
            // 检查是否已取消
            if self.is_cancelled() {
                return Err("扫描已取消".to_string());
            }

            // 限制条目数量
            if entry_count >= MAX_ENTRIES_PER_DIR {
                warn!("目录 {} 条目过多，只处理前 {} 个条目", path.display(), MAX_ENTRIES_PER_DIR);
                break;
            }
            entry_count += 1;

            // 更新进度
            processed_entries += 1;
            if processed_entries % 100 == 0 { // 每100个条目更新一次进度
                self.update_progress(&path.display().to_string(), processed_entries, MAX_ENTRIES_PER_DIR as u64);
            }

            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    error!("读取条目失败: {}", e);
                    continue;
                }
            };

            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                // 递归扫描子目录
                match self.scan_directory_recursive(&entry_path, depth + 1) {
                    Ok(subdir_info) => {
                        total_size += subdir_info.size;
                        file_count += subdir_info.file_count + 1; // +1 为目录本身
                        subdirectories.push(subdir_info);
                    }
                    Err(e) => {
                        if e == "扫描已取消" {
                            return Err(e);
                        }
                        error!("扫描子目录失败 {}: {}", entry_path.display(), e);
                        continue;
                    }
                }
            } else {
                // 处理文件
                match entry.metadata() {
                    Ok(metadata) => {
                        total_size += metadata.len();
                        file_count += 1;
                    }
                    Err(e) => {
                        error!("获取文件元数据失败 {}: {}", entry_path.display(), e);
                        continue;
                    }
                }
            }
        }

        // 获取目录本身的修改时间
        let last_modified = fs::metadata(path)
            .and_then(|m| m.modified().or_else(|_| m.created()))
            .ok();

        // 按大小排序子目录（从大到小）
        subdirectories.sort_by(|a, b| b.size.cmp(&a.size));

        // 计算是否为大型文件夹
        let is_large_folder = total_size >= self.large_folder_threshold;
        if is_large_folder {
            self.increment_large_folders_found();
        }

        // 计算子目录的相对占比
        let mut subdirs_with_percentage = subdirectories;
        if total_size > 0 {
            for subdir in &mut subdirs_with_percentage {
                subdir.size_percentage = (subdir.size as f64 / total_size as f64) * 100.0;
            }
        }

        Ok(DirectoryInfo {
            path: path_str,
            name,
            size: total_size,
            file_count,
            subdirectories: subdirs_with_percentage,
            last_modified,
            is_large_folder,
            size_percentage: 0.0, // 根目录的占比设为0，由上层计算
        })
    }

    /// 获取目录的简要信息（不递归）
    pub async fn get_directory_info(&self, path: &Path) -> Result<DirectoryInfo, String> {
        let path_str = path.to_string_lossy().to_string();
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path_str)
            .to_string();

        let metadata = fs::metadata(path)
            .map_err(|e| format!("获取目录元数据失败: {}", e))?;

        let size = if metadata.is_dir() {
            // 计算目录总大小
            self.calculate_directory_size(path)?
        } else {
            metadata.len()
        };

        let last_modified = metadata.modified().or_else(|_| metadata.created()).ok();

        Ok(DirectoryInfo {
            path: path_str,
            name,
            size,
            file_count: 0, // 简要信息不包含文件计数
            subdirectories: Vec::new(),
            last_modified,
            is_large_folder: size >= self.large_folder_threshold,
            size_percentage: 0.0,
        })
    }

    /// 计算目录总大小
    fn calculate_directory_size(&self, path: &Path) -> Result<u64, String> {
        let mut total_size = 0u64;

        let entries = fs::read_dir(path)
            .map_err(|e| format!("读取目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                total_size += self.calculate_directory_size(&entry_path)?;
            } else {
                let metadata = entry.metadata()
                    .map_err(|e| format!("获取文件元数据失败: {}", e))?;
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }
}


/// 工具函数：格式化文件大小
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    const THRESHOLD: f64 = 1024.0;

    if size == 0 {
        return "0 B".to_string();
    }

    let size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= THRESHOLD && unit_index < UNITS.len() - 1 {
        unit_index += 1;
    }

    let size_in_unit = size_f / THRESHOLD.powi(unit_index as i32);
    
    if unit_index == 0 {
        format!("{} {}", size_in_unit as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size_in_unit, UNITS[unit_index])
    }
}

/// 工具函数：获取文件大小百分比
pub fn get_size_percentage(size: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (size as f64 / total as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(1023), "1023 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1536), "1.50 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_get_size_percentage() {
        assert_eq!(get_size_percentage(0, 100), 0.0);
        assert_eq!(get_size_percentage(50, 100), 50.0);
        assert_eq!(get_size_percentage(100, 100), 100.0);
        assert_eq!(get_size_percentage(25, 100), 25.0);
    }
}