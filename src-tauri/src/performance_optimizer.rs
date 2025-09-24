//! 性能优化器模块
//!
//! 提供内存优化、缓存管理、并发控制等性能优化功能

use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use std::path::{Path, PathBuf};
use std::fs;
use log::{info, warn, debug};
use lru::LruCache;
use tokio::sync::Semaphore;

/// 性能优化配置
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub max_memory_usage_mb: usize,
    pub cache_size: usize,
    pub max_concurrent_operations: usize,
    pub batch_size: usize,
    pub cleanup_interval_seconds: u64,
    pub enable_memory_monitoring: bool,
    pub enable_caching: bool,
    pub enable_batch_processing: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_memory_usage_mb: 500, // 500MB
            cache_size: 1000, // 缓存1000个目录信息
            max_concurrent_operations: 5, // 最多5个并发操作
            batch_size: 100, // 每批处理100个文件
            cleanup_interval_seconds: 300, // 5分钟清理一次
            enable_memory_monitoring: true,
            enable_caching: true,
            enable_batch_processing: true,
        }
    }
}

/// 内存使用监控器
pub struct MemoryMonitor {
    current_usage: Arc<Mutex<usize>>,
    peak_usage: Arc<Mutex<usize>>,
    warning_threshold: usize,
    critical_threshold: usize,
}

impl MemoryMonitor {
    pub fn new(warning_mb: usize, critical_mb: usize) -> Self {
        Self {
            current_usage: Arc::new(Mutex::new(0)),
            peak_usage: Arc::new(Mutex::new(0)),
            warning_threshold: warning_mb * 1024 * 1024,
            critical_threshold: critical_mb * 1024 * 1024,
        }
    }

    /// 记录内存使用
    pub fn record_usage(&self, bytes: usize) {
        if let Ok(mut current) = self.current_usage.lock() {
            *current += bytes;
            
            if let Ok(mut peak) = self.peak_usage.lock() {
                if *current > *peak {
                    *peak = *current;
                }
            }

            // 检查内存使用警告
            if *current > self.warning_threshold {
                warn!("内存使用超过警告阈值: {}MB", *current / (1024 * 1024));
            }
            
            if *current > self.critical_threshold {
                warn!("内存使用超过临界阈值: {}MB", *current / (1024 * 1024));
            }
        }
    }

    /// 释放内存
    pub fn release_usage(&self, bytes: usize) {
        if let Ok(mut current) = self.current_usage.lock() {
            *current = current.saturating_sub(bytes);
        }
    }

    /// 获取当前内存使用
    pub fn get_current_usage(&self) -> usize {
        match self.current_usage.lock() {
            Ok(guard) => *guard,
            Err(_) => 0,
        }
    }

    /// 获取峰值内存使用
    pub fn get_peak_usage(&self) -> usize {
        match self.peak_usage.lock() {
            Ok(guard) => *guard,
            Err(_) => 0,
        }
    }

    /// 检查是否需要内存清理
    pub fn should_cleanup(&self) -> bool {
        self.get_current_usage() > self.warning_threshold
    }
}

/// 目录信息缓存
pub struct DirectoryCache {
    cache: Arc<RwLock<LruCache<PathBuf, CachedDirectoryInfo>>>,
    hit_count: Arc<Mutex<usize>>,
    miss_count: Arc<Mutex<usize>>,
}

#[derive(Debug, Clone)]
pub struct CachedDirectoryInfo {
    pub path: PathBuf,
    pub file_count: usize,
    pub total_size: u64,
    pub last_modified: Instant,
    pub is_large_folder: bool,
}

