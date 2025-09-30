//! 综合测试套件模块
//! 
//! 这个模块包含了C-drive空间管理器的所有测试用例，包括：
//! - 单元测试：测试各个模块的独立功能
//! - 集成测试：测试模块间的协作
//! - 端到端测试：测试完整的用户场景

pub mod unit_tests;
pub mod integration_tests;
pub mod e2e_tests;
pub mod test_utils;
pub mod performance_tests;
pub mod appdata_analyzer_tests;
pub mod appdata_performance_tests;

use log::{info, error};
use std::time::Instant;

/// 测试结果统计
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestStatistics {
    pub total_tests: u32,
    pub passed_tests: u32,
    pub failed_tests: u32,
    pub skipped_tests: u32,
    pub total_duration_ms: u64,
}

impl TestStatistics {
    pub fn success_rate(&self) -> f64 {
        if self.total_tests > 0 {
            (self.passed_tests as f64 / self.total_tests as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn add_result(&mut self, passed: bool, skipped: bool, duration_ms: u64) {
        self.total_tests += 1;
        self.total_duration_ms += duration_ms;

        if skipped {
            self.skipped_tests += 1;
        } else if passed {
            self.passed_tests += 1;
        } else {
            self.failed_tests += 1;
        }
    }
}

/// 测试运行器
pub struct TestRunner {
    statistics: TestStatistics,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            statistics: TestStatistics::default(),
        }
    }

    /// 运行所有测试
    pub async fn run_all_tests(&mut self) -> TestStatistics {
        info!("开始运行综合测试套件");
        let start_time = Instant::now();

        // 运行单元测试
        info!("运行单元测试...");
        self.run_unit_tests().await;

        // 运行集成测试
        info!("运行集成测试...");
        self.run_integration_tests().await;

        // 运行端到端测试
        info!("运行端到端测试...");
        self.run_e2e_tests().await;

        // 运行性能测试
        info!("运行性能测试...");
        self.run_performance_tests().await;

        let duration = start_time.elapsed().as_millis() as u64;
        self.statistics.total_duration_ms = duration;

        info!("测试完成 - 总计: {}, 通过: {}, 失败: {}, 跳过: {}, 耗时: {}ms, 成功率: {:.1}%",
              self.statistics.total_tests,
              self.statistics.passed_tests,
              self.statistics.failed_tests,
              self.statistics.skipped_tests,
              duration,
              self.statistics.success_rate());

        self.statistics.clone()
    }

    /// 运行单元测试
    pub async fn run_unit_tests(&mut self) {
        self.run_test("磁盘分析器单元测试", || unit_tests::test_disk_analyzer()).await;
        self.run_test("文件操作单元测试", || unit_tests::test_file_operations()).await;
        self.run_test("迁移服务单元测试", || unit_tests::test_migration_service()).await;
        self.run_test("错误恢复单元测试", || unit_tests::test_error_recovery()).await;
        self.run_test("操作日志单元测试", || unit_tests::test_operation_logger()).await;
        info!("AppData分析器单元测试已在独立测试文件中运行");
    }

    /// 运行单个测试
    async fn run_test<F, Fut>(&mut self, test_name: &str, test_func: F)
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<(), TestError>> + Send,
    {
        let start_time = Instant::now();
        let result = test_func().await;
        let duration = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(_) => {
                info!("✅ {} - 通过 ({}ms)", test_name, duration);
                self.statistics.add_result(true, false, duration);
            },
            Err(e) => {
                error!("❌ {} - 失败: {} ({}ms)", test_name, e, duration);
                self.statistics.add_result(false, false, duration);
            },
        }
    }

    /// 运行集成测试
    pub async fn run_integration_tests(&mut self) {
        self.run_test("扫描和迁移集成测试", || integration_tests::test_scan_and_migrate()).await;
        self.run_test("错误处理和恢复集成测试", || integration_tests::test_error_handling_and_recovery()).await;
        self.run_test("操作日志集成测试", || integration_tests::test_operation_logging()).await;
        self.run_test("路径验证集成测试", || integration_tests::test_path_validation()).await;
        self.run_test("备份和回滚集成测试", || integration_tests::test_backup_and_rollback()).await;
        self.run_test("AppData扫描集成测试", || integration_tests::test_appdata_scan_integration()).await;
    }

    /// 运行端到端测试
    pub async fn run_e2e_tests(&mut self) {
        self.run_test("完整迁移流程测试", || e2e_tests::test_complete_migration_workflow()).await;
        self.run_test("大文件夹处理测试", || e2e_tests::test_large_folder_handling()).await;
        self.run_test("错误恢复流程测试", || e2e_tests::test_error_recovery_workflow()).await;
        self.run_test("用户界面交互测试", || e2e_tests::test_ui_interactions()).await;
        self.run_test("并发操作测试", || e2e_tests::test_concurrent_operations()).await;
    }

