/**
 * 实时数据管理器
 * 专门用于管理AppData实时扫描和分析的数据流
 * 支持事件驱动、内存优化和性能监控
 */

import { ref, reactive, computed, watch, type Ref } from 'vue';
import type {
  IRealTimeDataManager,
  RealTimeDataManagerConfig,
  RealTimeDataManagerOptions,
  RealTimeScanData,
  DataUpdateBatch,
  SortUpdateRequest,
  MemoryUsageReport,
  PerformanceMetrics,
  RealTimeDataState,
  EventListener,
  EventHandlerContext,
  ScanEventHandler,
  CacheStatistics,
  DataConsistencyCheck,
  ErrorRecoveryStrategy,
  ConfigUpdate,
  EventEmitter
} from '../types/realtime';
import type { AppDataScanEvent, AppDataFirstLevelItem } from '../types/appdata';
import { appDataAPI } from './api';

/**
 * 默认配置
 */
const DEFAULT_CONFIG: RealTimeDataManagerConfig = {
  maxMemoryUsage: 150, // 150MB
  updateFrequency: 100, // 100ms
  cacheExpiration: 300, // 5分钟
  batchSize: 50,
  enableCaching: true,
  enableMemoryMonitoring: true,
  cleanupInterval: 60 // 1分钟
};

/**
 * 实时数据管理器实现
 * 管理AppData扫描的实时数据流和状态
 */
export class RealTimeDataManager implements IRealTimeDataManager {
  // 配置
  private _config: RealTimeDataManagerConfig;
  
  // 数据状态
  private _data: Ref<RealTimeScanData>;
  private _state: Ref<RealTimeDataState>;
  private _cache: Map<string, AppDataFirstLevelItem>;
  private _eventListeners: Map<string, EventListener>;
  private _eventHandlers: Map<string, ScanEventHandler[]>;
  
  // 性能监控
  private _metrics: Ref<PerformanceMetrics>;
  private _memoryReport: Ref<MemoryUsageReport>;
  
  // 扫描状态
  private _isScanning: Ref<boolean>;
  private _scanStartTime: number = 0;
  private _updateTimer: number | null = null;
  private _cleanupTimer: number | null = null;
  
  // 错误处理
  private _errorCount: Ref<number>;
  private _errorRecoveryStrategies: Map<string, ErrorRecoveryStrategy> = new Map();
  
  // 排序管理
  private _sortQueue: SortUpdateRequest[] = [];
  private _isSorting: Ref<boolean>;
  
  constructor(options: RealTimeDataManagerOptions = {}) {
    // 合并配置
    this._config = { ...DEFAULT_CONFIG, ...options.config };
    
    // 初始化响应式数据
    this._data = ref<RealTimeScanData>(this.createInitialData());
    this._state = ref<RealTimeDataState>(this.createInitialState());
    this._metrics = ref<PerformanceMetrics>(this.createInitialMetrics());
    this._memoryReport = ref<MemoryUsageReport>(this.createInitialMemoryReport());
    
    // 初始化状态
    this._isScanning = ref(false);
    this._errorCount = ref(0);
    this._isSorting = ref(false);
    
    // 初始化存储
    this._cache = new Map();
    this._eventListeners = new Map();
    this._eventHandlers = new Map();
    
    // 初始化错误恢复策略
    this.initializeErrorRecoveryStrategies();
    
    // 如果有提供事件处理器，注册它们
    if (options.eventHandlers) {
      options.eventHandlers.forEach(handler => this.registerEventHandler(handler));
    }
    
    // 设置性能监控回调
    if (options.performanceCallback) {
      this.setupPerformanceMonitoring(options.performanceCallback);
    }
    
    // 设置错误处理回调
    if (options.errorCallback) {
      this.setupErrorHandling(options.errorCallback);
    }
    
    // 启动内存监控
    if (this._config.enableMemoryMonitoring) {
      this.startMemoryMonitoring();
    }
  }
  
  //  getters
  get config(): RealTimeDataManagerConfig {
    return { ...this._config };
  }
  
  get state(): RealTimeDataState {
    return { ...this._state.value };
  }
  
  get metrics(): PerformanceMetrics {
    return { ...this._metrics.value };
  }
  
  get memoryReport(): MemoryUsageReport {
    return { ...this._memoryReport.value };
  }
  
