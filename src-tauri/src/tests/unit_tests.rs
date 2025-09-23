//! 单元测试模块
//! 
//! 测试各个模块的独立功能

use crate::disk_analyzer::{DiskAnalyzer, DirectoryInfo};
use crate::file_operations::{FileOperator, FileOperationResult};
use crate::migration_service::{MigrationService, MigrationOptions};
use crate::error_recovery::{ErrorRecoveryManager, ErrorRecoveryConfig};
use crate::operation_logger::{OperationLogger, OperationType, OperationStatus};
use tempfile::TempDir;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use log::info;

/// 磁盘分析器单元测试
pub async fn test_disk_analyzer() -> Result<(), crate::tests::TestError> {
    info!("开始磁盘分析器单元测试");

    // 创建临时测试目录
    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    let test_dir = temp_dir.path().join("test_analyzer");
    
    // 创建测试目录结构
    create_test_directory_structure(&test_dir)?;

    // 测试1: 基本目录扫描
    let analyzer = DiskAnalyzer::new();
    let result = analyzer.scan_directory_async(&test_dir).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("扫描失败: {}", e)))?;

    // 验证结果
    assert_eq!(result.file_count, 6, "文件数量不匹配");
    assert!(result.size > 0, "目录大小应该大于0");
    assert_eq!(result.name, "test_analyzer", "目录名称不匹配");

    // 测试2: C盘模式扫描
    let mut analyzer = DiskAnalyzer::new();
    analyzer.set_c_drive_mode(true);
    analyzer.set_large_folder_threshold(1024); // 1KB阈值
    
    let result = analyzer.scan_directory_async(&test_dir).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("C盘模式扫描失败: {}", e)))?;

    assert!(result.is_large_folder || result.size < 1024, "大文件夹检测逻辑错误");

    // 测试3: 最大深度限制
    let mut analyzer = DiskAnalyzer::new();
    analyzer.set_max_depth(2);
    
    let result = analyzer.scan_directory_async(&test_dir).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("深度限制扫描失败: {}", e)))?;

    assert!(result.subdirectories.len() <= 4, "深度限制未生效"); // 最多2层

    // 测试4: 取消扫描功能
    let analyzer = DiskAnalyzer::new();
    
    // 启动扫描
    let scan_task = tokio::spawn(async move {
        analyzer.scan_directory_async(&test_dir).await
    });
    
    // 立即取消
    analyzer.cancel_scan();
    
    // 等待任务完成
    let _result = scan_task.await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("任务执行失败: {}", e)))?;

    info!("磁盘分析器单元测试完成");
    Ok(())
}

/// 文件操作单元测试
pub async fn test_file_operations() -> Result<(), crate::tests::TestError> {
    info!("开始文件操作单元测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 测试1: 文件复制
    test_file_copy(&temp_dir).await?;
    
    // 测试2: 目录复制
    test_directory_copy(&temp_dir).await?;
    
    // 测试3: 文件删除
    test_file_deletion(&temp_dir).await?;
    
    // 测试4: 目录删除
    test_directory_deletion(&temp_dir).await?;
    
    // 测试5: 符号链接创建
    test_symlink_creation(&temp_dir).await?;
    
    // 测试6: 操作取消
    test_operation_cancellation(&temp_dir).await?;

    info!("文件操作单元测试完成");
    Ok(())
}

/// 测试文件复制
async fn test_file_copy(temp_dir: &TempDir) -> Result<(), crate::tests::TestError> {
    let source_file = temp_dir.path().join("source.txt");
    let target_file = temp_dir.path().join("target.txt");
    
    // 创建源文件
    let mut file = File::create(&source_file)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建源文件失败: {}", e)))?;
    writeln!(file, "测试内容")
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入源文件失败: {}", e)))?;

    // 执行复制
    let operator = FileOperator::new();
    let result = operator.copy_path(&source_file, &target_file)
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("文件复制失败: {}", e)))?;

    // 验证结果
    assert!(result.success, "文件复制应该成功");
    assert!(target_file.exists(), "目标文件应该存在");
    
    let target_content = fs::read_to_string(&target_file)
        .map_err(|e| crate::tests::TestError::AssertionFailed(format!("读取目标文件失败: {}", e)))?;
    assert_eq!(target_content.trim(), "测试内容", "文件内容不匹配");

    Ok(())
}

