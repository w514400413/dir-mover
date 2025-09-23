use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Local};
use log::{info, error, warn, debug};
use uuid::Uuid;

/// 错误类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    IoError(String),
    PermissionDenied(String),
    PathNotFound(String),
    PathAlreadyExists(String),
    InvalidPath(String),
    OperationCancelled(String),
    DiskSpaceInsufficient(String),
    SystemProtection(String),
    NetworkError(String),
    Timeout(String),
    Unknown(String),
}

/// 错误严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 恢复策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    Retry(u32), // 重试次数
    Rollback,   // 回滚
    Skip,       // 跳过
    Abort,      // 中止
    Manual,     // 手动处理
}

/// 错误恢复配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecoveryConfig {
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub enable_auto_recovery: bool,
    pub enable_partial_rollback: bool,
    pub max_rollback_size_mb: u64,
    pub backup_retention_hours: u64,
}

impl Default for ErrorRecoveryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 1000,
            enable_auto_recovery: true,
            enable_partial_rollback: true,
            max_rollback_size_mb: 1000, // 1GB
            backup_retention_hours: 24,
        }
    }
}

/// 错误恢复状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryState {
    pub operation_id: String,
    pub error_type: ErrorType,
    pub severity: ErrorSeverity,
    pub recovery_strategy: RecoveryStrategy,
    pub retry_count: u32,
    pub is_recovered: bool,
    pub recovery_message: Option<String>,
    pub backup_path: Option<PathBuf>,
    pub timestamp: DateTime<Local>,
}

/// 错误恢复管理器
pub struct ErrorRecoveryManager {
    config: ErrorRecoveryConfig,
    recovery_states: HashMap<String, RecoveryState>,
    backup_registry: HashMap<String, BackupInfo>,
}

/// 备份信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub backup_id: String,
    pub original_path: PathBuf,
    pub backup_path: PathBuf,
    pub backup_size: u64,
    pub created_at: DateTime<Local>,
    pub operation_type: String,
    pub is_active: bool,
}

impl ErrorRecoveryManager {
    /// 创建新的错误恢复管理器
    pub fn new(config: ErrorRecoveryConfig) -> Self {
        Self {
            config,
            recovery_states: HashMap::new(),
            backup_registry: HashMap::new(),
        }
    }

    /// 处理错误并尝试恢复
    pub async fn handle_error(
        &mut self,
        operation_id: &str,
        error: &FileOperationError,
        context: &RecoveryContext,
    ) -> Result<RecoveryResult, RecoveryError> {
        
        let error_type = self.classify_error(error);
        let severity = self.determine_severity(&error_type, context);
        let strategy = self.determine_recovery_strategy(&error_type, &severity, context);

        info!("处理错误 - 操作ID: {}, 错误类型: {:?}, 严重程度: {:?}, 恢复策略: {:?}", 
              operation_id, error_type, severity, strategy);

        let mut recovery_state = RecoveryState {
            operation_id: operation_id.to_string(),
            error_type: error_type.clone(),
            severity: severity.clone(),
            recovery_strategy: strategy.clone(),
            retry_count: 0,
            is_recovered: false,
            recovery_message: None,
            backup_path: None,
            timestamp: Local::now(),
        };

        let result = match strategy {
            RecoveryStrategy::Retry(max_retries) => {
                self.handle_retry(operation_id, &mut recovery_state, max_retries, context).await
            },
            RecoveryStrategy::Rollback => {
                self.handle_rollback(operation_id, &mut recovery_state, context).await
            },
            RecoveryStrategy::Skip => {
                self.handle_skip(operation_id, &mut recovery_state, context).await
            },
            RecoveryStrategy::Abort => {
                self.handle_abort(operation_id, &mut recovery_state, context).await
            },
            RecoveryStrategy::Manual => {
                self.handle_manual(operation_id, &mut recovery_state, context).await
            },
        };

        // 记录恢复状态
        self.recovery_states.insert(operation_id.to_string(), recovery_state.clone());

        result
    }