  /**
   * 创建初始数据
   */
  private createInitialData(): RealTimeScanData {
    return {
      items: [],
      totalSize: 0,
      scanProgress: 0,
      lastUpdateTime: Date.now(),
      isScanning: false,
      sortField: 'size',
      sortOrder: 'desc',
      errorCount: 0,
      cacheHitRate: 0
    };
  }
  
  /**
   * 创建初始状态
   */
  private createInitialState(): RealTimeDataState {
    return {
      isScanning: false,
      scanProgress: 0,
      currentPath: '',
      processedItems: 0,
      totalItems: 0,
      errorCount: 0,
      lastUpdateTime: Date.now(),
      scanStartTime: 0
    };
  }
  
  /**
   * 创建初始性能指标
   */
  private createInitialMetrics(): PerformanceMetrics {
    return {
      updateTime: 0,
      sortTime: 0,
      eventProcessingTime: 0,
      cacheHitRate: 0,
      throughput: 0,
      errorRate: 0,
      memoryEfficiency: 0
    };
  }
  
  /**
   * 创建初始内存报告
   */
  private createInitialMemoryReport(): MemoryUsageReport {
    return {
      currentUsage: 0,
      peakUsage: 0,
      memoryLimit: this._config.maxMemoryUsage,
      usagePercentage: 0,
      componentUsage: {
        cache: 0,
        data: 0,
        events: 0,
        other: 0
      }
    };
  }
  
  /**
   * 初始化错误恢复策略
   */
  private initializeErrorRecoveryStrategies(): void {
    this._errorRecoveryStrategies = new Map();
    
    // 权限错误
    this._errorRecoveryStrategies.set('permission_denied', {
      errorType: 'permission_denied',
      retryCount: 0,
      retryInterval: 0,
      fallbackStrategy: 'skip',
      isRecoverable: false,
      userMessage: '权限不足，跳过此项目'
    });
    
    // 路径不存在
    this._errorRecoveryStrategies.set('path_not_found', {
      errorType: 'path_not_found',
      retryCount: 1,
      retryInterval: 1000,
      fallbackStrategy: 'skip',
      isRecoverable: true,
      userMessage: '路径不存在，跳过此项目'
    });
    
    // 内存不足
    this._errorRecoveryStrategies.set('out_of_memory', {
      errorType: 'out_of_memory',
      retryCount: 0,
      retryInterval: 0,
      fallbackStrategy: 'pause',
      isRecoverable: false,
      userMessage: '内存不足，暂停扫描'
    });
    
    // 网络超时
    this._errorRecoveryStrategies.set('network_timeout', {
      errorType: 'network_timeout',
      retryCount: 3,
      retryInterval: 5000,
      fallbackStrategy: 'skip',
      isRecoverable: true,
      userMessage: '网络超时，重试3次后跳过'
    });
  }
  
  /**
   * 初始化
   */
  async initialize(): Promise<void> {
    try {
      console.log('初始化实时数据管理器...');
      
      // 启动定时清理
      this.startPeriodicCleanup();
      
      console.log('实时数据管理器初始化完成');
    } catch (error) {
      console.error('实时数据管理器初始化失败:', error);
      throw error;
    }
  }
  
  /**
   * 开始扫描
   */
  async startScan(options: any = {}): Promise<void> {
    if (this._isScanning.value) {
      console.warn('扫描已在进行中');
      return;
    }
    
    try {
      console.log('开始实时扫描...');
      
      this._isScanning.value = true;
      this._scanStartTime = Date.now();
      this._errorCount.value = 0;
      
      // 更新状态
      this._state.value = {
        ...this._state.value,
        isScanning: true,
        scanProgress: 0,
        currentPath: '',
        processedItems: 0,
        totalItems: 0,
        errorCount: 0,
        lastUpdateTime: Date.now(),
        scanStartTime: this._scanStartTime
      };
      
      // 更新数据状态
      this._data.value = {
        ...this._data.value,
        isScanning: true,
        scanProgress: 0,
        lastUpdateTime: Date.now(),
        errorCount: 0
      };
      
      // 清理缓存
      if (this._config.enableCaching) {
        this._cache.clear();
      }
      
      // 发射扫描开始事件
      this.emit('scan_started', {
        timestamp: Date.now(),
        options
      });
      
      // 使用流式API开始扫描
      this.startStreamingScan(options);
      
    } catch (error) {
      console.error('开始扫描失败:', error);
      this._isScanning.value = false;
      throw error;
    }
  }
  
