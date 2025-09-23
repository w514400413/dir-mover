//! 性能测试模块
//! 
//! 测试系统在各种负载下的性能表现

use crate::disk_analyzer::DiskAnalyzer;
use crate::file_operations::FileOperator;
use crate::migration_service::{MigrationService, MigrationOptions};
use tempfile::TempDir;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use log::info;
use sysinfo::{System, SystemExt, ProcessExt};

/// 磁盘扫描性能测试
pub async fn test_disk_scan_performance() -> Result<(), crate::tests::TestError> {
    info!("开始磁盘扫描性能测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 创建不同规模的测试目录
    let test_cases = vec![
        ("Small", 100, 10),      // 100个文件，10个子目录
        ("Medium", 1000, 50),    // 1000个文件，50个子目录
        ("Large", 5000, 100),    // 5000个文件，100个子目录
    ];

    let mut system = System::new_all();
    let mut results = Vec::new();

    for (size_name, file_count, subdir_count) in test_cases {
        info!("测试 {} 规模目录扫描性能...", size_name);
        
        let test_dir = temp_dir.path().join(format!("ScanPerf{}", size_name));
        create_performance_test_structure(&test_dir, file_count, subdir_count)?;

        // 监控系统资源
        system.refresh_all();
        let memory_before = system.used_memory();
        let cpu_before = system.global_cpu_info().cpu_usage();

        // 执行扫描
        let analyzer = DiskAnalyzer::new();
        let scan_start = Instant::now();
        
        let result = analyzer.scan_directory_async(&test_dir).await
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("扫描失败: {}", e)))?;
        
        let scan_duration = scan_start.elapsed();

        // 监控系统资源
        system.refresh_all();
        let memory_after = system.used_memory();
        let cpu_after = system.global_cpu_info().cpu_usage();
        
        let memory_used = memory_after.saturating_sub(memory_before);
        let cpu_usage = cpu_after - cpu_before;

        // 计算性能指标
        let scan_rate = result.file_count as f64 / scan_duration.as_secs_f64();
        let throughput = result.size as f64 / scan_duration.as_secs_f64() / (1024.0 * 1024.0); // MB/s

        let perf_result = PerformanceResult {
            test_name: format!("DiskScan_{}", size_name),
            file_count: result.file_count,
            total_size: result.size,
            duration: scan_duration,
            scan_rate,
            throughput_mbps: throughput,
            memory_used_mb: memory_used as f64 / (1024.0 * 1024.0),
            cpu_usage_percent: cpu_usage,
        };

        results.push(perf_result);

        info!("{} 扫描性能: {:.0} 文件/秒, {:.2} MB/秒, 内存使用: {:.1} MB, CPU: {:.1}%", 
              size_name, scan_rate, throughput, memory_used as f64 / (1024.0 * 1024.0), cpu_usage);
    }

    // 性能基准验证
    validate_scan_performance_baseline(&results)?;

    info!("磁盘扫描性能测试完成");
    Ok(())
}

