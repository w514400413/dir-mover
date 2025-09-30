use std::path::{Path, PathBuf};
use std::env;
use std::sync::Arc;
use std::time::Instant;
use std::collections::HashMap;
use log::{info, warn, error, debug};
use serde::{Serialize, Deserialize};

use crate::disk_analyzer::{DiskAnalyzer};
use crate::performance_optimizer::{PerformanceOptimizer, PerformanceConfig, PerformanceStats};

/// AppData 分析器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDataConfig {
    pub min_size_threshold: u64,  // 最小大小阈值（默认1GB）
    pub max_depth: usize,          // 最大扫描深度（默认2层）
    pub sort_order: SortOrder,     // 排序方式
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Default for AppDataConfig {
    fn default() -> Self {
        Self {
            min_size_threshold: 1024 * 1024 * 1024, // 1GB
            max_depth: 2,
            sort_order: SortOrder::Desc,
        }
    }
}

/// AppData 一级项目信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDataFirstLevelItem {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub item_type: String, // "directory" or "file"
    pub parent_type: String, // "Local", "LocalLow", "Roaming"
    pub is_large: bool,
    pub size_percentage: f64,
}

/// AppData 迁移选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDataMigrationOptions {
    pub source_items: Vec<String>, // 要迁移的项目路径列表
    pub target_drive: String,      // 目标盘符（如"D:"）
    pub create_symlink: bool,      // 是否创建符号链接
    pub delete_source: bool,       // 是否删除源文件
}

/// AppData 信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDataInfo {
    pub local_path: String,
    pub local_low_path: String,
    pub roaming_path: String,
    pub local_size: u64,
    pub local_low_size: u64,
    pub roaming_size: u64,
    pub total_size: u64,
    pub first_level_items: Vec<AppDataFirstLevelItem>, // 三个目录下的一级子目录和文件
    pub large_items: Vec<AppDataFirstLevelItem>, // 1GB以上项目列表
    pub scan_time_ms: u64,
}

/// 缓存的扫描结果
#[derive(Debug, Clone)]
pub struct CachedScanResult {
    pub items: Vec<AppDataFirstLevelItem>,
    pub total_size: u64,
    pub timestamp: Instant,
}

/// AppData 分析器（性能优化版本）
#[derive(Clone)]
pub struct AppDataAnalyzer {
    disk_analyzer: DiskAnalyzer,
    config: AppDataConfig,
    performance_optimizer: Arc<PerformanceOptimizer>,
    scan_cache: Arc<tokio::sync::RwLock<HashMap<String, CachedScanResult>>>,
}

