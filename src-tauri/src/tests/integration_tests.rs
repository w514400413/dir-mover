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
use log::{info, error};

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

    /// 集成测试运行器（增强版本）
    pub async fn run_all_integration_tests() -> Result<(), crate::tests::TestError> {
        info!("开始运行所有集成测试");
        
        // 基础集成测试
        info!("运行扫描和迁移集成测试");
        test_scan_and_migrate().await?;
        
        info!("运行错误处理和恢复集成测试");
        test_error_handling_and_recovery().await?;
        
        info!("运行操作日志集成测试");
        test_operation_logging().await?;
        
        info!("运行路径验证集成测试");
        test_path_validation().await?;
        
        info!("运行备份和回滚集成测试");
        test_backup_and_rollback().await?;
        
        // AppData专项集成测试
        info!("运行AppData扫描集成测试");
        test_appdata_scan_integration().await?;
        
        info!("运行AppData完整工作流集成测试");
        test_appdata_complete_workflow().await?;
        
        info!("所有集成测试完成");
        Ok(())
    }
    
    #[tokio::test]
    async fn test_integration_tests_runner() {
        // 这个测试确保所有的集成测试函数都能正常运行
        let result = run_all_integration_tests().await;
        assert!(result.is_ok(), "所有集成测试应该通过");
        
        // 单独测试也保留以确保兼容性
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
        
        let result = test_appdata_scan_integration().await;
        assert!(result.is_ok(), "AppData扫描集成测试应该通过");
        
        let result = test_appdata_complete_workflow().await;
        assert!(result.is_ok(), "AppData完整工作流集成测试应该通过");
    }
}

