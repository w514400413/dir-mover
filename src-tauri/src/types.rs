use serde::{Serialize, Deserialize};
use std::time::SystemTime;

/// 目录信息结构体（与前端共享）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub file_count: u64,
    pub subdirectories: Vec<DirectoryInfo>,
}

/// 扫描进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub current_path: String,
    pub processed_files: u64,
    pub total_files: u64,
    pub progress: f64,
    pub processed_directories: u64, // 新增：已处理目录数
    pub total_directories: u64,     // 新增：总目录数
    pub current_directory: String,  // 新增：当前处理的目录
    pub estimated_time_remaining: u64, // 新增：预计剩余时间（秒）
    pub scan_speed: f64,            // 新增：扫描速度（文件/秒）
    pub large_folders_found: u64,   // 新增：发现的大文件夹数量
}

/// 磁盘信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub total_space: u64,
    pub free_space: u64,
    pub used_space: u64,
}

/// 迁移选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationOptions {
    pub source_path: String,
    pub target_path: String,
    pub create_symlink: bool,
    pub delete_source: bool,
}

/// 迁移结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    pub success: bool,
    pub message: String,
    pub source_path: String,
    pub target_path: String,
    pub symlink_path: Option<String>,
}

/// 文件操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationResult {
    pub success: bool,
    pub message: String,
    pub source_path: String,
    pub target_path: Option<String>,
}

/// 系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub total_memory: u64,
    pub available_memory: u64,
    pub cpu_count: usize,
}

/// API响应包装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// 创建错误响应
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// 错误信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl ErrorInfo {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }
}

/// 进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub operation: String,
    pub current_item: String,
    pub processed: u64,
    pub total: u64,
    pub percentage: f64,
    pub status: String,
}

impl ProgressInfo {
    pub fn new(operation: &str, current_item: &str) -> Self {
        Self {
            operation: operation.to_string(),
            current_item: current_item.to_string(),
            processed: 0,
            total: 0,
            percentage: 0.0,
            status: "运行中".to_string(),
        }
    }

    pub fn update_progress(&mut self, processed: u64, total: u64) {
        self.processed = processed;
        self.total = total;
        self.percentage = if total > 0 {
            (processed as f64 / total as f64) * 100.0
        } else {
            0.0
        };
    }

    pub fn set_status(&mut self, status: &str) {
        self.status = status.to_string();
    }
}

/// 路径验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathValidationResult {
    pub valid: bool,
    pub message: String,
    pub suggestions: Vec<String>,
}

impl PathValidationResult {
    pub fn valid(message: &str) -> Self {
        Self {
            valid: true,
            message: message.to_string(),
            suggestions: Vec::new(),
        }
    }

    pub fn invalid(message: &str) -> Self {
        Self {
            valid: false,
            message: message.to_string(),
            suggestions: Vec::new(),
        }
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestions.push(suggestion.to_string());
        self
    }
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub is_directory: bool,
    pub modified_time: Option<SystemTime>,
    pub created_time: Option<SystemTime>,
    pub permissions: Option<String>,
}

/// 目录内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryContent {
    pub path: String,
    pub files: Vec<FileInfo>,
    pub total_size: u64,
    pub file_count: u64,
    pub directory_count: u64,
}

/// 排序选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOptions {
    pub field: String, // "name", "size", "modified_time"
    pub order: String, // "asc", "desc"
}

/// 过滤选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOptions {
    pub name_pattern: Option<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub file_types: Option<Vec<String>>, // 文件扩展名
    pub show_hidden: bool,
    pub show_empty: bool,
}

/// 分页信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
    pub total_pages: u32,
}

/// 分页响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: PaginationInfo,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, page: u32, page_size: u32, total: u64) -> Self {
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;
        
        Self {
            items,
            pagination: PaginationInfo {
                page,
                page_size,
                total,
                total_pages,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response() {
        let data = DirectoryInfo {
            path: "/test".to_string(),
            name: "test".to_string(),
            size: 1024,
            file_count: 10,
            subdirectories: Vec::new(),
        };

        let response = ApiResponse::success(data.clone());
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());

        let error_response = ApiResponse::<DirectoryInfo>::error("测试错误".to_string());
        assert!(!error_response.success);
        assert!(error_response.data.is_none());
        assert!(error_response.error.is_some());
    }

    #[test]
    fn test_progress_info() {
        let mut progress = ProgressInfo::new("扫描", "/test/path");
        assert_eq!(progress.operation, "扫描");
        assert_eq!(progress.percentage, 0.0);

        progress.update_progress(50, 100);
        assert_eq!(progress.percentage, 50.0);
        assert_eq!(progress.processed, 50);
        assert_eq!(progress.total, 100);
    }

    #[test]
    fn test_path_validation_result() {
        let valid = PathValidationResult::valid("路径有效");
        assert!(valid.valid);
        assert_eq!(valid.message, "路径有效");

        let invalid = PathValidationResult::invalid("路径无效");
        assert!(!invalid.valid);
        assert_eq!(invalid.message, "路径无效");
    }
}