  /**
   * 开始流式扫描
   */
  private startStreamingScan(options: any): void {
    // 这里应该调用appDataAPI的流式扫描方法
    // 为了演示，我们使用模拟数据
    this.simulateStreamingScan(options);
  }
  
  /**
   * 模拟流式扫描（用于演示）
   */
  private async simulateStreamingScan(options: any): Promise<void> {
    const mockItems = this.generateMockItems(50);
    let processedCount = 0;
    
    for (const item of mockItems) {
      if (!this._isScanning.value) break;
      
      // 模拟处理时间
      await new Promise(resolve => setTimeout(resolve, 100));
      
      // 创建更新批次
      const batch: DataUpdateBatch = {
        batchId: `batch_${Date.now()}`,
        timestamp: Date.now(),
        items: [item],
        updateType: 'add',
        isFullRefresh: false
      };
      
      // 处理更新
      await this.updateData(batch);
      
      processedCount++;
      
      // 更新进度
      const progress = (processedCount / mockItems.length) * 100;
      this.updateScanProgress(progress, item.path);
    }
    
    // 扫描完成
    if (this._isScanning.value) {
      this.completeScan();
    }
  }
  
  /**
   * 生成模拟数据
   */
  private generateMockItems(count: number): AppDataFirstLevelItem[] {
    const items: AppDataFirstLevelItem[] = [];
    const parentTypes = ['Local', 'LocalLow', 'Roaming'] as const;
    const names = ['Chrome', 'Firefox', 'Edge', 'VSCode', 'Node.js', 'Python', 'Java', 'Docker', 'Git', 'npm'];
    
    for (let i = 0; i < count; i++) {
      const size = Math.random() * 5 * 1024 * 1024 * 1024; // 0-5GB
      const isLarge = size >= 1024 * 1024 * 1024; // >= 1GB
      
      items.push({
        path: `C:\\Users\\User\\AppData\\${parentTypes[i % 3]}\\${names[i % names.length]}_${i}`,
        name: `${names[i % names.length]}_${i}`,
        size: Math.floor(size),
        itemType: 'directory',
        parentType: parentTypes[i % 3],
        isLarge,
        sizePercentage: 0 // 将在更新时计算
      });
    }
    
    return items.sort((a, b) => b.size - a.size); // 按大小降序排列
  }
  
  /**
   * 更新扫描进度
   */
  private updateScanProgress(progress: number, currentPath: string): void {
    this._state.value = {
      ...this._state.value,
      scanProgress: progress,
      currentPath
    };
    
    this._data.value = {
      ...this._data.value,
      scanProgress: progress,
      lastUpdateTime: Date.now()
    };
    
    // 发射进度更新事件
    this.emit('scan_progress', {
      progress,
      currentPath,
      timestamp: Date.now()
    });
  }
  
  /**
   * 停止扫描
   */
  async stopScan(): Promise<void> {
    if (!this._isScanning.value) {
      console.warn('没有正在进行的扫描');
      return;
    }
    
    console.log('停止扫描...');
    this._isScanning.value = false;
    
    // 更新状态
    this._state.value = {
      ...this._state.value,
      isScanning: false,
      lastUpdateTime: Date.now()
    };
    
    this._data.value = {
      ...this._data.value,
      isScanning: false,
      lastUpdateTime: Date.now()
    };
    
    // 发射扫描停止事件
    this.emit('scan_stopped', {
      timestamp: Date.now(),
      finalProgress: this._state.value.scanProgress
    });
  }
  
  /**
   * 暂停扫描
   */
  async pauseScan(): Promise<void> {
    // 实现暂停逻辑
    console.log('暂停扫描...');
    this.emit('scan_paused', { timestamp: Date.now() });
  }
  
  /**
   * 恢复扫描
   */
  async resumeScan(): Promise<void> {
    // 实现恢复逻辑
    console.log('恢复扫描...');
    this.emit('scan_resumed', { timestamp: Date.now() });
  }
  
