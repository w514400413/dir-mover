/**
 * 实时数据管理类型定义
 * 专门为AppData实时扫描和分析功能提供类型支持
 * 支持流式数据更新、事件驱动架构和性能监控
 */

import type {
  AppDataFirstLevelItem,
  AppDataScanEvent,
  SortOrder,
  AppDataSortField,
  RealTimeScanData,
  CacheStatistics
} from './appdata';

// 重新导出从appdata导入的类型，使它们可在realtime模块中使用
export type { RealTimeScanData, CacheStatistics } from './appdata';

/**
 * 实时数据管理器配置接口
 * 定义实时数据管理的行为参数和性能限制
 */
export interface RealTimeDataManagerConfig {
  /** 最大内存使用限制 (MB) - 默认150MB */
  maxMemoryUsage: number;
  /** 更新频率限制 (毫秒) - 默认100ms */
  updateFrequency: number;
  /** 缓存过期时间 (秒) - 默认300秒 (5分钟) */
  cacheExpiration: number;
  /** 批量更新大小 - 默认50个项目 */
  batchSize: number;
  /** 是否启用缓存 - 默认true */
  enableCaching: boolean;
  /** 是否启用内存监控 - 默认true */
  enableMemoryMonitoring: boolean;
  /** 自动清理间隔 (秒) - 默认60秒 */
  cleanupInterval: number;
}

/**
 * 事件监听器接口
 * 定义事件监听器的标准结构
 */
export interface EventListener<T = any> {
  /** 监听器ID - 用于识别和移除监听器 */
  id: string;
  /** 事件类型 - 监听的具体事件 */
  eventType: string;
  /** 回调函数 - 事件触发时执行 */
  callback: (data: T) => void;
  /** 优先级 - 用于事件处理顺序 */
  priority: number;
  /** 是否一次性监听器 - 执行后是否自动移除 */
  once: boolean;
}

/**
 * 事件发射器接口
 * 定义事件发射器的标准行为
 */
export interface EventEmitter {
  /** 添加事件监听器 */
  on<T>(eventType: string, callback: (data: T) => void, priority?: number): string;
  /** 添加一次性事件监听器 */
  once<T>(eventType: string, callback: (data: T) => void, priority?: number): string;
  /** 移除事件监听器 */
  off(listenerId: string): boolean;
  /** 发射事件 */
  emit<T>(eventType: string, data: T): void;
  /** 移除所有指定类型的事件监听器 */
  removeAllListeners(eventType?: string): void;
}

/**
 * 数据更新批次接口
 * 用于批量处理数据更新，提高性能
 */
export interface DataUpdateBatch {
  /** 批次ID - 唯一标识 */
  batchId: string;
  /** 更新时间戳 */
  timestamp: number;
  /** 更新的项目列表 */
  items: AppDataFirstLevelItem[];
  /** 更新类型 - 添加、更新、删除 */
  updateType: 'add' | 'update' | 'delete';
  /** 是否完整刷新 - 是否替换整个数据集 */
  isFullRefresh: boolean;
  /** 源事件 - 触发此批次的事件 */
  sourceEvent?: AppDataScanEvent;
}

/**
 * 排序更新请求接口
 * 用于请求动态排序更新
 */
export interface SortUpdateRequest {
  /** 排序字段 */
  sortField: AppDataSortField;
  /** 排序顺序 */
  sortOrder: SortOrder;
  /** 是否立即执行 - 否的话会加入队列 */
  immediate: boolean;
  /** 优先级 - 用于排序请求队列 */
  priority: number;
}

/**
 * 内存使用报告接口
 * 用于监控和报告内存使用情况
 */
