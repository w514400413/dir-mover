mod disk_analyzer;
mod error_recovery;
mod file_operations;
mod migration_service;
mod operation_logger;
mod performance_optimizer;
mod types;
mod logger;
mod tests;

use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;
use log::{info, error};

use disk_analyzer::{DiskAnalyzer, DirectoryInfo, format_file_size};
use error_recovery::{ErrorRecoveryManager, ErrorRecoveryConfig, RecoveryContext, RecoveryResult, RecoveryStatistics};
use migration_service::{MigrationService, MigrationOptions, MigrationResult, validate_migration_options};
use operation_logger::{OperationLogger, OperationType, OperationStatus, OperationLog, OperationStatistics};
use performance_optimizer::{PerformanceOptimizer, PerformanceConfig, PerformanceStats};
use types::{ScanProgress, PathValidationResult};

/// 应用状态
struct AppState {
    disk_analyzer: Arc<Mutex<DiskAnalyzer>>,
    migration_service: Arc<MigrationService>,
    operation_logger: Arc<Mutex<OperationLogger>>,
    error_recovery_manager: Arc<Mutex<ErrorRecoveryManager>>,
    performance_optimizer: Arc<Mutex<PerformanceOptimizer>>,
}

/// 扫描目录（异步版本）
#[tauri::command]
async fn scan_directory(path: String, c_drive_mode: bool, state: State<'_, AppState>) -> Result<DirectoryInfo, String> {
    info!("收到扫描目录请求: {}, C盘模式: {}", path, c_drive_mode);
    
    let path = Path::new(&path);
    
    if !path.exists() {
        error!("路径不存在: {}", path.display());
        return Err("路径不存在".to_string());
    }

    if !path.is_dir() {
        error!("路径不是目录: {}", path.display());
        return Err("路径不是目录".to_string());
    }

    let analyzer = state.disk_analyzer.clone();
    
    // 在异步任务中配置分析器
    {
        let mut analyzer = analyzer.lock().await;
        analyzer.reset_cancel_flag();
        analyzer.set_max_depth(5); // 增加扫描深度到5层
        analyzer.set_c_drive_mode(c_drive_mode); // 设置C盘专项扫描模式
        if c_drive_mode {
            analyzer.set_large_folder_threshold(1024 * 1024 * 1024); // C盘模式使用1GB作为大文件夹阈值
        } else {
            analyzer.set_large_folder_threshold(100 * 1024 * 1024); // 普通模式使用100MB作为大文件夹阈值
        }
        info!("分析器配置完成，最大深度: 5, C盘模式: {}", c_drive_mode);
    }
    
    info!("开始扫描目录: {}", path.display());
    
    // 使用异步扫描
    let result = {
        let analyzer = analyzer.lock().await;
        analyzer.scan_directory_async(path).await
    };
    
    match &result {
        Ok(info) => {
            info!("扫描完成: {} (文件数: {}, 大小: {}, 大文件夹: {})",
                  path.display(), info.file_count, format_file_size(info.size), info.is_large_folder);
        }
        Err(e) => {
            error!("扫描失败 {}: {}", path.display(), e);
        }
    }
    
    result
}

/// 获取扫描进度
#[tauri::command]
fn get_scan_progress(state: State<'_, AppState>) -> Result<types::ScanProgress, String> {
    let analyzer = state.disk_analyzer.clone();
    let analyzer = analyzer.blocking_lock();
    
    let progress = analyzer.get_scan_progress();
    info!("获取扫描进度: 当前路径: {}, 进度: {:.1}%, 已处理文件: {}, 总文件: {}, 发现大文件夹: {}",
          progress.current_path, progress.progress, progress.processed_files, progress.total_files, progress.large_folders_found);
    
    // 转换磁盘分析器的进度类型到types模块的进度类型
    Ok(types::ScanProgress {
        current_path: progress.current_path,
        processed_files: progress.processed_files,
        total_files: progress.total_files,
        progress: progress.progress,
        processed_directories: progress.processed_directories,
        total_directories: progress.total_directories,
        current_directory: progress.current_directory,
        estimated_time_remaining: progress.estimated_time_remaining,
        scan_speed: progress.scan_speed,
        large_folders_found: progress.large_folders_found,
    })
}