  /**
   * 完成扫描
   */
  private completeScan(): void {
    console.log('扫描完成');
    
    this._isScanning.value = false;
    const scanTime = Date.now() - this._scanStartTime;
    
    // 更新最终状态
    this._state.value = {
      ...this._state.value,
      isScanning: false,
      scanProgress: 100,
      lastUpdateTime: Date.now()
    };
    
    this._data.value = {
      ...this._data.value,
      isScanning: false,
      scanProgress: 100,
      lastUpdateTime: Date.now()
    };
    
    // 发射扫描完成事件
    this.emit('scan_completed', {
      totalItems: this._data.value.items.length,
      totalSize: this._data.value.totalSize,
      scanTimeMs: scanTime,
      timestamp: Date.now()
    });
  }
  
  /**
   * 更新数据
   */
  async updateData(batch: DataUpdateBatch): Promise<void> {
    const startTime = performance.now();
    
    try {
      // 根据更新类型处理数据
      switch (batch.updateType) {
        case 'add':
          await this.handleAddItems(batch.items);
          break;
        case 'update':
          await this.handleUpdateItems(batch.items);
          break;
        case 'delete':
          await this.handleDeleteItems(batch.items);
          break;
      }
      
      // 重新计算总大小和百分比
      this.recalculateTotals();
      
      // 更新缓存统计
      this.updateCacheStatistics();
      
      // 更新性能指标
      const updateTime = performance.now() - startTime;
      this.updateMetrics(updateTime);
      
      // 发射数据更新事件
      this.emit('data_updated', {
        batchId: batch.batchId,
        updateType: batch.updateType,
        itemCount: batch.items.length,
        updateTime
      });
      
    } catch (error) {
      console.error('数据更新失败:', error);
      this._errorCount.value++;
      throw error;
    }
  }
  
  /**
   * 处理添加项目
   */
  private async handleAddItems(items: AppDataFirstLevelItem[]): Promise<void> {
    // 添加到主数据
    this._data.value.items.push(...items);
    
    // 添加到缓存
    if (this._config.enableCaching) {
      items.forEach(item => {
        this._cache.set(item.path, item);
      });
    }
    
    // 更新处理计数
    this._state.value.processedItems += items.length;
  }
  
  /**
   * 处理更新项目
   */
  private async handleUpdateItems(items: AppDataFirstLevelItem[]): Promise<void> {
    items.forEach(updatedItem => {
      // 更新主数据
      const index = this._data.value.items.findIndex((item: AppDataFirstLevelItem) => item.path === updatedItem.path);
      if (index >= 0) {
        this._data.value.items[index] = updatedItem;
      }
      
      // 更新缓存
      if (this._config.enableCaching) {
        this._cache.set(updatedItem.path, updatedItem);
      }
    });
  }
  
  /**
   * 处理删除项目
   */
  private async handleDeleteItems(items: AppDataFirstLevelItem[]): Promise<void> {
    const pathsToDelete = new Set(items.map(item => item.path));
    
    // 从主数据中删除
    this._data.value.items = this._data.value.items.filter((item: AppDataFirstLevelItem) => !pathsToDelete.has(item.path));
    
    // 从缓存中删除
    if (this._config.enableCaching) {
      pathsToDelete.forEach(path => this._cache.delete(path));
    }
    
    // 更新处理计数
    this._state.value.processedItems = Math.max(0, this._state.value.processedItems - items.length);
  }
  
  /**
   * 重新计算总计
   */
  private recalculateTotals(): void {
    const items = this._data.value.items;
    const totalSize = items.reduce((sum: number, item: AppDataFirstLevelItem) => sum + item.size, 0);
    
    // 更新项目百分比
    items.forEach((item: AppDataFirstLevelItem) => {
      item.sizePercentage = totalSize > 0 ? (item.size / totalSize) * 100 : 0;
    });
    
    // 更新总大小
    this._data.value.totalSize = totalSize;
  }
  
