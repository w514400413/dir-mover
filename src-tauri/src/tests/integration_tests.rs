//! 集成测试模块
//!
//! 测试模块间的协作和交互

use crate::disk_analyzer::DiskAnalyzer;
use crate::file_operations::FileOperator;
use crate::migration_service::{MigrationService, MigrationOptions};
use crate::error_recovery::{ErrorRecoveryManager, ErrorRecoveryConfig, RecoveryContext};
use crate::operation_logger::{OperationLogger, OperationType, OperationStatus};
use crate::tests::test_utils::create_test_directory_structure;
use tempfile::TempDir;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use log::info;

/// 扫描和迁移集成测试
pub async fn test_scan_and_migrate() -> Result<(), crate::tests::TestError> {
    info!("开始扫描和迁移集成测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 创建复杂的测试目录结构
    let source_dir = temp_dir.path().join("integration_source");
    create_complex_test_structure(&source_dir)?;

    // 步骤1: 扫描源目录
    let analyzer = DiskAnalyzer::new();
    let scan_result = analyzer.scan_directory_async(&source_dir).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("扫描失败: {}", e)))?;

    info!("扫描完成 - 文件数: {}, 大小: {}", scan_result.file_count, scan_result.size);

    // 步骤2: 执行迁移
    let target_dir = temp_dir.path().join("integration_target");
    let service = MigrationService::new();
    
    let options = MigrationOptions {
        source_path: source_dir.display().to_string(),
        target_path: target_dir.display().to_string(),
        create_symlink: false,
        delete_source: false,
    };

    let migrate_result = service.migrate_folder(options).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("迁移失败: {}", e)))?;

    assert!(migrate_result.success, "迁移应该成功");

    // 步骤3: 验证迁移结果
    let target_scan_result = analyzer.scan_directory_async(&target_dir).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("目标扫描失败: {}", e)))?;

    // 验证文件数量和大小基本一致（允许小的差异）
    assert!(
        (scan_result.file_count as i32 - target_scan_result.file_count as i32).abs() <= 1,
        "文件数量应该基本一致"
    );
    
    assert!(
        (scan_result.size as i64 - target_scan_result.size as i64).abs() <= 1024, // 1KB差异
        "文件大小应该基本一致"
    );

    info!("扫描和迁移集成测试完成");
    Ok(())
}

/// 错误处理和恢复集成测试
pub async fn test_error_handling_and_recovery() -> Result<(), crate::tests::TestError> {
    info!("开始错误处理和恢复集成测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 初始化错误恢复管理器
    let config = ErrorRecoveryConfig::default();
    let mut recovery_manager = ErrorRecoveryManager::new(config);

    // 测试1: 权限错误恢复
    test_permission_error_recovery(&mut recovery_manager, &temp_dir).await?;
    
    // 测试2: 路径不存在错误恢复
    test_path_not_found_recovery(&mut recovery_manager, &temp_dir).await?;
    
    // 测试3: 磁盘空间不足恢复
    test_disk_space_recovery(&mut recovery_manager, &temp_dir).await?;

    info!("错误处理和恢复集成测试完成");
    Ok(())
}

/// 测试权限错误恢复
async fn test_permission_error_recovery(
    recovery_manager: &mut ErrorRecoveryManager,
    temp_dir: &TempDir
) -> Result<(), crate::tests::TestError> {
    use crate::file_operations::FileOperationError;
    
    // 创建一个模拟的权限错误
    let permission_error = FileOperationError::PermissionDenied("权限被拒绝".to_string());
    
    let context = RecoveryContext::new(
        "permission_test".to_string(),
        temp_dir.path().to_path_buf(),
        None,
        "test_phase".to_string(),
    );

    // 处理错误
    let result = recovery_manager.handle_error("perm_test_1", &permission_error, &context).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("错误处理失败: {}", e)))?;

    // 权限错误应该被标记为需要手动处理
    assert_eq!(result.recovery_type, crate::error_recovery::RecoveryType::Manual, "权限错误应该需要手动处理");

    info!("权限错误恢复测试完成");
    Ok(())
}

