import { invoke } from "@tauri-apps/api/core";
import { DirectoryInfo, ScanProgress, MigrationOptions, MigrationResult } from "../types/directory";

// 操作日志类型定义
export interface OperationLog {
  id: string;
  timestamp: string;
  operation_type: 'Scan' | 'Migrate' | 'Delete' | 'CreateSymlink' | 'Validate' | 'Cancel' | 'Error';
  status: 'Started' | 'InProgress' | 'Completed' | 'Failed' | 'Cancelled';
  source_path: string;
  target_path?: string;
  details: string;
  error_message?: string;
  duration_ms?: number;
  file_count?: number;
  total_size?: number;
  user: string;
  session_id: string;
}

export interface OperationStatistics {
  total_operations: number;
  completed_operations: number;
  failed_operations: number;
  cancelled_operations: number;
  total_bytes_transferred: number;
  total_files_processed: number;
  total_duration_ms: number;
  average_duration_ms: number;
}

// 错误恢复类型定义
export interface RecoveryStatistics {
  total_operations: number;
  successful_recoveries: number;
  failed_recoveries: number;
  retry_successes: number;
  rollback_successes: number;
  skip_count: number;
  abort_count: number;
  manual_count: number;
}

// 测试结果类型定义
export interface TestStatistics {
  total_tests: number;
  passed_tests: number;
  failed_tests: number;
  skipped_tests: number;
  total_duration_ms: number;
}

export interface TestDetail {
  name: string;
  status: 'Passed' | 'Failed' | 'Skipped';
  duration_ms: number;
  error_message?: string;
}

/**
 * 磁盘扫描API
 */
export const diskAPI = {
  /**
   * 扫描指定路径的目录结构
   * @param path 要扫描的路径
   * @param cDriveMode 是否启用C盘专项扫描模式
   * @returns 目录信息
   */
  async scanDirectory(path: string, cDriveMode: boolean = false): Promise<DirectoryInfo> {
    console.log('API: 开始扫描目录', path, 'C盘模式:', cDriveMode);
    try {
      const result = await invoke<DirectoryInfo>("scan_directory", {
        path,
        cDriveMode
      });
      console.log('API: 扫描目录成功', result);
      return result;
    } catch (error) {
      console.error("API: 扫描目录失败:", error);
      throw new Error(`扫描目录失败: ${error}`);
    }
  },

  /**
   * 获取扫描进度
   * @returns 扫描进度信息
   */
  async getScanProgress(): Promise<ScanProgress> {
    try {
      const result = await invoke<ScanProgress>("get_scan_progress");
      return result;
    } catch (error) {
      console.error("获取扫描进度失败:", error);
      throw new Error(`获取扫描进度失败: ${error}`);
    }
  },

  /**
   * 停止扫描
   */
  async stopScan(): Promise<void> {
    try {
      await invoke("stop_scan");
    } catch (error) {
      console.error("停止扫描失败:", error);
      throw new Error(`停止扫描失败: ${error}`);
    }
  }
};

/**
 * 性能优化API
 */
export const performanceAPI = {
  /**
   * 获取性能统计信息
   * @returns 性能统计信息
   */
  async getPerformanceStats(): Promise<any> {
    try {
      const result = await invoke<any>("get_performance_stats");
      return result;
    } catch (error) {
      console.error("获取性能统计失败:", error);
      throw new Error(`获取性能统计失败: ${error}`);
    }
  },

  /**
   * 优化磁盘扫描
   * @param path 要扫描的路径
   * @param enableCaching 是否启用缓存
   * @returns 目录信息
   */
  async optimizeDiskScan(path: string, enableCaching: boolean = true): Promise<DirectoryInfo> {
    try {
      const result = await invoke<DirectoryInfo>("optimize_disk_scan", { path, enableCaching });
      return result;
    } catch (error) {
      console.error("优化磁盘扫描失败:", error);
      throw new Error(`优化磁盘扫描失败: ${error}`);
    }
  },

  /**
   * 运行内存清理
   * @returns 是否执行了清理
   */
  async runMemoryCleanup(): Promise<boolean> {
    try {
      const result = await invoke<boolean>("run_memory_cleanup");
      return result;
    } catch (error) {
      console.error("运行内存清理失败:", error);
      throw new Error(`运行内存清理失败: ${error}`);
    }
  },

  /**
   * 获取系统性能基准
   * @returns 性能基准信息
   */
  async getPerformanceBenchmark(): Promise<PerformanceBenchmark> {
    try {
      const result = await invoke<PerformanceBenchmark>("get_performance_benchmark");
      return result;
    } catch (error) {
      console.error("获取性能基准失败:", error);
      throw new Error(`获取性能基准失败: ${error}`);
    }
  }
};