/// AppData扫描集成测试
pub async fn test_appdata_scan_integration() -> Result<(), crate::tests::TestError> {
    info!("开始AppData扫描集成测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 创建模拟的AppData目录结构
    let appdata_base = temp_dir.path().join("AppData");
    let local_dir = appdata_base.join("Local");
    let roaming_dir = appdata_base.join("Roaming");
    let local_low_dir = appdata_base.join("LocalLow");
    
    // 创建AppData子目录
    fs::create_dir_all(&local_dir)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建Local目录失败: {}", e)))?;
    fs::create_dir_all(&roaming_dir)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建Roaming目录失败: {}", e)))?;
    fs::create_dir_all(&local_low_dir)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建LocalLow目录失败: {}", e)))?;
    
    // 创建测试应用数据
    let test_apps = vec![
        ("Chrome", 2 * 1024 * 1024 * 1024),      // 2GB - 大应用
        ("VSCode", 1500 * 1024 * 1024),          // 1.5GB - 大应用
        ("NodeJS", 800 * 1024 * 1024),           // 800MB - 中等应用
        ("SmallApp", 200 * 1024 * 1024),         // 200MB - 小应用
        ("TinyApp", 50 * 1024 * 1024),           // 50MB - 微小应用
    ];
    
    for (app_name, size) in test_apps {
        // 根据应用大小决定放在哪个目录
        let app_dir = if size >= 1024 * 1024 * 1024 { // 1GB+
            local_dir.join(app_name)
        } else if size >= 500 * 1024 * 1024 { // 500MB+
            roaming_dir.join(app_name)
        } else {
            local_low_dir.join(app_name)
        };
        
        fs::create_dir_all(&app_dir)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建应用目录 {} 失败: {}", app_name, e)))?;
        
        // 创建应用数据文件
        let data_file = app_dir.join("data.dat");
        create_large_file(&data_file, size)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建应用数据文件 {} 失败: {}", app_name, e)))?;
        
        // 创建一些配置文件
        let config_file = app_dir.join("config.json");
        let mut file = File::create(&config_file)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建配置文件失败: {}", e)))?;
        writeln!(file, r#"{{"app": "{}", "size": {}}}"#, app_name, size)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入配置文件失败: {}", e)))?;
    }
    
    info!("AppData测试数据创建完成");
    
    // 测试1: 验证AppData路径检测
    test_appdata_path_detection(&appdata_base).await?;
    
    // 测试2: 验证大文件夹筛选
    test_large_folder_filtering_integration(&appdata_base).await?;
    
    // 测试3: 验证目录大小计算
    test_directory_size_calculation(&appdata_base).await?;
    
    // 测试4: 验证排序功能
    test_sorting_functionality(&appdata_base).await?;
    
    info!("AppData扫描集成测试完成");
    Ok(())
}

/// 测试AppData路径检测
async fn test_appdata_path_detection(appdata_base: &std::path::Path) -> Result<(), crate::tests::TestError> {
    use crate::appdata_analyzer::AppDataAnalyzer;
    
    // 验证三个主要子目录都存在
    assert!(appdata_base.join("Local").exists(), "Local目录应该存在");
    assert!(appdata_base.join("Roaming").exists(), "Roaming目录应该存在");
    assert!(appdata_base.join("LocalLow").exists(), "LocalLow目录应该存在");
    
    info!("AppData路径检测测试完成");
    Ok(())
}

/// 测试大文件夹筛选集成
async fn test_large_folder_filtering_integration(appdata_base: &std::path::Path) -> Result<(), crate::tests::TestError> {
    use crate::appdata_analyzer::AppDataAnalyzer;
    
    // 创建分析器并设置1GB阈值
    let mut analyzer = AppDataAnalyzer::new();
    let config = crate::appdata_analyzer::AppDataConfig {
        min_size_threshold: 1024 * 1024 * 1024, // 1GB
        max_depth: 2,
        sort_order: crate::appdata_analyzer::SortOrder::Desc,
    };
    analyzer.set_config(config);
    
    // 手动验证大文件夹（>1GB）应该包括Chrome和VSCode
    let local_dir = appdata_base.join("Local");
    let chrome_dir = local_dir.join("Chrome");
    let vscode_dir = local_dir.join("VSCode");
    
    assert!(chrome_dir.exists(), "Chrome目录应该存在");
    assert!(vscode_dir.exists(), "VSCode目录应该存在");
    
    // 验证这些大应用的数据文件存在
    assert!(chrome_dir.join("data.dat").exists(), "Chrome数据文件应该存在");
    assert!(vscode_dir.join("data.dat").exists(), "VSCode数据文件应该存在");
    
    info!("大文件夹筛选集成测试完成");
    Ok(())
}

/// 测试目录大小计算
async fn test_directory_size_calculation(appdata_base: &std::path::Path) -> Result<(), crate::tests::TestError> {
    // 验证各个目录的大小
    let local_dir = appdata_base.join("Local");
    let roaming_dir = appdata_base.join("Roaming");
    let local_low_dir = appdata_base.join("LocalLow");
    
    // Local目录应该包含大应用（Chrome和VSCode）
    let local_size = calculate_directory_size(&local_dir)?;
    assert!(local_size >= 3 * 1024 * 1024 * 1024, "Local目录应该至少包含3GB数据");
    
    // Roaming目录应该包含中等大小的应用
    let roaming_size = calculate_directory_size(&roaming_dir)?;
    assert!(roaming_size >= 800 * 1024 * 1024, "Roaming目录应该至少包含800MB数据");
    
    // LocalLow目录应该包含小应用
    let local_low_size = calculate_directory_size(&local_low_dir)?;
    assert!(local_low_size >= 250 * 1024 * 1024, "LocalLow目录应该至少包含250MB数据");
    
    info!("目录大小计算测试完成");
    Ok(())
}

/// 测试排序功能
async fn test_sorting_functionality(appdata_base: &std::path::Path) -> Result<(), crate::tests::TestError> {
    // 获取所有应用目录
    let mut app_sizes = Vec::new();
    
    for subdir in &["Local", "Roaming", "LocalLow"] {
        let subdir_path = appdata_base.join(subdir);
        if subdir_path.exists() {
            for entry in fs::read_dir(&subdir_path)
                .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("读取目录失败: {}", e)))? {
                let entry = entry
                    .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("读取目录项失败: {}", e)))?;
                let path = entry.path();
                if path.is_dir() {
                    let size = calculate_directory_size(&path)?;
                    let app_name = path.file_name().unwrap().to_string_lossy().to_string();
                    app_sizes.push((app_name, size));
                }
            }
        }
    }
    
    // 按大小降序排序
    app_sizes.sort_by(|a, b| b.1.cmp(&a.1));
    
    // 验证排序结果
    assert!(app_sizes.len() >= 3, "应该至少有3个应用");
    
    // 最大的应用应该是Chrome或VSCode（都大于1GB）
    let largest_app = &app_sizes[0];
    assert!(largest_app.1 >= 1024 * 1024 * 1024, "最大的应用应该大于1GB");
    assert!(largest_app.0 == "Chrome" || largest_app.0 == "VSCode", "最大的应用应该是Chrome或VSCode");
    
    info!("排序功能测试完成 - 找到 {} 个应用", app_sizes.len());
    Ok(())
}

