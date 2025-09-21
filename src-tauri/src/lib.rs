mod disk_analyzer;
mod file_operations;
mod migration_service;
mod types;
mod logger;

use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;
use log::{info, error};

use disk_analyzer::{DiskAnalyzer, DirectoryInfo, format_file_size};
use migration_service::{MigrationService, MigrationOptions, MigrationResult, validate_migration_options};
use types::{ScanProgress, PathValidationResult};

/// 应用状态
struct AppState {
    disk_analyzer: Arc<Mutex<DiskAnalyzer>>,
    migration_service: Arc<MigrationService>,
}

/// 扫描目录（异步版本）
#[tauri::command]
async fn scan_directory(path: String, state: State<'_, AppState>) -> Result<DirectoryInfo, String> {
    info!("收到扫描目录请求: {}", path);
    
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
        info!("分析器配置完成，最大深度: 5");
    }
    
    info!("开始扫描目录: {}", path.display());
    
    // 使用异步扫描
    let result = {
        let analyzer = analyzer.lock().await;
        analyzer.scan_directory_async(path).await
    };
    
    match &result {
        Ok(info) => {
            info!("扫描完成: {} (文件数: {}, 大小: {})",
                  path.display(), info.file_count, format_file_size(info.size));
        }
        Err(e) => {
            error!("扫描失败 {}: {}", path.display(), e);
        }
    }
    
    result
}

/// 获取扫描进度
#[tauri::command]
fn get_scan_progress(_state: State<'_, AppState>) -> Result<ScanProgress, String> {
    // 这里可以实现更复杂的进度跟踪逻辑
    // 暂时返回一个简单的进度信息
    Ok(ScanProgress {
        current_path: String::new(),
        processed_files: 0,
        total_files: 0,
        progress: 0.0,
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
    
    let app_state = AppState {
        disk_analyzer: Arc::new(Mutex::new(DiskAnalyzer::new())),
        migration_service: Arc::new(MigrationService::new()),
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
            format_size
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
