/**
 * AppData 类型定义 - 专门用于Windows用户AppData目录空间分析
 * 支持Local、LocalLow、Roaming三个主要子目录的分析和迁移
 */

import type { DirectoryInfo } from './directory';

/**
 * AppData 排序选项类型
 * 用于定义扫描结果的排序方式
 */
export type SortOrder = 'asc' | 'desc';

/**
 * AppData 配置接口
 * 定义AppData扫描的基本配置参数
 * 满足 FR-3: 空间计算和展示 要求
 */
export interface AppDataConfig {
  /** 最小大小阈值（默认1GB）- 用于筛选大文件 */
  minSizeThreshold: number;
  /** 最大扫描深度（默认2层）- 限制扫描范围 */
  maxDepth: number;
  /** 排序方式（默认desc）- 控制结果展示顺序 */
  sortOrder: SortOrder;
}

/**
 * AppData 一级项目接口
 * 表示AppData三个主要目录下的一级子目录和文件
 * 满足 FR-3: 空间计算和展示 要求，支持1GB筛选和动态排序
 */
export interface AppDataFirstLevelItem {
  /** 完整路径 - 用于文件系统操作 */
  path: string;
  /** 名称 - 用于界面显示 */
  name: string;
  /** 大小（字节）- 用于排序和筛选 */
  size: number;
  /** 项目类型："directory" 或 "file" */
  itemType: 'directory' | 'file';
  /** 父目录类型："Local"、"LocalLow"、"Roaming" */
  parentType: 'Local' | 'LocalLow' | 'Roaming';
  /** 是否大于1GB - 用于快速筛选大文件 */
  isLarge: boolean;
  /** 占总大小百分比 - 用于可视化展示 */
  sizePercentage: number;
}

/**
 * AppData 迁移选项接口
 * 定义AppData文件迁移的配置参数
 * 满足 FR-5: 迁移功能增强 要求，支持目标盘符选择和迁移控制
 */
export interface AppDataMigrationOptions {
  /** 要迁移的项目路径列表 - 只包含1GB以上的大文件 */
  sourceItems: string[];
  /** 目标盘符（如"D:"）- 用户选择的目标位置 */
  targetDrive: string;
  /** 是否创建符号链接 - 保持应用程序兼容性 */
  createSymlink: boolean;
  /** 是否删除源文件 - 空间清理选项 */
  deleteSource: boolean;
}

/**
 * AppData 信息接口（更新版本）
 * 包含完整的AppData扫描结果和统计信息
 * 满足 FR-3: 空间计算和展示 要求，支持一级项目展示和1GB筛选
 */
export interface AppDataInfo {
  /** AppData\Local路径 - Windows本地应用数据 */
  localPath: string;
  /** AppData\LocalLow路径 - 低完整性应用数据 */
  localLowPath: string;
  /** AppData\Roaming路径 - 漫游应用数据 */
  roamingPath: string;
  /** Local目录总大小（字节） */
  localSize: number;
  /** LocalLow目录总大小（字节） */
  localLowSize: number;
  /** Roaming目录总大小（字节） */
  roamingSize: number;
  /** 总大小（字节）- 三个目录总和 */
  totalSize: number;
  /** 三个目录下的一级子目录和文件 - 用于界面展示 */
  firstLevelItems: AppDataFirstLevelItem[];
  /** 1GB以上项目列表 - 经过筛选的大文件 */
  largeItems: AppDataFirstLevelItem[];
  /** 扫描耗时（毫秒）- 性能指标 */
  scanTimeMs: number;
}

/**
 * AppData 扫描选项接口
 * 定义AppData扫描的可选参数
 * 满足 FR-4: 排序和筛选功能 要求
 */
export interface AppDataScanOptions {
  /** 最小大小阈值（默认1GB）- 用于筛选大文件 */
  minSizeThreshold?: number;
  /** 最大扫描深度（默认2层）- 限制扫描范围 */
  maxDepth?: number;
  /** 排序方式（默认desc）- 控制结果展示顺序 */
  sortOrder?: SortOrder;
  /** 是否只显示大文件（默认true）- 界面筛选选项 */
  showOnlyLarge?: boolean;
}