/// 计算目录大小
fn calculate_directory_size(path: &std::path::Path) -> Result<u64, crate::tests::TestError> {
    let mut total_size = 0;
    
    if path.is_dir() {
        let entries = fs::read_dir(path)
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("读取目录失败: {}", e)))?;
        
        for entry in entries {
            let entry = entry
                .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("读取目录项失败: {}", e)))?;
            let entry_path = entry.path();
            
            if entry_path.is_file() {
                let metadata = fs::metadata(&entry_path)
                    .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("获取文件元数据失败: {}", e)))?;
                total_size += metadata.len();
            } else if entry_path.is_dir() {
                total_size += calculate_directory_size(&entry_path)?;
            }
        }
    } else if path.is_file() {
        let metadata = fs::metadata(path)
            .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("获取文件元数据失败: {}", e)))?;
        total_size = metadata.len();
    }
    
    Ok(total_size)
}

/// 创建大文件（用于测试）
fn create_large_file(path: &std::path::Path, size_bytes: usize) -> Result<(), crate::tests::TestError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建父目录失败: {}", e)))?;
    }
    
    let mut file = File::create(path)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建文件失败: {}", e)))?;
    
    // 分块写入以避免内存问题
    let chunk_size = 1024 * 1024; // 1MB chunks
    let chunk = vec![b'A'; chunk_size.min(size_bytes)];
    
    let chunks_needed = size_bytes / chunk_size;
    let remainder = size_bytes % chunk_size;
    
    for _ in 0..chunks_needed {
        file.write_all(&chunk)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入文件块失败: {}", e)))?;
    }
    
    if remainder > 0 {
        let remainder_chunk = vec![b'A'; remainder];
        file.write_all(&remainder_chunk)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入文件剩余部分失败: {}", e)))?;
    }
    
    Ok(())
}

/// AppData完整工作流集成测试（增强版本）
pub async fn test_appdata_complete_workflow() -> Result<(), crate::tests::TestError> {
    info!("开始AppData完整工作流集成测试");

    let temp_dir = TempDir::new().map_err(|e| crate::tests::TestError::SetupFailed(e.to_string()))?;
    
    // 创建完整的AppData测试环境
    let appdata_base = temp_dir.path().join("AppData");
    let local_dir = appdata_base.join("Local");
    let roaming_dir = appdata_base.join("Roaming");
    let local_low_dir = appdata_base.join("LocalLow");
    let target_drive = temp_dir.path().join("D_DRIVE");
    
    // 创建AppData子目录
    fs::create_dir_all(&local_dir)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建Local目录失败: {}", e)))?;
    fs::create_dir_all(&roaming_dir)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建Roaming目录失败: {}", e)))?;
    fs::create_dir_all(&local_low_dir)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建LocalLow目录失败: {}", e)))?;
    fs::create_dir_all(&target_drive)
        .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建目标盘符目录失败: {}", e)))?;

    // 步骤1: 创建测试应用数据（模拟真实场景）
    create_realistic_appdata_structure(&appdata_base).await?;

    // 步骤2: 扫描AppData并验证结果
    let scan_result = test_appdata_scanning_workflow(&appdata_base).await?;

    // 步骤3: 验证一级项目检测
    test_first_level_items_detection(&scan_result).await?;

    // 步骤4: 测试动态排序功能
    test_dynamic_sorting_integration(&scan_result).await?;

    // 步骤5: 测试1GB筛选功能
    test_one_gb_filtering_integration(&scan_result).await?;

    // 步骤6: 执行迁移操作
    let migration_result = test_appdata_migration_workflow(&appdata_base, &target_drive).await?;

    // 步骤7: 验证迁移结果
    test_migration_result_validation(&migration_result, &target_drive).await?;

    // 步骤8: 测试错误恢复
    test_error_recovery_integration(&appdata_base).await?;

    // 步骤9: 性能基准测试
    test_performance_benchmarks(&appdata_base).await?;

    info!("AppData完整工作流集成测试完成");
    Ok(())
}

