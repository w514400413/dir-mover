use log::{LevelFilter, info};
use simplelog::*;
use std::fs::File;
use std::path::PathBuf;
use chrono::Local;

/// 日志配置结构体
pub struct LoggerConfig {
    pub log_dir: PathBuf,
    pub log_level: LevelFilter,
    pub max_file_size: u64,
    pub max_files: u32,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            log_dir: get_default_log_dir(),
            log_level: LevelFilter::Info,
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 5,
        }
    }
}

/// 获取默认日志目录
fn get_default_log_dir() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
    path.push("dir-mover");
    path.push("logs");
    path
}

/// 初始化日志系统
pub fn init_logger(config: LoggerConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 创建日志目录
    std::fs::create_dir_all(&config.log_dir)?;
    
    // 生成日志文件名
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let log_file = config.log_dir.join(format!("dir-mover-{}.log", timestamp));
    
    // 使用默认配置
    let file_config = ConfigBuilder::new().build();
    let term_config = ConfigBuilder::new().build();
    
    // 初始化组合日志记录器
    CombinedLogger::init(vec![
        // 终端输出
        TermLogger::new(
            config.log_level,
            term_config,
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        // 文件输出
        WriteLogger::new(
            config.log_level,
            file_config,
            File::create(&log_file)?,
        ),
    ])?;
    
    info!("日志系统初始化完成");
    info!("日志文件: {}", log_file.display());
    info!("日志级别: {}", config.log_level);
    
    Ok(())
}

/// 获取当前日志目录
pub fn get_log_directory() -> PathBuf {
    get_default_log_dir()
}

/// 清理旧日志文件
pub fn cleanup_old_logs(max_files: u32) -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = get_default_log_dir();
    if !log_dir.exists() {
        return Ok(());
    }
    
    let mut log_files: Vec<_> = std::fs::read_dir(&log_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "log")
                .unwrap_or(false)
        })
        .collect();
    
    // 按修改时间排序（简化版本）
    log_files.sort_by(|a, b| {
        let time_a = a.metadata()
            .ok()
            .and_then(|meta| meta.modified().ok())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let time_b = b.metadata()
            .ok()
            .and_then(|meta| meta.modified().ok())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        time_a.cmp(&time_b)
    });
    
    // 删除最旧的文件
    while log_files.len() > max_files as usize {
        if let Some(old_file) = log_files.first() {
            std::fs::remove_file(old_file.path())?;
            log_files.remove(0);
        }
    }
    
    Ok(())
}