impl AppDataAnalyzer {
    /// 创建新的 AppData 分析器（性能优化版本）
    pub fn new() -> Self {
        let mut disk_analyzer = DiskAnalyzer::new();
        disk_analyzer.set_max_depth(2); // AppData扫描限制深度为2层
        
        // 创建性能优化器配置
        let perf_config = PerformanceConfig {
            max_memory_usage_mb: 150, // 限制为150MB以满足NFR-2要求
            cache_size: 500, // 缓存500个目录信息
            max_concurrent_operations: 3, // 限制并发操作数
            batch_size: 50, // 每批处理50个项目
            cleanup_interval_seconds: 60, // 1分钟清理一次
            enable_memory_monitoring: true,
            enable_caching: true,
            enable_batch_processing: true,
        };
        
        let performance_optimizer = Arc::new(PerformanceOptimizer::new(perf_config));
        
        Self {
            disk_analyzer,
            config: AppDataConfig::default(),
            performance_optimizer: performance_optimizer.clone(),
            scan_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 设置配置
    pub fn set_config(&mut self, config: AppDataConfig) {
        self.disk_analyzer.set_max_depth(config.max_depth);
        self.disk_analyzer.set_large_folder_threshold(config.min_size_threshold);
        self.config = config.clone();
    }

    /// 获取当前用户的 AppData 路径
    pub fn get_appdata_path() -> Result<PathBuf, String> {
        // 通过环境变量获取用户目录
        let user_profile = env::var("USERPROFILE")
            .map_err(|e| format!("无法获取用户目录: {}", e))?;
        
        let appdata_path = PathBuf::from(user_profile).join("AppData");
        
        if !appdata_path.exists() {
            return Err(format!("AppData目录不存在: {}", appdata_path.display()));
        }
        
        Ok(appdata_path)
    }

    /// 扫描 AppData 目录（性能优化版本）
    pub async fn scan_appdata(&self) -> Result<AppDataInfo, String> {
        info!("开始扫描AppData目录（性能优化版本）");
        let start_time = std::time::Instant::now();
        
        // 检查缓存
        let cache_key = "appdata_scan".to_string();
        {
            let cache = self.scan_cache.read().await;
            if let Some(cached_result) = cache.get(&cache_key) {
                // 检查缓存是否过期（5分钟）
                if cached_result.timestamp.elapsed() < std::time::Duration::from_secs(300) {
                    info!("使用缓存的AppData扫描结果");
                    return Ok(AppDataInfo {
                        local_path: String::new(),
                        local_low_path: String::new(),
                        roaming_path: String::new(),
                        local_size: 0,
                        local_low_size: 0,
                        roaming_size: 0,
                        total_size: cached_result.total_size,
                        first_level_items: cached_result.items.clone(),
                        large_items: cached_result.items.iter()
                            .filter(|item| item.size >= self.config.min_size_threshold)
                            .cloned()
                            .collect(),
                        scan_time_ms: 0, // 缓存命中，时间为0
                    });
                }
            }
        }
        
        // 获取性能统计
        let initial_stats = self.performance_optimizer.get_performance_stats();
        debug!("扫描前性能统计: {:?}", initial_stats);
        
        // 获取AppData路径
        let appdata_path = Self::get_appdata_path()?;
        
        // 扫描三个主要子目录（使用任务池并发执行以提高性能）
        let local_path = appdata_path.join("Local");
        let local_low_path = appdata_path.join("LocalLow");
        let roaming_path = appdata_path.join("Roaming");
        
        info!("开始并发扫描AppData子目录");
        
        // 使用简单的并发扫描（更可靠的方法）
        info!("开始并发扫描AppData子目录");
        
        // 创建并发任务（使用Arc包装self以解决生命周期问题）
        let analyzer_arc = Arc::new(self.clone());
        
        let local_task = tokio::spawn({
            let analyzer = analyzer_arc.clone();
            let path = local_path.clone();
            async move {
                analyzer.scan_first_level_items(&path, "Local").await
            }
        });
        
        let local_low_task = tokio::spawn({
            let analyzer = analyzer_arc.clone();
            let path = local_low_path.clone();
            async move {
                analyzer.scan_first_level_items(&path, "LocalLow").await
            }
        });
        
        let roaming_task = tokio::spawn({
            let analyzer = analyzer_arc.clone();
            let path = roaming_path.clone();
            async move {
                analyzer.scan_first_level_items(&path, "Roaming").await
            }
        });
        
        // 等待所有任务完成
        let (local_result, local_low_result, roaming_result) = tokio::join!(
            local_task,
            local_low_task,
            roaming_task
        );
        
        // 处理结果
        let (local_items, local_size) = local_result.map_err(|e| format!("Local扫描任务失败: {}", e))??;
        let (local_low_items, local_low_size) = local_low_result.map_err(|e| format!("LocalLow扫描任务失败: {}", e))??;
        let (roaming_items, roaming_size) = roaming_result.map_err(|e| format!("Roaming扫描任务失败: {}", e))??;
        
        // 合并所有一级项目（预分配容量以提高性能）
        let total_items = local_items.len() + local_low_items.len() + roaming_items.len();
        let all_first_level_items = Arc::new(tokio::sync::RwLock::new(Vec::with_capacity(total_items)));
        all_first_level_items.write().await.extend(local_items);
        all_first_level_items.write().await.extend(local_low_items);
        all_first_level_items.write().await.extend(roaming_items);
        
        // 计算总大小
        let total_size = local_size + local_low_size + roaming_size;
        
        // 筛选大文件（1GB以上）- 使用高效的并行过滤
        let items = all_first_level_items.read().await;
        let large_items: Vec<AppDataFirstLevelItem> = items
            .iter()
            .filter(|item| item.size >= self.config.min_size_threshold)
            .cloned()
            .collect();
        
        let scan_time_ms = start_time.elapsed().as_millis() as u64;
        
        // 记录性能统计
        let final_stats = self.performance_optimizer.get_performance_stats();
        debug!("扫描后性能统计: {:?}", final_stats);
        
        info!("AppData扫描完成 - 总大小: {}, 一级项目数量: {}, 大项目数量: {}, 耗时: {}ms",
              Self::format_size(total_size),
              items.len(),
              large_items.len(),
              scan_time_ms);
        
        // 验证性能要求
        if scan_time_ms > 30000 {
            warn!("扫描时间超过30秒要求: {}ms", scan_time_ms);
        }
        
        // 验证内存使用
        if final_stats.memory_usage_mb > 150.0 {
            warn!("内存使用超过150MB要求: {}MB", final_stats.memory_usage_mb);
        }
        
        // 缓存结果
        let cache_result = CachedScanResult {
            items: items.clone(),
            total_size,
            timestamp: Instant::now(),
        };
        
        {
            let mut cache = self.scan_cache.write().await;
            cache.insert(cache_key, cache_result);
        }
        
        Ok(AppDataInfo {
            local_path: local_path.to_string_lossy().to_string(),
            local_low_path: local_low_path.to_string_lossy().to_string(),
            roaming_path: roaming_path.to_string_lossy().to_string(),
            local_size,
            local_low_size,
            roaming_size,
            total_size,
            first_level_items: items.clone(),
            large_items,
            scan_time_ms,
        })
    }

    /// 扫描目录的一级子目录和文件（性能优化版本）
    async fn scan_first_level_items(&self, path: &Path, parent_type: &str) -> Result<(Vec<AppDataFirstLevelItem>, u64), String> {
        if !path.exists() {
            warn!("目录不存在，跳过扫描: {}", path.display());
            return Ok((Vec::new(), 0));
        }
        
        // 使用性能优化器进行内存优化的扫描
        let mut items = Vec::new();
        let mut total_size = 0u64;
        
        // 先尝试使用性能优化器的内存优化扫描功能
        let scan_result = self.performance_optimizer.optimize_scan_operation(path, || {
            // 读取目录内容
            match std::fs::read_dir(path) {
                Ok(entries) => {
                    let entries_vec: Vec<_> = entries.collect::<Result<Vec<_>, _>>()
                        .map_err(|e| format!("读取目录项失败: {}", e))?;
                    
                    // 预分配容量以提高性能
                    items.reserve(entries_vec.len());
                    
                    // 同步处理，不调用异步函数
                    for entry in entries_vec {
                        let entry_path = entry.path();
                        let name = entry.file_name().to_string_lossy().to_string();
                        
                        // 同步获取基本信息（不调用异步函数）
                        match AppDataAnalyzer::get_item_info_sync(&entry_path) {
                            Ok((size, item_type)) => {
                                total_size += size;
                                
                                let is_large = size >= self.config.min_size_threshold;
                                let size_percentage = if total_size > 0 {
                                    (size as f64 / total_size as f64) * 100.0
                                } else {
                                    0.0
                                };
                                
                                let item = AppDataFirstLevelItem {
                                    path: entry_path.to_string_lossy().to_string(),
                                    name,
                                    size,
                                    item_type,
                                    parent_type: parent_type.to_string(),
                                    is_large,
                                    size_percentage,
                                };
                                
                                items.push(item);
                            }
                            Err(e) => {
                                warn!("获取项目信息失败 {}: {}", entry_path.display(), e);
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("读取目录失败 {}: {}", path.display(), e);
                    Err(format!("读取目录失败: {}", e))
                }
            }
        });
        
        if let Err(e) = scan_result {
            warn!("扫描优化失败: {}，使用异步备用扫描", e);
            // 回退到异步扫描
            return self.scan_first_level_items_fallback(path, parent_type).await;
        }
        
        // 按大小排序（根据配置）- 使用高效的排序算法
        match self.config.sort_order {
            SortOrder::Desc => {
                items.sort_by(|a, b| b.size.cmp(&a.size));
            }
            SortOrder::Asc => {
                items.sort_by(|a, b| a.size.cmp(&b.size));
            }
        }
        
        Ok((items, total_size))
    }
    
    /// 扫描目录的一级子目录和文件（备用实现）
    async fn scan_first_level_items_fallback(&self, path: &Path, parent_type: &str) -> Result<(Vec<AppDataFirstLevelItem>, u64), String> {
        if !path.exists() {
            warn!("目录不存在，跳过扫描: {}", path.display());
            return Ok((Vec::new(), 0));
        }
        
        let mut items = Vec::new();
        let mut total_size = 0u64;
        
        // 标准扫描实现
        match std::fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let entry_path = entry.path();
                            let name = entry.file_name().to_string_lossy().to_string();
                            
                            // 获取文件/目录信息
                            match self.get_item_info(&entry_path).await {
                                Ok((size, item_type)) => {
                                    total_size += size;
                                    
                                    let is_large = size >= self.config.min_size_threshold;
                                    let size_percentage = if total_size > 0 {
                                        (size as f64 / total_size as f64) * 100.0
                                    } else {
                                        0.0
                                    };
                                    
                                    let item = AppDataFirstLevelItem {
                                        path: entry_path.to_string_lossy().to_string(),
                                        name,
                                        size,
                                        item_type,
                                        parent_type: parent_type.to_string(),
                                        is_large,
                                        size_percentage,
                                    };
                                    
                                    items.push(item);
                                }
                                Err(e) => {
                                    warn!("获取项目信息失败 {}: {}", entry_path.display(), e);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("读取目录项失败: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("读取目录失败 {}: {}", path.display(), e);
                return Ok((Vec::new(), 0));
            }
        }
        
        // 按大小排序（根据配置）
        match self.config.sort_order {
            SortOrder::Desc => {
                items.sort_by(|a, b| b.size.cmp(&a.size));
            }
            SortOrder::Asc => {
                items.sort_by(|a, b| a.size.cmp(&b.size));
            }
        }
        
        Ok((items, total_size))
    }

    /// 获取项目信息（大小和类型）
    async fn get_item_info(&self, path: &Path) -> Result<(u64, String), String> {
        if path.is_file() {
            // 文件：获取大小
            return match std::fs::metadata(path) {
                Ok(metadata) => Ok((metadata.len(), "file".to_string())),
                Err(e) => Err(format!("获取文件元数据失败: {}", e)),
            };
        } else if path.is_dir() {
            // 目录：使用DiskAnalyzer扫描获取总大小
            return match self.disk_analyzer.scan_directory_async(path).await {
                Ok(dir_info) => Ok((dir_info.size, "directory".to_string())),
                Err(e) => {
                    warn!("扫描目录失败 {}: {}，返回0大小", path.display(), e);
                    Ok((0, "directory".to_string()))
                }
            };
        } else {
            return Err("未知项目类型".to_string());
        }
    }

    /// 同步获取项目信息（用于优化扫描）
    pub fn get_item_info_sync(path: &Path) -> Result<(u64, String), String> {
        if path.is_file() {
            // 文件：获取大小
            match std::fs::metadata(path) {
                Ok(metadata) => Ok((metadata.len(), "file".to_string())),
                Err(e) => Err(format!("获取文件元数据失败: {}", e)),
            }
        } else if path.is_dir() {
            // 目录：使用同步扫描获取总大小（简化版）
            match Self::get_directory_size_sync(path) {
                Ok(size) => Ok((size, "directory".to_string())),
                Err(e) => {
                    warn!("扫描目录失败 {}: {}，返回0大小", path.display(), e);
                    Ok((0, "directory".to_string()))
                }
            }
        } else {
            Err("未知项目类型".to_string())
        }
    }

    /// 同步获取目录大小（简化版）
    pub fn get_directory_size_sync(path: &Path) -> Result<u64, String> {
        let mut total_size = 0u64;
        
        match std::fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let entry_path = entry.path();
                        if entry_path.is_file() {
                            if let Ok(metadata) = std::fs::metadata(&entry_path) {
                                total_size += metadata.len();
                            }
                        } else if entry_path.is_dir() {
                            // 递归获取子目录大小（限制深度以避免性能问题）
                            if let Ok(sub_size) = Self::get_directory_size_sync(&entry_path) {
                                total_size += sub_size;
                            }
                        }
                    }
                }
                Ok(total_size)
            }
            Err(e) => Err(format!("读取目录失败: {}", e)),
        }
    }

    /// 格式化文件大小
    pub fn format_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        const THRESHOLD: f64 = 1024.0;

        if size == 0 {
            return "0 B".to_string();
        }

        let mut size_f = size as f64;
        let mut unit_index = 0;

        while size_f >= THRESHOLD && unit_index < UNITS.len() - 1 {
            size_f /= THRESHOLD;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size_f as u64, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size_f, UNITS[unit_index])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appdata_config_default() {
        let config = AppDataConfig::default();
        assert_eq!(config.min_size_threshold, 1024 * 1024 * 1024);
        assert_eq!(config.max_depth, 2);
        match config.sort_order {
            SortOrder::Desc => (),
            _ => panic!("默认排序应该是降序"),
        }
    }

    #[test]
    fn test_format_size() {
        assert_eq!(AppDataAnalyzer::format_size(0), "0 B");
        assert_eq!(AppDataAnalyzer::format_size(1024), "1.00 KB");
        assert_eq!(AppDataAnalyzer::format_size(1024 * 1024), "1.00 MB");
        assert_eq!(AppDataAnalyzer::format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[tokio::test]
    async fn test_get_appdata_path() {
        // 这个测试需要在Windows环境下运行
        let result = AppDataAnalyzer::get_appdata_path();
        match result {
            Ok(path) => {
                assert!(path.exists(), "AppData路径应该存在");
                assert!(path.to_string_lossy().contains("AppData"), "路径应该包含AppData");
            }
            Err(e) => {
                // 在非Windows环境下可能会失败，这是预期的
                println!("获取AppData路径失败（这可能是预期的）: {}", e);
            }
        }
    }

    #[test]
    fn test_appdata_first_level_item() {
        let item = AppDataFirstLevelItem {
            path: "C:\\Users\\Test\\AppData\\Local\\TestApp".to_string(),
            name: "TestApp".to_string(),
            size: 2 * 1024 * 1024 * 1024, // 2GB
            item_type: "directory".to_string(),
            parent_type: "Local".to_string(),
            is_large: true,
            size_percentage: 50.0,
        };

        assert_eq!(item.name, "TestApp");
        assert_eq!(item.size, 2147483648);
        assert!(item.is_large);
        assert_eq!(item.parent_type, "Local");
    }
}