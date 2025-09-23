//! 测试工具模块
//! 
//! 提供测试所需的通用工具函数和辅助结构

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use log::{info, warn};
use tempfile::TempDir;

/// 测试数据生成器
pub struct TestDataGenerator {
    temp_dir: TempDir,
}

impl TestDataGenerator {
    /// 创建新的测试数据生成器
    pub fn new() -> Result<Self, std::io::Error> {
        let temp_dir = TempDir::new()?;
        Ok(Self { temp_dir })
    }

    /// 获取临时目录路径
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// 生成标准测试目录结构
    pub fn create_standard_structure(&self, name: &str) -> Result<PathBuf, std::io::Error> {
        let base_path = self.temp_dir.path().join(name);
        create_standard_test_structure(&base_path)?;
        Ok(base_path)
    }

    /// 生成大文件测试数据
    pub fn create_large_files(&self, name: &str, count: usize, size_mb: usize) -> Result<PathBuf, std::io::Error> {
        let base_path = self.temp_dir.path().join(name);
        fs::create_dir_all(&base_path)?;
        
        for i in 0..count {
            let file_path = base_path.join(format!("large_file_{}.dat", i));
            create_large_file(&file_path, size_mb * 1024 * 1024)?;
        }
        
        Ok(base_path)
    }

    /// 生成深度嵌套目录
    pub fn create_deep_nested_structure(&self, name: &str, depth: usize) -> Result<PathBuf, std::io::Error> {
        let base_path = self.temp_dir.path().join(name);
        create_deep_nested_dirs(&base_path, depth)?;
        Ok(base_path)
    }

    /// 生成包含特殊字符的文件名
    pub fn create_special_characters(&self, name: &str) -> Result<PathBuf, std::io::Error> {
        let base_path = self.temp_dir.path().join(name);
        fs::create_dir_all(&base_path)?;
        
        let special_names = vec![
            "文件 with spaces.txt",
            "文件-with-dashes.txt",
            "文件_with_underscores.txt",
            "文件.with.dots.txt",
            "文件(with)parentheses.txt",
            "文件[with]brackets.txt",
            "文件{with}braces.txt",
        ];
        
        for filename in special_names {
            let file_path = base_path.join(filename);
            let mut file = File::create(&file_path)?;
            writeln!(file, "Content for {}", filename)?;
        }
        
        Ok(base_path)
    }

    /// 生成权限测试数据
    #[cfg(unix)]
    pub fn create_permission_test_data(&self, name: &str) -> Result<PathBuf, std::io::Error> {
        use std::os::unix::fs::PermissionsExt;
        
        let base_path = self.temp_dir.path().join(name);
        fs::create_dir_all(&base_path)?;
        
        // 创建只读文件
        let readonly_file = base_path.join("readonly.txt");
        File::create(&readonly_file)?;
        let mut perms = fs::metadata(&readonly_file)?.permissions();
        perms.set_mode(0o444); // 只读
        fs::set_permissions(&readonly_file, perms)?;
        
        // 创建无权限目录
        let no_access_dir = base_path.join("no_access");
        fs::create_dir_all(&no_access_dir)?;
        let mut perms = fs::metadata(&no_access_dir)?.permissions();
        perms.set_mode(0o000); // 无权限
        fs::set_permissions(&no_access_dir, perms)?;
        
        Ok(base_path)
    }
}

/// 性能测量工具
pub struct PerformanceMeasurer {
    start_time: Instant,
    name: String,
}

impl PerformanceMeasurer {
    /// 创建新的性能测量器
    pub fn new(name: &str) -> Self {
        info!("开始性能测量: {}", name);
        Self {
            start_time: Instant::now(),
            name: name.to_string(),
        }
    }

    /// 结束测量并返回结果
    pub fn finish(self) -> PerformanceMetrics {
        let duration = self.start_time.elapsed();
        let metrics = PerformanceMetrics {
            name: self.name.clone(),
            duration,
            memory_peak: measure_peak_memory(),
        };
        
        info!("性能测量完成: {} - 耗时: {:?}", self.name, duration);
        metrics
    }
}

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub name: String,
    pub duration: Duration,
    pub memory_peak: usize,
}

/// 内存使用测量
fn measure_peak_memory() -> usize {
    // 简化的内存测量，实际应用中可以使用更精确的内存监控
    // 这里返回一个估算值
    0
}