export interface MemoryUsageReport {
  /** 当前内存使用 (MB) */
  currentUsage: number;
  /** 峰值内存使用 (MB) */
  peakUsage: number;
  /** 内存限制 (MB) */
  memoryLimit: number;
  /** 使用率百分比 */
  usagePercentage: number;
  /** 建议操作 - 当接近限制时的建议 */
  recommendedAction?: 'cleanup' | 'pause' | 'reduce_batch_size';
  /** 详细组件内存使用 */
  componentUsage: {
    cache: number;
    data: number;
    events: number;
    other: number;
  };
}

/**
 * 性能指标接口
 * 用于收集和报告性能数据
 */
export interface PerformanceMetrics {
  /** 更新时间 (毫秒) - 数据更新的平均时间 */
  updateTime: number;
  /** 排序时间 (毫秒) - 排序操作的平均时间 */
  sortTime: number;
  /** 事件处理时间 (毫秒) - 事件处理的平均时间 */
  eventProcessingTime: number;
  /** 缓存命中率 (百分比) */
  cacheHitRate: number;
  /** 数据吞吐量 (项目/秒) - 每秒处理的项目数 */
  throughput: number;
  /** 错误率 (百分比) - 操作失败的比例 */
  errorRate: number;
  /** 内存效率 - 每MB内存处理的项目数 */
  memoryEfficiency: number;
}

/**
 * 实时数据状态接口
 * 描述实时数据管理器的当前状态
 */
export interface RealTimeDataState {
  /** 是否正在扫描 */
  isScanning: boolean;
  /** 当前扫描进度 (0-100) */
  scanProgress: number;
  /** 当前扫描路径 */
  currentPath: string;
  /** 已处理项目数 */
  processedItems: number;
  /** 总项目数 */
  totalItems: number;
  /** 错误计数 */
  errorCount: number;
  /** 最后更新时间 */
  lastUpdateTime: number;
  /** 扫描开始时间 */
  scanStartTime: number;
  /** 预计完成时间 */
  estimatedCompletionTime?: number;
}

/**
 * 错误恢复策略接口
 * 定义不同类型错误的恢复策略
 */
export interface ErrorRecoveryStrategy {
  /** 错误类型 */
  errorType: string;
  /** 重试次数 */
  retryCount: number;
  /** 重试间隔 (毫秒) */
  retryInterval: number;
  /** 回退策略 - 失败时的处理方式 */
  fallbackStrategy: 'skip' | 'pause' | 'abort';
  /** 是否可恢复 */
  isRecoverable: boolean;
  /** 用户提示消息 */
  userMessage: string;
}

/**
 * 数据一致性检查接口
 * 用于验证数据的一致性和完整性
 */
export interface DataConsistencyCheck {
  /** 检查ID */
  checkId: string;
  /** 检查类型 */
  checkType: 'size' | 'count' | 'structure' | 'timestamp';
  /** 预期值 */
  expectedValue: any;
  /** 实际值 */
  actualValue: any;
  /** 是否通过 */
  passed: boolean;
  /** 差异描述 */
  discrepancy?: string;
  /** 建议操作 */
  recommendedAction?: string;
}

/**
 * 实时数据管理器接口
 * 定义实时数据管理器的核心功能
 */
export interface IRealTimeDataManager extends EventEmitter {
  /** 配置 */
  readonly config: RealTimeDataManagerConfig;
  /** 当前状态 */
  readonly state: RealTimeDataState;
  /** 性能指标 */
  readonly metrics: PerformanceMetrics;
  /** 内存使用报告 */
  readonly memoryReport: MemoryUsageReport;

  /** 初始化 */
  initialize(): Promise<void>;
  /** 开始扫描 */
  startScan(options?: any): Promise<void>;
  /** 停止扫描 */
  stopScan(): Promise<void>;
  /** 暂停扫描 */
  pauseScan(): Promise<void>;
  /** 恢复扫描 */
  resumeScan(): Promise<void>;
  
  /** 获取当前数据 */
  getCurrentData(): RealTimeScanData;
  /** 更新数据 */
  updateData(batch: DataUpdateBatch): Promise<void>;
  /** 请求排序更新 */
  requestSort(update: SortUpdateRequest): Promise<void>;
  