    /// 处理重试策略
    async fn handle_retry(
        &mut self,
        operation_id: &str,
        recovery_state: &mut RecoveryState,
        max_retries: u32,
        context: &RecoveryContext,
    ) -> Result<RecoveryResult, RecoveryError> {
        
        while recovery_state.retry_count < max_retries {
            recovery_state.retry_count += 1;
            
            info!("重试操作 {} - 第 {} 次", operation_id, recovery_state.retry_count);
            
            // 等待重试延迟
            tokio::time::sleep(tokio::time::Duration::from_millis(self.config.retry_delay_ms)).await;
            
            // 尝试重新执行操作
            match self.retry_operation(context).await {
                Ok(_) => {
                    recovery_state.is_recovered = true;
                    recovery_state.recovery_message = Some(format!("重试成功，共重试 {} 次", recovery_state.retry_count));
                    
                    return Ok(RecoveryResult {
                        success: true,
                        recovery_type: RecoveryType::Retry,
                        message: "操作通过重试恢复成功".to_string(),
                        backup_path: None,
                    });
                },
                Err(e) => {
                    warn!("重试 {} 失败: {}", recovery_state.retry_count, e);
                    
                    // 如果达到最大重试次数，尝试其他策略
                    if recovery_state.retry_count >= max_retries {
                        // 尝试回滚作为备选策略
                        if self.config.enable_partial_rollback {
                            return self.handle_rollback(operation_id, recovery_state, context).await;
                        } else {
                            return Err(RecoveryError::MaxRetriesExceeded(format!("达到最大重试次数: {}", max_retries)));
                        }
                    }
                }
            }
        }
        
        Err(RecoveryError::MaxRetriesExceeded("重试次数耗尽".to_string()))
    }

    /// 处理回滚策略
    async fn handle_rollback(
        &mut self,
        operation_id: &str,
        recovery_state: &mut RecoveryState,
        context: &RecoveryContext,
    ) -> Result<RecoveryResult, RecoveryError> {
        
        info!("执行回滚操作: {}", operation_id);
        
        // 检查是否有可用的备份
        if let Some(backup_info) = self.find_backup_for_operation(operation_id) {
            match self.perform_rollback(&backup_info, context).await {
                Ok(_) => {
                    recovery_state.is_recovered = true;
                    recovery_state.recovery_message = Some("回滚成功".to_string());
                    recovery_state.backup_path = Some(backup_info.backup_path.clone());
                    
                    Ok(RecoveryResult {
                        success: true,
                        recovery_type: RecoveryType::Rollback,
                        message: "操作通过回滚恢复成功".to_string(),
                        backup_path: Some(backup_info.backup_path),
                    })
                },
                Err(e) => {
                    error!("回滚失败: {}", e);
                    Err(RecoveryError::RollbackFailed(format!("回滚操作失败: {}", e)))
                }
            }
        } else {
            // 如果没有备份，尝试创建临时备份并回滚
            match self.create_emergency_backup_and_rollback(context).await {
                Ok(backup_path) => {
                    recovery_state.is_recovered = true;
                    recovery_state.recovery_message = Some("紧急回滚成功".to_string());
                    recovery_state.backup_path = Some(backup_path.clone());
                    
                    Ok(RecoveryResult {
                        success: true,
                        recovery_type: RecoveryType::Rollback,
                        message: "通过紧急备份回滚成功".to_string(),
                        backup_path: Some(backup_path),
                    })
                },
                Err(e) => {
                    error!("紧急回滚失败: {}", e);
                    Err(RecoveryError::RollbackFailed(format!("紧急回滚失败: {}", e)))
                }
            }
        }
    }

    /// 处理跳过策略
    async fn handle_skip(
        &mut self,
        operation_id: &str,
        recovery_state: &mut RecoveryState,
        _context: &RecoveryContext,
    ) -> Result<RecoveryResult, RecoveryError> {
        
        info!("跳过操作: {}", operation_id);
        
        recovery_state.is_recovered = true;
        recovery_state.recovery_message = Some("操作已跳过".to_string());
        
        Ok(RecoveryResult {
            success: true,
            recovery_type: RecoveryType::Skip,
            message: "操作已跳过，继续执行后续步骤".to_string(),
            backup_path: None,
        })
    }

    /// 处理中止策略
    async fn handle_abort(
        &mut self,
        operation_id: &str,
        recovery_state: &mut RecoveryState,
        _context: &RecoveryContext,
    ) -> Result<RecoveryResult, RecoveryError> {
        
        info!("中止操作: {}", operation_id);
        
        recovery_state.is_recovered = false;
        recovery_state.recovery_message = Some("操作已中止".to_string());
        
        Ok(RecoveryResult {
            success: false,
            recovery_type: RecoveryType::Abort,
            message: "操作已中止，需要手动干预".to_string(),
            backup_path: None,
        })
    }