/// 文件系统操作计时器
pub struct FileSystemTimer {
    operation_count: usize,
    total_duration: Duration,
}

impl FileSystemTimer {
    pub fn new() -> Self {
        Self {
            operation_count: 0,
            total_duration: Duration::ZERO,
        }
    }

    /// 计时文件系统操作
    pub fn time_operation<F, R>(&mut self, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        
        self.operation_count += 1;
        self.total_duration += duration;
        
        result
    }

    /// 获取平均操作时间
    pub fn average_duration(&self) -> Duration {
        if self.operation_count > 0 {
            self.total_duration / self.operation_count as u32
        } else {
            Duration::ZERO
        }
    }

    /// 获取操作统计
    pub fn get_stats(&self) -> FileSystemStats {
        FileSystemStats {
            operation_count: self.operation_count,
            total_duration: self.total_duration,
            average_duration: self.average_duration(),
            operations_per_second: if self.total_duration.as_secs_f64() > 0.0 {
                self.operation_count as f64 / self.total_duration.as_secs_f64()
            } else {
                0.0
            },
        }
    }
}

/// 文件系统操作统计
#[derive(Debug, Clone)]
pub struct FileSystemStats {
    pub operation_count: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub operations_per_second: f64,
}

/// 测试断言工具
pub struct TestAsserter;

impl TestAsserter {
    /// 断言目录存在且包含指定数量的文件
    pub fn assert_directory_contents(
        dir_path: &Path,
        expected_file_count: usize,
        expected_subdir_count: usize,
    ) -> Result<(), crate::tests::TestError> {
        if !dir_path.exists() {
            return Err(crate::tests::TestError::AssertionFailed(format!(
                "目录不存在: {}", dir_path.display()
            )));
        }

        let mut file_count = 0;
        let mut subdir_count = 0;

        for entry in fs::read_dir(dir_path)
            .map_err(|e| crate::tests::TestError::AssertionFailed(format!("读取目录失败: {}", e)))?
        {
            let entry = entry
                .map_err(|e| crate::tests::TestError::AssertionFailed(format!("读取目录项失败: {}", e)))?;
            
            if entry.path().is_file() {
                file_count += 1;
            } else if entry.path().is_dir() {
                subdir_count += 1;
            }
        }

        if file_count != expected_file_count {
            return Err(crate::tests::TestError::AssertionFailed(format!(
                "文件数量不匹配: 期望 {}, 实际 {}", expected_file_count, file_count
            )));
        }

        if subdir_count != expected_subdir_count {
            return Err(crate::tests::TestError::AssertionFailed(format!(
                "子目录数量不匹配: 期望 {}, 实际 {}", expected_subdir_count, subdir_count
            )));
        }

        Ok(())
    }

    /// 断言文件内容匹配
    pub fn assert_file_content(
        file_path: &Path,
        expected_content: &str,
    ) -> Result<(), crate::tests::TestError> {
        if !file_path.exists() {
            return Err(crate::tests::TestError::AssertionFailed(format!(
                "文件不存在: {}", file_path.display()
            )));
        }

        let actual_content = fs::read_to_string(file_path)
            .map_err(|e| crate::tests::TestError::AssertionFailed(format!("读取文件失败: {}", e)))?;

        if actual_content.trim() != expected_content.trim() {
            return Err(crate::tests::TestError::AssertionFailed(format!(
                "文件内容不匹配:\n期望: {}\n实际: {}", expected_content, actual_content
            )));
        }

        Ok(())
    }

    /// 断言文件大小在预期范围内
    pub fn assert_file_size_range(
        file_path: &Path,
        min_size: u64,
        max_size: u64,
    ) -> Result<(), crate::tests::TestError> {
        if !file_path.exists() {
            return Err(crate::tests::TestError::AssertionFailed(format!(
                "文件不存在: {}", file_path.display()
            )));
        }

        let metadata = fs::metadata(file_path)
            .map_err(|e| crate::tests::TestError::AssertionFailed(format!("获取文件元数据失败: {}", e)))?;
        
        let file_size = metadata.len();

        if file_size < min_size {
            return Err(crate::tests::TestError::AssertionFailed(format!(
                "文件大小太小: {} < {}", file_size, min_size
            )));
        }

        if file_size > max_size {
            return Err(crate::tests::TestError::AssertionFailed(format!(
                "文件大小太大: {} > {}", file_size, max_size
            )));
        }

        Ok(())
    }

