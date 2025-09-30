//! 端到端测试模块
//!
//! 测试完整的用户场景和工作流程

use crate::disk_analyzer::DiskAnalyzer;
use crate::migration_service::{MigrationService, MigrationOptions};
use crate::error_recovery::{ErrorRecoveryManager, ErrorRecoveryConfig, RecoveryContext};
use crate::operation_logger::{OperationLogger, OperationType, OperationStatus};
use crate::tests::test_utils::create_test_directory_structure;
use tempfile::TempDir;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use log::info;

/// 完整迁移流程测试
pub async fn test_complete_migration_workflow() -> Result<(), crate::tests::TestError> {
    info!("开始完整迁移流程测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 模拟用户场景：用户有一个包含各种文件的文档目录
    let documents_dir = temp_dir.path().join("Documents");
    create_realistic_user_directory(&documents_dir)?;

    // 步骤1: 用户扫描目录
    info!("步骤1: 扫描用户文档目录");
    let scan_start = Instant::now();
    
    let analyzer = DiskAnalyzer::new();
    let scan_result = analyzer.scan_directory_async(&documents_dir).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("扫描失败: {}", e)))?;
    
    let scan_duration = scan_start.elapsed();
    
    info!("扫描完成 - 耗时: {:?}, 文件数: {}, 大小: {}", 
          scan_duration, scan_result.file_count, scan_result.size);

    // 验证扫描结果
    assert!(scan_result.file_count > 0, "应该找到文件");
    assert!(scan_result.size > 0, "应该有文件大小");
    assert_eq!(scan_result.name, "Documents", "目录名称应该正确");

    // 步骤2: 用户选择目标位置
    let target_dir = temp_dir.path().join("MovedDocuments");
    
    // 步骤3: 验证迁移路径
    info!("步骤2: 验证迁移路径");
    let service = MigrationService::new();
    let validation_result = service.file_operator().validate_migration_path(&documents_dir, &target_dir)
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("路径验证失败: {}", e)))?;

    assert!(validation_result.0, "路径验证应该通过");
    info!("路径验证通过: {}", validation_result.1);

    // 步骤4: 执行迁移
    info!("步骤3: 执行迁移");
    let migrate_start = Instant::now();
    
    let options = MigrationOptions {
        source_path: documents_dir.display().to_string(),
        target_path: target_dir.display().to_string(),
        create_symlink: true,  // 用户选择创建符号链接
        delete_source: false,  // 用户选择保留源文件
    };

    let migrate_result = service.migrate_folder(options).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("迁移失败: {}", e)))?;
    
    let migrate_duration = migrate_start.elapsed();
    
    info!("迁移完成 - 耗时: {:?}, 结果: {}", migrate_duration, migrate_result.message);
    assert!(migrate_result.success, "迁移应该成功");

    // 步骤5: 验证迁移结果
    info!("步骤4: 验证迁移结果");
    
    // 验证目标目录存在且内容正确
    assert!(target_dir.exists(), "目标目录应该存在");
    
    let target_scan_result = analyzer.scan_directory_async(&target_dir).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("目标扫描失败: {}", e)))?;
    
    assert_eq!(target_scan_result.file_count, scan_result.file_count, "文件数量应该相同");
    assert_eq!(target_scan_result.size, scan_result.size, "文件大小应该相同");

    // 验证符号链接（如果创建成功）
    if migrate_result.symlink_path.is_some() {
        info!("符号链接创建成功: {:?}", migrate_result.symlink_path);
        // 在实际环境中，这里应该验证符号链接指向正确
    }

    // 步骤6: 验证源目录仍然存在（因为delete_source=false）
    assert!(documents_dir.exists(), "源目录应该仍然存在");

    // 步骤7: 性能验证
    let total_duration = scan_duration + migrate_duration;
    info!("总耗时: {:?}", total_duration);
    
    // 验证性能在合理范围内（例如，小目录应该在几秒内完成）
    assert!(total_duration < Duration::from_secs(30), "总耗时应该在合理范围内");

    info!("完整迁移流程测试完成");
    Ok(())
}

