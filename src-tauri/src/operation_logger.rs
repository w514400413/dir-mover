use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use uuid::Uuid;

/// 操作类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationType {
    Scan,
    Migrate,
    Delete,
    CreateSymlink,
    Validate,
    Cancel,
    Error,
}

/// 操作状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationStatus {
    Started,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// 操作日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLog {
    pub id: String,
    pub timestamp: DateTime<Local>,
    pub operation_type: OperationType,
    pub status: OperationStatus,
    pub source_path: String,
    pub target_path: Option<String>,
    pub details: String,
    pub error_message: Option<String>,
    pub duration_ms: Option<u64>,
    pub file_count: Option<u64>,
    pub total_size: Option<u64>,
    pub user: String,
    pub session_id: String,
}

impl OperationLog {
    pub fn new(
        operation_type: OperationType,
        source_path: String,
        target_path: Option<String>,
        user: String,
        session_id: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Local::now(),
            operation_type,
            status: OperationStatus::Started,
            source_path,
            target_path,
            details: String::new(),
            error_message: None,
            duration_ms: None,
            file_count: None,
            total_size: None,
            user,
            session_id,
        }
    }

    pub fn with_details(mut self, details: String) -> Self {
        self.details = details;
        self
    }

    pub fn with_status(mut self, status: OperationStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error_message = Some(error);
        self.status = OperationStatus::Failed;
        self
    }

    pub fn with_metrics(mut self, file_count: u64, total_size: u64, duration_ms: u64) -> Self {
        self.file_count = Some(file_count);
        self.total_size = Some(total_size);
        self.duration_ms = Some(duration_ms);
        self
    }

    pub fn complete(mut self) -> Self {
        self.status = OperationStatus::Completed;
        self
    }

    pub fn fail(mut self, error: String) -> Self {
        self.status = OperationStatus::Failed;
        self.error_message = Some(error);
        self
    }

    pub fn cancel(mut self) -> Self {
        self.status = OperationStatus::Cancelled;
        self
    }
}

/// 操作日志管理器
#[derive(Clone)]
pub struct OperationLogger {
    log_file: PathBuf,
    session_id: String,
    current_user: String,
}

impl OperationLogger {
    pub fn new(log_dir: PathBuf, session_id: String, current_user: String) -> Result<Self, Box<dyn std::error::Error>> {
        // 创建日志目录
        std::fs::create_dir_all(&log_dir)?;
        
        // 生成操作日志文件名
        let timestamp = Local::now().format("%Y-%m");
        let log_file = log_dir.join(format!("operations-{}.log", timestamp));
        
        Ok(Self {
            log_file,
            session_id,
            current_user,
        })
    }

    /// 记录操作开始
    pub fn log_operation_start(
        &self,
        operation_type: OperationType,
        source_path: String,
        target_path: Option<String>,
        details: String,
    ) -> Result<OperationLog, Box<dyn std::error::Error>> {
        let log = OperationLog::new(
            operation_type,
            source_path,
            target_path,
            self.current_user.clone(),
            self.session_id.clone(),
        ).with_details(details);

        self.write_log(&log)?;
        Ok(log)
    }

    /// 更新操作状态
    pub fn update_operation_status(
        &self,
        log: &mut OperationLog,
        status: OperationStatus,
        details: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log.status = status;
        if let Some(details) = details {
            log.details = details;
        }
        self.write_log(log)?;
        Ok(())
    }

    /// 完成操作
    pub fn complete_operation(
        &self,
        log: &mut OperationLog,
        file_count: u64,
        total_size: u64,
        duration_ms: u64,
        details: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log.status = OperationStatus::Completed;
        log.file_count = Some(file_count);
        log.total_size = Some(total_size);
        log.duration_ms = Some(duration_ms);
        if let Some(details) = details {
            log.details = details;
        }
        self.write_log(log)?;
        Ok(())
    }

    /// 记录操作失败
    pub fn fail_operation(
        &self,
        log: &mut OperationLog,
        error: String,
        details: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log.status = OperationStatus::Failed;
        log.error_message = Some(error);
        if let Some(details) = details {
            log.details = details;
        }
        self.write_log(log)?;
        Ok(())
    }

    /// 取消操作
    pub fn cancel_operation(
        &self,
        log: &mut OperationLog,
        details: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log.status = OperationStatus::Cancelled;
        if let Some(details) = details {
            log.details = details;
        }
        self.write_log(log)?;
        Ok(())
    }

    /// 写入日志文件
    fn write_log(&self, log: &OperationLog) -> Result<(), Box<dyn std::error::Error>> {
        let log_entry = serde_json::to_string(log)? + "\n";
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)?;
        
        file.write_all(log_entry.as_bytes())?;
        file.flush()?;
        
