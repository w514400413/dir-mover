import { invoke } from "@tauri-apps/api/core";
import { DirectoryInfo, ScanProgress, MigrationOptions, MigrationResult } from "../types/directory";

/**
 * 磁盘扫描API
 */
export const diskAPI = {
  /**
   * 扫描指定路径的目录结构
   * @param path 要扫描的路径
   * @returns 目录信息
   */
  async scanDirectory(path: string): Promise<DirectoryInfo> {
    console.log('API: 开始扫描目录', path);
    try {
      const result = await invoke<DirectoryInfo>("scan_directory", { path });
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