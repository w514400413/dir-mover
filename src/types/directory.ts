// 目录信息接口
export interface DirectoryInfo {
  path: string;
  name: string;
  size: number;
  fileCount: number;
  subdirectories: DirectoryInfo[];
  isExpanded?: boolean;
  isLargeFolder?: boolean;      // 新增：标识大文件夹
  sizePercentage?: number;      // 新增：相对父目录的占比
}

// 扫描进度接口
export interface ScanProgress {
  currentPath: string;
  processedFiles: number;
  totalFiles: number;
  progress: number;
  processedDirectories?: number;     // 新增：已处理目录数
  totalDirectories?: number;         // 新增：总目录数
  currentDirectory?: string;         // 新增：当前处理的目录
  estimatedTimeRemaining?: number;   // 新增：预计剩余时间（秒）
  scanSpeed?: number;                // 新增：扫描速度（文件/秒）
  largeFoldersFound?: number;        // 新增：发现的大文件夹数量
}

// 迁移选项接口
export interface MigrationOptions {
  sourcePath: string;
  targetPath: string;
  createSymlink: boolean;
  deleteSource: boolean;
}

// 迁移结果接口
export interface MigrationResult {
  success: boolean;
  message: string;
  sourcePath: string;
  targetPath: string;
  symlinkPath?: string;
}

// 排序选项
export interface SortOptions {
  field: 'name' | 'size' | 'fileCount';
  order: 'asc' | 'desc';
}

// 筛选选项
export interface FilterOptions {
  minSize?: number;
  maxSize?: number;
  namePattern?: string;
  showEmpty?: boolean;
}