  /** 获取缓存统计 */
  getCacheStatistics(): CacheStatistics;
  /** 清理缓存 */
  cleanupCache(): Promise<void>;
  /** 强制内存清理 */
  forceMemoryCleanup(): Promise<void>;
  
  /** 执行数据一致性检查 */
  performConsistencyCheck(): Promise<DataConsistencyCheck[]>;
  /** 获取错误恢复策略 */
  getErrorRecoveryStrategy(errorType: string): ErrorRecoveryStrategy;
  
  /** 销毁 - 清理资源 */
  destroy(): Promise<void>;
}

/**
 * 扫描事件处理器接口
 * 定义扫描事件的处理逻辑
 */
export interface ScanEventHandler {
  /** 处理器名称 */
  name: string;
  /** 支持的事件类型 */
  supportedEventTypes: string[];
  /** 处理事件 */
  handleEvent(event: AppDataScanEvent, context: EventHandlerContext): Promise<void>;
  /** 获取优先级 */
  getPriority(): number;
}

/**
 * 事件处理器上下文接口
 * 提供事件处理所需的上下文信息
 */
export interface EventHandlerContext {
  /** 数据管理器实例 */
  dataManager: IRealTimeDataManager;
  /** 当前数据状态 */
  currentData: RealTimeScanData;
  /** 事件发射器 */
  eventEmitter: EventEmitter;
  /** 性能监控器 */
  performanceMonitor: PerformanceMetrics;
}

/**
 * 数据验证结果接口
 * 用于报告数据验证的结果
 */
export interface DataValidationResult {
  /** 是否有效 */
  isValid: boolean;
  /** 验证类型 */
  validationType: string;
  /** 错误信息 */
  errors: string[];
  /** 警告信息 */
  warnings: string[];
  /** 建议操作 */
  recommendations: string[];
}

/**
 * 实时配置更新接口
 * 用于动态更新配置
 */
export interface ConfigUpdate {
  /** 配置项名称 */
  key: keyof RealTimeDataManagerConfig;
  /** 新值 */
  value: any;
  /** 是否立即生效 */
  immediate: boolean;
  /** 验证函数 - 可选的配置验证 */
  validator?: (value: any) => boolean;
}

/**
 * 数据导出选项接口
 * 用于导出实时数据
 */
export interface DataExportOptions {
  /** 导出格式 */
  format: 'json' | 'csv' | 'xml';
  /** 包含的字段 */
  includeFields: string[];
  /** 过滤器 - 可选的数据过滤 */
  filter?: (item: AppDataFirstLevelItem) => boolean;
  /** 是否包含统计信息 */
  includeStatistics: boolean;
  /** 是否包含元数据 */
  includeMetadata: boolean;
}

/**
 * 数据导出结果接口
 * 导出操作的结果
 */
export interface DataExportResult {
  /** 是否成功 */
  success: boolean;
  /** 导出数据 */
  data?: string;
  /** 文件路径 - 如果保存到文件 */
  filePath?: string;
  /** 错误信息 */
  error?: string;
  /** 导出时间 */
  exportTime: number;
  /** 数据大小 */
  dataSize: number;
}

/**
 * 实时数据管理器创建选项接口
 * 用于创建实时数据管理器实例
 */
export interface RealTimeDataManagerOptions {
  /** 配置 - 可选的自定义配置 */
  config?: Partial<RealTimeDataManagerConfig>;
  /** 事件处理器 - 可选的自定义事件处理器 */
  eventHandlers?: ScanEventHandler[];
  /** 性能监控回调 - 可选的性能监控 */
  performanceCallback?: (metrics: PerformanceMetrics) => void;
  /** 错误处理回调 - 可选的错误处理 */
  errorCallback?: (error: Error, context: string) => void;
  /** 日志级别 - 可选的日志配置 */
  logLevel?: 'debug' | 'info' | 'warn' | 'error';
}