// 性能基准类型定义
export interface PerformanceBenchmark {
  memory_usage_score: number;
  cache_performance_score: number;
  concurrency_score: number;
  overall_performance_score: number;
  recommendations: string[];
}

/**
 * 测试API
 */
export const testAPI = {
  /**
   * 运行综合测试套件
   * @returns 测试统计信息
   */
  async runComprehensiveTests(): Promise<TestStatistics> {
    try {
      const result = await invoke<TestStatistics>("run_comprehensive_tests");
      return result;
    } catch (error) {
      console.error("运行综合测试失败:", error);
      throw new Error(`运行综合测试失败: ${error}`);
    }
  },

  /**
   * 运行特定类型的测试套件
   * @param testType 测试类型 (unit, integration, e2e, performance)
   * @returns 测试统计信息
   */
  async runTestSuite(testType: 'unit' | 'integration' | 'e2e' | 'performance'): Promise<TestStatistics> {
    try {
      const result = await invoke<TestStatistics>("run_test_suite", { testType });
      return result;
    } catch (error) {
      console.error(`运行${testType}测试失败:`, error);
      throw new Error(`运行${testType}测试失败: ${error}`);
    }
  },

  /**
   * 生成测试报告
   * @param outputPath 输出路径
   * @returns 是否成功
   */
  async generateTestReport(outputPath: string): Promise<boolean> {
    try {
      const result = await invoke<boolean>("generate_test_report", { outputPath });
      return result;
    } catch (error) {
      console.error("生成测试报告失败:", error);
      throw new Error(`生成测试报告失败: ${error}`);
    }
  }
};

/**
 * 错误恢复API
 */
export const errorRecoveryAPI = {
  /**
   * 获取恢复统计信息
   * @returns 恢复统计信息
   */
  async getRecoveryStatistics(): Promise<RecoveryStatistics> {
    try {
      const result = await invoke<RecoveryStatistics>("get_recovery_statistics");
      return result;
    } catch (error) {
      console.error("获取恢复统计失败:", error);
      throw new Error(`获取恢复统计失败: ${error}`);
    }
  },

  /**
   * 清理过期备份
   * @returns 清理的备份数量
   */
  async cleanupExpiredBackups(): Promise<number> {
    try {
      const result = await invoke<number>("cleanup_expired_backups");
      return result;
    } catch (error) {
      console.error("清理过期备份失败:", error);
      throw new Error(`清理过期备份失败: ${error}`);
    }
  },

  /**
   * 测试错误恢复功能
   * @returns 是否成功
   */
  async testErrorRecovery(): Promise<boolean> {
    try {
      const result = await invoke<boolean>("test_error_recovery");
      return result;
    } catch (error) {
      console.error("测试错误恢复失败:", error);
      throw new Error(`测试错误恢复失败: ${error}`);
    }
  }
};

/**
 * 文件夹迁移API
 */
export const migrationAPI = {
  /**
   * 迁移文件夹到目标位置
   * @param options 迁移选项
   * @returns 迁移结果
   */
  async migrateDirectory(options: MigrationOptions): Promise<MigrationResult> {
    try {
      const result = await invoke<MigrationResult>("migrate_directory", {
        sourcePath: options.sourcePath,
        targetPath: options.targetPath,
        createSymlink: options.createSymlink,
        deleteSource: options.deleteSource
      });
      return result;
    } catch (error) {
      console.error("迁移目录失败:", error);
      throw new Error(`迁移目录失败: ${error}`);
    }
  },

  /**
   * 验证迁移路径
   * @param sourcePath 源路径
   * @param targetPath 目标路径
   * @returns 验证结果
   */
  async validateMigrationPath(sourcePath: string, targetPath: string): Promise<{
    valid: boolean;
    message: string;
  }> {
    try {
      const result = await invoke<{ valid: boolean; message: string }>(
        "validate_migration_path",
        { sourcePath, targetPath }
      );
      return result;
    } catch (error) {
      console.error("验证迁移路径失败:", error);
      throw new Error(`验证迁移路径失败: ${error}`);
    }
  }
};