        Ok(())
    }

    /// 读取最近的日志
    pub fn get_recent_logs(&self, limit: usize) -> Result<Vec<OperationLog>, Box<dyn std::error::Error>> {
        if !self.log_file.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.log_file)?;
        let reader = BufReader::new(file);
        let mut logs = Vec::new();

        // 读取所有日志
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(log) = serde_json::from_str::<OperationLog>(&line) {
                    logs.push(log);
                }
            }
        }

        // 按时间排序并取最近的
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        logs.truncate(limit);
        logs.reverse(); // 恢复时间顺序
        
        Ok(logs)
    }

    /// 按类型筛选日志
    pub fn get_logs_by_type(&self, operation_type: OperationType, limit: usize) -> Result<Vec<OperationLog>, Box<dyn std::error::Error>> {
        let all_logs = self.get_recent_logs(1000)?; // 获取足够多的日志
        let filtered_logs: Vec<_> = all_logs
            .into_iter()
            .filter(|log| std::mem::discriminant(&log.operation_type) == std::mem::discriminant(&operation_type))
            .take(limit)
            .collect();
        Ok(filtered_logs)
    }

    /// 获取失败的操作日志
    pub fn get_failed_operations(&self, limit: usize) -> Result<Vec<OperationLog>, Box<dyn std::error::Error>> {
        let all_logs = self.get_recent_logs(1000)?;
        let failed_logs: Vec<_> = all_logs
            .into_iter()
            .filter(|log| std::mem::discriminant(&log.status) == std::mem::discriminant(&OperationStatus::Failed))
            .take(limit)
            .collect();
        Ok(failed_logs)
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> Result<OperationStatistics, Box<dyn std::error::Error>> {
        let logs = self.get_recent_logs(10000)?; // 获取最近10000条日志
        
        let mut stats = OperationStatistics::default();
        
        for log in logs {
            stats.total_operations += 1;
            
            match log.status {
                OperationStatus::Completed => stats.completed_operations += 1,
                OperationStatus::Failed => stats.failed_operations += 1,
                OperationStatus::Cancelled => stats.cancelled_operations += 1,
                _ => {}
            }
            
            if let Some(size) = log.total_size {
                stats.total_bytes_transferred += size;
            }
            
            if let Some(files) = log.file_count {
                stats.total_files_processed += files;
            }
            
            if let Some(duration) = log.duration_ms {
                stats.total_duration_ms += duration;
            }
        }
        
        stats.average_duration_ms = if stats.completed_operations > 0 {
            stats.total_duration_ms / stats.completed_operations
        } else {
            0
        };
        
        Ok(stats)
    }

    /// 清理旧的操作日志
    pub fn cleanup_old_logs(&self, days_to_keep: u32) -> Result<(), Box<dyn std::error::Error>> {
        let cutoff_date = Local::now() - chrono::Duration::days(days_to_keep as i64);
        
        if !self.log_file.exists() {
            return Ok(());
        }

        let file = File::open(&self.log_file)?;
        let reader = BufReader::new(file);
        let mut valid_logs = Vec::new();

        // 读取所有有效的日志
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(log) = serde_json::from_str::<OperationLog>(&line) {
                    if log.timestamp >= cutoff_date {
                        valid_logs.push(log);
                    }
                }
            }
        }

        // 重写日志文件
        let mut file = File::create(&self.log_file)?;
        for log in valid_logs {
            let log_entry = serde_json::to_string(&log)? + "\n";
            file.write_all(log_entry.as_bytes())?;
        }
        file.flush()?;

        Ok(())
    }
}

/// 操作统计信息
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OperationStatistics {
    pub total_operations: u64,
    pub completed_operations: u64,
    pub failed_operations: u64,
    pub cancelled_operations: u64,
    pub total_bytes_transferred: u64,
    pub total_files_processed: u64,
    pub total_duration_ms: u64,
    pub average_duration_ms: u64,
}

impl OperationStatistics {
    pub fn success_rate(&self) -> f64 {
        if self.total_operations > 0 {
            (self.completed_operations as f64 / self.total_operations as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn average_transfer_speed_mbps(&self) -> f64 {
        if self.total_duration_ms > 0 {
            let total_mb = self.total_bytes_transferred as f64 / (1024.0 * 1024.0);
            let total_seconds = self.total_duration_ms as f64 / 1000.0;
            total_mb / total_seconds
        } else {
            0.0
        }
    }
}

/// 操作日志导出功能
pub fn export_logs_to_csv(
    logs: &[OperationLog],
    output_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(output_path)?;
    
    // 写入CSV头部
    writeln!(file, "ID,时间戳,操作类型,状态,源路径,目标路径,详情,错误信息,耗时(ms),文件数,总大小,用户,会话ID")?;
    
    // 写入数据
    for log in logs {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            log.id,
            log.timestamp.format("%Y-%m-%d %H:%M:%S"),
            format!("{:?}", log.operation_type),
            format!("{:?}", log.status),
            log.source_path,
            log.target_path.as_deref().unwrap_or(""),
            log.details.replace(",", ";"),
            log.error_message.as_deref().unwrap_or(""),
            log.duration_ms.map(|d| d.to_string()).unwrap_or_default(),
            log.file_count.map(|f| f.to_string()).unwrap_or_default(),
            log.total_size.map(|s| s.to_string()).unwrap_or_default(),
            log.user,
            log.session_id
        )?;
    }
    
    Ok(())
}