  /**
   * 请求排序更新
   */
  async requestSort(update: SortUpdateRequest): Promise<void> {
    if (this._isSorting.value && !update.immediate) {
      // 如果正在排序且不是立即执行，加入队列
      this._sortQueue.push(update);
      return;
    }
    
    this._isSorting.value = true;
    const startTime = performance.now();
    
    try {
      // 更新排序设置
      this._data.value.sortField = update.sortField as 'name' | 'size';
      this._data.value.sortOrder = update.sortOrder;
      
      // 执行排序
      await this.performSort(update.sortField, update.sortOrder);
      
      // 更新性能指标
      const sortTime = performance.now() - startTime;
      this._metrics.value.sortTime = sortTime;
      
      // 发射排序完成事件
      this.emit('sort_completed', {
        sortField: update.sortField,
        sortOrder: update.sortOrder,
        sortTime,
        itemCount: this._data.value.items.length
      });
      
    } catch (error) {
      console.error('排序失败:', error);
      throw error;
    } finally {
      this._isSorting.value = false;
      
      // 处理队列中的下一个排序请求
      if (this._sortQueue.length > 0) {
        const nextUpdate = this._sortQueue.shift()!;
        await this.requestSort(nextUpdate);
      }
    }
  }
  
  /**
   * 执行排序
   */
  private async performSort(sortField: string, sortOrder: string): Promise<void> {
    const items = [...this._data.value.items];
    
    items.sort((a, b) => {
      let compareResult = 0;
      
      switch (sortField) {
        case 'name':
          compareResult = a.name.localeCompare(b.name);
          break;
        case 'size':
          compareResult = a.size - b.size;
          break;
        default:
          compareResult = a.size - b.size; // 默认按大小排序
      }
      
      return sortOrder === 'desc' ? -compareResult : compareResult;
    });
    
    this._data.value.items = items;
  }
  
  /**
   * 获取当前数据
   */
  getCurrentData(): RealTimeScanData {
    return { ...this._data.value };
  }
  
  /**
   * 获取缓存统计
   */
  getCacheStatistics(): CacheStatistics {
    const hitRate = this._metrics.value.cacheHitRate;
    const cacheSize = this._cache.size;
    
    return {
      hitCount: Math.floor(hitRate * 100),
      missCount: Math.floor((1 - hitRate) * 100),
      evictionCount: 0, // 简化实现
      memoryUsage: this.estimateCacheMemoryUsage(),
      cleanupCount: 0, // 简化实现
      lastCleanupTime: Date.now()
    };
  }
  
  /**
   * 估算缓存内存使用量
   */
  private estimateCacheMemoryUsage(): number {
    // 简化估算：每个项目大约1KB
    return (this._cache.size * 1) / 1024; // MB
  }
  
  /**
   * 更新缓存统计
   */
  private updateCacheStatistics(): void {
    const cacheHitRate = this._cache.size > 0 ? 0.8 : 0; // 模拟缓存命中率
    this._data.value.cacheHitRate = cacheHitRate;
    this._metrics.value.cacheHitRate = cacheHitRate;
  }
  
  /**
   * 更新性能指标
   */
  private updateMetrics(updateTime: number): void {
    this._metrics.value.updateTime = updateTime;
    
    // 计算吞吐量
    const itemsPerSecond = this._state.value.processedItems / (updateTime / 1000);
    this._metrics.value.throughput = itemsPerSecond;
    
    // 计算错误率
    const totalOperations = this._state.value.processedItems;
    const errorRate = totalOperations > 0 ? (this._errorCount.value / totalOperations) : 0;
    this._metrics.value.errorRate = errorRate;
    
    // 计算内存效率
    const memoryEfficiency = this._memoryReport.value.currentUsage > 0 
      ? (this._data.value.items.length / this._memoryReport.value.currentUsage) 
      : 0;
    this._metrics.value.memoryEfficiency = memoryEfficiency;
  }
  
  /**
   * 清理缓存
   */
  async cleanupCache(): Promise<void> {
    console.log('清理缓存...');
    this._cache.clear();
    this.emit('cache_cleaned', { timestamp: Date.now() });
  }
  
  /**
   * 强制内存清理
   */
  async forceMemoryCleanup(): Promise<void> {
    console.log('强制内存清理...');
    
    // 清理缓存
    await this.cleanupCache();
    
    // 重置数据（保留必要信息）
    const essentialData = {
      items: this._data.value.items.slice(0, 100), // 只保留前100个项目
      totalSize: this._data.value.totalSize,
      scanProgress: this._data.value.scanProgress,
      lastUpdateTime: Date.now(),
      isScanning: this._data.value.isScanning,
      sortField: this._data.value.sortField,
      sortOrder: this._data.value.sortOrder,
      errorCount: this._data.value.errorCount,
      cacheHitRate: 0
    };
    
    this._data.value = essentialData;
    
    // 触发垃圾回收提示
    if (global.gc) {
      global.gc();
    }
    
    this.emit('memory_cleanup_completed', { timestamp: Date.now() });
  }
  