/// 文件迁移性能测试
pub async fn test_migration_performance() -> Result<(), crate::tests::TestError> {
    info!("开始文件迁移性能测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 创建不同大小的测试数据
    let test_cases = vec![
        ("SmallFiles", 100, 1024),      // 100个1KB文件
        ("MediumFiles", 50, 10240),     // 50个10KB文件
        ("LargeFiles", 10, 102400),     // 10个100KB文件
        ("MixedFiles", 200, 5120),      // 200个5KB混合文件
    ];

    let mut system = System::new_all();
    let mut results = Vec::new();

    for (test_name, file_count, file_size) in test_cases {
        info!("测试 {} 迁移性能...", test_name);
        
        let source_dir = temp_dir.path().join(format!("MigrateSource{}", test_name));
        let target_dir = temp_dir.path().join(format!("MigrateTarget{}", test_name));
        
        // 创建测试文件
        create_files_with_size(&source_dir, file_count, file_size)?;

        // 监控系统资源
        system.refresh_all();
        let memory_before = system.used_memory();
        let cpu_before = system.global_cpu_info().cpu_usage();

        // 执行迁移
        let service = MigrationService::new();
        let migrate_start = Instant::now();
        
        let options = MigrationOptions {
            source_path: source_dir.display().to_string(),
            target_path: target_dir.display().to_string(),
            create_symlink: false,
            delete_source: false,
        };

        let result = service.migrate_folder(options).await
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("迁移失败: {}", e)))?;
        
        let migrate_duration = migrate_start.elapsed();

        // 监控系统资源
        system.refresh_all();
        let memory_after = system.used_memory();
        let cpu_after = system.global_cpu_info().cpu_usage();
        
        let memory_used = memory_after.saturating_sub(memory_before);
        let cpu_usage = cpu_after - cpu_before;

        // 计算性能指标
        let total_size = (file_count * file_size) as u64;
        let migrate_rate = file_count as f64 / migrate_duration.as_secs_f64();
        let throughput = total_size as f64 / migrate_duration.as_secs_f64() / (1024.0 * 1024.0); // MB/s

        let perf_result = PerformanceResult {
            test_name: format!("Migration_{}", test_name),
            file_count,
            total_size,
            duration: migrate_duration,
            scan_rate: migrate_rate,
            throughput_mbps: throughput,
            memory_used_mb: memory_used as f64 / (1024.0 * 1024.0),
            cpu_usage_percent: cpu_usage,
        };

        results.push(perf_result);

        info!("{} 迁移性能: {:.0} 文件/秒, {:.2} MB/秒, 内存使用: {:.1} MB, CPU: {:.1}%", 
              test_name, migrate_rate, throughput, memory_used as f64 / (1024.0 * 1024.0), cpu_usage);
    }

    // 性能基准验证
    validate_migration_performance_baseline(&results)?;

    info!("文件迁移性能测试完成");
    Ok(())
}

/// 内存使用测试
pub async fn test_memory_usage() -> Result<(), crate::tests::TestError> {
    info!("开始内存使用测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 创建大目录结构
    let test_dir = temp_dir.path().join("MemoryTest");
    create_large_folder_structure(&test_dir, 5000, 100)?; // 5000个文件，100个子目录

    let mut system = System::new_all();
    
    // 测试1: 扫描内存使用
    info!("测试扫描内存使用...");
    system.refresh_all();
    let memory_before_scan = system.used_memory();

    let analyzer = DiskAnalyzer::new();
    let _scan_result = analyzer.scan_directory_async(&test_dir).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("扫描失败: {}", e)))?;

    system.refresh_all();
    let memory_after_scan = system.used_memory();
    let scan_memory_used = memory_after_scan.saturating_sub(memory_before_scan);

    info!("扫描内存使用: {:.1} MB", scan_memory_used as f64 / (1024.0 * 1024.0));

    // 测试2: 迁移内存使用
    let target_dir = temp_dir.path().join("MemoryTarget");
    
    info!("测试迁移内存使用...");
    system.refresh_all();
    let memory_before_migrate = system.used_memory();

    let service = MigrationService::new();
    let options = MigrationOptions {
        source_path: test_dir.display().to_string(),
        target_path: target_dir.display().to_string(),
        create_symlink: false,
        delete_source: false,
    };

    let _migrate_result = service.migrate_folder(options).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("迁移失败: {}", e)))?;

    system.refresh_all();
    let memory_after_migrate = system.used_memory();
    let migrate_memory_used = memory_after_migrate.saturating_sub(memory_before_migrate);

    info!("迁移内存使用: {:.1} MB", migrate_memory_used as f64 / (1024.0 * 1024.0));

    // 测试3: 内存泄漏检测（多次操作）
    info!("测试内存泄漏...");
    
    let mut memory_usage_history = Vec::new();
    
    for i in 0..5 {
        system.refresh_all();
        let memory_before = system.used_memory();
        
        // 执行一次完整操作
        let temp_subdir = temp_dir.path().join(format!("leak_test_{}", i));
        create_test_directory_structure(&temp_subdir)?;
        
        let temp_target = temp_dir.path().join(format!("leak_target_{}", i));
        let service = MigrationService::new();
        
        let options = MigrationOptions {
            source_path: temp_subdir.display().to_string(),
            target_path: temp_target.display().to_string(),
            create_symlink: false,
            delete_source: true,
        };
        
        service.migrate_folder(options).await
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("迁移失败: {}", e)))?;
        
        system.refresh_all();
        let memory_after = system.used_memory();
        let memory_used = memory_after.saturating_sub(memory_before);
        
        memory_usage_history.push(memory_used);
        info!("第 {} 次操作内存使用: {:.1} MB", i + 1, memory_used as f64 / (1024.0 * 1024.0));
    }

    // 分析内存使用趋势
    let avg_memory = memory_usage_history.iter().sum::<u64>() as f64 / memory_usage_history.len() as f64;
    let max_memory = *memory_usage_history.iter().max().unwrap_or(&0);
    
    info!("平均内存使用: {:.1} MB", avg_memory as f64 / (1024.0 * 1024.0));
    info!("最大内存使用: {:.1} MB", max_memory as f64 / (1024.0 * 1024.0));

    // 验证内存使用在合理范围内
    assert!(scan_memory_used < 100 * 1024 * 1024, "扫描内存使用应该小于100MB"); // 100MB
    assert!(migrate_memory_used < 200 * 1024 * 1024, "迁移内存使用应该小于200MB"); // 200MB

    info!("内存使用测试完成");
    Ok(())
}