/// 停止扫描
#[tauri::command]
fn stop_scan(state: State<'_, AppState>) -> Result<(), String> {
    let analyzer = state.disk_analyzer.clone();
    let analyzer = analyzer.blocking_lock();
    
    analyzer.cancel_scan();
    info!("扫描取消请求已发送");
    
    Ok(())
}

/// 迁移目录
#[tauri::command]
fn migrate_directory(
    source_path: String,
    target_path: String,
    create_symlink: bool,
    delete_source: bool,
    state: State<'_, AppState>
) -> Result<MigrationResult, String> {
    let options = MigrationOptions {
        source_path: source_path.clone(),
        target_path: target_path.clone(),
        create_symlink,
        delete_source,
    };

    // 验证迁移选项
    validate_migration_options(&options)?;

    let service = &state.migration_service;
    // 使用 block_on 来执行异步操作
    let runtime = tokio::runtime::Handle::current();
    runtime.block_on(service.migrate_folder(options))
}

/// 验证迁移路径
#[tauri::command]
fn validate_migration_path(
    source_path: String,
    target_path: String,
    state: State<'_, AppState>
) -> Result<PathValidationResult, String> {
    let source = Path::new(&source_path);
    let target = Path::new(&target_path);

    let result = state.migration_service.file_operator().validate_migration_path(source, target);
    let result = result.map_err(|e| e.to_string())?;
    
    Ok(PathValidationResult {
        valid: result.0,
        message: result.1,
        suggestions: Vec::new(),
    })
}

/// 获取磁盘信息
#[tauri::command]
fn get_disk_info() -> Result<Vec<types::DiskInfo>, String> {
    // 这里应该实现获取系统磁盘信息的逻辑
    // 暂时返回模拟数据
    Ok(vec![
        types::DiskInfo {
            name: "C:".to_string(),
            total_space: 256 * 1024 * 1024 * 1024, // 256GB
            free_space: 128 * 1024 * 1024 * 1024, // 128GB
            used_space: 128 * 1024 * 1024 * 1024, // 128GB
        }
    ])
}

/// 检查路径是否存在
#[tauri::command]
fn path_exists(path: String) -> Result<bool, String> {
    let path = Path::new(&path);
    Ok(path.exists())
}

/// 格式化文件大小
#[tauri::command]
fn format_size(size: u64) -> Result<String, String> {
    Ok(disk_analyzer::format_file_size(size))
}

/// 获取操作日志
#[tauri::command]
async fn get_operation_logs(limit: u32, state: State<'_, AppState>) -> Result<Vec<operation_logger::OperationLog>, String> {
    let logger = state.operation_logger.clone();
    let logger = logger.lock().await;
    
    match logger.get_recent_logs(limit as usize) {
        Ok(logs) => Ok(logs),
        Err(e) => Err(format!("获取操作日志失败: {}", e))
    }
}

/// 获取操作统计信息
#[tauri::command]
async fn get_operation_statistics(state: State<'_, AppState>) -> Result<OperationStatistics, String> {
    let logger = state.operation_logger.clone();
    let logger = logger.lock().await;
    
    match logger.get_statistics() {
        Ok(stats) => Ok(stats),
        Err(e) => Err(format!("获取操作统计失败: {}", e))
    }
}

/// 导出操作日志
#[tauri::command]
async fn export_operation_logs(output_path: String, state: State<'_, AppState>) -> Result<bool, String> {
    let logger = state.operation_logger.clone();
    let logger = logger.lock().await;
    
    match logger.get_recent_logs(10000) {
        Ok(logs) => {
            let output_path = std::path::PathBuf::from(output_path);
            match operation_logger::export_logs_to_csv(&logs, &output_path) {
                Ok(_) => Ok(true),
                Err(e) => Err(format!("导出日志失败: {}", e))
            }
        },
        Err(e) => Err(format!("获取日志失败: {}", e))
    }
}

/// 清理旧的操作日志
#[tauri::command]
async fn cleanup_old_operation_logs(days_to_keep: u32, state: State<'_, AppState>) -> Result<bool, String> {
    let logger = state.operation_logger.clone();
    let logger = logger.lock().await;
    
    match logger.cleanup_old_logs(days_to_keep) {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("清理旧日志失败: {}", e))
    }
}