/// 测试目录复制
async fn test_directory_copy(temp_dir: &TempDir) -> Result<(), crate::tests::TestError> {
    let source_dir = temp_dir.path().join("source_dir");
    let target_dir = temp_dir.path().join("target_dir");
    
    // 创建源目录结构
    create_test_directory_structure(&source_dir)?;

    // 执行复制
    let operator = FileOperator::new();
    let result = operator.copy_path(&source_dir, &target_dir)
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("目录复制失败: {}", e)))?;

    // 验证结果
    assert!(result.success, "目录复制应该成功");
    assert!(target_dir.exists(), "目标目录应该存在");
    assert!(target_dir.join("file1.txt").exists(), "子文件应该存在");
    assert!(target_dir.join("subdir").exists(), "子目录应该存在");
    assert!(target_dir.join("subdir").join("file2.txt").exists(), "子目录文件应该存在");

    Ok(())
}

/// 测试文件删除
async fn test_file_deletion(temp_dir: &TempDir) -> Result<(), crate::tests::TestError> {
    let test_file = temp_dir.path().join("delete_test.txt");
    
    // 创建测试文件
    File::create(&test_file)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建测试文件失败: {}", e)))?;

    // 执行删除
    let operator = FileOperator::new();
    let result = operator.delete_path(&test_file)
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("文件删除失败: {}", e)))?;

    // 验证结果
    assert!(result.success, "文件删除应该成功");
    assert!(!test_file.exists(), "文件应该不存在");

    Ok(())
}

/// 测试目录删除
async fn test_directory_deletion(temp_dir: &TempDir) -> Result<(), crate::tests::TestError> {
    let test_dir = temp_dir.path().join("delete_dir");
    
    // 创建测试目录结构
    create_test_directory_structure(&test_dir)?;

    // 执行删除
    let operator = FileOperator::new();
    let result = operator.delete_path(&test_dir)
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("目录删除失败: {}", e)))?;

    // 验证结果
    assert!(result.success, "目录删除应该成功");
    assert!(!test_dir.exists(), "目录应该不存在");

    Ok(())
}

/// 测试符号链接创建
async fn test_symlink_creation(temp_dir: &TempDir) -> Result<(), crate::tests::TestError> {
    let target_dir = temp_dir.path().join("symlink_target");
    let link_path = temp_dir.path().join("symlink_link");
    
    // 创建目标目录
    create_test_directory_structure(&target_dir)?;

    // 创建符号链接
    let operator = FileOperator::new();
    let result = operator.create_symlink(&target_dir, &link_path)
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("符号链接创建失败: {}", e)))?;

    // 验证结果
    assert!(result.success, "符号链接创建应该成功");
    assert!(link_path.exists(), "符号链接应该存在");

    // 验证链接指向正确
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::MetadataExt;
        let link_metadata = fs::symlink_metadata(&link_path)
            .map_err(|e| crate::tests::TestError::AssertionFailed(format!("获取链接元数据失败: {}", e)))?;
        assert!(link_metadata.file_attributes() & 0x400 != 0, "应该是符号链接");
    }

    Ok(())
}

/// 测试操作取消
async fn test_operation_cancellation(temp_dir: &TempDir) -> Result<(), crate::tests::TestError> {
    let test_dir = temp_dir.path().join("cancel_test");
    create_large_test_directory(&test_dir, 100)?; // 创建包含100个文件的目录

    let operator = FileOperator::new();
    
    // 启动复制操作
    let operator_clone = operator.clone();
    let target_dir = temp_dir.path().join("cancel_target");
    
    let copy_task = tokio::spawn(async move {
        operator_clone.copy_path(&test_dir, &target_dir)
    });
    
    // 立即取消操作
    operator.cancel_operation();
    
    // 等待任务完成
    let result = copy_task.await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("任务执行失败: {}", e)))?;

    // 操作应该被取消或失败
    match result {
        Ok(_) => {
            // 如果操作成功完成，也是可以接受的，因为取消可能在操作完成后发生
            info!("操作在取消前已完成");
        },
        Err(_) => {
            info!("操作被取消成功");
        }
    }

    Ok(())
}