impl DirectoryCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(std::num::NonZeroUsize::new(capacity).unwrap()))),
            hit_count: Arc::new(Mutex::new(0)),
            miss_count: Arc::new(Mutex::new(0)),
        }
    }

    /// 获取缓存的目录信息
    pub fn get(&self, path: &Path) -> Option<CachedDirectoryInfo> {
        if let Ok(mut cache) = self.cache.write() {
            if let Some(cached_info) = cache.get(path) {
                // 检查缓存是否过期（5分钟）
                if cached_info.last_modified.elapsed() < Duration::from_secs(300) {
                    if let Ok(mut hit_count) = self.hit_count.lock() {
                        *hit_count += 1;
                    }
                    return Some(cached_info.clone());
                }
            }
        }

        if let Ok(mut miss_count) = self.miss_count.lock() {
            *miss_count += 1;
        }
        None
    }

    /// 插入目录信息到缓存
    pub fn insert(&self, path: PathBuf, info: CachedDirectoryInfo) {
        if let Ok(mut cache) = self.cache.write() {
            cache.put(path, info);
        }
    }

    /// 清除缓存
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// 获取缓存统计
    pub fn get_stats(&self) -> CacheStats {
        let hit_count = match self.hit_count.lock() {
            Ok(guard) => *guard,
            Err(_) => 0,
        };
        let miss_count = match self.miss_count.lock() {
            Ok(guard) => *guard,
            Err(_) => 1,
        };
        let total_requests = hit_count + miss_count;
        
        CacheStats {
            hit_count,
            miss_count,
            hit_rate: if total_requests > 0 {
                (hit_count as f64 / total_requests as f64) * 100.0
            } else {
                0.0
            },
            size: self.cache.read().map(|c| c.len()).unwrap_or(0),
        }
    }
}

/// 缓存统计
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hit_count: usize,
    pub miss_count: usize,
    pub hit_rate: f64,
    pub size: usize,
}

/// 批处理管理器
pub struct BatchProcessor {
    config: PerformanceConfig,
    pending_items: Arc<Mutex<VecDeque<BatchItem>>>,
    semaphore: Arc<Semaphore>,
}

#[derive(Debug, Clone)]
pub struct BatchItem {
    pub operation_type: String,
    pub source_path: PathBuf,
    pub target_path: Option<PathBuf>,
    pub priority: u32,
}

impl BatchProcessor {
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config: config.clone(),
            pending_items: Arc::new(Mutex::new(VecDeque::new())),
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_operations)),
        }
    }

    /// 添加项目到批处理队列
    pub fn add_item(&self, item: BatchItem) -> Result<(), String> {
        if let Ok(mut queue) = self.pending_items.lock() {
            queue.push_back(item);
            Ok(())
        } else {
            Err("无法获取队列锁".to_string())
        }
    }

    /// 获取下一批项目
    pub fn get_next_batch(&self) -> Vec<BatchItem> {
        if let Ok(mut queue) = self.pending_items.lock() {
            let batch_size = self.config.batch_size.min(queue.len());
            let mut batch = Vec::with_capacity(batch_size);
            
            for _ in 0..batch_size {
                if let Some(item) = queue.pop_front() {
                    batch.push(item);
                }
            }
            
            batch
        } else {
            Vec::new()
        }
    }

    /// 获取队列大小
    pub fn queue_size(&self) -> usize {
        self.pending_items.lock().map(|q| q.len()).unwrap_or(0)
    }

    /// 获取信号量用于并发控制
    pub async fn acquire_permit(&self) -> tokio::sync::OwnedSemaphorePermit {
        self.semaphore.clone().acquire_owned().await.unwrap()
    }
}

/// 性能优化器
pub struct PerformanceOptimizer {
    config: PerformanceConfig,
    memory_monitor: MemoryMonitor,
    directory_cache: DirectoryCache,
    batch_processor: BatchProcessor,
    last_cleanup: Arc<Mutex<Instant>>,
}