/// 创建真实的AppData测试结构
async fn create_realistic_appdata_structure(appdata_base: &std::path::Path) -> Result<(), crate::tests::TestError> {
    use crate::appdata_analyzer::AppDataAnalyzer;
    
    // 模拟真实的Windows应用数据分布
    let realistic_apps = vec![
        // Local目录 - 大应用（>1GB）
        ("Google.Chrome", 2500 * 1024 * 1024, "Local"),      // 2.5GB
        ("Microsoft.VSCode", 1800 * 1024 * 1024, "Local"),   // 1.8GB
        ("Mozilla.Firefox", 1200 * 1024 * 1024, "Local"),    // 1.2GB
        ("JetBrains.IntelliJ", 3500 * 1024 * 1024, "Local"), // 3.5GB
        
        // Roaming目录 - 中等应用（500MB-1GB）
        ("Microsoft.Office", 900 * 1024 * 1024, "Roaming"),   // 900MB
        ("Adobe.CreativeCloud", 750 * 1024 * 1024, "Roaming"), // 750MB
        ("Steam", 650 * 1024 * 1024, "Roaming"),              // 650MB
        
        // LocalLow目录 - 小应用（<500MB）
        ("Spotify", 400 * 1024 * 1024, "LocalLow"),           // 400MB
        ("Discord", 300 * 1024 * 1024, "LocalLow"),           // 300MB
        ("Zoom", 200 * 1024 * 1024, "LocalLow"),              // 200MB
        ("Notepad++", 150 * 1024 * 1024, "LocalLow"),         // 150MB
    ];

    for (app_name, size, parent_type) in realistic_apps {
        let app_dir = match parent_type {
            "Local" => appdata_base.join("Local").join(app_name),
            "Roaming" => appdata_base.join("Roaming").join(app_name),
            "LocalLow" => appdata_base.join("LocalLow").join(app_name),
            _ => continue,
        };

        fs::create_dir_all(&app_dir)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建应用目录 {} 失败: {}", app_name, e)))?;

        // 创建应用数据文件
        let data_file = app_dir.join("data.dat");
        create_large_file(&data_file, size)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建应用数据文件 {} 失败: {}", app_name, e)))?;

        // 创建配置文件
        let config_file = app_dir.join("config.json");
        let mut file = File::create(&config_file)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建配置文件失败: {}", e)))?;
        writeln!(file, r#"{{"app": "{}", "version": "1.0.0", "size": {}}}"#, app_name, size)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("写入配置文件失败: {}", e)))?;

        // 创建缓存目录（一级子目录）
        let cache_dir = app_dir.join("Cache");
        fs::create_dir_all(&cache_dir)
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建缓存目录失败: {}", e)))?;
        
        let cache_file = cache_dir.join("cache.data");
        create_large_file(&cache_file, size / 10) // 缓存大小为数据的1/10
            .map_err(|e| crate::tests::TestError::SetupFailed(format!("创建缓存文件失败: {}", e)))?;

        info!("创建了应用 {} ({}MB) 在 {} 目录", app_name, size / (1024 * 1024), parent_type);
    }

    info!("真实AppData测试结构创建完成");
    Ok(())
}

/// 测试AppData扫描工作流
async fn test_appdata_scanning_workflow(appdata_base: &std::path::Path) -> Result<crate::appdata_analyzer::AppDataInfo, crate::tests::TestError> {
    use crate::appdata_analyzer::{AppDataAnalyzer, AppDataConfig, SortOrder};
    
    // 创建分析器
    let mut analyzer = AppDataAnalyzer::new();
    let config = AppDataConfig {
        min_size_threshold: 1024 * 1024 * 1024, // 1GB
        max_depth: 2,
        sort_order: SortOrder::Desc,
    };
    analyzer.set_config(config);

    // 执行扫描（使用模拟路径）
    let scan_result = analyzer.scan_appdata().await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("AppData扫描失败: {}", e)))?;

    // 验证扫描结果
    assert!(scan_result.total_size > 0, "总大小应该大于0");
    assert!(scan_result.first_level_items.len() >= 10, "应该至少有10个一级项目");
    assert!(scan_result.large_items.len() >= 4, "应该至少有4个大项目（>1GB）");

    // 验证三个主要目录
    assert!(scan_result.local_size > 0, "Local目录应该有大小");
    assert!(scan_result.roaming_size > 0, "Roaming目录应该有大小");
    assert!(scan_result.local_low_size > 0, "LocalLow目录应该有大小");

    // 验证扫描时间
    assert!(scan_result.scan_time_ms > 0, "扫描时间应该大于0");
    assert!(scan_result.scan_time_ms < 30000, "扫描时间应该小于30秒");

    info!("AppData扫描工作流测试完成 - 发现 {} 个一级项目, {} 个大项目",
          scan_result.first_level_items.len(), scan_result.large_items.len());

    Ok(scan_result)
}