/// 测试路径不存在错误恢复
async fn test_path_not_found_recovery(
    recovery_manager: &mut ErrorRecoveryManager,
    temp_dir: &TempDir
) -> Result<(), crate::tests::TestError> {
    use crate::file_operations::FileOperationError;
    
    // 创建一个模拟的路径不存在错误
    let path_error = FileOperationError::PathNotFound("/non/existent/path".to_string());
    
    let context = RecoveryContext::new(
        "path_test".to_string(),
        temp_dir.path().to_path_buf(),
        None,
        "test_phase".to_string(),
    );

    // 处理错误
    let result = recovery_manager.handle_error("path_test_1", &path_error, &context).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("错误处理失败: {}", e)))?;

    // 路径不存在错误应该尝试重试
    assert_eq!(result.recovery_type, crate::error_recovery::RecoveryType::Retry, "路径错误应该尝试重试");

    info!("路径不存在错误恢复测试完成");
    Ok(())
}

/// 测试磁盘空间不足恢复
async fn test_disk_space_recovery(
    recovery_manager: &mut ErrorRecoveryManager,
    temp_dir: &TempDir
) -> Result<(), crate::tests::TestError> {
    use crate::error_recovery::{ErrorType, ErrorSeverity};
    
    // 创建一个模拟的磁盘空间不足错误
    let disk_error = ErrorType::DiskSpaceInsufficient("磁盘空间不足".to_string());
    let severity = ErrorSeverity::Critical;
    
    let context = RecoveryContext::new(
        "disk_test".to_string(),
        temp_dir.path().to_path_buf(),
        None,
        "test_phase".to_string(),
    );

    // 确定恢复策略
    let strategy = recovery_manager.determine_recovery_strategy(&disk_error, &severity, &context);
    
    // 磁盘空间不足应该中止操作
    assert_eq!(strategy, crate::error_recovery::RecoveryStrategy::Abort, "磁盘空间不足应该中止操作");

    info!("磁盘空间不足恢复测试完成");
    Ok(())
}

/// 操作日志集成测试
pub async fn test_operation_logging() -> Result<(), crate::tests::TestError> {
    info!("开始操作日志集成测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    let log_dir = temp_dir.path().join("integration_logs");
    
    let logger = OperationLogger::new(
        log_dir.clone(),
        "integration_session".to_string(),
        "integration_user".to_string(),
    ).map_err(|e| crate::tests::TestError::SetupFailed(format!("日志器创建失败: {}", e)))?;

    // 测试1: 完整操作生命周期记录
    test_complete_operation_lifecycle(&logger).await?;
    
    // 测试2: 并发操作记录
    test_concurrent_operation_logging(logger.clone()).await?;
    
    // 测试3: 错误操作记录
    test_error_operation_logging(&logger).await?;

    info!("操作日志集成测试完成");
    Ok(())
}

/// 测试完整操作生命周期记录
async fn test_complete_operation_lifecycle(logger: &OperationLogger) -> Result<(), crate::tests::TestError> {
    // 记录操作开始
    let mut log = logger.log_operation_start(
        OperationType::Migrate,
        "/source/path".to_string(),
        Some("/target/path".to_string()),
        "完整迁移测试".to_string(),
    ).map_err(|e| crate::tests::TestError::ExecutionFailed(format!("操作开始记录失败: {}", e)))?;

    // 模拟操作进行中
    logger.update_operation_status(&mut log, OperationStatus::InProgress, Some("正在复制文件...".to_string()))
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("状态更新失败: {}", e)))?;

    // 模拟操作完成
    logger.complete_operation(&mut log, 100, 1024 * 1024 * 10, 30000, Some("迁移成功完成".to_string()))
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("操作完成记录失败: {}", e)))?;

    // 验证日志记录
    let recent_logs = logger.get_recent_logs(1)
        .map_err(|e| crate::tests::TestError::AssertionFailed(format!("查询日志失败: {}", e)))?;

    assert!(!recent_logs.is_empty(), "应该能找到记录的日志");
    
    let latest_log = &recent_logs[0];
    assert_eq!(latest_log.operation_type, OperationType::Migrate, "操作类型不匹配");
    assert_eq!(latest_log.status, OperationStatus::Completed, "操作状态不匹配");
    assert_eq!(latest_log.file_count, Some(100), "文件数量不匹配");
    assert_eq!(latest_log.total_size, Some(1024 * 1024 * 10), "总大小不匹配");

    info!("完整操作生命周期记录测试完成");
    Ok(())
}

