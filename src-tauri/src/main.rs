// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::info;

fn main() {
    // 初始化日志系统
    if let Err(e) = dir_mover_lib::init_logger() {
        eprintln!("初始化日志系统失败: {}", e);
    }
    
    info!("应用程序启动");
    dir_mover_lib::run()
}
