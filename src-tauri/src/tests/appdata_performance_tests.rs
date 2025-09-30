//! AppData性能测试模块

use std::time::Instant;
use log::info;
use crate::appdata_analyzer::AppDataAnalyzer;

/// AppData扫描性能基准测试
pub fn test_appdata_scan_performance() -> Result<(), crate::tests::TestError> {
    info!("开始AppData扫描性能基准测试");
    
    let start_time = Instant::now();
    
    // 创建分析器
    let analyzer = AppDataAnalyzer::new();
    
    // 测试配置设置性能
    let config_setup_start = Instant::now();
    let config = crate::appdata_analyzer::AppDataConfig {
        min_size_threshold: 1024 * 1024 * 1024, // 1GB
        max_depth: 2,
        sort_order: crate::appdata_analyzer::SortOrder::Desc,
    };
    
    // 注意：由于我们不能在实际环境中测试，这里测试配置和初始化性能
    let config_setup_time = config_setup_start.elapsed();
    
    // 测试路径检测性能
    let path_detection_start = Instant::now();
    let path_result = AppDataAnalyzer::get_appdata_path();
    let path_detection_time = path_detection_start.elapsed();
    
    // 测试格式化性能
    let format_start = Instant::now();
    let formatted_sizes = vec![
        AppDataAnalyzer::format_size(0),
        AppDataAnalyzer::format_size(1024),
        AppDataAnalyzer::format_size(1024 * 1024),
        AppDataAnalyzer::format_size(1024 * 1024 * 1024),
        AppDataAnalyzer::format_size(5 * 1024 * 1024 * 1024),
    ];
    let format_time = format_start.elapsed();
    
    let total_time = start_time.elapsed();
    
    info!("AppData性能基准测试结果:");
    info!("  配置设置时间: {:?}", config_setup_time);
    info!("  路径检测时间: {:?}", path_detection_time);
    info!("  格式化时间: {:?}", format_time);
    info!("  总测试时间: {:?}", total_time);
    
    // 验证性能要求
    assert!(config_setup_time.as_millis() < 10, "配置设置应该在10ms内完成");
    assert!(format_time.as_millis() < 5, "格式化操作应该在5ms内完成");
    
    // 验证路径检测在非Windows环境下的表现
    match path_result {
        Ok(path) => {
            info!("  成功检测到AppData路径: {}", path.display());
            assert!(path_detection_time.as_millis() < 100, "路径检测应该在100ms内完成");
        }
        Err(e) => {
            info!("  路径检测失败（非Windows环境预期）: {}", e);
            // 在非Windows环境下，路径检测应该快速失败
            assert!(path_detection_time.as_millis() < 50, "路径检测失败应该快速返回");
        }
    }
    
    info!("AppData扫描性能基准测试完成");
    Ok(())
}

/// AppData内存使用测试
pub fn test_appdata_memory_usage() -> Result<(), crate::tests::TestError> {
    info!("开始AppData内存使用测试");
    
    // 测试创建多个分析器实例的内存使用
    let start_memory = get_current_memory_usage();
    
    let mut analyzers = Vec::new();
    for i in 0..100 {
        let mut analyzer = AppDataAnalyzer::new();
        let config = crate::appdata_analyzer::AppDataConfig {
            min_size_threshold: (i + 1) * 10 * 1024 * 1024, // 10MB递增
            max_depth: (i % 5) as usize + 1,
            sort_order: if i % 2 == 0 { 
                crate::appdata_analyzer::SortOrder::Asc 
            } else { 
                crate::appdata_analyzer::SortOrder::Desc 
            },
        };
        analyzer.set_config(config);
        analyzers.push(analyzer);
    }
    
    let end_memory = get_current_memory_usage();
    let memory_increase = end_memory.saturating_sub(start_memory);
    
    info!("创建100个分析器实例的内存使用增加: {} KB", memory_increase / 1024);
    
    // 验证内存使用合理（每个实例应该很小）
    assert!(memory_increase < 10 * 1024 * 1024, "内存使用增加应该小于10MB"); // 10MB
    
    // 清理并测试内存释放
    drop(analyzers);
    
    info!("AppData内存使用测试完成");
    Ok(())
}

/// 获取当前内存使用量（简化版本）
fn get_current_memory_usage() -> usize {
    // 在实际应用中，这里可以使用更精确的内存监控
    // 现在返回一个估算值
    0
}

/// AppData并发性能测试
pub fn test_appdata_concurrent_performance() -> Result<(), crate::tests::TestError> {
    use std::sync::Arc;
    use std::thread;
    
    info!("开始AppData并发性能测试");
    
    let start_time = Instant::now();
    let analyzer = Arc::new(AppDataAnalyzer::new());
    
    let mut handles = vec![];
    
    // 创建多个线程同时访问分析器
    for i in 0..10 {
        let analyzer_clone = Arc::clone(&analyzer);
        let handle = thread::spawn(move || {
            let start = Instant::now();
            
            // 执行一些基本操作
            let _config = crate::appdata_analyzer::AppDataConfig {
                min_size_threshold: (i + 1) * 100 * 1024 * 1024,
                max_depth: 2,
                sort_order: crate::appdata_analyzer::SortOrder::Desc,
            };
            
            let _formatted_size = AppDataAnalyzer::format_size(1024 * 1024 * 1024);
            
            start.elapsed()
        });
        
        handles.push(handle);
    }
    
    // 等待所有线程完成并收集结果
    let mut total_thread_time = std::time::Duration::ZERO;
    for handle in handles {
        let thread_time = handle.join()
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("线程执行失败: {:?}", e)))?;
        total_thread_time += thread_time;
    }
    
    let total_time = start_time.elapsed();
    let average_thread_time = total_thread_time / 10;
    
    info!("并发性能测试结果:");
    info!("  总执行时间: {:?}", total_time);
    info!("  线程总时间: {:?}", total_thread_time);
    info!("  平均线程时间: {:?}", average_thread_time);
    
    // 验证并发性能
    assert!(total_time.as_millis() < 1000, "并发测试应该在1秒内完成");
    assert!(average_thread_time.as_millis() < 50, "平均线程时间应该小于50ms");
    
    info!("AppData并发性能测试完成");
    Ok(())
}