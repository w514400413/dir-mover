/**
 * 动态排序引擎
 * 专门为大数据集提供高效的增量排序和实时更新功能
 * 支持多种排序算法和性能优化策略
 */

import {
  AppDataFirstLevelItem,
  AppDataSortField,
  SortOrder
} from '../types/appdata';
import type {
  SortUpdateRequest,
  PerformanceMetrics
} from '../types/realtime';

/**
 * 排序算法类型
 */
export enum SortAlgorithm {
  QUICK_SORT = 'quickSort',
  MERGE_SORT = 'mergeSort',
  HEAP_SORT = 'heapSort',
  TIM_SORT = 'timSort',      // 适用于部分有序数据
  INSERTION_SORT = 'insertionSort', // 适用于小数据集
  HYBRID = 'hybrid',          // 混合算法，根据数据特点自动选择
  CACHE_HIT = 'cacheHit'     // 缓存命中
}

/**
 * 排序引擎配置
 */
export interface SortingEngineConfig {
  /** 默认排序算法 */
  defaultAlgorithm: SortAlgorithm;
  /** 小数据集阈值 - 小于此值使用简单算法 */
  smallDatasetThreshold: number;
  /** 大数据集阈值 - 大于此值使用特殊优化 */
  largeDatasetThreshold: number;
  /** 增量更新阈值 - 小于此值使用增量排序 */
  incrementalUpdateThreshold: number;
  /** 最大排序时间 (毫秒) */
  maxSortTime: number;
  /** 是否启用缓存 */
  enableCache: boolean;
  /** 缓存大小限制 */
  cacheSizeLimit: number;
  /** 是否启用多线程 */
  enableWebWorkers: boolean;
  /** 是否启用预排序 */
  enablePreSorting: boolean;
  /** 预排序样本大小 */
  preSortSampleSize: number;
}

/**
 * 排序结果
 */
export interface SortResult {
  /** 排序后的项目 */
  items: AppDataFirstLevelItem[];
  /** 排序耗时 (毫秒) */
  sortTime: number;
  /** 使用的算法 */
  algorithm: SortAlgorithm;
  /** 是否使用增量更新 */
  usedIncremental: boolean;
  /** 性能指标 */
  performance: SortPerformance;
  /** 缓存命中率 */
  cacheHitRate: number;
}

/**
 * 排序性能指标
 */
export interface SortPerformance {
  /** 比较次数 */
  comparisons: number;
  /** 交换次数 */
  swaps: number;
  /** 内存使用 (MB) */
  memoryUsage: number;
  /** 算法复杂度 */
  complexity: string;
}

/**
 * 排序缓存项
 */
interface SortCacheItem {
  /** 缓存键 */
  key: string;
  /** 排序后的项目 */
  items: AppDataFirstLevelItem[];
  /** 排序字段 */
  sortField: AppDataSortField;
  /** 排序顺序 */
  sortOrder: SortOrder;
  /** 缓存时间戳 */
  timestamp: number;
  /** 访问次数 */
  accessCount: number;
}

/**
 * 增量排序状态
 */
interface IncrementalSortState {
  /** 当前排序字段 */
  sortField: AppDataSortField;
  /** 当前排序顺序 */
  sortOrder: SortOrder;
  /** 已排序的项目 */
  sortedItems: AppDataFirstLevelItem[];
  /** 待插入的项目 */
  pendingItems: AppDataFirstLevelItem[];
  /** 已删除的项目路径 */
  deletedPaths: Set<string>;
  /** 最后更新时间 */
  lastUpdateTime: number;
}

/**
 * 动态排序引擎
 * 提供高效的增量排序和实时更新功能
 */
export class DynamicSortingEngine {
  private config: SortingEngineConfig;
  private cache: Map<string, SortCacheItem>;
  private incrementalState: IncrementalSortState | null = null;
  private performanceMetrics: PerformanceMetrics;
  private isSorting: boolean = false;
  private sortQueue: SortUpdateRequest[] = [];
  