    /// 处理手动恢复策略
    async fn handle_manual(
        &mut self,
        operation_id: &str,
        recovery_state: &mut RecoveryState,
        _context: &RecoveryContext,
    ) -> Result<RecoveryResult, RecoveryError> {
        
        info!("需要手动恢复: {}", operation_id);
        
        recovery_state.is_recovered = false;
        recovery_state.recovery_message = Some("需要手动恢复".to_string());
        
        Ok(RecoveryResult {
            success: false,
            recovery_type: RecoveryType::Manual,
            message: "错误需要手动处理，请查看日志获取详细信息".to_string(),
            backup_path: None,
        })
    }

    /// 创建备份
    pub async fn create_backup(
        &mut self,
        source_path: &Path,
        operation_type: &str,
        operation_id: &str,
    ) -> Result<BackupInfo, BackupError> {
        
        // 检查备份大小限制
        let source_size = self.calculate_directory_size(source_path)?;
        let max_backup_size = self.config.max_rollback_size_mb * 1024 * 1024;
        
        if source_size > max_backup_size {
            return Err(BackupError::BackupTooLarge(format!(
                "备份大小 {} 超过最大限制 {}", 
                format_size(source_size), 
                format_size(max_backup_size)
            )));
        }

        let backup_id = Uuid::new_v4().to_string();
        let backup_path = self.generate_backup_path(source_path, &backup_id);
        
        info!("创建备份 - 操作ID: {}, 源路径: {}, 备份路径: {}", 
              operation_id, source_path.display(), backup_path.display());

        // 执行备份
        match self.perform_backup(source_path, &backup_path).await {
            Ok(_) => {
                let backup_info = BackupInfo {
                    backup_id: backup_id.clone(),
                    original_path: source_path.to_path_buf(),
                    backup_path: backup_path.clone(),
                    backup_size: source_size,
                    created_at: Local::now(),
                    operation_type: operation_type.to_string(),
                    is_active: true,
                };

                // 注册备份
                self.backup_registry.insert(backup_id.clone(), backup_info.clone());
                
                info!("备份创建成功: {}", backup_id);
                Ok(backup_info)
            },
            Err(e) => {
                error!("备份创建失败: {}", e);
                Err(BackupError::BackupFailed(format!("备份操作失败: {}", e)))
            }
        }
    }

    /// 执行备份操作
    async fn perform_backup(&self, source: &Path, backup_path: &Path) -> Result<(), String> {
        // 创建备份目录
        fs::create_dir_all(backup_path.parent().unwrap())
            .map_err(|e| format!("创建备份目录失败: {}", e))?;

        // 复制文件或目录
        if source.is_file() {
            fs::copy(source, backup_path)
                .map_err(|e| format!("文件备份失败: {}", e))?;
        } else {
            // 递归复制目录
            self.copy_directory_recursive(source, backup_path)?;
        }

        Ok(())
    }

