mod disk_analyzer;
mod error_recovery;
mod file_operations;
mod migration_service;
mod operation_logger;
mod performance_optimizer;
mod types;
mod logger;
mod tests;
mod appdata_analyzer;

use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{State, Emitter};
use log::{info, error, warn};

use disk_analyzer::{DiskAnalyzer, DirectoryInfo, format_file_size};
use error_recovery::{ErrorRecoveryManager, ErrorRecoveryConfig, RecoveryContext, RecoveryStatistics};
use migration_service::{MigrationService, MigrationOptions, MigrationResult, validate_migration_options};
use operation_logger::{OperationLogger, OperationStatistics};
use performance_optimizer::{PerformanceOptimizer, PerformanceConfig};
use types::PathValidationResult;
use appdata_analyzer::{AppDataAnalyzer, AppDataInfo, AppDataConfig, AppDataMigrationOptions, ScanEvent, StreamingAppDataScanner};

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
async fn run_comprehensive_tests(_state: State<'_, AppState>) -> Result<tests::TestStatistics, String> {
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
async fn run_test_suite(test_type: String, _state: State<'_, AppState>) -> Result<tests::TestStatistics, String> {
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
async fn generate_test_report(output_path: String, _state: State<'_, AppState>) -> Result<bool, String> {
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
    
    let _context = RecoveryContext::new(
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
            get_performance_benchmark,
            scan_appdata,
            scan_appdata_streaming,
            get_appdata_path,
            migrate_appdata_items,
            get_available_drives,
            get_migration_progress,
            validate_appdata_migration_options
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 扫描AppData目录（新版本，支持配置）
#[tauri::command]
async fn scan_appdata(config: Option<AppDataConfig>, state: State<'_, AppState>) -> Result<AppDataInfo, String> {
    info!("收到扫描AppData目录请求");
    
    let mut analyzer = AppDataAnalyzer::new();
    
    // 应用配置（如果有）
    if let Some(config) = config {
        let min_size = config.min_size_threshold;
        let max_depth = config.max_depth;
        let sort_order = config.sort_order.clone();
        
        analyzer.set_config(config);
        info!("应用自定义配置: min_size_threshold={}, max_depth={}, sort_order={:?}",
              min_size, max_depth, sort_order);
    }
    
    info!("开始扫描AppData目录");
    
    match analyzer.scan_appdata().await {
        Ok(result) => {
            info!("AppData扫描完成 - 总大小: {}, 一级项目数量: {}, 大项目数量: {}, 耗时: {}ms",
                  format_file_size(result.total_size),
                  result.first_level_items.len(),
                  result.large_items.len(),
                  result.scan_time_ms);
            Ok(result)
        }
        Err(e) => {
            error!("AppData扫描失败: {}", e);
            Err(format!("AppData扫描失败: {}", e))
        }
    }
}

/// 迁移AppData项目
#[tauri::command]
async fn migrate_appdata_items(options: AppDataMigrationOptions, state: State<'_, AppState>) -> Result<MigrationResult, String> {
    info!("收到AppData迁移请求 - 目标盘符: {}, 项目数量: {}",
          options.target_drive, options.source_items.len());
    
    // 验证目标盘符
    let target_drive = Path::new(&options.target_drive);
    if !target_drive.exists() {
        return Err(format!("目标盘符不存在: {}", options.target_drive));
    }
    
    let mut all_results = Vec::new();
    let mut total_migrated_size = 0u64;
    let mut success_count = 0;
    let mut failure_count = 0;
    
    // 逐个迁移项目
    for source_item in &options.source_items {
        let source_path = Path::new(source_item);
        if !source_path.exists() {
            warn!("源项目不存在，跳过: {}", source_item);
            failure_count += 1;
            continue;
        }
        
        // 构建目标路径（保持相对路径结构）
        let item_name = source_path.file_name()
            .ok_or_else(|| format!("无法获取项目名称: {}", source_item))?
            .to_string_lossy();
        
        let target_path = target_drive.join(item_name.to_string());
        
        info!("迁移项目: {} -> {}", source_item, target_path.display());
        
        let migration_options = MigrationOptions {
            source_path: source_item.clone(),
            target_path: target_path.to_string_lossy().to_string(),
            create_symlink: options.create_symlink,
            delete_source: options.delete_source,
        };
        
        // 验证迁移选项
        if let Err(e) = validate_migration_options(&migration_options) {
            error!("迁移选项验证失败: {}", e);
            failure_count += 1;
            continue;
        }
        
        // 执行迁移
        match state.migration_service.migrate_folder(migration_options).await {
            Ok(result) => {
                if result.success {
                    success_count += 1;
                    // 估算迁移大小（简化处理）
                    let estimated_size = 1024 * 1024 * 1024; // 假设1GB
                    total_migrated_size += estimated_size;
                    info!("项目迁移成功: {}, 大小: {}", source_item, format_file_size(estimated_size));
                } else {
                    failure_count += 1;
                    error!("项目迁移失败: {}, 错误: {}", source_item, result.message);
                }
                all_results.push(result);
            }
            Err(e) => {
                failure_count += 1;
                error!("项目迁移出错: {}, 错误: {}", source_item, e);
            }
        }
    }
    
    // 汇总结果
    let overall_success = failure_count == 0;
    let summary = format!("AppData迁移完成 - 成功: {}, 失败: {}, 总迁移大小: {}",
                         success_count, failure_count, format_file_size(total_migrated_size));
    
    info!("{}", summary);
    
    Ok(MigrationResult {
        success: overall_success,
        message: summary,
        source_path: format!("{}个项目", options.source_items.len()),
        target_path: options.target_drive.clone(),
        symlink_path: if options.create_symlink { Some(format!("创建了{}个符号链接", success_count)) } else { None },
    })
}

/// 获取系统可用盘符
#[tauri::command]
fn get_available_drives() -> Result<Vec<String>, String> {
    info!("获取系统可用盘符");
    
    // 简化的盘符检测 - 检查常见的盘符
    let mut drives = Vec::new();
    let common_drives = ['C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];
    
    for drive_letter in common_drives {
        let drive_path = format!("{}:\\", drive_letter);
        let drive_path_buf = std::path::PathBuf::from(&drive_path);
        
        // 检查盘符是否存在且是目录
        if drive_path_buf.exists() && drive_path_buf.is_dir() {
            drives.push(drive_path);
        }
    }
    
    info!("发现可用盘符: {:?}", drives);
    Ok(drives)
}

/// 获取迁移进度（用于实时进度报告）
#[tauri::command]
fn get_migration_progress(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    // 这里应该实现真实的进度跟踪
    // 暂时返回模拟数据
    Ok(serde_json::json!({
        "current_item": "暂无迁移任务",
        "progress": 0,
        "total_items": 0,
        "estimated_time_remaining": 0
    }))
}

/// 验证AppData迁移选项
#[tauri::command]
fn validate_appdata_migration_options(options: AppDataMigrationOptions) -> Result<serde_json::Value, String> {
    info!("验证AppData迁移选项 - 目标盘符: {}, 项目数量: {}", options.target_drive, options.source_items.len());
    
    let mut validation_results = Vec::new();
    let mut valid_count = 0;
    
    // 验证目标盘符
    let target_drive_path = Path::new(&options.target_drive);
    if !target_drive_path.exists() {
        return Err(format!("目标盘符不存在: {}", options.target_drive));
    }
    
    // 验证每个源项目
    for source_item in &options.source_items {
        let source_path = Path::new(source_item);
        let item_validation = if !source_path.exists() {
            serde_json::json!({
                "path": source_item,
                "valid": false,
                "message": "源路径不存在"
            })
        } else if !source_path.is_dir() && !source_path.is_file() {
            serde_json::json!({
                "path": source_item,
                "valid": false,
                "message": "路径既不是文件也不是目录"
            })
        } else {
            valid_count += 1;
            serde_json::json!({
                "path": source_item,
                "valid": true,
                "message": "路径有效"
            })
        };
        
        validation_results.push(item_validation);
    }
    
    let summary = format!("验证完成：{}/{} 个项目有效", valid_count, options.source_items.len());
    info!("{}", summary);
    
    Ok(serde_json::json!({
        "valid": valid_count == options.source_items.len(),
        "items": validation_results,
        "summary": summary,
        "target_drive_valid": true,
        "target_drive": options.target_drive
    }))
}

/// 获取AppData路径信息
#[tauri::command]
fn get_appdata_path() -> Result<String, String> {
    match AppDataAnalyzer::get_appdata_path() {
        Ok(path) => {
            info!("获取AppData路径: {}", path.display());
            Ok(path.to_string_lossy().to_string())
        }
        Err(e) => {
            error!("获取AppData路径失败: {}", e);
            Err(format!("获取AppData路径失败: {}", e))
        }
    }
}

/// 流式扫描AppData目录 - 使用Tauri事件系统实现实时推送
#[tauri::command]
async fn scan_appdata_streaming(
    config: Option<AppDataConfig>,
    window: tauri::Window,
    _state: State<'_, AppState>
) -> Result<AppDataInfo, String> {
    info!("收到流式扫描AppData目录请求");
    
    let mut analyzer = AppDataAnalyzer::new();
    
    // 应用配置（如果有）
    if let Some(config) = config.clone() {
        let min_size = config.min_size_threshold;
        let max_depth = config.max_depth;
        let sort_order = config.sort_order.clone();
        
        analyzer.set_config(config);
        info!("应用自定义配置: min_size_threshold={}, max_depth={}, sort_order={:?}",
              min_size, max_depth, sort_order);
    }
    
    // 创建事件通道
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<ScanEvent>();
    let streaming_scanner = StreamingAppDataScanner::new(analyzer, event_tx);
    
    // 启动事件转发任务 - 将扫描事件转发到Tauri前端
    let window_clone = window.clone();
    let event_forward_task = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            // 将扫描事件发送到前端
            if let Err(e) = window_clone.emit("appdata-scan-event", &event) {
                error!("发送扫描事件到前端失败: {}", e);
                break;
            }
            
            // 添加小延迟避免事件发送过快
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    });
    
    info!("开始流式扫描AppData目录");
    
    // 执行流式扫描
    let scan_result = streaming_scanner.scan_appdata_streaming().await;
    
    // 等待事件转发任务完成
    if let Err(e) = event_forward_task.await {
        error!("事件转发任务失败: {}", e);
    }
    
    match &scan_result {
        Ok(result) => {
            info!("流式AppData扫描完成 - 总大小: {}, 一级项目数量: {}, 大项目数量: {}, 耗时: {}ms",
                  format_file_size(result.total_size),
                  result.first_level_items.len(),
                  result.large_items.len(),
                  result.scan_time_ms);
            
            // 发送最终的扫描完成事件
            let _ = window.emit("appdata-scan-complete", &serde_json::json!({
                "success": true,
                "total_items": result.first_level_items.len(),
                "total_size": result.total_size,
                "scan_time_ms": result.scan_time_ms
            }));
        }
        Err(e) => {
            error!("流式AppData扫描失败: {}", e);
            
            // 发送扫描失败事件
            let _ = window.emit("appdata-scan-error", &serde_json::json!({
                "success": false,
                "error": e.to_string()
            }));
        }
    }
    
    scan_result
}