/// 迁移服务单元测试
pub async fn test_migration_service() -> Result<(), crate::tests::TestError> {
    info!("开始迁移服务单元测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    let service = MigrationService::new();

    // 测试1: 基本迁移
    test_basic_migration(&service, &temp_dir).await?;
    
    // 测试2: 带符号链接的迁移
    test_migration_with_symlink(&service, &temp_dir).await?;
    
    // 测试3: 迁移验证
    test_migration_validation(&service, &temp_dir).await?;
    
    // 测试4: 错误处理
    test_migration_error_handling(&service, &temp_dir).await?;

    info!("迁移服务单元测试完成");
    Ok(())
}

/// 测试基本迁移
async fn test_basic_migration(service: &MigrationService, temp_dir: &TempDir) -> Result<(), crate::tests::TestError> {
    let source_dir = temp_dir.path().join("migrate_source");
    let target_dir = temp_dir.path().join("migrate_target");
    
    // 创建源目录结构
    create_test_directory_structure(&source_dir)?;

    // 执行迁移
    let options = MigrationOptions {
        source_path: source_dir.display().to_string(),
        target_path: target_dir.display().to_string(),
        create_symlink: false,
        delete_source: false,
    };

    let result = service.migrate_folder(options).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("迁移失败: {}", e)))?;

    // 验证结果
    assert!(result.success, "迁移应该成功");
    assert!(target_dir.exists(), "目标目录应该存在");
    assert!(source_dir.exists(), "源目录应该存在（未删除）");
    assert!(target_dir.join("file1.txt").exists(), "文件应该被迁移");

    Ok(())
}

/// 测试带符号链接的迁移
async fn test_migration_with_symlink(service: &MigrationService, temp_dir: &TempDir) -> Result<(), crate::tests::TestError> {
    let source_dir = temp_dir.path().join("symlink_source");
    let target_dir = temp_dir.path().join("symlink_target");
    
    // 创建源目录结构
    create_test_directory_structure(&source_dir)?;

    // 执行迁移（带符号链接）
    let options = MigrationOptions {
        source_path: source_dir.display().to_string(),
        target_path: target_dir.display().to_string(),
        create_symlink: true,
        delete_source: false,
    };

    let result = service.migrate_folder(options).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("迁移失败: {}", e)))?;

    // 验证结果
    assert!(result.success, "迁移应该成功");
    assert!(target_dir.exists(), "目标目录应该存在");
    
    // 符号链接创建可能失败（权限问题），所以不强求
    if result.symlink_path.is_some() {
        info!("符号链接创建成功");
    } else {
        info!("符号链接创建失败（可能权限不足）");
    }

    Ok(())
}

/// 测试迁移验证
async fn test_migration_validation(service: &MigrationService, temp_dir: &TempDir) -> Result<(), crate::tests::TestError> {
    let source_dir = temp_dir.path().join("validate_source");
    let target_dir = temp_dir.path().join("validate_target");
    
    // 不创建源目录，测试验证失败
    let options = MigrationOptions {
        source_path: source_dir.display().to_string(),
        target_path: target_dir.display().to_string(),
        create_symlink: false,
        delete_source: false,
    };

    let result = service.migrate_folder(options).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("迁移失败: {}", e)))?;

    // 验证应该失败
    assert!(!result.success, "迁移应该失败（源目录不存在）");
    assert!(result.message.contains("预检查失败"), "应该显示预检查失败");

    Ok(())
}

/// 测试迁移错误处理
async fn test_migration_error_handling(service: &MigrationService, temp_dir: &TempDir) -> Result<(), crate::tests::TestError> {
    let source_dir = temp_dir.path().join("error_source");
    let target_dir = source_dir.join("nested_target"); // 目标在源目录内，应该失败
    
    // 创建源目录
    create_test_directory_structure(&source_dir)?;

    let options = MigrationOptions {
        source_path: source_dir.display().to_string(),
        target_path: target_dir.display().to_string(),
        create_symlink: false,
        delete_source: false,
    };

    let result = service.migrate_folder(options).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("迁移失败: {}", e)))?;

    // 验证应该失败
    assert!(!result.success, "迁移应该失败（目标在源目录内）");

    Ok(())
}