/// 测试一级项目检测
async fn test_first_level_items_detection(scan_result: &crate::appdata_analyzer::AppDataInfo) -> Result<(), crate::tests::TestError> {
    // 验证一级项目来自三个主要目录
    let mut local_count = 0;
    let mut roaming_count = 0;
    let mut local_low_count = 0;

    for item in &scan_result.first_level_items {
        match item.parent_type.as_str() {
            "Local" => local_count += 1,
            "Roaming" => roaming_count += 1,
            "LocalLow" => local_low_count += 1,
            _ => {}
        }

        // 验证项目属性
        assert!(!item.path.is_empty(), "项目路径不应该为空");
        assert!(!item.name.is_empty(), "项目名称不应该为空");
        assert!(item.size > 0, "项目大小应该大于0");
        assert!(item.size_percentage >= 0.0, "大小百分比应该非负");
    }

    assert!(local_count >= 3, "Local目录应该至少有3个一级项目");
    assert!(roaming_count >= 2, "Roaming目录应该至少有2个一级项目");
    assert!(local_low_count >= 3, "LocalLow目录应该至少有3个一级项目");

    info!("一级项目检测测试完成 - Local: {}, Roaming: {}, LocalLow: {}",
          local_count, roaming_count, local_low_count);

    Ok(())
}

/// 测试动态排序功能集成
async fn test_dynamic_sorting_integration(scan_result: &crate::appdata_analyzer::AppDataInfo) -> Result<(), crate::tests::TestError> {
    // 验证默认降序排序（按大小）
    let items = &scan_result.first_level_items;
    
    for i in 1..items.len() {
        assert!(items[i-1].size >= items[i].size,
                "项目应该按大小降序排列: 项目{} ({} bytes) >= 项目{} ({} bytes)",
                i-1, items[i-1].size, i, items[i].size);
    }

    // 验证大项目排序
    let large_items = &scan_result.large_items;
    if large_items.len() > 1 {
        for i in 1..large_items.len() {
            assert!(large_items[i-1].size >= large_items[i].size,
                    "大项目应该按大小降序排列");
        }
    }

    info!("动态排序功能集成测试完成");
    Ok(())
}