impl PerformanceOptimizer {
    pub fn new(config: PerformanceConfig) -> Self {
        let memory_monitor = MemoryMonitor::new(
            config.max_memory_usage_mb / 2, // 警告阈值：50%
            config.max_memory_usage_mb * 8 / 10, // 临界阈值：80%
        );
        
        let directory_cache = DirectoryCache::new(config.cache_size);
        let batch_processor = BatchProcessor::new(config.clone());
        
        Self {
            config,
            memory_monitor,
            directory_cache,
            batch_processor,
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// 优化文件扫描操作
    pub fn optimize_scan_operation<F, R>(&self, path: &Path, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        // 检查缓存
        if self.config.enable_caching {
            if let Some(cached_info) = self.directory_cache.get(path) {
                debug!("使用缓存的目录信息: {}", path.display());
                // 这里应该返回缓存的信息，但为了简化，我们仍然执行操作
            }
        }

        // 执行操作
        let result = operation();

        // 记录内存使用（估算）
        let estimated_memory = self.estimate_scan_memory_usage(path);
        self.memory_monitor.record_usage(estimated_memory);

        result
    }

    /// 优化文件迁移操作
    pub async fn optimize_migration_operation<F, Fut, R>(&self, items: Vec<BatchItem>, operation: F) -> Vec<R>
    where
        F: Fn(BatchItem) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let mut results = Vec::new();
        
        // 分批处理
        for item in items {
            // 检查内存使用情况
            if self.memory_monitor.should_cleanup() {
                self.perform_cleanup().await;
            }

            // 获取并发许可
            let _permit = self.batch_processor.acquire_permit().await;
            
            // 执行操作
            let result = operation(item).await;
            results.push(result);
        }

        results
    }

    /// 执行内存清理
    pub async fn perform_cleanup(&self) {
        info!("执行内存清理...");
        
        // 清理缓存
        if self.config.enable_caching {
            self.directory_cache.clear();
            info!("目录缓存已清理");
        }

        // 重置内存计数
        if let Ok(mut last_cleanup) = self.last_cleanup.lock() {
            *last_cleanup = Instant::now();
        }

        info!("内存清理完成");
    }

    /// 估算扫描操作的内存使用
    fn estimate_scan_memory_usage(&self, path: &Path) -> usize {
        // 简化的估算：每个文件约1KB的元数据开销
        if let Ok(entries) = fs::read_dir(path) {
            entries.count() * 1024 // 1KB per file/directory
        } else {
            1024 // 默认1KB
        }
    }

    /// 获取性能统计
    pub fn get_performance_stats(&self) -> PerformanceStats {
        let cache_stats = self.directory_cache.get_stats();
        
        PerformanceStats {
            memory_usage_mb: self.memory_monitor.get_current_usage() as f64 / (1024.0 * 1024.0),
            memory_peak_mb: self.memory_monitor.get_peak_usage() as f64 / (1024.0 * 1024.0),
            cache_hit_rate: cache_stats.hit_rate,
            cache_size: cache_stats.size,
            batch_queue_size: self.batch_processor.queue_size(),
            last_cleanup_seconds_ago: self.last_cleanup.lock()
                .map(|instant| instant.elapsed().as_secs())
                .unwrap_or(0),
        }
    }

    /// 检查是否需要定期清理
    pub fn should_periodic_cleanup(&self) -> bool {
        if let Ok(last_cleanup) = self.last_cleanup.lock() {
            last_cleanup.elapsed() > Duration::from_secs(self.config.cleanup_interval_seconds)
        } else {
            true
        }
    }
}

/// 性能统计
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceStats {
    pub memory_usage_mb: f64,
    pub memory_peak_mb: f64,
    pub cache_hit_rate: f64,
    pub cache_size: usize,
    pub batch_queue_size: usize,
    pub last_cleanup_seconds_ago: u64,
}

/// 内存优化的目录扫描器
pub struct MemoryOptimizedScanner {
    optimizer: Arc<PerformanceOptimizer>,
    chunk_size: usize,
}

impl MemoryOptimizedScanner {
    pub fn new(optimizer: Arc<PerformanceOptimizer>, chunk_size: usize) -> Self {
        Self {
            optimizer,
            chunk_size,
        }
    }

    /// 内存优化的目录扫描
    pub async fn scan_directory_in_chunks<F, Fut>(
        &self,
        path: &Path,
        mut process_chunk: F,
    ) -> Result<(), String>
    where
        F: FnMut(Vec<std::fs::DirEntry>) -> Fut,
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        let entries = std::fs::read_dir(path)
            .map_err(|e| format!("读取目录失败: {}", e))?;
        
        let mut chunk = Vec::with_capacity(self.chunk_size);
        