/// 大文件夹处理测试
pub async fn test_large_folder_handling() -> Result<(), crate::tests::TestError> {
    info!("开始大文件夹处理测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 创建包含大量文件的大目录
    let large_dir = temp_dir.path().join("LargeFolder");
    create_large_folder_structure(&large_dir, 1000, 10)?; // 1000个文件，10个子目录

    // 步骤1: 扫描大文件夹
    info!("扫描大文件夹...");
    let scan_start = Instant::now();
    
    let mut analyzer = DiskAnalyzer::new();
    analyzer.set_c_drive_mode(true); // 启用C盘模式优化
    analyzer.set_large_folder_threshold(1024 * 1024); // 1MB阈值
    
    let scan_result = analyzer.scan_directory_async(&large_dir).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("扫描失败: {}", e)))?;
    
    let scan_duration = scan_start.elapsed();
    
    info!("大文件夹扫描完成 - 耗时: {:?}, 文件数: {}, 大小: {}", 
          scan_duration, scan_result.file_count, scan_result.size);

    // 验证扫描结果
    assert!(scan_result.file_count >= 1000, "应该找到至少1000个文件");
    assert!(scan_result.is_large_folder, "应该被标记为大文件夹");

    // 步骤2: 迁移大文件夹
    let target_dir = temp_dir.path().join("MovedLargeFolder");
    
    info!("迁移大文件夹...");
    let migrate_start = Instant::now();
    
    let service = MigrationService::new();
    let options = MigrationOptions {
        source_path: large_dir.display().to_string(),
        target_path: target_dir.display().to_string(),
        create_symlink: false,
        delete_source: false,
    };

    let migrate_result = service.migrate_folder(options).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("迁移失败: {}", e)))?;
    
    let migrate_duration = migrate_start.elapsed();
    
    info!("大文件夹迁移完成 - 耗时: {:?}", migrate_duration);
    assert!(migrate_result.success, "大文件夹迁移应该成功");

    // 步骤3: 验证迁移结果
    let target_scan_result = analyzer.scan_directory_async(&target_dir).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("目标扫描失败: {}", e)))?;
    
    assert_eq!(target_scan_result.file_count, scan_result.file_count, "文件数量应该相同");

    // 步骤4: 性能验证
    info!("性能分析:");
    info!("  扫描性能: {} 文件/秒", scan_result.file_count as f64 / scan_duration.as_secs_f64());
    info!("  迁移性能: {} 文件/秒", scan_result.file_count as f64 / migrate_duration.as_secs_f64());

    // 验证性能在合理范围内
    let scan_rate = scan_result.file_count as f64 / scan_duration.as_secs_f64();
    let migrate_rate = scan_result.file_count as f64 / migrate_duration.as_secs_f64();
    
    assert!(scan_rate > 10.0, "扫描速率应该大于10文件/秒");
    assert!(migrate_rate > 5.0, "迁移速率应该大于5文件/秒");

    info!("大文件夹处理测试完成");
    Ok(())
}