    /// 递归复制目录
    fn copy_directory_recursive(&self, source: &Path, target: &Path) -> Result<(), String> {
        fs::create_dir_all(target)
            .map_err(|e| format!("创建目标目录失败: {}", e))?;

        let entries = fs::read_dir(source)
            .map_err(|e| format!("读取源目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let entry_path = entry.path();
            let entry_name = entry.file_name();
            let target_entry_path = target.join(&entry_name);

            if entry_path.is_dir() {
                self.copy_directory_recursive(&entry_path, &target_entry_path)?;
            } else {
                fs::copy(&entry_path, &target_entry_path)
                    .map_err(|e| format!("复制文件失败 {}: {}", entry_path.display(), e))?;
            }
        }

        Ok(())
    }

    /// 执行回滚
    async fn perform_rollback(&self, backup_info: &BackupInfo, context: &RecoveryContext) -> Result<(), String> {
        info!("执行回滚 - 备份ID: {}, 原始路径: {}", 
              backup_info.backup_id, backup_info.original_path.display());

        // 检查原始路径是否存在
        if backup_info.original_path.exists() {
            // 如果存在，先创建临时备份
            let temp_backup_path = self.generate_temp_backup_path(&backup_info.original_path);
            self.perform_backup(&backup_info.original_path, &temp_backup_path).await?;
        }

        // 删除原始路径（如果是目录）
        if backup_info.original_path.is_dir() {
            fs::remove_dir_all(&backup_info.original_path)
                .map_err(|e| format!("删除原始目录失败: {}", e))?;
        } else if backup_info.original_path.is_file() {
            fs::remove_file(&backup_info.original_path)
                .map_err(|e| format!("删除原始文件失败: {}", e))?;
        }

        // 恢复备份
        self.perform_backup(&backup_info.backup_path, &backup_info.original_path).await?;

        info!("回滚完成: {}", backup_info.backup_id);
        Ok(())
    }

    /// 创建紧急备份并回滚
    async fn create_emergency_backup_and_rollback(&self, context: &RecoveryContext) -> Result<PathBuf, String> {
        // 这个函数用于在没有预先备份的情况下创建紧急备份
        // 实现取决于具体的上下文
        Err("紧急备份功能未实现".to_string())
    }

    /// 重试操作
    async fn retry_operation(&self, context: &RecoveryContext) -> Result<(), String> {
        // 这个函数应该根据上下文重新执行失败的操作
        // 实现取决于具体的操作类型
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(())
    }

    /// 错误分类
    pub fn classify_error(&self, error: &FileOperationError) -> ErrorType {
        match error {
            FileOperationError::IoError(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    ErrorType::PermissionDenied(e.to_string())
                } else if e.kind() == std::io::ErrorKind::NotFound {
                    ErrorType::PathNotFound(e.to_string())
                } else if e.kind() == std::io::ErrorKind::AlreadyExists {
                    ErrorType::PathAlreadyExists(e.to_string())
                } else {
                    ErrorType::IoError(e.to_string())
                }
            },
            FileOperationError::PermissionDenied(msg) => ErrorType::PermissionDenied(msg.clone()),
            FileOperationError::PathNotFound(msg) => ErrorType::PathNotFound(msg.clone()),
            FileOperationError::PathAlreadyExists(msg) => ErrorType::PathAlreadyExists(msg.clone()),
            FileOperationError::InvalidPath(msg) => ErrorType::InvalidPath(msg.clone()),
            FileOperationError::OperationCancelled(msg) => ErrorType::OperationCancelled(msg.clone()),
        }
    }

    /// 确定错误严重程度
    pub fn determine_severity(&self, error_type: &ErrorType, context: &RecoveryContext) -> ErrorSeverity {
        match error_type {
            ErrorType::OperationCancelled(_) => ErrorSeverity::Low,
            ErrorType::PathAlreadyExists(_) => ErrorSeverity::Low,
            ErrorType::PathNotFound(_) => ErrorSeverity::Medium,
            ErrorType::PermissionDenied(_) => ErrorSeverity::Medium,
            ErrorType::InvalidPath(_) => ErrorSeverity::High,
            ErrorType::IoError(_) => ErrorSeverity::High,
            ErrorType::DiskSpaceInsufficient(_) => ErrorSeverity::Critical,
            ErrorType::SystemProtection(_) => ErrorSeverity::Critical,
            ErrorType::NetworkError(_) => ErrorSeverity::Medium,
            ErrorType::Timeout(_) => ErrorSeverity::Medium,
            ErrorType::Unknown(_) => ErrorSeverity::High,
        }
    }

    /// 确定恢复策略
    pub fn determine_recovery_strategy(&self, error_type: &ErrorType, severity: &ErrorSeverity, context: &RecoveryContext) -> RecoveryStrategy {
        if !self.config.enable_auto_recovery {
            return RecoveryStrategy::Manual;
        }

        match (error_type, severity) {
            (ErrorType::OperationCancelled(_), _) => RecoveryStrategy::Skip,
            (ErrorType::PathAlreadyExists(_), ErrorSeverity::Low) => RecoveryStrategy::Skip,
            (ErrorType::PathNotFound(_), ErrorSeverity::Medium) => RecoveryStrategy::Retry(2),
            (ErrorType::PermissionDenied(_), ErrorSeverity::Medium) => RecoveryStrategy::Manual,
            (ErrorType::IoError(_), ErrorSeverity::High) => RecoveryStrategy::Retry(3),
            (ErrorType::DiskSpaceInsufficient(_), ErrorSeverity::Critical) => RecoveryStrategy::Abort,
            (ErrorType::SystemProtection(_), ErrorSeverity::Critical) => RecoveryStrategy::Abort,
            _ => RecoveryStrategy::Manual,
        }
    }

    /// 查找操作的备份
    fn find_backup_for_operation(&self, operation_id: &str) -> Option<BackupInfo> {
        self.backup_registry.values()
            .find(|backup| backup.operation_type.contains(operation_id) && backup.is_active)
            .cloned()
    }