  /**
   * 执行数据一致性检查
   */
  async performConsistencyCheck(): Promise<DataConsistencyCheck[]> {
    const checks: DataConsistencyCheck[] = [];
    
    // 检查项目总数一致性
    checks.push({
      checkId: 'item_count_consistency',
      checkType: 'count',
      expectedValue: this._state.value.processedItems,
      actualValue: this._data.value.items.length,
      passed: this._state.value.processedItems === this._data.value.items.length,
      discrepancy: this._state.value.processedItems !== this._data.value.items.length 
        ? `处理项目数 (${this._state.value.processedItems}) 与实际项目数 (${this._data.value.items.length}) 不匹配` 
        : undefined
    });
    
    // 检查总大小一致性
    const calculatedTotalSize = this._data.value.items.reduce((sum: number, item: AppDataFirstLevelItem) => sum + item.size, 0);
    checks.push({
      checkId: 'total_size_consistency',
      checkType: 'size',
      expectedValue: this._data.value.totalSize,
      actualValue: calculatedTotalSize,
      passed: this._data.value.totalSize === calculatedTotalSize,
      discrepancy: this._data.value.totalSize !== calculatedTotalSize 
        ? `记录的总大小 (${this._data.value.totalSize}) 与计算的总大小 (${calculatedTotalSize}) 不匹配` 
        : undefined
    });
    
    return checks;
  }
  
  /**
   * 获取错误恢复策略
   */
  getErrorRecoveryStrategy(errorType: string): ErrorRecoveryStrategy {
    return this._errorRecoveryStrategies.get(errorType) || {
      errorType: 'unknown',
      retryCount: 0,
      retryInterval: 0,
      fallbackStrategy: 'skip',
      isRecoverable: false,
      userMessage: '发生未知错误'
    };
  }
  
  /**
   * 启动内存监控
   */
  private startMemoryMonitoring(): void {
    // 简化实现：定期更新内存报告
    setInterval(() => {
      this.updateMemoryReport();
    }, 5000); // 每5秒更新一次
  }
  
  /**
   * 更新内存报告
   */
  private updateMemoryReport(): void {
    // 估算内存使用（简化实现）
    const dataSize = this._data.value.items.length * 1; // 假设每个项目1KB
    const cacheSize = this._cache.size * 1; // 假设每个缓存项目1KB
    const eventSize = this._eventListeners.size * 0.5; // 假设每个监听器0.5KB
    
    const currentUsage = (dataSize + cacheSize + eventSize) / 1024; // 转换为MB
    const usagePercentage = (currentUsage / this._config.maxMemoryUsage) * 100;
    
    this._memoryReport.value = {
      currentUsage,
      peakUsage: Math.max(this._memoryReport.value.peakUsage, currentUsage),
      memoryLimit: this._config.maxMemoryUsage,
      usagePercentage,
      recommendedAction: usagePercentage > 90 ? 'cleanup' : undefined,
      componentUsage: {
        cache: cacheSize / 1024,
        data: dataSize / 1024,
        events: eventSize / 1024,
        other: 0
      }
    };
    
    // 如果内存使用过高，自动清理
    if (usagePercentage > 95) {
      console.warn('内存使用过高，自动清理...');
      this.forceMemoryCleanup();
    }
  }
  
  /**
   * 启动定期清理
   */
  private startPeriodicCleanup(): void {
    this._cleanupTimer = window.setInterval(() => {
      this.performPeriodicCleanup();
    }, this._config.cleanupInterval * 1000);
  }
  
  /**
   * 执行定期清理
   */
  private async performPeriodicCleanup(): Promise<void> {
    console.log('执行定期清理...');
    
    // 清理过期缓存
    if (this._config.enableCaching) {
      const now = Date.now();
      const expirationMs = this._config.cacheExpiration * 1000;
      
      for (const [path, item] of this._cache.entries()) {
        // 简化实现：假设缓存项目没有时间戳，这里只是示例
        // 实际实现中应该在缓存项目中添加时间戳
        if (Math.random() < 0.1) { // 随机清理10%的缓存
          this._cache.delete(path);
        }
      }
    }
    
    // 更新内存报告
    this.updateMemoryReport();
  }
  