/// 运行综合测试套件
#[tauri::command]
async fn run_comprehensive_tests(state: State<'_, AppState>) -> Result<tests::TestStatistics, String> {
    info!("开始运行综合测试套件");
    
    let mut test_runner = tests::TestRunner::new();
    let statistics = test_runner.run_all_tests().await;
    
    info!("综合测试完成 - 总计: {}, 通过: {}, 失败: {}, 成功率: {:.1}%",
          statistics.total_tests,
          statistics.passed_tests,
          statistics.failed_tests,
          statistics.success_rate());
    
    Ok(statistics)
}

/// 运行特定类型的测试
#[tauri::command]
async fn run_test_suite(test_type: String, state: State<'_, AppState>) -> Result<tests::TestStatistics, String> {
    info!("运行 {} 测试套件", test_type);
    
    let mut test_runner = tests::TestRunner::new();
    let statistics = match test_type.as_str() {
        "unit" => {
            test_runner.run_unit_tests().await;
            test_runner.get_statistics().clone()
        },
        "integration" => {
            test_runner.run_integration_tests().await;
            test_runner.get_statistics().clone()
        },
        "e2e" => {
            test_runner.run_e2e_tests().await;
            test_runner.get_statistics().clone()
        },
        "performance" => {
            test_runner.run_performance_tests().await;
            test_runner.get_statistics().clone()
        },
        _ => return Err(format!("未知的测试类型: {}", test_type)),
    };
    
    Ok(statistics.clone())
}

/// 生成测试报告
#[tauri::command]
async fn generate_test_report(output_path: String, state: State<'_, AppState>) -> Result<bool, String> {
    info!("生成测试报告: {}", output_path);
    
    // 运行测试获取结果
    let mut test_runner = tests::TestRunner::new();
    let statistics = test_runner.run_all_tests().await;
    
    // 生成详细的测试报告
    let report_content = tests::TestReportGenerator::generate_html_report(&statistics, vec![]);
    
    // 写入报告文件
    let report_path = std::path::PathBuf::from(output_path);
    std::fs::write(&report_path, report_content)
        .map_err(|e| format!("写入测试报告失败: {}", e))?;
    
    info!("测试报告已生成: {}", report_path.display());
    Ok(true)
}

/// 获取恢复统计信息
#[tauri::command]
async fn get_recovery_statistics(state: State<'_, AppState>) -> Result<RecoveryStatistics, String> {
    let recovery_manager = state.error_recovery_manager.clone();
    let recovery_manager = recovery_manager.lock().await;
    
    let stats = recovery_manager.get_recovery_statistics();
    Ok(stats)
}

/// 清理过期备份
#[tauri::command]
async fn cleanup_expired_backups(state: State<'_, AppState>) -> Result<u32, String> {
    let recovery_manager = state.error_recovery_manager.clone();
    let mut recovery_manager = recovery_manager.lock().await;
    
    match recovery_manager.cleanup_expired_backups() {
        Ok(count) => Ok(count),
        Err(e) => Err(format!("清理过期备份失败: {}", e))
    }
}

/// 测试错误恢复
#[tauri::command]
async fn test_error_recovery(state: State<'_, AppState>) -> Result<bool, String> {
    let recovery_manager = state.error_recovery_manager.clone();
    let recovery_manager = recovery_manager.lock().await;
    
    // 创建一个测试错误
    use crate::file_operations::FileOperationError;
    let test_error = FileOperationError::PermissionDenied("测试权限错误".to_string());
    
    let context = RecoveryContext::new(
        "test_operation".to_string(),
        std::path::PathBuf::from("test_path"),
        None,
        "test_phase".to_string(),
    );
    
    // 这里我们只是测试错误分类，不实际执行恢复
    let error_type = recovery_manager.classify_error(&test_error);
    info!("测试错误分类: {:?}", error_type);
    
    Ok(true)
}