/// 测试并发操作记录
async fn test_concurrent_operation_logging(logger: OperationLogger) -> Result<(), crate::tests::TestError> {
    use tokio::task;
    use std::sync::Arc;
    
    // 创建多个并发操作 - 每个任务使用独立的 logger
    let mut handles = vec![];
    
    for i in 0..5 {
        let handle = task::spawn(async move {
            // 为每个并发任务创建独立的 logger
            let temp_dir = tempfile::TempDir::new().unwrap();
            let task_logger = OperationLogger::new(
                temp_dir.path().to_path_buf(),
                format!("concurrent_session_{}", i),
                "test_user".to_string(),
            ).unwrap();
            
            let mut log = task_logger.log_operation_start(
                OperationType::Scan,
                format!("/test/path/{}", i),
                None,
                format!("并发扫描测试 {}", i),
            ).unwrap();
            
            task_logger.complete_operation(&mut log, 10 * (i + 1), 1024 * (i + 1), 1000, None).unwrap();
            
            // 返回操作结果用于验证
            (task_logger, format!("/test/path/{}", i))
        });
        
        handles.push(handle);
    }
    
    // 等待所有操作完成并收集结果
    let mut completed_tasks = 0;
    for handle in handles {
        let (task_logger, path) = handle.await
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("并发任务失败: {}", e)))?;
        
        // 验证每个任务的操作都被记录
        let logs = task_logger.get_recent_logs(1)
            .map_err(|e| crate::tests::TestError::AssertionFailed(format!("查询任务日志失败: {}", e)))?;
        
        if !logs.is_empty() && logs[0].source_path == path {
            completed_tasks += 1;
        }
    }
    
    assert!(completed_tasks >= 5, "应该至少有5个并发操作成功完成");

    info!("并发操作记录测试完成");
    Ok(())
}

/// 测试错误操作记录
async fn test_error_operation_logging(logger: &OperationLogger) -> Result<(), crate::tests::TestError> {
    // 记录一个失败的操作
    let mut log = logger.log_operation_start(
        OperationType::Delete,
        "/non/existent/path".to_string(),
        None,
        "删除不存在的路径".to_string(),
    ).map_err(|e| crate::tests::TestError::ExecutionFailed(format!("操作开始记录失败: {}", e)))?;

    // 模拟操作失败
    logger.fail_operation(&mut log, "路径不存在".to_string(), Some("无法删除不存在的路径".to_string()))
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("操作失败记录失败: {}", e)))?;

    // 验证错误日志
    let failed_logs = logger.get_failed_operations(1)
        .map_err(|e| crate::tests::TestError::AssertionFailed(format!("查询失败日志失败: {}", e)))?;
    
    assert!(!failed_logs.is_empty(), "应该能找到失败的日志");
    
    let failed_log = &failed_logs[0];
    assert_eq!(failed_log.status, OperationStatus::Failed, "操作状态应该是失败");
    assert!(failed_log.error_message.is_some(), "应该有错误信息");
    assert_eq!(failed_log.error_message.as_ref().unwrap(), "路径不存在", "错误信息不匹配");

    info!("错误操作记录测试完成");
    Ok(())
}

/// 路径验证集成测试
pub async fn test_path_validation() -> Result<(), crate::tests::TestError> {
    info!("开始路径验证集成测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    let operator = FileOperator::new();

    // 测试1: 有效路径验证
    test_valid_path_validation(&operator, &temp_dir).await?;
    
    // 测试2: 无效路径验证
    test_invalid_path_validation(&operator, &temp_dir).await?;
    
    // 测试3: 危险路径验证
    test_dangerous_path_validation(&operator, &temp_dir).await?;

    info!("路径验证集成测试完成");
    Ok(())
}

/// 测试有效路径验证
async fn test_valid_path_validation(operator: &FileOperator, temp_dir: &TempDir) -> Result<(), crate::tests::TestError> {
    let source_dir = temp_dir.path().join("valid_source");
    let target_dir = temp_dir.path().join("valid_target");
    
    // 创建源目录
    create_test_directory_structure(&source_dir)?;

    // 验证路径
    let (valid, message) = operator.validate_migration_path(&source_dir, &target_dir)
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("路径验证失败: {}", e)))?;

    assert!(valid, "有效路径应该通过验证");
    assert_eq!(message, "路径验证通过", "验证消息不匹配");

    info!("有效路径验证测试完成");
    Ok(())
}

