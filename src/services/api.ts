import { invoke } from "@tauri-apps/api/core";
import { DirectoryInfo, ScanProgress, MigrationOptions, MigrationResult } from "../types/directory";
import { AppDataInfo, AppDataScanOptions, AppDataScanResult, AppDataConfig, AppDataMigrationOptions, AppDataMigrationResult, AppDataDriveInfo } from "../types/appdata";

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

/**
 * AppData 扫描API（更新版本）
 */
export const appDataAPI = {
  /**
   * 扫描AppData目录（新版本，支持配置）
   * @param options 扫描选项（可选）
   * @returns AppData信息
   */
  async scanAppData(options?: AppDataScanOptions): Promise<AppDataInfo> {
    console.log('API: 开始扫描AppData目录', options);
    try {
      // 构建配置参数
      const config: AppDataConfig = {
        minSizeThreshold: options?.minSizeThreshold || 1024 * 1024 * 1024, // 默认1GB
        maxDepth: options?.maxDepth || 2, // 默认2层
        sortOrder: options?.sortOrder || 'desc' // 默认降序
      };

      const result = await invoke<AppDataInfo>("scan_appdata", { config });
      console.log('API: AppData扫描成功', result);
      return result;
    } catch (error) {
      console.error("API: AppData扫描失败:", error);
      throw new Error(`AppData扫描失败: ${error}`);
    }
  },

  /**
   * 流式扫描AppData目录 - 支持实时进度推送
   * @param options 扫描选项（可选）
   * @param onEvent 扫描事件回调函数
   * @returns AppData信息
   */
  async scanAppDataStreaming(
    options?: AppDataScanOptions,
    onEvent?: (event: any) => void
  ): Promise<AppDataInfo> {
    console.log('API: 开始流式扫描AppData目录', options);
    
    try {
      // 设置事件监听器
      if (onEvent) {
        // 监听扫描事件
        const unlistenScan = await (window as any).addEventListener('appdata-scan-event', (event: any) => {
          onEvent(event.payload);
        });
        
        // 监听扫描完成事件
        const unlistenComplete = await (window as any).addEventListener('appdata-scan-complete', (event: any) => {
          onEvent({ type: 'scan_complete', data: event.payload });
        });
        
        // 监听扫描错误事件
        const unlistenError = await (window as any).addEventListener('appdata-scan-error', (event: any) => {
          onEvent({ type: 'scan_error', data: event.payload });
        });
        
        // 构建配置参数
        const config: AppDataConfig = {
          minSizeThreshold: options?.minSizeThreshold || 1024 * 1024 * 1024, // 默认1GB
          maxDepth: options?.maxDepth || 2, // 默认2层
          sortOrder: options?.sortOrder || 'desc' // 默认降序
        };

        try {
          const result = await invoke<AppDataInfo>("scan_appdata_streaming", { config });
          console.log('API: 流式AppData扫描成功', result);
          
          // 清理事件监听器
          try {
            if (unlistenScan && typeof unlistenScan === 'function') {
              unlistenScan();
            }
          } catch (e) {
            console.warn('清理扫描事件监听器失败:', e);
          }
          
          try {
            if (unlistenComplete && typeof unlistenComplete === 'function') {
              unlistenComplete();
            }
          } catch (e) {
            console.warn('清理完成事件监听器失败:', e);
          }
          
          try {
            if (unlistenError && typeof unlistenError === 'function') {
              unlistenError();
            }
          } catch (e) {
            console.warn('清理错误事件监听器失败:', e);
          }
          
          return result;
        } catch (error) {
          // 清理事件监听器
          try {
            if (unlistenScan && typeof unlistenScan === 'function') {
              unlistenScan();
            }
          } catch (e) {
            console.warn('清理扫描事件监听器失败:', e);
          }
          
          try {
            if (unlistenComplete && typeof unlistenComplete === 'function') {
              unlistenComplete();
            }
          } catch (e) {
            console.warn('清理完成事件监听器失败:', e);
          }
          
          try {
            if (unlistenError && typeof unlistenError === 'function') {
              unlistenError();
            }
          } catch (e) {
            console.warn('清理错误事件监听器失败:', e);
          }
          throw error;
        }
      } else {
        // 无事件回调的简单扫描
        const config: AppDataConfig = {
          minSizeThreshold: options?.minSizeThreshold || 1024 * 1024 * 1024,
          maxDepth: options?.maxDepth || 2,
          sortOrder: options?.sortOrder || 'desc'
        };

        const result = await invoke<AppDataInfo>("scan_appdata_streaming", { config });
        console.log('API: 流式AppData扫描成功', result);
        return result;
      }
    } catch (error) {
      console.error("API: 流式AppData扫描失败:", error);
      throw new Error(`流式AppData扫描失败: ${error}`);
    }
  },

  /**
   * 获取AppData路径
   * @returns AppData路径
   */
  async getAppDataPath(): Promise<string> {
    try {
      const result = await invoke<string>("get_appdata_path");
      console.log('API: 获取AppData路径成功', result);
      return result;
    } catch (error) {
      console.error("API: 获取AppData路径失败:", error);
      throw new Error(`获取AppData路径失败: ${error}`);
    }
  },

  /**
   * 迁移AppData项目（增强版本，支持进度报告）
   * @param options 迁移选项
   * @param onProgress 进度回调函数（可选）
   * @returns 迁移结果
   */
  async migrateAppDataItems(
    options: AppDataMigrationOptions,
    onProgress?: (progress: { currentItem: string; progress: number; totalItems: number }) => void
  ): Promise<AppDataMigrationResult> {
    console.log('API: 开始迁移AppData项目', options);
    
    try {
      // 验证迁移选项
      this.validateMigrationOptions(options);
      
      // 如果提供了进度回调，设置进度监听
      if (onProgress) {
        // 模拟进度报告（实际实现需要后端支持）
        const totalItems = options.sourceItems.length;
        let currentIndex = 0;
        
        // 开始迁移前报告进度
        onProgress({
          currentItem: '开始迁移...',
          progress: 0,
          totalItems
        });
        
        // 执行迁移
        const result = await invoke<MigrationResult>("migrate_appdata_items", { options });
        
        // 模拟进度更新（实际应该由后端驱动）
        for (const item of options.sourceItems) {
          currentIndex++;
          onProgress({
            currentItem: item,
            progress: Math.round((currentIndex / totalItems) * 100),
            totalItems
          });
          
          // 模拟处理时间
          await new Promise(resolve => setTimeout(resolve, 100));
        }
        
        // 转换结果格式
        const migrationResult: AppDataMigrationResult = {
          success: result.success,
          message: result.message,
          migratedItems: result.success ? options.sourceItems.length : 0,
          failedItems: result.success ? 0 : options.sourceItems.length,
          totalSize: 0, // 可以从迁移服务获取实际大小
          targetDrive: options.targetDrive
        };
        
        console.log('API: AppData迁移成功', migrationResult);
        return migrationResult;
      } else {
        // 无进度回调的简单迁移
        const result = await invoke<MigrationResult>("migrate_appdata_items", { options });
        
        const migrationResult: AppDataMigrationResult = {
          success: result.success,
          message: result.message,
          migratedItems: result.success ? options.sourceItems.length : 0,
          failedItems: result.success ? 0 : options.sourceItems.length,
          totalSize: 0,
          targetDrive: options.targetDrive
        };
        
        console.log('API: AppData迁移成功', migrationResult);
        return migrationResult;
      }
    } catch (error) {
      console.error("API: AppData迁移失败:", error);
      
      // 详细的错误处理
      const errorMessage = this.formatMigrationError(error);
      
      // 返回错误结果
      return {
        success: false,
        message: errorMessage,
        migratedItems: 0,
        failedItems: options.sourceItems.length,
        totalSize: 0,
        targetDrive: options.targetDrive
      };
    }
  },

  /**
   * 获取系统可用盘符
   * @returns 可用盘符列表
   */
  async getAvailableDrives(): Promise<string[]> {
    try {
      const drives = await invoke<string[]>("get_available_drives");
      console.log('API: 获取可用盘符成功', drives);
      return drives;
    } catch (error) {
      console.error("API: 获取可用盘符失败:", error);
      
      // 返回默认盘符
      return ['D:\\', 'E:\\', 'F:\\'];
    }
  },

  /**
   * 获取迁移进度（增强版本）
   * @returns 当前迁移进度
   */
  async getMigrationProgress(): Promise<{
    currentItem: string;
    progress: number;
    totalItems: number;
    estimatedTimeRemaining?: number;
  }> {
    try {
      const result = await invoke<{
        current_item: string;
        progress: number;
        total_items: number;
        estimated_time_remaining?: number;
      }>("get_migration_progress");
      
      return {
        currentItem: result.current_item,
        progress: result.progress,
        totalItems: result.total_items,
        estimatedTimeRemaining: result.estimated_time_remaining
      };
    } catch (error) {
      console.error('API: 获取迁移进度失败:', error);
      return {
        currentItem: '无法获取进度信息',
        progress: 0,
        totalItems: 0
      };
    }
  },

  /**
   * 验证AppData迁移选项
   * @param options 迁移选项
   * @returns 验证结果
   */
  async validateAppDataMigrationOptions(options: AppDataMigrationOptions): Promise<{
    valid: boolean;
    items: Array<{
      path: string;
      valid: boolean;
      message?: string;
    }>;
    summary: string;
    targetDriveValid: boolean;
    targetDrive: string;
  }> {
    try {
      const result = await invoke<{
        valid: boolean;
        items: Array<{
          path: string;
          valid: boolean;
          message?: string;
        }>;
        summary: string;
        target_drive_valid: boolean;
        target_drive: string;
      }>("validate_appdata_migration_options", { options });
      
      return {
        valid: result.valid,
        items: result.items,
        summary: result.summary,
        targetDriveValid: result.target_drive_valid,
        targetDrive: result.target_drive
      };
    } catch (error) {
      console.error('API: 验证迁移选项失败:', error);
      throw new Error(`验证迁移选项失败: ${error}`);
    }
  },

  /**
   * 获取AppData扫描结果（带错误处理）
   * @param options 扫描选项（可选）
   * @returns 扫描结果包装
   */
  async getAppDataScanResult(options?: AppDataScanOptions): Promise<AppDataScanResult> {
    try {
      const data = await this.scanAppData(options);
      return {
        success: true,
        data,
        timestamp: Date.now()
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        timestamp: Date.now()
      };
    }
  },

  /**
   * 格式化AppData信息用于显示
   * @param info AppData信息
   * @returns 格式化后的信息
   */
  formatAppDataInfo(info: AppDataInfo): AppDataInfo {
    return {
      ...info,
      // 可以在这里添加额外的格式化逻辑
    };
  },

  /**
   * 获取AppData统计信息（更新版本）
   * @param info AppData信息
   * @returns 统计信息
   */
  getAppDataStatistics(info: AppDataInfo) {
    const totalItems = info.firstLevelItems.length;
    const largeItems = info.largeItems.length;
    const averageItemSize = totalItems > 0 ? info.totalSize / totalItems : 0;
    const largestItem = info.largeItems.length > 0 ? info.largeItems[0] : null;

    return {
      totalApps: totalItems,
      totalSize: info.totalSize,
      largeApps: largeItems,
      averageAppSize: averageItemSize,
      largestApp: largestItem?.name || '',
      largestAppSize: largestItem?.size || 0,
      scanDate: new Date()
    };
  },

  /**
   * 验证迁移选项的有效性
   * @param options 迁移选项
   * @throws 如果选项无效则抛出错误
   */
  validateMigrationOptions(options: AppDataMigrationOptions): void {
    if (!options.sourceItems || options.sourceItems.length === 0) {
      throw new Error('迁移源项目列表不能为空');
    }
    
    if (!options.targetDrive || options.targetDrive.trim() === '') {
      throw new Error('目标盘符不能为空');
    }
    
    // 验证目标盘符格式
    const drivePattern = /^[A-Za-z]:[\\/]?$/;
    if (!drivePattern.test(options.targetDrive)) {
      throw new Error('目标盘符格式无效，应为如 "D:" 或 "D:\" 的格式');
    }
    
    // 验证源项目路径
    for (const item of options.sourceItems) {
      if (!item || item.trim() === '') {
        throw new Error('迁移源项目路径不能为空');
      }
    }
  },

  /**
   * 格式化迁移错误信息
   * @param error 原始错误
   * @returns 格式化的错误消息
   */
  formatMigrationError(error: unknown): string {
    if (error instanceof Error) {
      // 常见的迁移错误类型
      const errorMessage = error.message.toLowerCase();
      
      if (errorMessage.includes('permission')) {
        return '迁移失败：权限不足，请确保有足够的权限访问源文件和目标位置';
      } else if (errorMessage.includes('space') || errorMessage.includes('disk')) {
        return '迁移失败：目标盘符空间不足，请清理空间后重试';
      } else if (errorMessage.includes('exists')) {
        return '迁移失败：目标位置已存在同名文件或文件夹';
      } else if (errorMessage.includes('not found') || errorMessage.includes('不存在')) {
        return '迁移失败：源文件或目标盘符不存在';
      } else if (errorMessage.includes('symlink')) {
        return '迁移失败：符号链接创建失败，可能需要管理员权限';
      } else {
        return `迁移失败：${error.message}`;
      }
    } else if (typeof error === 'string') {
      return `迁移失败：${error}`;
    } else {
      return '迁移失败：发生未知错误';
    }
  },


  /**
   * 批量验证迁移路径
   * @param items 要迁移的项目路径列表
   * @param targetDrive 目标盘符
   * @returns 验证结果列表
   */
  async validateMigrationItems(items: string[], targetDrive: string): Promise<{
    valid: boolean;
    items: Array<{
      path: string;
      valid: boolean;
      message?: string;
    }>;
    summary: string;
  }> {
    const validationResults = [];
    let validCount = 0;

    for (const item of items) {
      try {
        // 检查源路径是否存在
        const exists = await systemAPI.pathExists(item);
        if (!exists) {
          validationResults.push({
            path: item,
            valid: false,
            message: '源路径不存在'
          });
          continue;
        }

        // 检查目标盘符空间（简化检查）
        const diskInfo = await systemAPI.getDiskInfo();
        const targetDisk = diskInfo.find(disk =>
          disk.name.toUpperCase().startsWith(targetDrive.toUpperCase().charAt(0))
        );

        if (!targetDisk) {
          validationResults.push({
            path: item,
            valid: false,
            message: '目标盘符信息不可用'
          });
          continue;
        }

        // 这里应该获取文件大小进行空间检查
        // 简化处理，假设有足够的空间
        validationResults.push({
          path: item,
          valid: true,
          message: '路径有效'
        });
        validCount++;

      } catch (error) {
        validationResults.push({
          path: item,
          valid: false,
          message: '验证过程中发生错误'
        });
      }
    }

    const summary = `验证完成：${validCount}/${items.length} 个项目有效`;

    return {
      valid: validCount === items.length,
      items: validationResults,
      summary
    };
  }
};