    /// 生成备份路径
    fn generate_backup_path(&self, source_path: &Path, backup_id: &str) -> PathBuf {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let source_name = source_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("backup");
        
        let backup_dir = std::env::temp_dir().join("dir_mover_backups");
        backup_dir.join(format!("{}_{}_{}", source_name, timestamp, backup_id))
    }

    /// 生成临时备份路径
    fn generate_temp_backup_path(&self, source_path: &Path) -> PathBuf {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let source_name = source_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("temp_backup");
        
        let temp_dir = std::env::temp_dir().join("dir_mover_temp");
        temp_dir.join(format!("{}_{}_temp", source_name, timestamp))
    }

    /// 计算目录大小
    fn calculate_directory_size(&self, path: &Path) -> Result<u64, String> {
        if !path.exists() {
            return Err("路径不存在".to_string());
        }

        if path.is_file() {
            return fs::metadata(path)
                .map(|m| m.len())
                .map_err(|e| format!("获取文件元数据失败: {}", e));
        }

        let mut total_size = 0u64;
        let entries = fs::read_dir(path)
            .map_err(|e| format!("读取目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                total_size += self.calculate_directory_size(&entry_path)?;
            } else {
                total_size += entry.metadata()
                    .map_err(|e| format!("获取文件元数据失败: {}", e))?
                    .len();
            }
        }

        Ok(total_size)
    }

    /// 清理过期备份
    pub fn cleanup_expired_backups(&mut self) -> Result<u32, String> {
        let cutoff_time = Local::now() - chrono::Duration::hours(self.config.backup_retention_hours as i64);
        let mut cleaned_count = 0;

        let backup_ids: Vec<String> = self.backup_registry.keys().cloned().collect();
        
        for backup_id in backup_ids {
            if let Some(backup_info) = self.backup_registry.get(&backup_id) {
                if backup_info.created_at < cutoff_time {
                    match self.remove_backup(&backup_id) {
                        Ok(_) => {
                            cleaned_count += 1;
                            info!("清理过期备份: {}", backup_id);
                        },
                        Err(e) => {
                            warn!("清理备份失败 {}: {}", backup_id, e);
                        }
                    }
                }
            }
        }

        Ok(cleaned_count)
    }

    /// 删除备份
    fn remove_backup(&mut self, backup_id: &str) -> Result<(), String> {
        if let Some(backup_info) = self.backup_registry.remove(backup_id) {
            // 删除备份文件
            if backup_info.backup_path.exists() {
                if backup_info.backup_path.is_dir() {
                    fs::remove_dir_all(&backup_info.backup_path)
                        .map_err(|e| format!("删除备份目录失败: {}", e))?;
                } else {
                    fs::remove_file(&backup_info.backup_path)
                        .map_err(|e| format!("删除备份文件失败: {}", e))?;
                }
            }
            
            Ok(())
        } else {
            Err("备份不存在".to_string())
        }
    }

    /// 获取恢复统计
    pub fn get_recovery_statistics(&self) -> RecoveryStatistics {
        let total_operations = self.recovery_states.len() as u32;
        let successful_recoveries = self.recovery_states.values()
            .filter(|state| state.is_recovered)
            .count() as u32;
        
        let retry_successes = self.recovery_states.values()
            .filter(|state| state.is_recovered && matches!(state.recovery_strategy, RecoveryStrategy::Retry(_)))
            .count() as u32;
        
        let rollback_successes = self.recovery_states.values()
            .filter(|state| state.is_recovered && matches!(state.recovery_strategy, RecoveryStrategy::Rollback))
            .count() as u32;

        RecoveryStatistics {
            total_operations,
            successful_recoveries,
            failed_recoveries: total_operations - successful_recoveries,
            retry_successes,
            rollback_successes,
            skip_count: self.recovery_states.values()
                .filter(|state| matches!(state.recovery_strategy, RecoveryStrategy::Skip))
                .count() as u32,
            abort_count: self.recovery_states.values()
                .filter(|state| matches!(state.recovery_strategy, RecoveryStrategy::Abort))
                .count() as u32,
            manual_count: self.recovery_states.values()
                .filter(|state| matches!(state.recovery_strategy, RecoveryStrategy::Manual))
                .count() as u32,
        }
    }
}

/// 恢复上下文
#[derive(Debug, Clone)]
pub struct RecoveryContext {
    pub operation_type: String,
    pub source_path: PathBuf,
    pub target_path: Option<PathBuf>,
    pub operation_phase: String,
    pub previous_operations: Vec<String>,
    pub user_preferences: HashMap<String, String>,
}

