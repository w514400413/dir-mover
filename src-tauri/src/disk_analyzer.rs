use std::path::Path;
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
}

/// 扫描进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub current_path: String,
    pub processed_files: u64,
    pub total_files: u64,
    pub progress: f64,
}

/// 磁盘分析器
#[derive(Clone)]
pub struct DiskAnalyzer {
    max_depth: usize,
    cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
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
        }
    }

    /// 设置最大扫描深度
    pub fn set_max_depth(&mut self, depth: usize) {
        self.max_depth = depth;
    }

    /// 取消当前扫描
    pub fn cancel_scan(&self) {
        self.cancel_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// 重置取消标志
    pub fn reset_cancel_flag(&self) {
        self.cancel_flag.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    /// 检查是否已取消
    fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(std::sync::atomic::Ordering::Relaxed)
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
        let result = self.scan_directory_recursive(path, 0);
        match &result {
            Ok(info) => {
                info!("磁盘分析器: 扫描完成 {} (文件数: {}, 大小: {})",
                      path.display(), info.file_count, info.size);
            }
            Err(e) => {
                error!("磁盘分析器: 扫描失败 {}: {}", path.display(), e);
            }
        }
        result
    }

    /// 递归扫描目录
    fn scan_directory_recursive(&self, path: &Path, depth: usize) -> Result<DirectoryInfo, String> {
        // 检查是否已取消
        if self.is_cancelled() {
            return Err("扫描已取消".to_string());
        }

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

        Ok(DirectoryInfo {
            path: path_str,
            name,
            size: total_size,
            file_count,
            subdirectories,
            last_modified,
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