/// 大文件处理性能测试
pub async fn test_large_file_performance() -> Result<(), crate::tests::TestError> {
    info!("开始大文件处理性能测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 创建不同大小的大文件
    let test_cases = vec![
        ("10MB", 10 * 1024 * 1024),
        ("50MB", 50 * 1024 * 1024),
        ("100MB", 100 * 1024 * 1024),
    ];

    let mut system = System::new_all();
    let mut results = Vec::new();

    for (test_name, file_size) in test_cases {
        info!("测试 {} 文件处理性能...", test_name);
        
        let source_dir = temp_dir.path().join(format!("LargeFileSource{}", test_name));
        let target_dir = temp_dir.path().join(format!("LargeFileTarget{}", test_name));
        
        // 创建大文件
        create_large_file(&source_dir, "large_file.dat", file_size)?;

        // 监控系统资源
        system.refresh_all();
        let memory_before = system.used_memory();
        let cpu_before = system.global_cpu_info().cpu_usage();

        // 执行迁移
        let service = MigrationService::new();
        let migrate_start = Instant::now();
        
        let options = MigrationOptions {
            source_path: source_dir.display().to_string(),
            target_path: target_dir.display().to_string(),
            create_symlink: false,
            delete_source: false,
        };

        let result = service.migrate_folder(options).await
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("迁移失败: {}", e)))?;
        
        let migrate_duration = migrate_start.elapsed();

        // 监控系统资源
        system.refresh_all();
        let memory_after = system.used_memory();
        let cpu_after = system.global_cpu_info().cpu_usage();
        
        let memory_used = memory_after.saturating_sub(memory_before);
        let cpu_usage = cpu_after - cpu_before;

        // 计算性能指标
        let throughput = file_size as f64 / migrate_duration.as_secs_f64() / (1024.0 * 1024.0); // MB/s

        let perf_result = PerformanceResult {
            test_name: format!("LargeFile_{}", test_name),
            file_count: 1,
            total_size: file_size as u64,
            duration: migrate_duration,
            scan_rate: 1.0 / migrate_duration.as_secs_f64(),
            throughput_mbps: throughput,
            memory_used_mb: memory_used as f64 / (1024.0 * 1024.0),
            cpu_usage_percent: cpu_usage,
        };

        results.push(perf_result);

        info!("{} 大文件迁移性能: {:.2} MB/秒, 耗时: {:?}, 内存使用: {:.1} MB", 
              test_name, throughput, migrate_duration, memory_used as f64 / (1024.0 * 1024.0));
    }

    // 验证大文件处理性能
    for result in &results {
        assert!(result.throughput_mbps > 10.0, "大文件迁移速率应该大于10MB/s");
        assert!(result.duration < Duration::from_secs(30), "大文件迁移应该在30秒内完成");
    }

    info!("大文件处理性能测试完成");
    Ok(())
}