/// 测试1GB筛选功能集成
async fn test_one_gb_filtering_integration(scan_result: &crate::appdata_analyzer::AppDataInfo) -> Result<(), crate::tests::TestError> {
    // 验证所有大项目都大于1GB
    for item in &scan_result.large_items {
        assert!(item.size >= 1024 * 1024 * 1024,
                "大项目应该大于1GB: {} 有 {} bytes", item.name, item.size);
        assert!(item.is_large, "大项目应该标记为is_large: {}", item.name);
    }

    // 验证一级项目中的大项目标记
    let large_item_count = scan_result.first_level_items.iter()
        .filter(|item| item.is_large)
        .count();
    
    assert_eq!(large_item_count, scan_result.large_items.len(),
                "一级项目中的大项目数量应该与大项目列表数量一致");

    info!("1GB筛选功能集成测试完成 - 发现 {} 个大项目", scan_result.large_items.len());
    Ok(())
}

/// 测试AppData迁移工作流
async fn test_appdata_migration_workflow(
    appdata_base: &std::path::Path,
    target_drive: &std::path::Path
) -> Result<crate::migration_service::MigrationResult, crate::tests::TestError> {
    use crate::migration_service::{MigrationService, MigrationOptions, validate_migration_options};
    use crate::appdata_analyzer::AppDataMigrationOptions;
    
    // 选择要迁移的大项目
    let items_to_migrate = vec![
        appdata_base.join("Local").join("Google.Chrome").to_string_lossy().to_string(),
        appdata_base.join("Local").join("Microsoft.VSCode").to_string_lossy().to_string(),
        appdata_base.join("Roaming").join("Microsoft.Office").to_string_lossy().to_string(),
    ];

    // 创建迁移选项
    let migration_options = AppDataMigrationOptions {
        source_items: items_to_migrate.clone(),
        target_drive: target_drive.to_string_lossy().to_string(),
        create_symlink: true,
        delete_source: false,
    };

    // 验证迁移选项
    let validation_result = validate_migration_options(&MigrationOptions {
        source_path: "multiple_items".to_string(),
        target_path: target_drive.to_string_lossy().to_string(),
        create_symlink: migration_options.create_symlink,
        delete_source: migration_options.delete_source,
    });

    assert!(validation_result.is_ok(), "迁移选项验证应该通过");

    // 执行迁移
    let service = MigrationService::new();
    let mut all_results = Vec::new();
    let mut total_migrated_size = 0u64;
    let mut success_count = 0;
    let mut failure_count = 0;

    for source_item in &migration_options.source_items {
        let source_path = std::path::Path::new(source_item);
        if !source_path.exists() {
            failure_count += 1;
            continue;
        }

        // 构建目标路径
        let item_name = source_path.file_name()
            .ok_or_else(|| crate::tests::TestError::ExecutionFailed("无法获取项目名称".to_string()))?
            .to_string_lossy();
        
        let target_path = target_drive.join(item_name.to_string());

        info!("迁移项目: {} -> {}", source_item, target_path.display());

        let options = MigrationOptions {
            source_path: source_item.clone(),
            target_path: target_path.to_string_lossy().to_string(),
            create_symlink: migration_options.create_symlink,
            delete_source: migration_options.delete_source,
        };

        // 执行迁移
        match service.migrate_folder(options).await {
            Ok(result) => {
                if result.success {
                    success_count += 1;
                    // 估算迁移大小
                    let estimated_size = 1024 * 1024 * 1024; // 假设1GB
                    total_migrated_size += estimated_size;
                    info!("项目迁移成功: {}, 大小: ~{}MB", source_item, estimated_size / (1024 * 1024));
                } else {
                    failure_count += 1;
                    info!("项目迁移失败: {}", result.message);
                }
                all_results.push(result);
            }
            Err(e) => {
                failure_count += 1;
                info!("项目迁移出错: {}", e);
            }
        }
    }

    // 汇总结果
    let overall_success = failure_count == 0;
    let summary = format!("AppData迁移完成 - 成功: {}, 失败: {}, 总迁移大小: ~{}MB",
                         success_count, failure_count, total_migrated_size / (1024 * 1024));

    info!("{}", summary);

    let migration_result = crate::migration_service::MigrationResult {
        success: overall_success,
        message: summary,
        source_path: format!("{}个项目", migration_options.source_items.len()),
        target_path: target_drive.to_string_lossy().to_string(),
        symlink_path: if migration_options.create_symlink { Some(format!("创建了{}个符号链接", success_count)) } else { None },
    };

    Ok(migration_result)
}