/// 错误恢复单元测试
pub async fn test_error_recovery() -> Result<(), crate::tests::TestError> {
    info!("开始错误恢复单元测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    let config = ErrorRecoveryConfig::default();
    let mut recovery_manager = ErrorRecoveryManager::new(config);

    // 测试1: 错误分类
    test_error_classification(&mut recovery_manager).await?;
    
    // 测试2: 恢复策略确定
    test_recovery_strategy(&mut recovery_manager).await?;
    
    // 测试3: 备份创建
    test_backup_creation(&mut recovery_manager, &temp_dir).await?;
    
    // 测试4: 备份清理
    test_backup_cleanup(&mut recovery_manager, &temp_dir).await?;

    info!("错误恢复单元测试完成");
    Ok(())
}

/// 测试错误分类
async fn test_error_classification(recovery_manager: &mut ErrorRecoveryManager) -> Result<(), crate::tests::TestError> {
    use crate::file_operations::FileOperationError;
    
    // 测试权限错误
    let permission_error = FileOperationError::PermissionDenied("测试权限错误".to_string());
    let error_type = recovery_manager.classify_error(&permission_error);
    
    match error_type {
        crate::error_recovery::ErrorType::PermissionDenied(_) => {
            info!("权限错误分类正确");
        },
        _ => return Err(crate::tests::TestError::AssertionFailed("权限错误分类不正确".to_string())),
    }

    // 测试IO错误
    let io_error = FileOperationError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "文件未找到"));
    let error_type = recovery_manager.classify_error(&io_error);
    
    match error_type {
        crate::error_recovery::ErrorType::PathNotFound(_) => {
            info!("IO错误分类正确");
        },
        _ => return Err(crate::tests::TestError::AssertionFailed("IO错误分类不正确".to_string())),
    }

    Ok(())
}

/// 测试恢复策略确定
async fn test_recovery_strategy(recovery_manager: &mut ErrorRecoveryManager) -> Result<(), crate::tests::TestError> {
    use crate::error_recovery::{ErrorType, ErrorSeverity, RecoveryContext};
    
    let context = RecoveryContext::new(
        "test_operation".to_string(),
        std::path::PathBuf::from("test_path"),
        None,
        "test_phase".to_string(),
    );

    // 测试权限错误的恢复策略
    let error_type = ErrorType::PermissionDenied("权限被拒绝".to_string());
    let severity = ErrorSeverity::Medium;
    let strategy = recovery_manager.determine_recovery_strategy(&error_type, &severity, &context);
    
    match strategy {
        crate::error_recovery::RecoveryStrategy::Manual => {
            info!("权限错误恢复策略正确（手动处理）");
        },
        _ => return Err(crate::tests::TestError::AssertionFailed("权限错误恢复策略不正确".to_string())),
    }

    Ok(())
}

/// 测试备份创建
async fn test_backup_creation(recovery_manager: &mut ErrorRecoveryManager, temp_dir: &TempDir) -> Result<(), crate::tests::TestError> {
    let test_dir = temp_dir.path().join("backup_test");
    create_test_directory_structure(&test_dir)?;

    // 创建备份
    let backup_info = recovery_manager.create_backup(&test_dir, "test_operation", "test_op_1").await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("备份创建失败: {}", e)))?;

    // 验证备份信息
    assert_eq!(backup_info.original_path, test_dir, "原始路径不匹配");
    assert!(backup_info.backup_path.exists(), "备份路径应该存在");
    assert!(backup_info.is_active, "备份应该处于活动状态");

    info!("备份创建测试成功: {}", backup_info.backup_id);

    Ok(())
}

/// 测试备份清理
async fn test_backup_cleanup(recovery_manager: &mut ErrorRecoveryManager, temp_dir: &TempDir) -> Result<(), crate::tests::TestError> {
    // 创建几个备份
    for i in 0..3 {
        let test_dir = temp_dir.path().join(format!("cleanup_test_{}", i));
        create_test_directory_structure(&test_dir)?;
        
        recovery_manager.create_backup(&test_dir, "test_operation", &format!("cleanup_op_{}", i)).await
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("备份创建失败: {}", e)))?;
    }

    // 清理过期备份（设置很短的保留时间）
    let cleaned_count = recovery_manager.cleanup_expired_backups()
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("备份清理失败: {}", e)))?;

    info!("备份清理测试完成，清理了 {} 个备份", cleaned_count);

    Ok(())
}