/// 并发性能测试
pub async fn test_concurrent_performance() -> Result<(), crate::tests::TestError> {
    info!("开始并发性能测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 创建多个测试目录
    let dir_count = 10;
    let mut test_dirs = Vec::new();
    
    for i in 0..dir_count {
        let test_dir = temp_dir.path().join(format!("ConcurrentPerf{}", i));
        create_test_directory_structure(&test_dir)?;
        test_dirs.push(test_dir);
    }

    let mut system = System::new_all();

    // 测试1: 并发扫描性能
    info!("测试并发扫描性能...");
    system.refresh_all();
    let memory_before = system.used_memory();

    let analyzer = DiskAnalyzer::new();
    let scan_start = Instant::now();
    
    let mut scan_handles = vec![];
    for dir in &test_dirs {
        let analyzer_clone = analyzer.clone();
        let dir_clone = dir.clone();
        
        let handle = tokio::spawn(async move {
            analyzer_clone.scan_directory_async(&dir_clone).await
        });
        
        scan_handles.push(handle);
    }
    
    // 等待所有扫描完成
    let mut scan_results = vec![];
    for handle in scan_handles {
        let result = handle.await
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("并发扫描任务失败: {}", e)))?
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("扫描失败: {}", e)))?;
        
        scan_results.push(result);
    }
    
    let concurrent_scan_duration = scan_start.elapsed();

    system.refresh_all();
    let memory_after = system.used_memory();
    let concurrent_memory_used = memory_after.saturating_sub(memory_before);

    // 测试2: 并发迁移性能
    info!("测试并发迁移性能...");
    system.refresh_all();
    let memory_before_migrate = system.used_memory();

    let service = MigrationService::new();
    let migrate_start = Instant::now();
    
    let mut migrate_handles = vec![];
    for (i, source_dir) in test_dirs.iter().enumerate() {
        let service_clone = service.clone();
        let source_clone = source_dir.clone();
        let target_clone = temp_dir.path().join(format!("ConcurrentTarget{}", i));
        
        let handle = tokio::spawn(async move {
            let options = MigrationOptions {
                source_path: source_clone.display().to_string(),
                target_path: target_clone.display().to_string(),
                create_symlink: false,
                delete_source: false,
            };
            
            service_clone.migrate_folder(options).await
        });
        
        migrate_handles.push(handle);
    }
    
    // 等待所有迁移完成
    let mut migrate_results = vec![];
    for handle in migrate_handles {
        let result = handle.await
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("并发迁移任务失败: {}", e)))?
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("迁移失败: {}", e)))?;
        
        migrate_results.push(result);
    }
    
    let concurrent_migrate_duration = migrate_start.elapsed();

    system.refresh_all();
    let memory_after_migrate = system.used_memory();
    let concurrent_migrate_memory_used = memory_after_migrate.saturating_sub(memory_before_migrate);

    // 性能分析
    let avg_scan_time = concurrent_scan_duration / dir_count as u32;
    let avg_migrate_time = concurrent_migrate_duration / dir_count as u32;
    
    info!("并发性能分析:");
    info!("  并发扫描总耗时: {:?}", concurrent_scan_duration);
    info!("  并发迁移总耗时: {:?}", concurrent_migrate_duration);
    info!("  平均扫描时间: {:?}", avg_scan_time);
    info!("  平均迁移时间: {:?}", avg_migrate_time);
    info!("  并发扫描内存使用: {:.1} MB", concurrent_memory_used as f64 / (1024.0 * 1024.0));
    info!("  并发迁移内存使用: {:.1} MB", concurrent_migrate_memory_used as f64 / (1024.0 * 1024.0));

    // 验证并发性能
    assert!(avg_scan_time < Duration::from_secs(2), "平均扫描时间应该在2秒内");
    assert!(avg_migrate_time < Duration::from_secs(3), "平均迁移时间应该在3秒内");
    assert!(concurrent_memory_used < 500 * 1024 * 1024, "并发内存使用应该小于500MB");

    // 测试3: 并发 vs 串行性能对比
    info!("测试串行性能用于对比...");
    
    let serial_start = Instant::now();
    for dir in &test_dirs {
        let target_clone = temp_dir.path().join(format!("SerialTarget{}", test_dirs.iter().position(|d| d == dir).unwrap()));
        let options = MigrationOptions {
            source_path: dir.display().to_string(),
            target_path: target_clone.display().to_string(),
            create_symlink: false,
            delete_source: false,
        };
        
        service.migrate_folder(options).await
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("串行迁移失败: {}", e)))?;
    }
    let serial_duration = serial_start.elapsed();

    info!("性能对比:");
    info!("  并发总耗时: {:?}", concurrent_migrate_duration);
    info!("  串行总耗时: {:?}", serial_duration);
    info!("  并发加速比: {:.2}x", serial_duration.as_secs_f64() / concurrent_migrate_duration.as_secs_f64());

    // 验证并发有性能提升
    assert!(concurrent_migrate_duration < serial_duration, "并发应该比串行更快");

    info!("并发性能测试完成");
    Ok(())
}