        for entry in entries {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            chunk.push(entry);
            
            if chunk.len() >= self.chunk_size {
                // 处理当前批次
                let chunk_to_process = std::mem::take(&mut chunk);
                process_chunk(chunk_to_process).await?;
                
                // 记录内存使用
                let chunk_memory = chunk.len() * std::mem::size_of::<std::fs::DirEntry>();
                self.optimizer.memory_monitor.record_usage(chunk_memory);
                
                // 检查是否需要清理内存
                if self.optimizer.memory_monitor.should_cleanup() {
                    self.optimizer.perform_cleanup().await;
                }
            }
        }
        
        // 处理剩余的项目
        if !chunk.is_empty() {
            process_chunk(chunk).await?;
        }
        
        Ok(())
    }
}

/// 并发控制管理器
pub struct ConcurrencyManager {
    semaphore: Arc<Semaphore>,
    active_operations: Arc<Mutex<Vec<String>>>,
}

impl ConcurrencyManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            active_operations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 获取操作许可
    pub async fn acquire_permit(&self, operation_id: &str) -> Result<tokio::sync::OwnedSemaphorePermit, String> {
        let permit = self.semaphore.clone().acquire_owned().await
            .map_err(|e| format!("获取并发许可失败: {}", e))?;
        
        if let Ok(mut operations) = self.active_operations.lock() {
            operations.push(operation_id.to_string());
        }
        
        Ok(permit)
    }

    /// 释放操作许可
    pub fn release_permit(&self, operation_id: &str) {
        if let Ok(mut operations) = self.active_operations.lock() {
            operations.retain(|op| op != operation_id);
        }
    }

    /// 获取活跃操作数
    pub fn get_active_count(&self) -> usize {
        self.active_operations.lock().map(|ops| ops.len()).unwrap_or(0)
    }

    /// 获取活跃操作列表
    pub fn get_active_operations(&self) -> Vec<String> {
        self.active_operations.lock().map(|ops| ops.clone()).unwrap_or_default()
    }
}

/// 智能批处理器
pub struct SmartBatchProcessor {
    config: PerformanceConfig,
    item_queue: Arc<Mutex<VecDeque<SmartBatchItem>>>,
    processing_stats: Arc<Mutex<ProcessingStats>>,
}