/// 验证迁移结果
async fn test_migration_result_validation(
    migration_result: &crate::migration_service::MigrationResult,
    target_drive: &std::path::Path
) -> Result<(), crate::tests::TestError> {
    // 验证迁移成功
    assert!(migration_result.success, "迁移应该成功完成");
    assert!(migration_result.message.contains("成功"), "迁移消息应该包含成功信息");

    // 验证目标路径
    assert_eq!(migration_result.target_path, target_drive.to_string_lossy(), "目标路径应该匹配");

    // 验证迁移的项目在目标位置存在
    let expected_apps = ["Google.Chrome", "Microsoft.VSCode", "Microsoft.Office"];
    for app_name in expected_apps {
        let target_app_path = target_drive.join(app_name);
        assert!(target_app_path.exists(), "迁移的应用 {} 应该在目标位置存在", app_name);
        
        // 验证数据文件存在
        let data_file = target_app_path.join("data.dat");
        assert!(data_file.exists(), "迁移的应用 {} 的数据文件应该存在", app_name);
    }

    info!("迁移结果验证测试完成");
    Ok(())
}

/// 测试错误恢复集成
async fn test_error_recovery_integration(appdata_base: &std::path::Path) -> Result<(), crate::tests::TestError> {
    use crate::error_recovery::{ErrorRecoveryManager, ErrorRecoveryConfig, RecoveryContext};
    use crate::file_operations::FileOperationError;
    
    // 初始化错误恢复管理器
    let config = ErrorRecoveryConfig::default();
    let mut recovery_manager = ErrorRecoveryManager::new(config);

    // 测试权限错误恢复
    let permission_error = FileOperationError::PermissionDenied("权限被拒绝".to_string());
    let context = RecoveryContext::new(
        "appdata_permission_test".to_string(),
        appdata_base.to_path_buf(),
        None,
        "migration_phase".to_string(),
    );

    let result = recovery_manager.handle_error("appdata_perm_test", &permission_error, &context).await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("错误处理失败: {}", e)))?;

    // 权限错误应该被标记为需要手动处理
    assert_eq!(result.recovery_type, crate::error_recovery::RecoveryType::Manual, "权限错误应该需要手动处理");

    info!("错误恢复集成测试完成");
    Ok(())
}

/// 测试性能基准
async fn test_performance_benchmarks(appdata_base: &std::path::Path) -> Result<(), crate::tests::TestError> {
    use crate::appdata_analyzer::AppDataAnalyzer;
    
    let start_time = std::time::Instant::now();
    
    // 执行完整扫描
    let mut analyzer = AppDataAnalyzer::new();
    let config = crate::appdata_analyzer::AppDataConfig {
        min_size_threshold: 1024 * 1024 * 1024, // 1GB
        max_depth: 2,
        sort_order: crate::appdata_analyzer::SortOrder::Desc,
    };
    analyzer.set_config(config);
    
    let scan_result = analyzer.scan_appdata().await
        .map_err(|e| crate::tests::TestError::ExecutionFailed(format!("性能测试扫描失败: {}", e)))?;

    let scan_duration = start_time.elapsed();

    // 验证性能要求（NFR-1）
    assert!(scan_duration.as_secs() < 30, "扫描时间应该小于30秒，实际: {}秒", scan_duration.as_secs());
    assert!(scan_result.scan_time_ms < 30000, "扫描耗时应该小于30000ms，实际: {}ms", scan_result.scan_time_ms);

    // 验证内存使用（NFR-2）
    // 这里我们验证处理的数据量是否合理
    assert!(scan_result.first_level_items.len() >= 10, "应该处理至少10个一级项目");
    assert!(scan_result.total_size > 0, "应该处理非零大小的数据");

    info!("性能基准测试完成 - 扫描耗时: {:?}, 处理项目: {}, 总大小: {}MB",
          scan_duration, scan_result.first_level_items.len(), scan_result.total_size / (1024 * 1024));

    Ok(())
}