impl RecoveryContext {
    pub fn new(
        operation_type: String,
        source_path: PathBuf,
        target_path: Option<PathBuf>,
        operation_phase: String,
    ) -> Self {
        Self {
            operation_type,
            source_path,
            target_path,
            operation_phase,
            previous_operations: Vec::new(),
            user_preferences: HashMap::new(),
        }
    }

    pub fn with_preferences(mut self, preferences: HashMap<String, String>) -> Self {
        self.user_preferences = preferences;
        self
    }
}

/// 恢复结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryResult {
    pub success: bool,
    pub recovery_type: RecoveryType,
    pub message: String,
    pub backup_path: Option<PathBuf>,
}

/// 恢复类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryType {
    Retry,
    Rollback,
    Skip,
    Abort,
    Manual,
}

/// 恢复错误
#[derive(Debug)]
pub enum RecoveryError {
    MaxRetriesExceeded(String),
    RollbackFailed(String),
    BackupFailed(String),
    RecoveryImpossible(String),
    UserCancelled(String),
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryError::MaxRetriesExceeded(msg) => write!(f, "重试次数超限: {}", msg),
            RecoveryError::RollbackFailed(msg) => write!(f, "回滚失败: {}", msg),
            RecoveryError::BackupFailed(msg) => write!(f, "备份失败: {}", msg),
            RecoveryError::RecoveryImpossible(msg) => write!(f, "无法恢复: {}", msg),
            RecoveryError::UserCancelled(msg) => write!(f, "用户取消: {}", msg),
        }
    }
}

/// 备份错误
#[derive(Debug)]
pub enum BackupError {
    BackupFailed(String),
    BackupTooLarge(String),
    BackupNotFound(String),
    InvalidBackupPath(String),
}

impl std::fmt::Display for BackupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupError::BackupFailed(msg) => write!(f, "备份失败: {}", msg),
            BackupError::BackupTooLarge(msg) => write!(f, "备份过大: {}", msg),
            BackupError::BackupNotFound(msg) => write!(f, "备份未找到: {}", msg),
            BackupError::InvalidBackupPath(msg) => write!(f, "无效备份路径: {}", msg),
        }
    }
}

/// 恢复统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStatistics {
    pub total_operations: u32,
    pub successful_recoveries: u32,
    pub failed_recoveries: u32,
    pub retry_successes: u32,
    pub rollback_successes: u32,
    pub skip_count: u32,
    pub abort_count: u32,
    pub manual_count: u32,
}

impl RecoveryStatistics {
    pub fn success_rate(&self) -> f64 {
        if self.total_operations > 0 {
            (self.successful_recoveries as f64 / self.total_operations as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// 文件操作错误（需要导入）
use crate::file_operations::FileOperationError;

/// 格式化文件大小
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.2} {}", size, UNITS[unit_index])
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_error_recovery_manager() {
        let config = ErrorRecoveryConfig::default();
        let mut manager = ErrorRecoveryManager::new(config);

        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("test.txt");
        
        // 创建测试文件
        let mut file = File::create(&source_file).unwrap();
        writeln!(file, "测试内容").unwrap();

        let context = RecoveryContext::new(
            "test_operation".to_string(),
            source_file.clone(),
            None,
            "test_phase".to_string(),
        );

        // 测试权限错误
        let permission_error = FileOperationError::PermissionDenied("测试权限错误".to_string());
        let result = manager.handle_error("test_op_1", &permission_error, &context).await;
        
        // 权限错误应该被分类为需要手动处理
        assert!(result.is_ok() || result.is_err()); // 根据策略可能成功或失败
    }

    #[test]
    fn test_error_classification() {
        let config = ErrorRecoveryConfig::default();
        let manager = ErrorRecoveryManager::new(config);

        let io_error = FileOperationError::IoError(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "权限被拒绝"));
        let classified = manager.classify_error(&io_error);
        
        match classified {
            ErrorType::PermissionDenied(_) => assert!(true),
            _ => panic!("错误分类不正确"),
        }
    }

    #[test]
    fn test_recovery_statistics() {
        let config = ErrorRecoveryConfig::default();
        let manager = ErrorRecoveryManager::new(config);
        
        let stats = manager.get_recovery_statistics();
        assert_eq!(stats.total_operations, 0);
        assert_eq!(stats.success_rate(), 0.0);
    }
}