/// 测试无效路径验证
async fn test_invalid_path_validation(operator: &FileOperator, temp_dir: &TempDir) -> Result<(), crate::tests::TestError> {
    let source_dir = temp_dir.path().join("invalid_source");
    let target_dir = temp_dir.path().join("invalid_target");
    
    // 不创建源目录，使其不存在

    // 验证路径
    let (valid, message) = operator.validate_migration_path(&source_dir, &target_dir)
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("路径验证失败: {}", e)))?;

    assert!(!valid, "无效路径应该验证失败");
    assert!(message.contains("源路径不存在"), "应该提示源路径不存在");

    info!("无效路径验证测试完成");
    Ok(())
}

/// 测试危险路径验证
async fn test_dangerous_path_validation(operator: &FileOperator, temp_dir: &TempDir) -> Result<(), crate::tests::TestError> {
    let source_dir = temp_dir.path().join("dangerous_source");
    let target_dir = source_dir.join("nested_target"); // 目标在源目录内
    
    // 创建源目录
    create_test_directory_structure(&source_dir)?;

    // 验证路径
    let (valid, message) = operator.validate_migration_path(&source_dir, &target_dir)
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("路径验证失败: {}", e)))?;

    assert!(!valid, "危险路径应该验证失败");
    assert!(message.contains("不能将目录迁移到其子目录中"), "应该提示循环迁移风险");

    info!("危险路径验证测试完成");
    Ok(())
}

/// 备份和回滚集成测试
pub async fn test_backup_and_rollback() -> Result<(), crate::tests::TestError> {
    info!("开始备份和回滚集成测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 初始化错误恢复管理器
    let config = ErrorRecoveryConfig::default();
    let mut recovery_manager = ErrorRecoveryManager::new(config);

    // 测试1: 备份创建和验证
    test_backup_creation_and_verification(&mut recovery_manager, &temp_dir).await?;
    
    // 测试2: 回滚操作
    test_rollback_operation(&mut recovery_manager, &temp_dir).await?;
    
    // 测试3: 备份清理
    test_backup_cleanup_integration(&mut recovery_manager, &temp_dir).await?;

    info!("备份和回滚集成测试完成");
    Ok(())
}

/// 测试备份创建和验证
async fn test_backup_creation_and_verification(
    recovery_manager: &mut ErrorRecoveryManager,
    temp_dir: &TempDir
) -> Result<(), crate::tests::TestError> {
    let test_dir = temp_dir.path().join("backup_integration_test");
    create_test_directory_structure(&test_dir)?;

    // 创建备份
    let backup_info = recovery_manager.create_backup(&test_dir, "integration_test", "backup_op_1").await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("备份创建失败: {}", e)))?;

    // 验证备份
    assert!(backup_info.backup_path.exists(), "备份路径应该存在");
    
    // 比较原始目录和备份目录的内容
    let original_files = count_files_in_directory(&test_dir)?;
    let backup_files = count_files_in_directory(&backup_info.backup_path)?;
    
    assert_eq!(original_files, backup_files, "备份文件数量应该与原始目录相同");

    info!("备份创建和验证测试完成");
    Ok(())
}

/// 测试回滚操作
async fn test_rollback_operation(
    recovery_manager: &mut ErrorRecoveryManager,
    temp_dir: &TempDir
) -> Result<(), crate::tests::TestError> {
    let original_dir = temp_dir.path().join("rollback_original");
    create_test_directory_structure(&original_dir)?;

    // 创建备份
    let backup_info = recovery_manager.create_backup(&original_dir, "rollback_test", "rollback_op_1").await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("备份创建失败: {}", e)))?;

    // 修改原始目录（模拟失败的迁移）
    fs::remove_file(original_dir.join("file1.txt"))
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("删除文件失败: {}", e)))?;

    // 执行回滚
    let context = RecoveryContext::new(
        "rollback_test".to_string(),
        original_dir.clone(),
        None,
        "rollback_phase".to_string(),
    );

    use crate::file_operations::FileOperationError;
    let mock_error = FileOperationError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "模拟错误"));
    
    let result = recovery_manager.handle_error("rollback_test_1", &mock_error, &context).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("回滚处理失败: {}", e)))?;

    // 验证回滚结果
    assert!(result.success, "回滚应该成功");
    assert!(original_dir.join("file1.txt").exists(), "文件应该被恢复");

    info!("回滚操作测试完成");
    Ok(())
}