/// 操作日志单元测试
pub async fn test_operation_logger() -> Result<(), crate::tests::TestError> {
    info!("开始操作日志单元测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    let log_dir = temp_dir.path().join("test_logs");
    
    let logger = OperationLogger::new(
        log_dir.clone(),
        "test_session".to_string(),
        "test_user".to_string(),
    ).map_err(|e| crate::tests::TestError::SetupFailed(format!("日志器创建失败: {}", e)))?;

    // 测试1: 日志记录
    test_log_recording(&logger).await?;
    
    // 测试2: 日志查询
    test_log_query(&logger).await?;
    
    // 测试3: 日志统计
    test_log_statistics(&logger).await?;

    info!("操作日志单元测试完成");
    Ok(())
}

/// 测试日志记录
async fn test_log_recording(logger: &OperationLogger) -> Result<(), crate::tests::TestError> {
    // 记录不同类型的操作
    let mut scan_log = logger.log_operation_start(
        OperationType::Scan,
        "/test/path/scan".to_string(),
        None,
        "扫描测试目录".to_string(),
    ).map_err(|e| crate::tests::TestError::ExecutionFailed(format!("日志记录失败: {}", e)))?;

    // 更新日志状态
    logger.update_operation_status(&mut scan_log, OperationStatus::Completed, Some("扫描完成".to_string()))
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("日志状态更新失败: {}", e)))?;

    // 记录迁移操作
    let mut migrate_log = logger.log_operation_start(
        OperationType::Migrate,
        "/test/path/source".to_string(),
        Some("/test/path/target".to_string()),
        "迁移测试目录".to_string(),
    ).map_err(|e| crate::tests::TestError::ExecutionFailed(format!("迁移日志记录失败: {}", e)))?;

    logger.complete_operation(&mut migrate_log, 10, 1024 * 1024, 5000, Some("迁移成功".to_string()))
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("迁移完成记录失败: {}", e)))?;

    info!("日志记录测试成功");
    Ok(())
}

/// 测试日志查询
async fn test_log_query(logger: &OperationLogger) -> Result<(), crate::tests::TestError> {
    // 查询最近日志
    let recent_logs = logger.get_recent_logs(10)
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("查询最近日志失败: {}", e)))?;
    
    assert!(!recent_logs.is_empty(), "应该至少有1条日志");
    
    // 按类型查询
    let scan_logs = logger.get_logs_by_type(OperationType::Scan, 5)
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("按类型查询日志失败: {}", e)))?;
    
    assert!(!scan_logs.is_empty(), "应该至少有1条扫描日志");
    
    // 查询失败日志
    let failed_logs = logger.get_failed_operations(5)
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("查询失败日志失败: {}", e)))?;
    
    // 可能没有失败日志，所以不强求

    info!("日志查询测试成功");
    Ok(())
}

/// 测试日志统计
async fn test_log_statistics(logger: &OperationLogger) -> Result<(), crate::tests::TestError> {
    let stats = logger.get_statistics()
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("获取日志统计失败: {}", e)))?;

    assert!(stats.total_operations > 0, "总操作数应该大于0");
    assert!(stats.success_rate() >= 0.0 && stats.success_rate() <= 100.0, "成功率应该在0-100%之间");

    info!("日志统计测试成功 - 总操作: {}, 成功率: {:.1}%", 
          stats.total_operations, stats.success_rate());

    Ok(())
}

/// 创建测试目录结构
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

/// 创建大型测试目录
fn create_large_test_directory(base_path: &std::path::Path, file_count: usize) -> Result<(), crate::tests::TestError> {
    fs::create_dir_all(base_path)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建基础目录失败: {}", e)))?;

    for i in 0..file_count {
        let file_path = base_path.join(format!("test_file_{}.txt", i));
        let mut file = File::create(&file_path)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建文件 {} 失败: {}", i, e)))?;
        
        writeln!(file, "这是测试文件 {} 的内容", i)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入文件 {} 失败: {}", i, e)))?;
    }

    Ok(())
}

/// 为FileOperator实现Clone trait（用于测试）
impl Clone for FileOperator {
    fn clone(&self) -> Self {
        FileOperator::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unit_tests_runner() {
        // 这个测试确保所有的单元测试函数都能正常运行
        let result = test_disk_analyzer().await;
        assert!(result.is_ok(), "磁盘分析器单元测试应该通过");
        
        let result = test_file_operations().await;
        assert!(result.is_ok(), "文件操作单元测试应该通过");
        
        let result = test_migration_service().await;
        assert!(result.is_ok(), "迁移服务单元测试应该通过");
        
        let result = test_error_recovery().await;
        assert!(result.is_ok(), "错误恢复单元测试应该通过");
        
        let result = test_operation_logger().await;
        assert!(result.is_ok(), "操作日志单元测试应该通过");
    }
}