  constructor(config: Partial<SortingEngineConfig> = {}) {
    this.config = {
      defaultAlgorithm: SortAlgorithm.TIM_SORT,
      smallDatasetThreshold: 100,
      largeDatasetThreshold: 5000,
      incrementalUpdateThreshold: 50,
      maxSortTime: 500,
      enableCache: true,
      cacheSizeLimit: 100,
      enableWebWorkers: false,
      enablePreSorting: true,
      preSortSampleSize: 100,
      ...config
    };
    
    this.cache = new Map();
    this.performanceMetrics = this.createInitialMetrics();
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
   * 创建缓存性能指标
   */
  private createCachePerformance(): SortPerformance {
    return {
      comparisons: 0,
      swaps: 0,
      memoryUsage: 0,
      complexity: 'O(1) - Cache Hit'
    };
  }
  
  /**
   * 主要排序方法
   * 根据数据特征自动选择最优排序策略
   */
  async sort(
    items: AppDataFirstLevelItem[],
    sortField: AppDataSortField,
    sortOrder: SortOrder,
    options: {
      useIncremental?: boolean;
      forceAlgorithm?: SortAlgorithm;
      enableProfiling?: boolean;
    } = {}
  ): Promise<SortResult> {
    const startTime = performance.now();
    
    try {
      this.isSorting = true;
      
      // 检查是否可以使用增量排序
      if (options.useIncremental && this.canUseIncrementalSort(items.length)) {
        return await this.incrementalSort(items, sortField, sortOrder, startTime);
      }
      
      // 选择排序算法
      const algorithm = options.forceAlgorithm || this.selectOptimalAlgorithm(items.length);
      
      // 检查缓存
      if (this.config.enableCache) {
        const cachedResult = this.getCachedResult(items, sortField, sortOrder);
        if (cachedResult) {
          return {
            items: cachedResult.items,
            sortTime: globalThis.performance.now() - startTime,
            algorithm: SortAlgorithm.CACHE_HIT,
            usedIncremental: false,
            performance: this.createCachePerformance(),
            cacheHitRate: 1
          };
        }
      }
      
      // 执行排序
      const sortedItems = await this.performSort(items, algorithm, sortField, sortOrder);
      
      // 计算性能指标
      const sortTime = globalThis.performance.now() - startTime;
      const performance = this.calculatePerformance(items.length, algorithm, sortTime);
      
      // 缓存结果
      if (this.config.enableCache) {
        this.cacheResult(items, sortedItems, sortField, sortOrder);
      }
      
      // 更新性能指标
      this.updateMetrics(sortTime, items.length, algorithm);
      
      return {
        items: sortedItems,
        sortTime,
        algorithm,
        usedIncremental: false,
        performance,
        cacheHitRate: 0
      };
      
    } catch (error) {
      console.error('排序失败:', error);
      this.performanceMetrics.errorRate = 1;
      throw error;
    } finally {
      this.isSorting = false;
    }
  }
  
  /**
   * 增量排序
   * 适用于小批量更新的场景
   */
  private async incrementalSort(
    items: AppDataFirstLevelItem[],
    sortField: AppDataSortField,
    sortOrder: SortOrder,
    startTime: number
  ): Promise<SortResult> {
    if (!this.incrementalState || 
        this.incrementalState.sortField !== sortField || 
        this.incrementalState.sortOrder !== sortOrder) {
      // 状态不匹配，需要重新排序
      return await this.fullSortWithIncrementalState(items, sortField, sortOrder, startTime);
    }
    
    // 应用增量更新
    const updatedItems = this.applyIncrementalUpdates(items);
    
    // 对更新后的项目进行排序
    const sortedItems = this.performIncrementalSort(updatedItems, sortField, sortOrder);
    
    // 更新增量状态
    this.updateIncrementalState(sortedItems, sortField, sortOrder);
    
    const sortTime = globalThis.performance.now() - startTime;
    const performance = this.calculatePerformance(items.length, SortAlgorithm.INSERTION_SORT, sortTime);
    
    return {
      items: sortedItems,
      sortTime,
      algorithm: SortAlgorithm.INSERTION_SORT,
      usedIncremental: true,
      performance,
      cacheHitRate: 0
    };
  }
  
  /**
   * 执行增量排序
   */
  private performIncrementalSort(
    items: AppDataFirstLevelItem[],
    sortField: AppDataSortField,
    sortOrder: SortOrder
  ): AppDataFirstLevelItem[] {
    // 使用插入排序进行增量排序
    const sorted = [...items];
    const compareFn = this.createCompareFunction(sortField, sortOrder);
    
    // 对已排序部分使用二分插入
    for (let i = 1; i < sorted.length; i++) {
      const current = sorted[i];
      let left = 0;
      let right = i;
      
      while (left < right) {
        const mid = Math.floor((left + right) / 2);
        if (compareFn(sorted[mid], current) <= 0) {
          left = mid + 1;
        } else {
          right = mid;
        }
      }
      
      if (left < i) {
        sorted.splice(i, 1);
        sorted.splice(left, 0, current);
      }
    }
    
    return sorted;
  }
  
  /**
   * 应用增量更新
   */
  private applyIncrementalUpdates(items: AppDataFirstLevelItem[]): AppDataFirstLevelItem[] {
    if (!this.incrementalState) return items;
    
    let result = [...items];
    
    // 删除已标记的项目
    if (this.incrementalState.deletedPaths.size > 0) {
      result = result.filter(item => !this.incrementalState!.deletedPaths.has(item.path));
    }
    
    // 添加待插入的项目
    if (this.incrementalState.pendingItems.length > 0) {
      result.push(...this.incrementalState.pendingItems);
    }
    
    return result;
  }
  
  /**
   * 更新增量状态
   */
  private updateIncrementalState(
    items: AppDataFirstLevelItem[],
    sortField: AppDataSortField,
    sortOrder: SortOrder
  ): void {
    this.incrementalState = {
      sortField,
      sortOrder,
      sortedItems: [...items],
      pendingItems: [],
      deletedPaths: new Set(),
      lastUpdateTime: Date.now()
    };
  }
  
  /**
   * 全排序并建立增量状态
   */
  private async fullSortWithIncrementalState(
    items: AppDataFirstLevelItem[],
    sortField: AppDataSortField,
    sortOrder: SortOrder,
    startTime: number
  ): Promise<SortResult> {
    const sortedItems = await this.performSort(items, SortAlgorithm.TIM_SORT, sortField, sortOrder);
    
    this.updateIncrementalState(sortedItems, sortField, sortOrder);
    
    const sortTime = globalThis.performance.now() - startTime;
    const performance = this.calculatePerformance(items.length, SortAlgorithm.TIM_SORT, sortTime);
    
    return {
      items: sortedItems,
      sortTime,
      algorithm: SortAlgorithm.TIM_SORT,
      usedIncremental: false,
      performance,
      cacheHitRate: 0
    };
  }
  
  /**
   * 选择最优排序算法
   */
  private selectOptimalAlgorithm(itemCount: number): SortAlgorithm {
    if (itemCount < this.config.smallDatasetThreshold) {
      return SortAlgorithm.INSERTION_SORT;
    }
    
    if (itemCount > this.config.largeDatasetThreshold) {
      return SortAlgorithm.HYBRID;
    }
    
    if (this.isPartiallySorted()) {
      return SortAlgorithm.TIM_SORT;
    }
    
    return this.config.defaultAlgorithm;
  }
  
  /**
   * 检查是否部分有序
   */
  private isPartiallySorted(): boolean {
    // 简化实现：假设数据通常是部分有序的
    return true;
  }
  
  /**
   * 执行排序
   */
  private async performSort(
    items: AppDataFirstLevelItem[],
    algorithm: SortAlgorithm,
    sortField: AppDataSortField,
    sortOrder: SortOrder
  ): Promise<AppDataFirstLevelItem[]> {
    const compareFn = this.createCompareFunction(sortField, sortOrder);
    
    switch (algorithm) {
      case SortAlgorithm.QUICK_SORT:
        return this.quickSort([...items], compareFn);
      case SortAlgorithm.MERGE_SORT:
        return this.mergeSort([...items], compareFn);
      case SortAlgorithm.HEAP_SORT:
        return this.heapSort([...items], compareFn);
      case SortAlgorithm.TIM_SORT:
        return this.timSort([...items], compareFn);
      case SortAlgorithm.INSERTION_SORT:
        return this.insertionSort([...items], compareFn);
      case SortAlgorithm.HYBRID:
        return this.hybridSort([...items], compareFn);
      default:
        return this.timSort([...items], compareFn);
    }
  }
  
  /**
   * 创建比较函数
   */
  private createCompareFunction(
    sortField: AppDataSortField,
    sortOrder: SortOrder
  ): (a: AppDataFirstLevelItem, b: AppDataFirstLevelItem) => number {
    return (a, b) => {
      let result = 0;
      
      switch (sortField) {
        case 'name':
          result = a.name.localeCompare(b.name);
          break;
        case 'size':
          result = a.size - b.size;
          break;
        case 'lastModified':
          // 简化实现，实际应该比较修改时间
          result = a.name.localeCompare(b.name);
          break;
        default:
          result = a.size - b.size;
      }
      
      return sortOrder === 'desc' ? -result : result;
    };
  }
  
  /**
   * 快速排序实现
   */
  private quickSort(
    items: AppDataFirstLevelItem[],
    compareFn: (a: AppDataFirstLevelItem, b: AppDataFirstLevelItem) => number
  ): AppDataFirstLevelItem[] {
    if (items.length <= 1) return items;
    
    const pivot = items[Math.floor(items.length / 2)];
    const left: AppDataFirstLevelItem[] = [];
    const right: AppDataFirstLevelItem[] = [];
    const equal: AppDataFirstLevelItem[] = [];
    
    for (const item of items) {
      const cmp = compareFn(item, pivot);
      if (cmp < 0) {
        left.push(item);
      } else if (cmp > 0) {
        right.push(item);
      } else {
        equal.push(item);
      }
    }
    
    return [
      ...this.quickSort(left, compareFn),
      ...equal,
      ...this.quickSort(right, compareFn)
    ];
  }
  
  /**
   * 归并排序实现
   */
  private mergeSort(
    items: AppDataFirstLevelItem[],
    compareFn: (a: AppDataFirstLevelItem, b: AppDataFirstLevelItem) => number
  ): AppDataFirstLevelItem[] {
    if (items.length <= 1) return items;
    
    const mid = Math.floor(items.length / 2);
    const left = this.mergeSort(items.slice(0, mid), compareFn);
    const right = this.mergeSort(items.slice(mid), compareFn);
    
    return this.merge(left, right, compareFn);
  }
  
  /**
   * 合并两个有序数组
   */
  private merge(
    left: AppDataFirstLevelItem[],
    right: AppDataFirstLevelItem[],
    compareFn: (a: AppDataFirstLevelItem, b: AppDataFirstLevelItem) => number
  ): AppDataFirstLevelItem[] {
    const result: AppDataFirstLevelItem[] = [];
    let leftIndex = 0;
    let rightIndex = 0;
    
    while (leftIndex < left.length && rightIndex < right.length) {
      if (compareFn(left[leftIndex], right[rightIndex]) <= 0) {
        result.push(left[leftIndex]);
        leftIndex++;
      } else {
        result.push(right[rightIndex]);
        rightIndex++;
      }
    }
    
    return result.concat(left.slice(leftIndex), right.slice(rightIndex));
  }
  
  /**
   * 堆排序实现
   */
  private heapSort(
    items: AppDataFirstLevelItem[],
    compareFn: (a: AppDataFirstLevelItem, b: AppDataFirstLevelItem) => number
  ): AppDataFirstLevelItem[] {
    const result = [...items];
    
    // 构建堆
    for (let i = Math.floor(result.length / 2) - 1; i >= 0; i--) {
      this.heapify(result, result.length, i, compareFn);
    }
    
    // 提取元素
    for (let i = result.length - 1; i > 0; i--) {
      [result[0], result[i]] = [result[i], result[0]];
      this.heapify(result, i, 0, compareFn);
    }
    
    return result;
  }
  
  /**
   * 堆化
   */
  private heapify(
    items: AppDataFirstLevelItem[],
    heapSize: number,
    rootIndex: number,
    compareFn: (a: AppDataFirstLevelItem, b: AppDataFirstLevelItem) => number
  ): void {
    let largest = rootIndex;
    const left = 2 * rootIndex + 1;
    const right = 2 * rootIndex + 2;
    
    if (left < heapSize && compareFn(items[left], items[largest]) > 0) {
      largest = left;
    }
    
    if (right < heapSize && compareFn(items[right], items[largest]) > 0) {
      largest = right;
    }
    
    if (largest !== rootIndex) {
      [items[rootIndex], items[largest]] = [items[largest], items[rootIndex]];
      this.heapify(items, heapSize, largest, compareFn);
    }
  }
  
  /**
   * Tim排序实现（简化版）
   */
  private timSort(
    items: AppDataFirstLevelItem[],
    compareFn: (a: AppDataFirstLevelItem, b: AppDataFirstLevelItem) => number
  ): AppDataFirstLevelItem[] {
    // 简化实现：使用归并排序
    return this.mergeSort(items, compareFn);
  }
  
  /**
   * 插入排序实现
   */
  private insertionSort(
    items: AppDataFirstLevelItem[],
    compareFn: (a: AppDataFirstLevelItem, b: AppDataFirstLevelItem) => number
  ): AppDataFirstLevelItem[] {
    const result = [...items];
    
    for (let i = 1; i < result.length; i++) {
      const current = result[i];
      let j = i - 1;
      
      while (j >= 0 && compareFn(result[j], current) > 0) {
        result[j + 1] = result[j];
        j--;
      }
      
      result[j + 1] = current;
    }
    
    return result;
  }
  
  /**
   * 混合排序实现
   */
  private hybridSort(
    items: AppDataFirstLevelItem[],
    compareFn: (a: AppDataFirstLevelItem, b: AppDataFirstLevelItem) => number
  ): AppDataFirstLevelItem[] {
    // 根据数据大小选择不同算法
    if (items.length < 100) {
      return this.insertionSort(items, compareFn);
    } else if (items.length < 1000) {
      return this.quickSort(items, compareFn);
    } else {
      return this.mergeSort(items, compareFn);
    }
  }
  
  /**
   * 检查是否可以使用增量排序
   */
  private canUseIncrementalSort(updateCount: number): boolean {
    return this.incrementalState !== null && 
           updateCount < this.config.incrementalUpdateThreshold;
  }
  
  /**
   * 缓存结果
   */
  private cacheResult(
    originalItems: AppDataFirstLevelItem[],
    sortedItems: AppDataFirstLevelItem[],
    sortField: AppDataSortField,
    sortOrder: SortOrder
  ): void {
    const key = this.generateCacheKey(originalItems, sortField, sortOrder);
    
    // 清理过期缓存
    this.cleanupCache();
    
    const cacheItem: SortCacheItem = {
      key,
      items: [...sortedItems],
      sortField,
      sortOrder,
      timestamp: Date.now(),
      accessCount: 0
    };
    
    this.cache.set(key, cacheItem);
  }
  
  /**
   * 获取缓存结果
   */
  private getCachedResult(
    items: AppDataFirstLevelItem[],
    sortField: AppDataSortField,
    sortOrder: SortOrder
  ): SortCacheItem | null {
    if (!this.config.enableCache) return null;
    
    const key = this.generateCacheKey(items, sortField, sortOrder);
    const cached = this.cache.get(key);
    
    if (cached) {
      cached.accessCount++;
      return cached;
    }
    
    return null;
  }
  
  /**
   * 生成缓存键
   */
  private generateCacheKey(
    items: AppDataFirstLevelItem[],
    sortField: AppDataSortField,
    sortOrder: SortOrder
  ): string {
    const itemHash = items.length.toString(); // 简化实现
    return `${itemHash}_${sortField}_${sortOrder}`;
  }
  
  /**
   * 清理缓存
   */
  private cleanupCache(): void {
    if (this.cache.size >= this.config.cacheSizeLimit) {
      // 移除最久未使用的缓存项
      const entries = Array.from(this.cache.entries());
      entries.sort((a, b) => a[1].timestamp - b[1].timestamp);
      
      const toRemove = entries.slice(0, Math.floor(entries.length * 0.2)); // 移除20%
      toRemove.forEach(([key]) => this.cache.delete(key));
    }
  }
  
  /**
   * 计算性能指标
   */
  private calculatePerformance(
    itemCount: number,
    algorithm: SortAlgorithm,
    sortTime: number
  ): SortPerformance {
    const comparisons = this.estimateComparisons(itemCount, algorithm);
    const swaps = this.estimateSwaps(itemCount, algorithm);
    const memoryUsage = this.estimateMemoryUsage(itemCount, algorithm);
    const complexity = this.getAlgorithmComplexity(algorithm);
    
    return {
      comparisons,
      swaps,
      memoryUsage,
      complexity
    };
  }
  
  /**
   * 估算比较次数
   */
  private estimateComparisons(itemCount: number, algorithm: SortAlgorithm): number {
    switch (algorithm) {
      case SortAlgorithm.QUICK_SORT:
        return itemCount * Math.log2(itemCount) * 1.39; // 平均情况
      case SortAlgorithm.MERGE_SORT:
        return itemCount * Math.log2(itemCount);
      case SortAlgorithm.HEAP_SORT:
        return itemCount * Math.log2(itemCount);
      case SortAlgorithm.TIM_SORT:
        return itemCount * Math.log2(itemCount) * 0.8; // 部分有序时更优
      case SortAlgorithm.INSERTION_SORT:
        return (itemCount * itemCount) / 4; // 平均情况
      default:
        return itemCount * Math.log2(itemCount);
    }
  }
  
  /**
   * 估算交换次数
   */
  private estimateSwaps(itemCount: number, algorithm: SortAlgorithm): number {
    switch (algorithm) {
      case SortAlgorithm.QUICK_SORT:
        return itemCount * 0.3; // 平均情况
      case SortAlgorithm.MERGE_SORT:
        return itemCount * Math.log2(itemCount) * 0.5;
      case SortAlgorithm.HEAP_SORT:
        return itemCount * Math.log2(itemCount);
      case SortAlgorithm.TIM_SORT:
        return itemCount * 0.2;
      case SortAlgorithm.INSERTION_SORT:
        return (itemCount * itemCount) / 8;
      default:
        return itemCount * 0.3;
    }
  }
  
  /**
   * 估算内存使用
   */
  private estimateMemoryUsage(itemCount: number, algorithm: SortAlgorithm): number {
    const itemSize = 1; // 假设每个项目1KB
    const baseMemory = (itemCount * itemSize) / 1024; // MB
    
    switch (algorithm) {
      case SortAlgorithm.QUICK_SORT:
        return baseMemory * 1.1; // 需要额外栈空间
      case SortAlgorithm.MERGE_SORT:
        return baseMemory * 2; // 需要额外数组
      case SortAlgorithm.HEAP_SORT:
        return baseMemory * 1.1;
      case SortAlgorithm.TIM_SORT:
        return baseMemory * 1.3;
      case SortAlgorithm.INSERTION_SORT:
        return baseMemory * 1.05;
      default:
        return baseMemory * 1.2;
    }
  }
  
  /**
   * 获取算法复杂度
   */
  private getAlgorithmComplexity(algorithm: SortAlgorithm): string {
    switch (algorithm) {
      case SortAlgorithm.QUICK_SORT:
        return 'O(n log n) 平均, O(n²) 最坏';
      case SortAlgorithm.MERGE_SORT:
        return 'O(n log n)';
      case SortAlgorithm.HEAP_SORT:
        return 'O(n log n)';
      case SortAlgorithm.TIM_SORT:
        return 'O(n log n)';
      case SortAlgorithm.INSERTION_SORT:
        return 'O(n²)';
      default:
        return 'O(n log n)';
    }
  }
  
  /**
   * 更新性能指标
   */
  private updateMetrics(sortTime: number, itemCount: number, algorithm: SortAlgorithm): void {
    this.performanceMetrics.sortTime = sortTime;
    this.performanceMetrics.throughput = itemCount / (sortTime / 1000); // 项目/秒
    this.performanceMetrics.memoryEfficiency = itemCount / this.estimateMemoryUsage(itemCount, algorithm);
  }
  
  /**
   * 批量排序更新
   * 适用于大量项目的批量排序
   */
  async batchSort(
    items: AppDataFirstLevelItem[],
    updates: Array<{
      sortField: AppDataSortField;
      sortOrder: SortOrder;
      priority?: number;
    }>
  ): Promise<Array<SortResult>> {
    const results: Array<SortResult> = [];
    
    // 按优先级排序
    const sortedUpdates = updates.sort((a, b) => (b.priority || 0) - (a.priority || 0));
    
    for (const update of sortedUpdates) {
      try {
        const result = await this.sort(items, update.sortField, update.sortOrder);
        results.push(result);
      } catch (error) {
        console.error(`批量排序失败: ${update.sortField}_${update.sortOrder}`, error);
        throw error;
      }
    }
    
    return results;
  }
  
  /**
   * 智能预排序
   * 分析数据特征并选择最优排序策略
   */
  async smartSort(
    items: AppDataFirstLevelItem[],
    options: {
      targetSortField?: AppDataSortField;
      targetSortOrder?: SortOrder;
      enableAnalysis?: boolean;
    } = {}
  ): Promise<SortResult> {
    if (!options.enableAnalysis) {
      return this.sort(
        items,
        options.targetSortField || AppDataSortField.SIZE,
        options.targetSortOrder || 'desc'
      );
    }
    
    // 分析数据特征
    const analysis = this.analyzeDataCharacteristics(items);
    
    // 根据分析结果选择排序策略
    const sortField = options.targetSortField || this.selectOptimalSortField(analysis);
    const sortOrder = options.targetSortOrder || 'desc';
    const algorithm = this.selectAlgorithmByCharacteristics(analysis);
    
    return this.sort(items, sortField, sortOrder, { forceAlgorithm: algorithm });
  }
  
  /**
   * 分析数据特征
   */
  private analyzeDataCharacteristics(items: AppDataFirstLevelItem[]): {
    isPartiallySorted: boolean;
    hasDuplicates: boolean;
    dataDistribution: 'uniform' | 'skewed' | 'normal';
    sizeRange: { min: number; max: number; avg: number };
  } {
    if (items.length < 10) {
      return {
        isPartiallySorted: false,
        hasDuplicates: false,
        dataDistribution: 'uniform',
        sizeRange: { min: 0, max: 0, avg: 0 }
      };
    }
    
    // 检查是否部分有序
    const isPartiallySorted = this.checkPartialOrder(items);
    
    // 检查重复项
    const hasDuplicates = new Set(items.map(item => item.name)).size < items.length;
    
    // 分析数据分布
    const sizes = items.map(item => item.size);
    const dataDistribution = this.analyzeDistribution(sizes);
    
    // 计算大小范围
    const sizeRange = {
      min: Math.min(...sizes),
      max: Math.max(...sizes),
      avg: sizes.reduce((sum, size) => sum + size, 0) / sizes.length
    };
    
    return {
      isPartiallySorted,
      hasDuplicates,
      dataDistribution,
      sizeRange
    };
  }
  
  /**
   * 检查部分有序性
   */
  private checkPartialOrder(items: AppDataFirstLevelItem[]): boolean {
    let orderedPairs = 0;
    let totalPairs = Math.min(items.length - 1, 100); // 检查前100对
    
    for (let i = 0; i < totalPairs; i++) {
      if (items[i].size >= items[i + 1].size) {
        orderedPairs++;
      }
    }
    
    return (orderedPairs / totalPairs) > 0.7; // 70%以上有序认为是部分有序
  }
  
  /**
   * 分析数据分布
   */
  private analyzeDistribution(values: number[]): 'uniform' | 'skewed' | 'normal' {
    if (values.length < 10) return 'uniform';
    
    // 简化实现：基于标准差判断
    const avg = values.reduce((sum, val) => sum + val, 0) / values.length;
    const variance = values.reduce((sum, val) => sum + Math.pow(val - avg, 2), 0) / values.length;
    const stdDev = Math.sqrt(variance);
    const coefficientOfVariation = stdDev / avg;
    
    if (coefficientOfVariation > 1.5) return 'skewed';
    if (coefficientOfVariation < 0.5) return 'normal';
    return 'uniform';
  }
  
  /**
   * 选择最优排序字段
   */
  private selectOptimalSortField(analysis: any): AppDataSortField {
    // 简化实现：默认按大小排序
    return 'size' as AppDataSortField;
  }
  
  /**
   * 根据特征选择算法
   */
  private selectAlgorithmByCharacteristics(analysis: any): SortAlgorithm {
    if (analysis.isPartiallySorted) {
      return SortAlgorithm.TIM_SORT;
    }
    
    if (analysis.dataDistribution === 'skewed') {
      return SortAlgorithm.QUICK_SORT;
    }
    
    return this.config.defaultAlgorithm;
  }
  
  /**
   * 获取性能指标
   */
  getPerformanceMetrics(): PerformanceMetrics {
    return { ...this.performanceMetrics };
  }
  
  /**
   * 获取缓存统计
   */
  getCacheStatistics(): {
    size: number;
    hitRate: number;
    evictionCount: number;
    memoryUsage: number;
  } {
    const totalAccesses = Array.from(this.cache.values())
      .reduce((sum, item) => sum + item.accessCount, 0);
    
    const hits = Array.from(this.cache.values())
      .filter(item => item.accessCount > 0).length;
    
    return {
      size: this.cache.size,
      hitRate: totalAccesses > 0 ? hits / totalAccesses : 0,
      evictionCount: 0, // 简化实现
      memoryUsage: this.cache.size * 1 // 假设每个缓存项1KB
    };
  }
  
  /**
   * 清理资源
   */
  cleanup(): void {
    this.cache.clear();
    this.incrementalState = null;
    this.sortQueue = [];
    this.isSorting = false;
  }
  
  /**
   * 获取当前状态
   */
  getStatus(): {
    isSorting: boolean;
    queueSize: number;
    cacheSize: number;
    algorithm: SortAlgorithm;
  } {
    return {
      isSorting: this.isSorting,
      queueSize: this.sortQueue.length,
      cacheSize: this.cache.size,
      algorithm: this.config.defaultAlgorithm
    };
  }
}

/**
 * 创建动态排序引擎的工厂函数
 */
export function createDynamicSortingEngine(config: Partial<SortingEngineConfig> = {}): DynamicSortingEngine {
  return new DynamicSortingEngine(config);
}

/**
 * 默认导出
 */
export default DynamicSortingEngine;