/**
 * 格式化文件大小
 * @param bytes 字节数
 * @returns 格式化后的字符串
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB", "PB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

/**
 * 获取文件大小占比
 * @param size 当前大小
 * @param total 总大小
 * @returns 百分比
 */
export function getSizePercentage(size: number, total: number): number {
  if (total === 0) return 0;
  return Math.round((size / total) * 100);
}

/**
 * 提取路径中的文件夹名称
 * @param path 路径
 * @returns 文件夹名称
 */
export function getDirectoryName(path: string): string {
  return path.split(/[\\/]/).pop() || path;
}

/**
 * 格式化时间（秒到可读格式）
 * @param seconds 秒数
 * @returns 格式化后的时间字符串
 */
export function formatTime(seconds: number): string {
  if (seconds < 60) {
    return `${Math.round(seconds)}秒`;
  } else {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = Math.round(seconds % 60);
    return `${minutes}分${remainingSeconds}秒`;
  }
}

/**
 * 从AppData路径提取应用名称
 * @param path 完整路径
 * @returns 应用名称
 */
export function getAppNameFromPath(path: string): string {
  const pathParts = path.split(/[\\/]/);
  // 通常应用名称在AppData的某个子目录中
  const appDataIndex = pathParts.findIndex(part => part.toLowerCase() === 'appdata');
  if (appDataIndex !== -1 && appDataIndex + 2 < pathParts.length) {
    return pathParts[appDataIndex + 2]; // 获取应用名称
  }
  return pathParts[pathParts.length - 1] || path;
}

/**
 * 根据文件大小获取颜色
 * @param size 文件大小（字节）
 * @returns 颜色代码
 */
export function getSizeColor(size: number): string {
  if (size >= 5 * 1024 * 1024 * 1024) { // 5GB+
    return '#F56C6C'; // 红色
  } else if (size >= 1024 * 1024 * 1024) { // 1GB+
    return '#E6A23C'; // 橙色
  } else {
    return '#67C23A'; // 绿色
  }
}