    /// 运行性能测试
    pub async fn run_performance_tests(&mut self) {
        self.run_test("磁盘扫描性能测试", || performance_tests::test_disk_scan_performance()).await;
        self.run_test("文件迁移性能测试", || performance_tests::test_migration_performance()).await;
        self.run_test("内存使用测试", || performance_tests::test_memory_usage()).await;
        self.run_test("大文件处理性能测试", || performance_tests::test_large_file_performance()).await;
        self.run_test("并发性能测试", || performance_tests::test_concurrent_performance()).await;
        // AppData性能测试函数是同步的，需要适配异步接口
        self.run_test("AppData扫描性能测试", || async {
            appdata_performance_tests::test_appdata_scan_performance()
                .map_err(|e| crate::tests::TestError::ExecutionFailed(e.to_string()))
        }).await;
        self.run_test("AppData内存使用测试", || async {
            appdata_performance_tests::test_appdata_memory_usage()
                .map_err(|e| crate::tests::TestError::ExecutionFailed(e.to_string()))
        }).await;
        self.run_test("AppData并发性能测试", || async {
            appdata_performance_tests::test_appdata_concurrent_performance()
                .map_err(|e| crate::tests::TestError::ExecutionFailed(e.to_string()))
        }).await;
    }

    /// 获取测试结果
    pub fn get_statistics(&self) -> &TestStatistics {
        &self.statistics
    }
}

/// 测试报告生成器
pub struct TestReportGenerator;

impl TestReportGenerator {
    /// 生成HTML测试报告
    pub fn generate_html_report(statistics: &TestStatistics, details: Vec<TestDetail>) -> String {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        
        format!(r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>C-drive空间管理器 - 测试报告</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background-color: #f0f0f0; padding: 20px; border-radius: 5px; }}
        .statistics {{ display: flex; gap: 20px; margin: 20px 0; }}
        .stat-card {{ background-color: #fff; border: 1px solid #ddd; padding: 15px; border-radius: 5px; text-align: center; }}
        .stat-value {{ font-size: 24px; font-weight: bold; }}
        .stat-label {{ color: #666; margin-top: 5px; }}
        .passed {{ color: #28a745; }}
        .failed {{ color: #dc3545; }}
        .skipped {{ color: #ffc107; }}
        .details {{ margin-top: 20px; }}
        .test-item {{ padding: 10px; margin: 5px 0; border-radius: 3px; }}
        .test-passed {{ background-color: #d4edda; }}
        .test-failed {{ background-color: #f8d7da; }}
        .test-skipped {{ background-color: #fff3cd; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>C-drive空间管理器 - 测试报告</h1>
        <p>生成时间: {}</p>
    </div>
    
    <div class="statistics">
        <div class="stat-card">
            <div class="stat-value">{}</div>
            <div class="stat-label">总测试数</div>
        </div>
        <div class="stat-card">
            <div class="stat-value passed">{}</div>
            <div class="stat-label">通过</div>
        </div>
        <div class="stat-card">
            <div class="stat-value failed">{}</div>
            <div class="stat-label">失败</div>
        </div>
        <div class="stat-card">
            <div class="stat-value skipped">{}</div>
            <div class="stat-label">跳过</div>
        </div>
        <div class="stat-card">
            <div class="stat-value">{}%</div>
            <div class="stat-label">成功率</div>
        </div>
    </div>
    
    <div class="details">
        <h2>测试详情</h2>
        {}
    </div>
    
    <div class="summary">
        <h2>测试总结</h2>
        <p>总耗时: {}ms</p>
        <p>平均每个测试耗时: {:.2}ms</p>
    </div>
</body>
</html>
        "#,
        timestamp,
        statistics.total_tests,
        statistics.passed_tests,
        statistics.failed_tests,
        statistics.skipped_tests,
        statistics.success_rate(),
        Self::generate_test_details_html(&details),
        statistics.total_duration_ms,
        if statistics.total_tests > 0 { statistics.total_duration_ms as f64 / statistics.total_tests as f64 } else { 0.0 }
        )
    }

    fn generate_test_details_html(details: &[TestDetail]) -> String {
        let mut html = String::new();
        
        for detail in details {
            let class = match detail.status {
                TestStatus::Passed => "test-passed",
                TestStatus::Failed => "test-failed",
                TestStatus::Skipped => "test-skipped",
            };
            
            html.push_str(&format!(
                r#"<div class="test-item {}">
                    <strong>{}</strong> - {} ({}ms)
                    {}
                </div>"#,
                class,
                detail.name,
                match detail.status {
                    TestStatus::Passed => "通过",
                    TestStatus::Failed => "失败",
                    TestStatus::Skipped => "跳过",
                },
                detail.duration_ms,
                if let Some(error) = &detail.error_message {
                    format!("<br><small>错误: {}</small>", error)
                } else {
                    String::new()
                }
            ));
        }
        
        html
    }
}

/// 测试详情
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestDetail {
    pub name: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub error_message: Option<String>,
}

/// 测试状态
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
}

/// 测试错误类型
#[derive(Debug)]
pub enum TestError {
    SetupFailed(String),
    ExecutionFailed(String),
    AssertionFailed(String),
    Timeout(String),
    CleanupFailed(String),
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestError::SetupFailed(msg) => write!(f, "测试设置失败: {}", msg),
            TestError::ExecutionFailed(msg) => write!(f, "测试执行失败: {}", msg),
            TestError::AssertionFailed(msg) => write!(f, "断言失败: {}", msg),
            TestError::Timeout(msg) => write!(f, "测试超时: {}", msg),
            TestError::CleanupFailed(msg) => write!(f, "清理失败: {}", msg),
        }
    }
}

impl std::error::Error for TestError {}

impl From<std::io::Error> for TestError {
    fn from(error: std::io::Error) -> Self {
        TestError::SetupFailed(format!("IO错误: {}", error))
    }
}