/**
 * AppData 扫描进度接口
 * 实时反馈扫描进度和状态
 * 满足 FR-2: 智能扫描引擎 要求，提供实时进度反馈
 */
export interface AppDataScanProgress {
  /** 当前扫描路径 - 用于显示当前处理位置 */
  currentPath: string;
  /** 已处理目录数 - 进度计算基础 */
  processedDirectories: number;
  /** 总目录数 - 用于计算完成百分比 */
  totalDirectories: number;
  /** 进度百分比（0-100）- 界面展示用 */
  progress: number;
  /** 预计剩余时间（秒）- 用户体验优化 */
  estimatedTimeRemaining?: number;
  /** 发现的大项目数量 - 实时结果预览 */
  largeItemsFound: number;
}

/**
 * AppData 目录信息接口
 * 包含三个主要AppData子目录的详细信息
 * 与现有 DirectoryInfo 类型保持兼容
 */
export interface AppDataDirectoryInfo {
  /** Local目录信息 - 本地应用数据 */
  local: DirectoryInfo | null;
  /** LocalLow目录信息 - 低完整性应用数据 */
  localLow: DirectoryInfo | null;
  /** Roaming目录信息 - 漫游应用数据 */
  roaming: DirectoryInfo | null;
}

/**
 * AppData 扫描结果接口
 * 封装扫描操作的完整结果
 * 提供统一的结果格式和错误处理
 */
export interface AppDataScanResult {
  /** 扫描是否成功 - 用于状态判断 */
  success: boolean;
  /** 扫描数据 - 成功时返回的数据 */
  data?: AppDataInfo;
  /** 错误信息 - 失败时的错误描述 */
  error?: string;
  /** 时间戳 - 操作时间记录 */
  timestamp: number;
}

/**
 * AppData 路径验证结果接口
 * 验证AppData路径有效性的结果
 * 满足 FR-1: AppData目录定位 要求
 */
export interface AppDataPathValidation {
  /** 路径是否有效 - 总体验证结果 */
  isValid: boolean;
  /** Local目录是否存在 */
  localExists: boolean;
  /** LocalLow目录是否存在 */
  localLowExists: boolean;
  /** Roaming目录是否存在 */
  roamingExists: boolean;
  /** 验证消息 - 额外的验证信息 */
  message?: string;
}

/**
 * AppData 统计信息接口
 * 提供AppData使用情况的统计概览
 * 用于生成报告和数据分析
 */
export interface AppDataStatistics {
  /** 应用总数 - 扫描到的应用数量 */
  totalApps: number;
  /** 总大小（字节）- 所有应用数据总和 */
  totalSize: number;
  /** 大应用数量（>1GB）- 符合筛选条件的应用 */
  largeApps: number;
  /** 平均应用大小（字节）- 统计指标 */
  averageAppSize: number;
  /** 最大应用名称 - 用于识别最大消费者 */
  largestApp: string;
  /** 最大应用大小（字节）- 具体大小数值 */
  largestAppSize: number;
  /** 扫描日期 - 数据时间戳 */
  scanDate: Date;
}

/**
 * AppData 应用信息接口（用于详细展示）
 * 单个应用的详细信息，支持三目录分解
 * 提供完整的应用数据画像
 */
export interface AppDataAppInfo {
  /** 应用名称 - 用于识别 */
  name: string;
  /** 应用路径 - 完整路径 */
  path: string;
  /** 应用大小（字节）- 总大小 */
  size: number;
  /** 格式化大小 - 人类可读格式（如"2.5GB"） */
  sizeFormatted: string;
  /** 是否为大应用（>1GB）- 筛选标识 */
  isLarge: boolean;
  /** Local部分大小（字节）- Local目录贡献 */
  localSize: number;
  /** LocalLow部分大小（字节）- LocalLow目录贡献 */
  localLowSize: number;
  /** Roaming部分大小（字节）- Roaming目录贡献 */
  roamingSize: number;
  /** 最后修改时间 - 活跃度指标 */
  lastModified?: Date;
  /** 文件数量 - 复杂度指标 */
  fileCount: number;
  /** 子文件夹数量 - 结构复杂度 */
  subfolderCount: number;
}