/// 创建性能测试文件结构
fn create_performance_test_structure(
    base_path: &std::path::Path, 
    file_count: usize, 
    subdir_count: usize
) -> Result<(), crate::tests::TestError> {
    // 创建子目录
    for i in 0..subdir_count {
        let subdir = base_path.join(format!("perf_subdir_{}", i));
        fs::create_dir_all(&subdir)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建性能测试子目录 {} 失败: {}", i, e)))?;
        
        // 在每个子目录中创建文件
        let files_per_subdir = file_count / subdir_count;
        for j in 0..files_per_subdir {
            let file_path = subdir.join(format!("perf_file_{}_{}.txt", i, j));
            let mut file = File::create(&file_path)
                .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建性能测试文件 {}_{} 失败: {}", i, j, e)))?;
            
            // 写入一些内容以模拟真实文件
            writeln!(file, "性能测试文件 {}_{} 的内容\n这是第二行内容\n这是第三行内容", i, j)
                .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入性能测试文件 {}_{} 失败: {}", i, j, e)))?;
        }
    }

    // 在根目录也创建一些文件
    let root_files = file_count % subdir_count;
    for i in 0..root_files {
        let file_path = base_path.join(format!("perf_root_file_{}.txt", i));
        let mut file = File::create(&file_path)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建性能测试根文件 {} 失败: {}", i, e)))?;
        
        writeln!(file, "性能测试根文件 {} 的内容", i)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入性能测试根文件 {} 失败: {}", i, e)))?;
    }

    Ok(())
}

/// 创建指定大小的文件
fn create_large_file(dir: &std::path::Path, filename: &str, size: usize) -> Result<(), crate::tests::TestError> {
    fs::create_dir_all(dir)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建目录失败: {}", e)))?;
    
    let file_path = dir.join(filename);
    let mut file = File::create(&file_path)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建大文件 {} 失败: {}", filename, e)))?;
    
    // 写入指定大小的内容
    let chunk_size = 1024; // 1KB chunks
    let chunk = vec![b'A'; chunk_size];
    let chunks_needed = size / chunk_size;
    let remainder = size % chunk_size;
    
    for _ in 0..chunks_needed {
        file.write_all(&chunk)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入大文件块失败: {}", e)))?;
    }
    
    if remainder > 0 {
        let remainder_chunk = vec![b'A'; remainder];
        file.write_all(&remainder_chunk)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入大文件剩余部分失败: {}", e)))?;
    }
    
    Ok(())
}

/// 创建指定数量和大小的文件
fn create_files_with_size(dir: &std::path::Path, count: usize, size: usize) -> Result<(), crate::tests::TestError> {
    fs::create_dir_all(dir)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建目录失败: {}", e)))?;
    
    for i in 0..count {
        let file_path = dir.join(format!("file_{}.txt", i));
        let mut file = File::create(&file_path)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建文件 {} 失败: {}", i, e)))?;
        
        // 写入指定大小的内容
        let content = format!("文件 {} 的内容\n", i);
        let content_bytes = content.as_bytes();
        let repetitions = size / content_bytes.len() + 1;
        
        let full_content = content.repeat(repetitions);
        let write_size = size.min(full_content.len());
        
        file.write_all(&full_content.as_bytes()[..write_size])
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入文件 {} 失败: {}", i, e)))?;
    }
    
    Ok(())
}

/// 性能测试结果
#[derive(Debug)]
struct PerformanceResult {
    test_name: String,
    file_count: usize,
    total_size: u64,
    duration: Duration,
    scan_rate: f64, // files per second
    throughput_mbps: f64, // MB per second
    memory_used_mb: f64,
    cpu_usage_percent: f32,
}

/// 验证扫描性能基线
fn validate_scan_performance_baseline(results: &[PerformanceResult]) -> Result<(), crate::tests::TestError> {
    for result in results {
        // 基础性能要求
        assert!(result.scan_rate > 5.0, "{}: 扫描速率应该大于5文件/秒", result.test_name);
        assert!(result.throughput_mbps > 1.0, "{}: 吞吐量应该大于1MB/秒", result.test_name);
        assert!(result.duration < Duration::from_secs(60), "{}: 扫描时间应该小于60秒", result.test_name);
        assert!(result.memory_used_mb < 500.0, "{}: 内存使用应该小于500MB", result.test_name);
        
        info!("{} 性能基线验证通过", result.test_name);
    }
    
    Ok(())
}