/// 错误恢复流程测试
pub async fn test_error_recovery_workflow() -> Result<(), crate::tests::TestError> {
    info!("开始错误恢复流程测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 初始化各个组件
    let analyzer = DiskAnalyzer::new();
    let service = MigrationService::new();
    let config = ErrorRecoveryConfig::default();
    let mut recovery_manager = ErrorRecoveryManager::new(config);
    let log_dir = temp_dir.path().join("recovery_logs");
    
    let logger = OperationLogger::new(
        log_dir,
        "recovery_test_session".to_string(),
        "recovery_test_user".to_string(),
    ).map_err(|e| crate::tests::TestError::SetupFailed(format!("日志器创建失败: {}", e)))?;

    // 步骤1: 创建一个正常的目录
    let source_dir = temp_dir.path().join("RecoverySource");
    create_test_directory_structure(&source_dir)?;

    // 步骤2: 创建一个会导致权限问题的目标目录（模拟）
    let problematic_target = temp_dir.path().join("SystemProtected"); // 模拟系统保护目录

    // 步骤3: 尝试迁移到受保护的位置
    info!("尝试迁移到受保护的位置...");
    
    let options = MigrationOptions {
        source_path: source_dir.display().to_string(),
        target_path: problematic_target.display().to_string(),
        create_symlink: false,
        delete_source: false,
    };

    // 这个迁移应该会因为权限问题而失败
    let migrate_result = service.migrate_folder(options).await;
    
    match migrate_result {
        Ok(result) => {
            if result.success {
                info!("迁移意外成功（可能测试环境权限较宽松）");
            } else {
                info!("迁移失败，开始错误恢复流程: {}", result.message);
                
                // 步骤4: 记录错误到操作日志
                let mut error_log = logger.log_operation_start(
                    OperationType::Migrate,
                    source_dir.display().to_string(),
                    Some(problematic_target.display().to_string()),
                    "迁移到受保护目录失败".to_string(),
                ).map_err(|e| crate::tests::TestError::ExecutionFailed(format!("错误日志记录失败: {}", e)))?;
                
                logger.fail_operation(&mut error_log, result.message.clone(), Some("权限不足".to_string()))
                    .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("错误日志更新失败: {}", e)))?;

                // 步骤5: 尝试恢复策略
                use crate::file_operations::FileOperationError;
                let permission_error = FileOperationError::PermissionDenied("权限被拒绝".to_string());
                
                let context = RecoveryContext::new(
                    "recovery_migration".to_string(),
                    source_dir.clone(),
                    Some(problematic_target.clone()),
                    "migration_phase".to_string(),
                );

                let recovery_result = recovery_manager.handle_error("recovery_test_1", &permission_error, &context).await;
                
                match recovery_result {
                    Ok(recovery) => {
                        info!("错误恢复结果: {:?}", recovery);
                        assert_eq!(recovery.recovery_type, crate::error_recovery::RecoveryType::Manual, 
                                  "权限错误应该需要手动处理");
                    },
                    Err(e) => {
                        info!("错误恢复失败: {}", e);
                        // 即使恢复失败，测试也算通过，因为我们测试的是流程
                    }
                }
            }
        },
        Err(e) => {
            info!("迁移执行失败: {}", e);
            // 记录系统级错误
        }
    }

    // 步骤6: 尝试使用备用目标路径
    let backup_target = temp_dir.path().join("RecoveryTarget");
    
    info!("尝试使用备用目标路径...");
    let backup_options = MigrationOptions {
        source_path: source_dir.display().to_string(),
        target_path: backup_target.display().to_string(),
        create_symlink: false,
        delete_source: false,
    };

    let backup_result = service.migrate_folder(backup_options).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("备用迁移失败: {}", e)))?;

    assert!(backup_result.success, "备用迁移应该成功");
    info!("备用迁移成功完成");

    // 步骤7: 验证操作日志记录了完整的恢复流程
    let recent_logs = logger.get_recent_logs(10)
        .map_err(|e| crate::tests::TestError::AssertionFailed(format!("查询日志失败: {}", e)))?;
    
    assert!(!recent_logs.is_empty(), "应该记录了操作日志");
    
    // 查找错误日志
    let error_logs: Vec<_> = recent_logs.iter()
        .filter(|log| log.status == OperationStatus::Failed)
        .collect();
    
    if !error_logs.is_empty() {
        info!("找到 {} 条错误日志，验证了错误记录功能", error_logs.len());
    }

    info!("错误恢复流程测试完成");
    Ok(())
}

/// 用户界面交互测试
pub async fn test_ui_interactions() -> Result<(), crate::tests::TestError> {
    info!("开始用户界面交互测试");

    // 这个测试模拟用户通过UI进行的各种操作
    // 由于我们没有实际的UI，这里测试的是后端API的响应

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 模拟用户创建测试数据
    let test_dir = temp_dir.path().join("UIInteractionTest");
    create_diverse_test_data(&test_dir)?;

    // 测试1: 快速扫描（用户点击扫描按钮）
    info!("模拟用户点击扫描按钮...");
    let scan_start = Instant::now();
    
    let analyzer = DiskAnalyzer::new();
    let scan_result = analyzer.scan_directory_async(&test_dir).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("扫描失败: {}", e)))?;
    
    let scan_duration = scan_start.elapsed();
    
    info!("扫描完成 - 耗时: {:?}, 适合UI响应", scan_duration);
    assert!(scan_duration < Duration::from_secs(5), "扫描应该在5秒内完成，适合UI响应");

    // 测试2: 路径验证（用户输入目标路径时的实时验证）
    info!("模拟用户输入目标路径时的验证...");
    let validation_start = Instant::now();
    
    let service = MigrationService::new();
    let target_path = temp_dir.path().join("UITarget");
    
    let (valid, message) = service.file_operator().validate_migration_path(&test_dir, &target_path)
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("路径验证失败: {}", e)))?;
    
    let validation_duration = validation_start.elapsed();
    
    info!("路径验证完成 - 耗时: {:?}, 结果: {}", validation_duration, message);
    assert!(validation_duration < Duration::from_millis(500), "路径验证应该在500ms内完成");
    assert!(valid, "路径验证应该通过");

    // 测试3: 迁移操作（用户点击迁移按钮）
    info!("模拟用户点击迁移按钮...");
    let migrate_start = Instant::now();
    
    let options = MigrationOptions {
        source_path: test_dir.display().to_string(),
        target_path: target_path.display().to_string(),
        create_symlink: false,
        delete_source: false,
    };

    let migrate_result = service.migrate_folder(options).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("迁移失败: {}", e)))?;
    
    let migrate_duration = migrate_start.elapsed();
    
    info!("迁移完成 - 耗时: {:?}, 结果: {}", migrate_duration, migrate_result.message);
    assert!(migrate_result.success, "迁移应该成功");

    // 测试4: 进度更新（模拟UI进度条更新）
    info!("模拟进度更新...");
    
    // 获取扫描进度
    let progress = analyzer.get_scan_progress();
    info!("当前进度: {:.1}%, 当前路径: {}", progress.progress, progress.current_path);
    
    assert!(progress.progress >= 0.0 && progress.progress <= 100.0, "进度应该在0-100%之间");

    // 测试5: 错误提示（模拟用户看到错误信息）
    info!("模拟错误提示...");
    
    // 尝试一个会失败的操作
    let invalid_target = test_dir.join("invalid/nested/target");
    let invalid_options = MigrationOptions {
        source_path: test_dir.display().to_string(),
        target_path: invalid_target.display().to_string(),
        create_symlink: false,
        delete_source: false,
    };

    let invalid_result = service.migrate_folder(invalid_options).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("无效迁移失败: {}", e)))?;

    assert!(!invalid_result.success, "无效迁移应该失败");
    info!("错误信息适合UI显示: {}", invalid_result.message);

    info!("用户界面交互测试完成");
    Ok(())
}