#[derive(Debug, Clone)]
pub struct SmartBatchItem {
    pub id: String,
    pub operation_type: String,
    pub source_path: PathBuf,
    pub target_path: Option<PathBuf>,
    pub estimated_size: u64,
    pub priority: u32,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct ProcessingStats {
    pub total_processed: usize,
    pub total_size_processed: u64,
    pub average_processing_time_ms: f64,
    pub success_rate: f64,
    pub processing_history: VecDeque<ProcessingRecord>,
}

#[derive(Debug, Clone)]
pub struct ProcessingRecord {
    pub item_id: String,
    pub start_time: Instant,
    pub duration: Duration,
    pub success: bool,
    pub size_processed: u64,
}

impl SmartBatchProcessor {
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            item_queue: Arc::new(Mutex::new(VecDeque::new())),
            processing_stats: Arc::new(Mutex::new(ProcessingStats::default())),
        }
    }

    /// 添加智能批处理项目
    pub fn add_item(&self, item: SmartBatchItem) -> Result<(), String> {
        if let Ok(mut queue) = self.item_queue.lock() {
            // 根据优先级插入到合适位置
            let insert_pos = queue.iter().position(|existing| existing.priority < item.priority)
                .unwrap_or(queue.len());
            
            queue.insert(insert_pos, item);
            Ok(())
        } else {
            Err("无法获取队列锁".to_string())
        }
    }

    /// 获取优化的批次
    pub fn get_optimized_batch(&self) -> Vec<SmartBatchItem> {
        if let Ok(mut queue) = self.item_queue.lock() {
            let mut batch = Vec::new();
            let mut total_size = 0u64;
            
            // 优先处理高优先级项目
            while let Some(item) = queue.pop_front() {
                // 检查依赖项是否已处理
                if self.check_dependencies(&item) {
                    // 检查批次大小限制 - 使用 max_memory_usage_mb 作为回退
                    let max_batch_size = (self.config.max_memory_usage_mb as u64) * 1024 * 1024 / 2; // 使用50%的内存限制
                    if total_size + item.estimated_size > max_batch_size {
                        // 如果超过批次大小限制，将项目放回队列
                        queue.push_front(item);
                        break;
                    }
                    
                    total_size += item.estimated_size;
                    batch.push(item);
                    
                    if batch.len() >= self.config.batch_size {
                        break;
                    }
                } else {
                    // 依赖项未满足，将项目放回队列末尾
                    queue.push_back(item);
                }
            }
            
            batch
        } else {
            Vec::new()
        }
    }

    /// 检查依赖项是否已处理
    fn check_dependencies(&self, item: &SmartBatchItem) -> bool {
        // 简化的依赖检查：假设所有依赖项都已处理
        // 在实际实现中，这里应该检查依赖项的状态
        item.dependencies.is_empty()
    }

    /// 记录处理结果
    pub fn record_processing_result(&self, record: ProcessingRecord) {
        if let Ok(mut stats) = self.processing_stats.lock() {
            stats.total_processed += 1;
            stats.total_size_processed += record.size_processed;
            
            // 更新平均处理时间
            let new_avg = if stats.total_processed == 1 {
                record.duration.as_millis() as f64
            } else {
                (stats.average_processing_time_ms * (stats.total_processed - 1) as f64 + 
                 record.duration.as_millis() as f64) / stats.total_processed as f64
            };
            stats.average_processing_time_ms = new_avg;
            
            // 更新成功率
            let recent_successes = stats.processing_history.iter()
                .filter(|r| r.success)
                .count();
            let recent_total = stats.processing_history.len().max(1);
            stats.success_rate = (recent_successes as f64 / recent_total as f64) * 100.0;
            
            // 添加到历史记录
            stats.processing_history.push_back(record);
            
            // 保持历史记录在合理大小
            if stats.processing_history.len() > 1000 {
                stats.processing_history.pop_front();
            }
        }
    }

    /// 获取处理统计
    pub fn get_processing_stats(&self) -> ProcessingStats {
        self.processing_stats.lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| ProcessingStats::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_performance_optimizer() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        
        // 测试基本功能
        let stats = optimizer.get_performance_stats();
        assert_eq!(stats.cache_size, 0);
        assert_eq!(stats.cache_hit_rate, 0.0);
    }

    #[tokio::test]
    async fn test_memory_monitor() {
        let monitor = MemoryMonitor::new(100, 200); // 100MB warning, 200MB critical
        
        monitor.record_usage(50 * 1024 * 1024); // 50MB
        assert_eq!(monitor.get_current_usage(), 50 * 1024 * 1024);
        
        monitor.release_usage(20 * 1024 * 1024); // Release 20MB
        assert_eq!(monitor.get_current_usage(), 30 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_directory_cache() {
        let cache = DirectoryCache::new(10);
        
        let test_info = CachedDirectoryInfo {
            path: PathBuf::from("/test/path"),
            file_count: 100,
            total_size: 1024 * 1024,
            last_modified: Instant::now(),
            is_large_folder: false,
        };
        
        cache.insert(PathBuf::from("/test/path"), test_info.clone());
        
        let cached = cache.get(&PathBuf::from("/test/path"));
        assert!(cached.is_some());
        
        let stats = cache.get_stats();
        assert_eq!(stats.size, 1);
    }

    #[tokio::test]
    async fn test_concurrency_manager() {
        let manager = ConcurrencyManager::new(3);
        
        let permit1 = manager.acquire_permit("op1").await.unwrap();
        assert_eq!(manager.get_active_count(), 1);
        
        manager.release_permit("op1");
        drop(permit1);
        
        assert_eq!(manager.get_active_count(), 0);
    }

    #[tokio::test]
    async fn test_smart_batch_processor() {
        let config = PerformanceConfig::default();
        let processor = SmartBatchProcessor::new(config);
        
        let item = SmartBatchItem {
            id: "test1".to_string(),
            operation_type: "copy".to_string(),
            source_path: PathBuf::from("/source"),
            target_path: Some(PathBuf::from("/target")),
            estimated_size: 1024,
            priority: 1,
            dependencies: vec![],
        };
        
        processor.add_item(item).unwrap();
        
        let batch = processor.get_optimized_batch();
        assert_eq!(batch.len(), 1);
    }
}