/// 验证迁移性能基线
fn validate_migration_performance_baseline(results: &[PerformanceResult]) -> Result<(), crate::tests::TestError> {
    for result in results {
        // 基础性能要求
        assert!(result.scan_rate > 2.0, "{}: 迁移速率应该大于2文件/秒", result.test_name);
        assert!(result.throughput_mbps > 0.5, "{}: 迁移吞吐量应该大于0.5MB/秒", result.test_name);
        assert!(result.duration < Duration::from_secs(120), "{}: 迁移时间应该小于120秒", result.test_name);
        assert!(result.memory_used_mb < 1000.0, "{}: 迁移内存使用应该小于1000MB", result.test_name);
        
        info!("{} 迁移性能基线验证通过", result.test_name);
    }
    
    Ok(())
}

/// 创建测试目录结构（复用unit_tests中的函数）
fn create_test_directory_structure(base_path: &std::path::Path) -> Result<(), crate::tests::TestError> {
    // 创建基础目录
    fs::create_dir_all(base_path)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建基础目录失败: {}", e)))?;

    // 创建文件
    let file1 = base_path.join("file1.txt");
    let mut f1 = File::create(&file1)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建文件1失败: {}", e)))?;
    writeln!(f1, "测试内容1")
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入文件1失败: {}", e)))?;

    let file2 = base_path.join("file2.txt");
    let mut f2 = File::create(&file2)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建文件2失败: {}", e)))?;
    writeln!(f2, "测试内容2")
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入文件2失败: {}", e)))?;

    // 创建子目录
    let subdir = base_path.join("subdir");
    fs::create_dir_all(&subdir)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建子目录失败: {}", e)))?;

    let subfile = subdir.join("subfile.txt");
    let mut sf = File::create(&subfile)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建子文件失败: {}", e)))?;
    writeln!(sf, "子目录测试内容")
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入子文件失败: {}", e)))?;

    Ok(())
}

/// 创建大型文件夹结构（复用e2e_tests中的函数）
fn create_large_folder_structure(
    base_path: &std::path::Path, 
    file_count: usize, 
    subdir_count: usize
) -> Result<(), crate::tests::TestError> {
    // 创建子目录
    for i in 0..subdir_count {
        let subdir = base_path.join(format!("subdir_{}", i));
        fs::create_dir_all(&subdir)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建子目录 {} 失败: {}", i, e)))?;
        
        // 在每个子目录中创建文件
        let files_per_subdir = file_count / subdir_count;
        for j in 0..files_per_subdir {
            let file_path = subdir.join(format!("file_{}_{}.txt", i, j));
            let mut file = File::create(&file_path)
                .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建文件 {}_{} 失败: {}", i, j, e)))?;
            
            writeln!(file, "这是子目录 {} 中的文件 {} 的内容", i, j)
                .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入文件 {}_{} 失败: {}", i, j, e)))?;
        }
    }

    // 在根目录也创建一些文件
    let root_files = file_count % subdir_count;
    for i in 0..root_files {
        let file_path = base_path.join(format!("root_file_{}.txt", i));
        let mut file = File::create(&file_path)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建根文件 {} 失败: {}", i, e)))?;
        
        writeln!(file, "这是根目录中的文件 {} 的内容", i)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入根文件 {} 失败: {}", i, e)))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_tests_runner() {
        // 这个测试确保所有的性能测试函数都能正常运行
        let result = test_disk_scan_performance().await;
        assert!(result.is_ok(), "磁盘扫描性能测试应该通过");
        
        let result = test_migration_performance().await;
        assert!(result.is_ok(), "文件迁移性能测试应该通过");
        
        let result = test_memory_usage().await;
        assert!(result.is_ok(), "内存使用测试应该通过");
        
        let result = test_large_file_performance().await;
        assert!(result.is_ok(), "大文件处理性能测试应该通过");
        
        let result = test_concurrent_performance().await;
        assert!(result.is_ok(), "并发性能测试应该通过");
    }
}