// 目录信息接口
export interface DirectoryInfo {
  path: string;
  name: string;
  size: number;
  fileCount: number;
  subdirectories: DirectoryInfo[];
  isExpanded?: boolean;
}

// 扫描进度接口
export interface ScanProgress {
  currentPath: string;
  processedFiles: number;
  totalFiles: number;
  progress: number;
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