/**
 * 系统信息API
 */
export const systemAPI = {
  /**
   * 获取系统磁盘信息
   * @returns 磁盘信息列表
   */
  async getDiskInfo(): Promise<Array<{
    name: string;
    totalSpace: number;
    freeSpace: number;
    usedSpace: number;
  }>> {
    try {
      const result = await invoke<Array<{
        name: string;
        totalSpace: number;
        freeSpace: number;
        usedSpace: number;
      }>>("get_disk_info");
      return result;
    } catch (error) {
      console.error("获取磁盘信息失败:", error);
      throw new Error(`获取磁盘信息失败: ${error}`);
    }
  },

  /**
   * 检查路径是否存在
   * @param path 路径
   * @returns 是否存在
   */
  async pathExists(path: string): Promise<boolean> {
    try {
      const result = await invoke<boolean>("path_exists", { path });
      return result;
    } catch (error) {
      console.error("检查路径失败:", error);
      throw new Error(`检查路径失败: ${error}`);
    }
  }
};

/**
 * 工具函数
 */
export const utils = {
  /**
   * 格式化文件大小
   * @param bytes 字节数
   * @returns 格式化后的字符串
   */
  formatFileSize(bytes: number): string {
    if (bytes === 0) return "0 B";
    
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB", "PB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  },

  /**
   * 获取文件大小占比
   * @param size 当前大小
   * @param total 总大小
   * @returns 百分比
   */
  getSizePercentage(size: number, total: number): number {
    if (total === 0) return 0;
    return Math.round((size / total) * 100);
  },

  /**
   * 提取路径中的文件夹名称
   * @param path 路径
   * @returns 文件夹名称
   */
  getDirectoryName(path: string): string {
    return path.split(/[\\/]/).pop() || path;
  }
};

/**
 * 操作日志API
 */
export const operationAPI = {
  /**
   * 获取操作日志
   * @param limit 限制数量
   * @returns 操作日志列表
   */
  async getOperationLogs(limit: number = 100): Promise<OperationLog[]> {
    try {
      const result = await invoke<OperationLog[]>("get_operation_logs", { limit });
      return result;
    } catch (error) {
      console.error("获取操作日志失败:", error);
      throw new Error(`获取操作日志失败: ${error}`);
    }
  },

  /**
   * 获取操作统计信息
   * @returns 操作统计信息
   */
  async getOperationStatistics(): Promise<OperationStatistics> {
    try {
      const result = await invoke<OperationStatistics>("get_operation_statistics");
      return result;
    } catch (error) {
      console.error("获取操作统计失败:", error);
      throw new Error(`获取操作统计失败: ${error}`);
    }
  },

  /**
   * 导出操作日志
   * @param outputPath 输出路径
   * @returns 是否成功
   */
  async exportOperationLogs(outputPath: string): Promise<boolean> {
    try {
      const result = await invoke<boolean>("export_operation_logs", { outputPath });
      return result;
    } catch (error) {
      console.error("导出操作日志失败:", error);
      throw new Error(`导出操作日志失败: ${error}`);
    }
  },

  /**
   * 清理旧的操作日志
   * @param daysToKeep 保留天数
   * @returns 是否成功
   */
  async cleanupOldOperationLogs(daysToKeep: number): Promise<boolean> {
    try {
      const result = await invoke<boolean>("cleanup_old_operation_logs", { daysToKeep });
      return result;
    } catch (error) {
      console.error("清理旧日志失败:", error);
      throw new Error(`清理旧日志失败: ${error}`);
    }
  }
};