/// 测试备份清理集成
async fn test_backup_cleanup_integration(
    recovery_manager: &mut ErrorRecoveryManager,
    temp_dir: &TempDir
) -> Result<(), crate::tests::TestError> {
    // 创建多个备份
    for i in 0..3 {
        let test_dir = temp_dir.path().join(format!("cleanup_test_{}", i));
        create_test_directory_structure(&test_dir)?;
        
        recovery_manager.create_backup(&test_dir, "cleanup_test", &format!("cleanup_op_{}", i)).await
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("备份创建失败: {}", e)))?;
    }

    // 获取备份统计
    let stats_before = recovery_manager.get_recovery_statistics();
    info!("清理前备份统计: {:?}", stats_before);

    // 执行清理（设置很短的保留时间）
    let cleaned_count = recovery_manager.cleanup_expired_backups()
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("备份清理失败: {}", e)))?;

    info!("清理了 {} 个过期备份", cleaned_count);

    // 获取清理后的统计
    let stats_after = recovery_manager.get_recovery_statistics();
    info!("清理后备份统计: {:?}", stats_after);

    Ok(())
}

/// 创建复杂的测试目录结构
fn create_complex_test_structure(base_path: &std::path::Path) -> Result<(), crate::tests::TestError> {
    // 创建多层目录结构
    let dirs = vec![
        "level1",
        "level1/level2a",
        "level1/level2b",
        "level1/level2a/level3",
        "level1/level2b/level3a",
        "level1/level2b/level3b",
    ];

    for dir in dirs {
        let dir_path = base_path.join(dir);
        fs::create_dir_all(&dir_path)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建目录 {} 失败: {}", dir, e)))?;
        
        // 在每个目录中创建文件
        for i in 0..3 {
            let file_path = dir_path.join(format!("file_{}.txt", i));
            let mut file = File::create(&file_path)
                .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建文件 {} 在 {} 失败: {}", i, dir, e)))?;
            
            writeln!(file, "这是 {} 目录中的文件 {} 的内容", dir, i)
                .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入文件 {} 在 {} 失败: {}", i, dir, e)))?;
        }
    }

    // 在根目录创建一些文件
    for i in 0..5 {
        let file_path = base_path.join(format!("root_file_{}.txt", i));
        let mut file = File::create(&file_path)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建根文件 {} 失败: {}", i, e)))?;
        
        writeln!(file, "这是根目录中的文件 {} 的内容", i)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入根文件 {} 失败: {}", i, e)))?;
    }

    Ok(())
}

/// 计算目录中的文件数量
fn count_files_in_directory(path: &std::path::Path) -> Result<usize, crate::tests::TestError> {
    let mut count = 0;
    
    if path.is_dir() {
        let entries = fs::read_dir(path)
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("读取目录失败: {}", e)))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| crate::tests::TestError::ExecutionFailed(format!("读取目录项失败: {}", e)))?;
            let entry_path = entry.path();
            
            if entry_path.is_file() {
                count += 1;
            } else if entry_path.is_dir() {
                count += count_files_in_directory(&entry_path)?;
            }
        }
    } else if path.is_file() {
        count = 1;
    }
    
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integration_tests_runner() {
        // 这个测试确保所有的集成测试函数都能正常运行
        let result = test_scan_and_migrate().await;
        assert!(result.is_ok(), "扫描和迁移集成测试应该通过");
        
        let result = test_error_handling_and_recovery().await;
        assert!(result.is_ok(), "错误处理和恢复集成测试应该通过");
        
        let result = test_operation_logging().await;
        assert!(result.is_ok(), "操作日志集成测试应该通过");
        
        let result = test_path_validation().await;
        assert!(result.is_ok(), "路径验证集成测试应该通过");
        
        let result = test_backup_and_rollback().await;
        assert!(result.is_ok(), "备份和回滚集成测试应该通过");
    }
}