    /// 断言操作在指定时间内完成
    pub fn assert_operation_timeout<F>(
        operation: F,
        timeout: Duration,
        operation_name: &str,
    ) -> Result<Duration, crate::tests::TestError>
    where
        F: FnOnce() -> Result<(), crate::tests::TestError>,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();

        if duration > timeout {
            return Err(crate::tests::TestError::AssertionFailed(format!(
                "{} 操作超时: {:?} > {:?}", operation_name, duration, timeout
            )));
        }

        if let Err(e) = result {
            return Err(crate::tests::TestError::AssertionFailed(format!(
                "{} 操作失败: {}", operation_name, e
            )));
        }

        Ok(duration)
    }
}

/// 随机数据生成器
pub struct RandomDataGenerator {
    seed: u64,
}

impl RandomDataGenerator {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// 生成随机字符串
    pub fn random_string(&mut self, length: usize) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        self.seed.hash(&mut hasher);
        let hash = hasher.finish();
        
        self.seed = hash;
        
        let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut result = String::with_capacity(length);
        
        for i in 0..length {
            let idx = (hash.wrapping_add(i as u64)) as usize % charset.len();
            result.push(charset[idx] as char);
        }
        
        result
    }

    /// 生成随机文件大小
    pub fn random_file_size(&mut self, min_kb: usize, max_kb: usize) -> usize {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        self.seed.hash(&mut hasher);
        let hash = hasher.finish();
        
        self.seed = hash;
        
        let range = max_kb - min_kb;
        let random_offset = (hash as usize) % range;
        (min_kb + random_offset) * 1024
    }
}

/// 标准测试目录结构创建函数
pub fn create_standard_test_structure(base_path: &Path) -> Result<(), std::io::Error> {
    // 创建基础目录
    fs::create_dir_all(base_path)?;

    // 创建文件
    let file1 = base_path.join("file1.txt");
    let mut f1 = File::create(&file1)?;
    writeln!(f1, "测试内容1")?;

    let file2 = base_path.join("file2.txt");
    let mut f2 = File::create(&file2)?;
    writeln!(f2, "测试内容2")?;

    // 创建子目录
    let subdir = base_path.join("subdir");
    fs::create_dir_all(&subdir)?;

    let subfile = subdir.join("subfile.txt");
    let mut sf = File::create(&subfile)?;
    writeln!(sf, "子目录测试内容")?;

    Ok(())
}

/// 创建大文件
pub fn create_large_file(file_path: &Path, size_bytes: usize) -> Result<(), std::io::Error> {
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(file_path)?;
    
    // 分块写入以避免内存问题
    let chunk_size = 1024 * 1024; // 1MB chunks
    let chunk = vec![b'A'; chunk_size.min(size_bytes)];
    
    let chunks_needed = size_bytes / chunk_size;
    let remainder = size_bytes % chunk_size;
    
    for _ in 0..chunks_needed {
        file.write_all(&chunk)?;
    }
    
    if remainder > 0 {
        let remainder_chunk = vec![b'A'; remainder];
        file.write_all(&remainder_chunk)?;
    }
    
    Ok(())
}

/// 创建深度嵌套目录
pub fn create_deep_nested_dirs(base_path: &Path, depth: usize) -> Result<(), std::io::Error> {
    let mut current_path = base_path.to_path_buf();
    
    for i in 0..depth {
        current_path = current_path.join(format!("level_{}", i));
        fs::create_dir_all(&current_path)?;
        
        // 在每个层级创建文件
        let file_path = current_path.join(format!("file_at_level_{}.txt", i));
        let mut file = File::create(&file_path)?;
        writeln!(file, "这是第 {} 层的文件", i)?;
    }
    
    Ok(())
}

/// 测试清理工具
pub struct TestCleanup;

impl TestCleanup {
    /// 安全删除测试目录
    pub fn cleanup_directory(dir_path: &Path) -> Result<(), std::io::Error> {
        if dir_path.exists() {
            fs::remove_dir_all(dir_path)?;
        }
        Ok(())
    }