/**
 * AppData 扫描状态枚举
 * 定义扫描操作的各种状态
 * 用于界面状态管理和用户反馈
 */
export enum AppDataScanStatus {
  /** 空闲 - 等待用户操作 */
  IDLE = 'idle',
  /** 扫描中 - 正在进行文件系统扫描 */
  SCANNING = 'scanning',
  /** 完成 - 扫描成功完成 */
  COMPLETED = 'completed',
  /** 错误 - 扫描过程中发生错误 */
  ERROR = 'error',
  /** 已取消 - 用户取消或系统中断 */
  CANCELLED = 'cancelled',
}

/**
 * AppData 排序字段枚举
 * 定义支持的排序字段
 * 满足 FR-4: 排序和筛选功能 要求
 */
export enum AppDataSortField {
  /** 按名称排序 - 字母顺序 */
  NAME = 'name',
  /** 按大小排序 - 默认排序方式（逆序） */
  SIZE = 'size',
  /** 按修改时间排序 - 时间顺序 */
  LAST_MODIFIED = 'lastModified',
}

/**
 * AppData 筛选选项接口
 * 定义灵活的数据筛选和排序选项
 * 支持动态数据探索和自定义视图
 */
export interface AppDataFilterOptions {
  /** 最小大小（字节）- 下限筛选 */
  minSize?: number;
  /** 最大大小（字节）- 上限筛选 */
  maxSize?: number;
  /** 名称匹配模式 - 支持通配符或正则 */
  namePattern?: string;
  /** 只显示大应用（>1GB）- 快速筛选 */
  showOnlyLarge?: boolean;
  /** 排序字段 - 控制展示顺序 */
  sortBy?: AppDataSortField;
  /** 排序方式 - 升序或降序 */
  sortOrder?: SortOrder;
}

/**
 * AppData 错误信息接口
 * 标准化的错误信息格式
 * 提供详细的错误诊断信息
 */
export interface AppDataError {
  /** 错误代码 - 用于程序识别 */
  code: string;
  /** 错误消息 - 用户友好的错误描述 */
  message: string;
  /** 详细错误信息 - 技术细节 */
  details?: string;
  /** 错误发生时间 - 时间戳 */
  timestamp: number;
  /** 相关路径 - 出错的具体路径 */
  path?: string;
}

/**
 * AppData 迁移结果接口
 * 封装迁移操作的完整结果
 * 满足 FR-5: 迁移功能增强 要求，提供详细的迁移反馈
 */
export interface AppDataMigrationResult {
  /** 迁移是否成功 - 总体结果 */
  success: boolean;
  /** 迁移结果消息 - 用户反馈 */
  message: string;
  /** 成功迁移的项目数量 - 成功统计 */
  migratedItems: number;
  /** 失败的项目数量 - 失败统计 */
  failedItems: number;
  /** 总迁移大小（字节）- 空间释放量 */
  totalSize: number;
  /** 目标盘符 - 迁移目的地 */
  targetDrive: string;
}

/**
 * AppData 盘符信息接口
 * 提供可用盘符的详细信息
 * 用于目标盘符选择和空间评估
 */
export interface AppDataDriveInfo {
  /** 盘符（如 "D:\"）- 盘符标识 */
  drive: string;
  /** 总空间（字节）- 盘符总容量 */
  totalSpace: number;
  /** 可用空间（字节）- 剩余可用空间 */
  freeSpace: number;
  /** 已用空间（字节）- 已使用空间 */
  usedSpace: number;
  /** 是否是系统盘 - 系统盘标识（通常是C:） */
  isSystemDrive: boolean;
}