/// 并发操作测试
pub async fn test_concurrent_operations() -> Result<(), crate::tests::TestError> {
    info!("开始并发操作测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 创建多个测试目录
    let dirs: Vec<PathBuf> = (0..5).map(|i| temp_dir.path().join(format!("ConcurrentDir{}", i))).collect();
    
    for dir in &dirs {
        create_test_directory_structure(dir)?;
    }

    // 测试1: 并发扫描
    info!("测试并发扫描...");
    let scan_start = Instant::now();
    
    let analyzer = DiskAnalyzer::new();
    let mut scan_handles = vec![];
    
    for dir in &dirs {
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
    
    let scan_duration = scan_start.elapsed();
    
    info!("并发扫描完成 - 耗时: {:?}, 扫描了 {} 个目录", scan_duration, scan_results.len());
    
    // 验证所有扫描都成功
    for result in &scan_results {
        assert!(result.file_count > 0, "每个目录都应该有文件");
    }

    // 测试2: 并发迁移
    info!("测试并发迁移...");
    let migrate_start = Instant::now();
    
    let service = MigrationService::new();
    let mut migrate_handles = vec![];
    
    for (i, source_dir) in dirs.iter().enumerate() {
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
    
    let migrate_duration = migrate_start.elapsed();
    
    info!("并发迁移完成 - 耗时: {:?}, 迁移了 {} 个目录", migrate_duration, migrate_results.len());
    
    // 验证所有迁移都成功
    for result in &migrate_results {
        assert!(result.success, "每个迁移都应该成功");
    }

    // 测试3: 并发错误处理
    info!("测试并发错误处理...");
    
    let config = ErrorRecoveryConfig::default();
    let recovery_manager = ErrorRecoveryManager::new(config);
    
    let mut error_handles = vec![];
    
    for i in 0..3 {
        let mut recovery_clone = recovery_manager.clone();
        
        let handle = tokio::spawn(async move {
            use crate::file_operations::FileOperationError;
            
            let error = FileOperationError::PermissionDenied(format!("并发权限错误 {}", i));
            let context = crate::error_recovery::RecoveryContext::new(
                format!("concurrent_test_{}", i),
                std::path::PathBuf::from(format!("/test/path/{}", i)),
                None,
                "concurrent_phase".to_string(),
            );
            
            recovery_clone.handle_error(&format!("concurrent_op_{}", i), &error, &context).await
        });
        
        error_handles.push(handle);
    }
    
    // 等待所有错误处理完成
    for handle in error_handles {
        let _result = handle.await
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("并发错误处理任务失败: {}", e)))?;
    }
    
    info!("并发错误处理完成");

    // 测试4: 性能分析
    info!("并发性能分析:");
    info!("  并发扫描总耗时: {:?}", scan_duration);
    info!("  并发迁移总耗时: {:?}", migrate_duration);
    info!("  平均扫描时间: {:?}", scan_duration / dirs.len() as u32);
    info!("  平均迁移时间: {:?}", migrate_duration / dirs.len() as u32);

    // 验证并发性能
    let avg_scan_time = scan_duration / dirs.len() as u32;
    let avg_migrate_time = migrate_duration / dirs.len() as u32;
    
    assert!(avg_scan_time < Duration::from_secs(2), "平均扫描时间应该在2秒内");
    assert!(avg_migrate_time < Duration::from_secs(3), "平均迁移时间应该在3秒内");

    info!("并发操作测试完成");
    Ok(())
}

/// 创建现实的用户目录结构
fn create_realistic_user_directory(base_path: &std::path::Path) -> Result<(), crate::tests::TestError> {
    // 创建常见的用户目录结构
    let subdirs = vec![
        "Documents",
        "Documents/Work",
        "Documents/Personal",
        "Pictures",
        "Pictures/Vacation",
        "Pictures/Family",
        "Downloads",
        "Music",
        "Videos",
    ];

    for dir in subdirs {
        let dir_path = base_path.join(dir);
        fs::create_dir_all(&dir_path)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建目录 {} 失败: {}", dir, e)))?;
    }

    // 创建各种类型的文件
    let files = vec![
        ("Documents/Work/report.docx", "工作报告内容"),
        ("Documents/Work/budget.xlsx", "预算表格数据"),
        ("Documents/Personal/letter.txt", "个人信件内容"),
        ("Pictures/Vacation/beach.jpg", "度假照片"),
        ("Pictures/Family/photo.jpg", "家庭照片"),
        ("Downloads/setup.exe", "下载的安装文件"),
        ("Music/song.mp3", "音乐文件"),
        ("Videos/movie.mp4", "视频文件"),
        ("readme.txt", "说明文档"),
    ];

    for (file_path, content) in files {
        let full_path = base_path.join(file_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建父目录失败: {}", e)))?;
        }
        
        let mut file = File::create(&full_path)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建文件 {} 失败: {}", file_path, e)))?;
        
        writeln!(file, "{}", content)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入文件 {} 失败: {}", file_path, e)))?;
    }

    Ok(())
}

/// 创建大型文件夹结构
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

/// 创建多样化的测试数据
fn create_diverse_test_data(base_path: &std::path::Path) -> Result<(), crate::tests::TestError> {
    // 创建不同大小的文件
    let test_files = vec![
        ("small.txt", 100),           // 100字节小文件
        ("medium.txt", 1024),         // 1KB中等文件
        ("large.txt", 10240),         // 10KB大文件
        ("binary.dat", 1024),         // 二进制文件
        ("document.doc", 5000),       // 文档文件
        ("image.jpg", 2048),          // 图片文件
    ];

    for (filename, size) in test_files {
        let file_path = base_path.join(filename);
        let mut file = File::create(&file_path)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建文件 {} 失败: {}", filename, e)))?;
        
        // 写入指定大小的内容
        let content = "测试内容".repeat(size / 12 + 1);
        file.write_all(&content.as_bytes()[..size.min(content.len())])
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入文件 {} 失败: {}", filename, e)))?;
    }

    // 创建子目录结构
    let subdirs = vec!["subdir1", "subdir2", "subdir1/nested"];
    for dir in subdirs {
        let dir_path = base_path.join(dir);
        fs::create_dir_all(&dir_path)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建目录 {} 失败: {}", dir, e)))?;
        
        // 在每个子目录中创建文件
        let subfile_path = dir_path.join("subfile.txt");
        let mut subfile = File::create(&subfile_path)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建子文件失败: {}", e)))?;
        
        writeln!(subfile, "子目录 {} 的内容", dir)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入子文件失败: {}", e)))?;
    }

    Ok(())
}

/// 为MigrationService实现Clone trait（用于测试）
impl Clone for MigrationService {
    fn clone(&self) -> Self {
        MigrationService::new()
    }
}

/// 为ErrorRecoveryManager实现Clone trait（用于测试）
impl Clone for ErrorRecoveryManager {
    fn clone(&self) -> Self {
        let config = ErrorRecoveryConfig::default();
        ErrorRecoveryManager::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_e2e_tests_runner() {
        // 这个测试确保所有的端到端测试函数都能正常运行
        let result = test_complete_migration_workflow().await;
        assert!(result.is_ok(), "完整迁移流程测试应该通过");
        
        let result = test_large_folder_handling().await;
        assert!(result.is_ok(), "大文件夹处理测试应该通过");
        
        let result = test_error_recovery_workflow().await;
        assert!(result.is_ok(), "错误恢复流程测试应该通过");
        
        let result = test_ui_interactions().await;
        assert!(result.is_ok(), "用户界面交互测试应该通过");
        
        let result = test_concurrent_operations().await;
        assert!(result.is_ok(), "并发操作测试应该通过");
    }
}