    /// 清理临时文件
    pub fn cleanup_temp_files(pattern: &str) -> Result<(), std::io::Error> {
        let temp_dir = std::env::temp_dir();
        
        for entry in fs::read_dir(&temp_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            
            if let Some(name_str) = file_name.to_str() {
                if name_str.contains(pattern) {
                    if entry.path().is_dir() {
                        fs::remove_dir_all(entry.path())?;
                    } else {
                        fs::remove_file(entry.path())?;
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// 基准测试比较器
pub struct BenchmarkComparator {
    baseline_results: Vec<PerformanceResult>,
}

#[derive(Debug, Clone)]
pub struct PerformanceResult {
    pub test_name: String,
    pub duration_ms: u64,
    pub memory_used_mb: f64,
    pub file_count: usize,
    pub total_size_mb: f64,
}

impl BenchmarkComparator {
    pub fn new(baseline_results: Vec<PerformanceResult>) -> Self {
        Self { baseline_results }
    }

    /// 比较当前结果与基线
    pub fn compare(&self, current_result: &PerformanceResult) -> BenchmarkComparison {
        if let Some(baseline) = self.baseline_results.iter().find(|r| r.test_name == current_result.test_name) {
            let duration_change = ((current_result.duration_ms as f64 - baseline.duration_ms as f64) / baseline.duration_ms as f64) * 100.0;
            let memory_change = ((current_result.memory_used_mb - baseline.memory_used_mb) / baseline.memory_used_mb) * 100.0;
            
            BenchmarkComparison {
                test_name: current_result.test_name.clone(),
                duration_change_percent: duration_change,
                memory_change_percent: memory_change,
                performance_regression: duration_change > 10.0 || memory_change > 10.0,
                improvement: duration_change < -5.0 && memory_change < -5.0,
            }
        } else {
            BenchmarkComparison {
                test_name: current_result.test_name.clone(),
                duration_change_percent: 0.0,
                memory_change_percent: 0.0,
                performance_regression: false,
                improvement: false,
            }
        }
    }
}

#[derive(Debug)]
pub struct BenchmarkComparison {
    pub test_name: String,
    pub duration_change_percent: f64,
    pub memory_change_percent: f64,
    pub performance_regression: bool,
    pub improvement: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_data_generator() {
        let generator = TestDataGenerator::new().unwrap();
        let test_dir = generator.create_standard_structure("test").unwrap();
        
        assert!(test_dir.exists());
        assert!(test_dir.join("file1.txt").exists());
        assert!(test_dir.join("subdir").exists());
    }

    #[test]
    fn test_performance_measurer() {
        let measurer = PerformanceMeasurer::new("test_operation");
        std::thread::sleep(Duration::from_millis(100));
        let metrics = measurer.finish();
        
        assert!(metrics.duration >= Duration::from_millis(100));
    }

    #[test]
    fn test_file_system_timer() {
        let mut timer = FileSystemTimer::new();
        
        timer.time_operation(|| {
            let _temp_dir = TempDir::new().unwrap();
        });
        
        let stats = timer.get_stats();
        assert!(stats.operation_count > 0);
        assert!(stats.average_duration > Duration::ZERO);
    }

    #[test]
    fn test_random_data_generator() {
        let mut generator = RandomDataGenerator::new(12345);
        let str1 = generator.random_string(10);
        let str2 = generator.random_string(10);
        
        assert_eq!(str1.len(), 10);
        assert_eq!(str2.len(), 10);
        // 使用相同种子应该生成相同的结果
        assert_eq!(str1, str2);
    }

    #[test]
    fn test_benchmark_comparator() {
        let baseline = vec![
            PerformanceResult {
                test_name: "test1".to_string(),
                duration_ms: 1000,
                memory_used_mb: 100.0,
                file_count: 100,
                total_size_mb: 10.0,
            }
        ];
        
        let comparator = BenchmarkComparator::new(baseline);
        let current = PerformanceResult {
            test_name: "test1".to_string(),
            duration_ms: 1100,
            memory_used_mb: 105.0,
            file_count: 100,
            total_size_mb: 10.0,
        };
        
        let comparison = comparator.compare(&current);
        assert!(comparison.duration_change_percent > 0.0);
        assert!(comparison.performance_regression);
    }
}