  /**
   * 设置性能监控
   */
  private setupPerformanceMonitoring(callback: (metrics: PerformanceMetrics) => void): void {
    // 定期调用回调函数
    setInterval(() => {
      callback(this._metrics.value);
    }, 1000); // 每秒更新一次
  }
  
  /**
   * 设置错误处理
   */
  private setupErrorHandling(callback: (error: Error, context: string) => void): void {
    // 监听错误事件
    this.on('error', (error: Error) => {
      callback(error, 'RealTimeDataManager');
    });
  }
  
  /**
   * 注册事件处理器
   */
  registerEventHandler(handler: ScanEventHandler): void {
    for (const eventType of handler.supportedEventTypes) {
      if (!this._eventHandlers.has(eventType)) {
        this._eventHandlers.set(eventType, []);
      }
      this._eventHandlers.get(eventType)!.push(handler);
    }
  }
  
  // 事件发射器实现
  on<T>(eventType: string, callback: (data: T) => void, priority: number = 0): string {
    const listenerId = `${eventType}_${Date.now()}_${Math.random()}`;
    const listener: EventListener<T> = {
      id: listenerId,
      eventType,
      callback,
      priority,
      once: false
    };
    
    this._eventListeners.set(listenerId, listener);
    return listenerId;
  }
  
  once<T>(eventType: string, callback: (data: T) => void, priority: number = 0): string {
    const listenerId = `${eventType}_${Date.now()}_${Math.random()}`;
    const listener: EventListener<T> = {
      id: listenerId,
      eventType,
      callback,
      priority,
      once: true
    };
    
    this._eventListeners.set(listenerId, listener);
    return listenerId;
  }
  
  off(listenerId: string): boolean {
    return this._eventListeners.delete(listenerId);
  }
  
  emit<T>(eventType: string, data: T): void {
    // 获取相关监听器
    const listeners = Array.from(this._eventListeners.values())
      .filter(listener => listener.eventType === eventType)
      .sort((a, b) => b.priority - a.priority);
    
    // 执行监听器
    for (const listener of listeners) {
      try {
        listener.callback(data);
        
        // 如果是一次性监听器，移除它
        if (listener.once) {
          this._eventListeners.delete(listener.id);
        }
      } catch (error) {
        console.error(`事件监听器执行失败: ${listener.id}`, error);
      }
    }
    
    // 调用事件处理器
    const handlers = this._eventHandlers.get(eventType) || [];
    for (const handler of handlers) {
      try {
        const context: EventHandlerContext = {
          dataManager: this,
          currentData: this._data.value,
          eventEmitter: this,
          performanceMonitor: this._metrics.value
        };
        
        handler.handleEvent(data as any, context);
      } catch (error) {
        console.error(`事件处理器执行失败: ${handler.name}`, error);
      }
    }
  }
  
  removeAllListeners(eventType?: string): void {
    if (eventType) {
      // 移除指定类型的所有监听器
      for (const [id, listener] of this._eventListeners.entries()) {
        if (listener.eventType === eventType) {
          this._eventListeners.delete(id);
        }
      }
    } else {
      // 移除所有监听器
      this._eventListeners.clear();
    }
  }
  
  /**
   * 销毁
   */
  async destroy(): Promise<void> {
    console.log('销毁实时数据管理器...');
    
    // 停止扫描
    if (this._isScanning.value) {
      await this.stopScan();
    }
    
    // 清理定时器
    if (this._updateTimer) {
      clearInterval(this._updateTimer);
      this._updateTimer = null;
    }
    
    if (this._cleanupTimer) {
      clearInterval(this._cleanupTimer);
      this._cleanupTimer = null;
    }
    
    // 清理监听器
    this.removeAllListeners();
    
    // 清理缓存
    this._cache.clear();
    
    // 清理处理器
    this._eventHandlers.clear();
    this._errorRecoveryStrategies.clear();
    
    console.log('实时数据管理器已销毁');
  }
}

/**
 * 创建实时数据管理器实例的工厂函数
 */
export function createRealTimeDataManager(options: RealTimeDataManagerOptions = {}): IRealTimeDataManager {
  return new RealTimeDataManager(options);
}

/**
 * 默认导出
 */
export default RealTimeDataManager;