/// 获取性能统计信息
#[tauri::command]
async fn get_performance_stats(state: State<'_, AppState>) -> Result<performance_optimizer::PerformanceStats, String> {
    let optimizer = state.performance_optimizer.clone();
    let optimizer = optimizer.lock().await;
    
    Ok(optimizer.get_performance_stats())
}

/// 优化磁盘扫描
#[tauri::command]
async fn optimize_disk_scan(path: String, state: State<'_, AppState>) -> Result<bool, String> {
    let optimizer = state.performance_optimizer.clone();
    let optimizer = optimizer.lock().await;
    
    let path = std::path::Path::new(&path);
    optimizer.optimize_scan_operation(path, || {
        // 这里执行实际的扫描优化逻辑
        true
    });
    
    Ok(true)
}

/// 运行内存清理
#[tauri::command]
async fn run_memory_cleanup(state: State<'_, AppState>) -> Result<bool, String> {
    let optimizer = state.performance_optimizer.clone();
    let optimizer = optimizer.lock().await;
    
    if optimizer.should_periodic_cleanup() {
        optimizer.perform_cleanup().await;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 获取性能基准测试
#[tauri::command]
async fn get_performance_benchmark(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let optimizer = state.performance_optimizer.clone();
    let optimizer = optimizer.lock().await;
    
    let stats = optimizer.get_performance_stats();
    
    // 创建基准测试报告
    let benchmark = serde_json::json!({
        "memory_usage_mb": stats.memory_usage_mb,
        "memory_peak_mb": stats.memory_peak_mb,
        "cache_hit_rate": stats.cache_hit_rate,
        "cache_size": stats.cache_size,
        "batch_queue_size": stats.batch_queue_size,
        "last_cleanup_seconds_ago": stats.last_cleanup_seconds_ago,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(benchmark)
}

/// 初始化日志系统
pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    use logger::{init_logger as init_simple_logger, LoggerConfig, cleanup_old_logs};
    use log::LevelFilter;
    
    // 创建日志配置
    let config = LoggerConfig {
        log_level: if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        },
        ..LoggerConfig::default()
    };
    
    // 初始化日志
    init_simple_logger(config)?;
    
    // 清理旧日志文件
    cleanup_old_logs(5)?;
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    info!("开始初始化应用程序状态");
    
    // 初始化操作日志器
    let log_dir = logger::get_log_directory();
    let session_id = uuid::Uuid::new_v4().to_string();
    let current_user = whoami::username();
    
    let operation_logger = match OperationLogger::new(log_dir.clone(), session_id.clone(), current_user.clone()) {
        Ok(logger) => {
            info!("操作日志系统初始化成功");
            logger
        }
        Err(e) => {
            error!("操作日志系统初始化失败: {}", e);
            panic!("无法初始化操作日志系统: {}", e);
        }
    };
    
    // 初始化错误恢复管理器
    let recovery_config = ErrorRecoveryConfig::default();
    let error_recovery_manager = ErrorRecoveryManager::new(recovery_config);
    info!("错误恢复管理器初始化成功");
    
    // 初始化性能优化器
    let perf_config = PerformanceConfig::default();
    let performance_optimizer = PerformanceOptimizer::new(perf_config);
    info!("性能优化器初始化成功");
    
    let app_state = AppState {
        disk_analyzer: Arc::new(Mutex::new(DiskAnalyzer::new())),
        migration_service: Arc::new(MigrationService::new()),
        operation_logger: Arc::new(Mutex::new(operation_logger)),
        error_recovery_manager: Arc::new(Mutex::new(error_recovery_manager)),
        performance_optimizer: Arc::new(Mutex::new(performance_optimizer)),
    };

    info!("应用程序状态初始化完成");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            scan_directory,
            get_scan_progress,
            stop_scan,
            migrate_directory,
            validate_migration_path,
            get_disk_info,
            path_exists,
            format_size,
            get_operation_logs,
            get_operation_statistics,
            export_operation_logs,
            cleanup_old_operation_logs,
            get_recovery_statistics,
            cleanup_expired_backups,
            test_error_recovery,
            run_comprehensive_tests,
            run_test_suite,
            generate_test_report,
            get_performance_stats,
            optimize_disk_scan,
            run_memory_cleanup,
            get_performance_benchmark
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
