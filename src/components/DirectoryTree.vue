<template>
  <div class="directory-tree">
    <!-- 工具栏 -->
    <div class="toolbar">
      <el-input
        v-model="scanPath"
        placeholder="输入要扫描的目录路径"
        :prefix-icon="Search"
        clearable
        style="width: 300px; margin-right: 10px"
      />
      <el-select v-model="sortField" placeholder="排序字段" style="width: 120px; margin-right: 10px">
        <el-option label="名称" value="name" />
        <el-option label="大小" value="size" />
        <el-option label="文件数" value="fileCount" />
      </el-select>
      <el-select v-model="sortOrder" placeholder="排序方式" style="width: 100px; margin-right: 10px">
        <el-option label="升序" value="asc" />
        <el-option label="降序" value="desc" />
      </el-select>
      <el-button @click="selectDirectory" :icon="Folder" type="primary">
        选择目录
      </el-button>
      <el-button @click="refreshData" :icon="Refresh" :loading="loading">
        刷新
      </el-button>
    </div>

    <!-- 性能控制面板 -->
    <div class="performance-controls" v-if="directoryData.length > 0">
      <el-switch
        v-model="virtualScrollEnabled"
        active-text="虚拟滚动"
        inactive-text="全部显示"
        size="small"
      />
      <el-switch
        v-model="lazyLoadEnabled"
        active-text="懒加载"
        inactive-text="预加载"
        size="small"
      />
      <el-select v-model="pageSize" placeholder="每页数量" size="small" style="width: 100px">
        <el-option label="50" :value="50" />
        <el-option label="100" :value="100" />
        <el-option label="200" :value="200" />
      </el-select>
      <el-button @click="optimizeMemoryUsage" size="small" type="text">
        内存优化
      </el-button>
      <span class="performance-info">
        显示: {{ displayData.length }}/{{ totalItemCount }} | 内存: {{ getMemoryUsage() }}MB
      </span>
    </div>

    <!-- 树形控制 -->
    <div class="tree-controls" v-if="directoryData.length > 0">
      <el-button @click="toggleAll" size="small">
        {{ allExpanded ? '折叠全部' : '展开全部' }}
      </el-button>
      <span class="tree-info">共 {{ totalItemCount }} 个项目</span>
    </div>

    <!-- 扫描进度 -->
    <div v-if="scanning" class="scan-progress">
      <el-progress :percentage="scanProgress" :status="scanStatus" />
      <div class="scan-info">
        <span>正在扫描: {{ currentScanPath }}</span>
        <el-button @click="stopScan" size="small" type="text">停止</el-button>
      </div>
    </div>

    <!-- 目录树表格 - 性能优化版本 -->
    <div class="tree-container" ref="treeContainer">
      <el-table
        :data="displayData"
        style="width: 100%"
        row-key="path"
        :tree-props="{ children: 'subdirectories', hasChildren: 'hasChildren' }"
        @row-contextmenu="handleRowContextMenu"
        v-loading="loading"
        :default-expand-all="false"
        :expand-row-keys="[]"
        :height="tableHeight"
        :max-height="tableHeight"
        @expand-change="handleExpandChange"
        @scroll="handleTableScroll"
      >
      <el-table-column prop="name" label="名称" min-width="250">
        <template #default="scope">
          <div class="directory-name">
            <el-icon v-if="scope.row.subdirectories.length > 0">
              <Folder />
            </el-icon>
            <el-icon v-else>
              <Document />
            </el-icon>
            <span class="name-text">{{ scope.row.name }}</span>
          </div>
        </template>
      </el-table-column>

      <el-table-column prop="size" label="大小" width="120" sortable>
        <template #default="scope">
          <div class="size-cell">
            <span>{{ formatSize(scope.row.size) }}</span>
            <el-progress 
              :percentage="getSizePercentage(scope.row.size)" 
              :stroke-width="4"
              :show-text="false"
            />
          </div>
        </template>
      </el-table-column>

      <el-table-column prop="fileCount" label="文件数" width="100" sortable />

      <el-table-column label="操作" width="150" fixed="right">
        <template #default="scope">
          <el-button
            size="small"
            type="primary"
            @click="showMigrationDialog(scope.row)"
          >
            迁移
          </el-button>
          <el-button
            size="small"
            @click="openDirectory(scope.row.path)"
          >
            打开
          </el-button>
        </template>
      </el-table-column>
    </el-table>
    </div>

    <!-- 右键菜单 -->
    <context-menu
      v-model:visible="contextMenuVisible"
      :position="contextMenuPosition"
      :directory="selectedDirectory"
      @migrate="showMigrationDialog"
      @open="openDirectory"
      @refresh="refreshData"
    />

    <!-- 迁移对话框 -->
    <migration-dialog
      v-model:visible="migrationDialogVisible"
      :source-directory="selectedDirectory"
      @confirmed="handleMigration"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { Search, Refresh, Folder, Document } from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import type { DirectoryInfo } from '../types/directory';
import { diskAPI, migrationAPI, systemAPI, utils } from '../services/api';
import ContextMenu from './ContextMenu.vue';
import MigrationDialog from './MigrationDialog.vue';

// 搜索文本
const searchText = ref('');

// 数据状态
const directoryData = ref<DirectoryInfo[]>([]);
const loading = ref(false);
const scanning = ref(false);
const scanProgress = ref(0);
const scanStatus = ref<'success' | 'exception' | 'warning'>('success');
const currentScanPath = ref('');
const allExpanded = ref(false);
const expandedKeys = ref<string[]>([]);

// 性能优化相关状态
const treeContainer = ref<HTMLElement | null>(null);
const tableHeight = ref(600); // 表格高度
const virtualScrollEnabled = ref(true); // 是否启用虚拟滚动
const pageSize = ref(50); // 每页显示的项目数
const currentPage = ref(1); // 当前页码
const totalItems = ref(0); // 总项目数
const lazyLoadEnabled = ref(true); // 是否启用懒加载
const loadedDirectories = ref<Set<string>>(new Set()); // 已加载的目录
const expandedDirectoryCache = ref<Map<string, DirectoryInfo[]>>(new Map()); // 展开的目录缓存

// 搜索和排序
const scanPath = ref('C:\\Users'); // 默认扫描路径
const sortField = ref<'name' | 'size' | 'fileCount'>('size');
const sortOrder = ref<'asc' | 'desc'>('desc');

// 右键菜单
const contextMenuVisible = ref(false);
const contextMenuPosition = ref({ x: 0, y: 0 });
const selectedDirectory = ref<DirectoryInfo | null>(null);

// 迁移对话框
const migrationDialogVisible = ref(false);

// 计算属性：过滤和排序后的数据
const filteredData = computed(() => {
  let filtered = directoryData.value;

  // 搜索过滤
  if (searchText.value) {
    filtered = filterDirectories(filtered, searchText.value);
  }

  // 排序
  filtered = sortDirectories(filtered, sortField.value, sortOrder.value);

  return filtered;
});

// 计算属性：虚拟滚动显示的数据
const displayData = computed(() => {
  if (!virtualScrollEnabled.value) {
    return filteredData.value;
  }
  
  // 实现虚拟滚动逻辑
  return getVirtualScrollData(filteredData.value);
});

// 计算属性：总项目数（用于显示）
const totalItemCount = computed(() => {
  return countItems(directoryData.value);
});

// 计算项目总数
function countItems(items: DirectoryInfo[]): number {
  let count = items.length;
  items.forEach(item => {
    if (item.subdirectories && item.subdirectories.length > 0) {
      count += countItems(item.subdirectories);
    }
  });
  return count;
}

// 过滤目录
function filterDirectories(directories: DirectoryInfo[], searchTerm: string): DirectoryInfo[] {
  return directories.filter(dir => {
    const matchesName = dir.name.toLowerCase().includes(searchTerm.toLowerCase());
    const hasMatchingSubdirs = dir.subdirectories.length > 0 &&
      filterDirectories(dir.subdirectories, searchTerm).length > 0;
    
    return matchesName || hasMatchingSubdirs;
  }).map(dir => ({
    ...dir,
    subdirectories: filterDirectories(dir.subdirectories, searchTerm)
  }));
}

// 排序目录
function sortDirectories(
  directories: DirectoryInfo[], 
  field: 'name' | 'size' | 'fileCount', 
  order: 'asc' | 'desc'
): DirectoryInfo[] {
  const sorted = [...directories].sort((a, b) => {
    let aValue = a[field];
    let bValue = b[field];
    
    if (typeof aValue === 'string') {
      aValue = aValue.toLowerCase();
      bValue = (bValue as string).toLowerCase();
    }
    
    if (order === 'asc') {
      return aValue < bValue ? -1 : aValue > bValue ? 1 : 0;
    } else {
      return aValue > bValue ? -1 : aValue < bValue ? 1 : 0;
    }
  });

  // 递归排序子目录
  return sorted.map(dir => ({
    ...dir,
    subdirectories: sortDirectories(dir.subdirectories, field, order)
  }));
}

// 格式化文件大小
function formatSize(bytes: number): string {
  return utils.formatFileSize(bytes);
}

// 获取大小百分比
function getSizePercentage(bytes: number): number {
  const totalSize = directoryData.value.reduce((sum: number, dir: DirectoryInfo) => sum + dir.size, 0);
  return utils.getSizePercentage(bytes, totalSize);
}

// 处理右键菜单
function handleRowContextMenu(row: DirectoryInfo, event: MouseEvent) {
  event.preventDefault();
  selectedDirectory.value = row;
  contextMenuPosition.value = { x: event.clientX, y: event.clientY };
  contextMenuVisible.value = true;
}

// 显示迁移对话框
function showMigrationDialog(directory: DirectoryInfo) {
  selectedDirectory.value = directory;
  migrationDialogVisible.value = true;
}

// 处理迁移
async function handleMigration(targetPath: string) {
  if (!selectedDirectory.value) return;

  try {
    loading.value = true;
    const result = await migrationAPI.migrateDirectory({
      sourcePath: selectedDirectory.value.path,
      targetPath,
      createSymlink: true,
      deleteSource: true
    });

    if (result.success) {
      ElMessage.success('迁移成功！');
      await refreshData();
    } else {
      ElMessage.error(`迁移失败: ${result.message}`);
    }
  } catch (error) {
    ElMessage.error(`迁移失败: ${error}`);
  } finally {
    loading.value = false;
    migrationDialogVisible.value = false;
  }
}

// 打开目录
async function openDirectory(path: string) {
  try {
    // 使用 Tauri 的 opener 插件打开目录
    const { openPath } = await import('@tauri-apps/plugin-opener');
    await openPath(path);
  } catch (error) {
    ElMessage.error(`打开目录失败: ${error}`);
  }
}

// 停止扫描
async function stopScan() {
  try {
    await diskAPI.stopScan();
    scanning.value = false;
    ElMessage.info('扫描已停止');
  } catch (error) {
    ElMessage.error(`停止扫描失败: ${error}`);
  }
}

// 选择目录
async function selectDirectory() {
  console.log('选择目录按钮被点击，当前路径:', scanPath.value);
  try {
    // 简化的目录选择，使用输入框路径
    if (scanPath.value) {
      console.log('开始扫描路径:', scanPath.value);
      await refreshData();
    } else {
      ElMessage.warning('请先输入目录路径');
    }
  } catch (error) {
    console.error('选择目录失败:', error);
    ElMessage.error(`选择目录失败: ${error}`);
  }
}

// 刷新数据
async function refreshData() {
  console.log('刷新数据函数被调用，路径:', scanPath.value);
  if (!scanPath.value) {
    ElMessage.warning('请先选择要扫描的目录');
    return;
  }

  try {
    loading.value = true;
    scanning.value = true;
    scanProgress.value = 0;
    currentScanPath.value = scanPath.value;
    
    console.log('开始调用API扫描目录:', scanPath.value);
    // 扫描指定目录
    const result = await diskAPI.scanDirectory(scanPath.value);
    console.log('扫描完成，结果:', result);
    console.log('结果中的子目录数量:', result.subdirectories?.length || 0);
    console.log('第一个子目录:', result.subdirectories?.[0]);
    console.log('第一个子目录的子目录:', result.subdirectories?.[0]?.subdirectories);
    
    // 检查数据结构是否正确
    if (result.subdirectories && result.subdirectories.length > 0) {
      console.log('检查子目录结构:');
      result.subdirectories.forEach((dir, index) => {
        console.log(`子目录 ${index}: ${dir.name}, 子目录数量: ${dir.subdirectories?.length || 0}`);
        if (dir.subdirectories && dir.subdirectories.length > 0) {
          console.log(`  第一个孙子目录: ${dir.subdirectories[0].name}`);
        }
      });
    }
    
    directoryData.value = [result];
    
    ElMessage.success('扫描完成！');
  } catch (error) {
    console.error('扫描失败:', error);
    ElMessage.error(`扫描失败: ${error}`);
    scanStatus.value = 'exception';
  } finally {
    loading.value = false;
    scanning.value = false;
  }
}

// 组件挂载时初始化
onMounted(() => {
  // 不再自动扫描，等待用户手动操作
  ElMessage.info('请选择要扫描的目录');
});

// 展开/折叠全部
function toggleAll() {
  if (allExpanded.value) {
    // 折叠全部
    expandedKeys.value = [];
    allExpanded.value = false;
  } else {
    // 展开全部
    expandedKeys.value = getAllPaths(directoryData.value);
    allExpanded.value = true;
  }
}

// 获取所有路径
function getAllPaths(items: DirectoryInfo[]): string[] {
  const paths: string[] = [];
  items.forEach(item => {
    paths.push(item.path);
    if (item.subdirectories && item.subdirectories.length > 0) {
      paths.push(...getAllPaths(item.subdirectories));
    }
  });
  return paths;
}

// 虚拟滚动数据处理
function getVirtualScrollData(items: DirectoryInfo[]): DirectoryInfo[] {
  if (!virtualScrollEnabled.value || items.length <= pageSize.value) {
    return items;
  }
  
  const start = (currentPage.value - 1) * pageSize.value;
  const end = start + pageSize.value;
  return items.slice(start, end);
}

// 处理表格滚动事件
function handleTableScroll(event: Event) {
  if (!virtualScrollEnabled.value) return;
  
  const target = event.target as HTMLElement;
  const scrollTop = target.scrollTop;
  const scrollHeight = target.scrollHeight;
  const clientHeight = target.clientHeight;
  
  // 检测是否需要加载更多数据
  if (scrollHeight - scrollTop - clientHeight < 100) {
    loadMoreData();
  }
}

// 加载更多数据
function loadMoreData() {
  if (currentPage.value * pageSize.value >= filteredData.value.length) {
    return; // 已经加载完所有数据
  }
  
  currentPage.value++;
}

// 处理展开/折叠事件
function handleExpandChange(row: DirectoryInfo, expanded: boolean) {
  if (!lazyLoadEnabled.value) return;
  
  if (expanded) {
    // 展开目录时，检查是否需要懒加载
    if (!loadedDirectories.value.has(row.path) && row.subdirectories.length === 0) {
      loadSubdirectories(row);
    }
  } else {
    // 折叠时，可以清理缓存以节省内存
    if (expandedDirectoryCache.value.has(row.path)) {
      expandedDirectoryCache.value.delete(row.path);
    }
  }
}

// 懒加载子目录
async function loadSubdirectories(parentDir: DirectoryInfo) {
  try {
    loading.value = true;
    
    // 检查缓存
    if (expandedDirectoryCache.value.has(parentDir.path)) {
      parentDir.subdirectories = expandedDirectoryCache.value.get(parentDir.path)!;
      loadedDirectories.value.add(parentDir.path);
      return;
    }
    
    // 在实际应用中，这里应该调用API获取子目录数据
    // 为了演示，我们使用模拟数据
    const mockSubdirs = generateMockSubdirectories(parentDir.path, 10);
    
    // 添加到缓存
    expandedDirectoryCache.value.set(parentDir.path, mockSubdirs);
    parentDir.subdirectories = mockSubdirs;
    loadedDirectories.value.add(parentDir.path);
    
  } catch (error) {
    console.error('懒加载子目录失败:', error);
    ElMessage.error(`加载子目录失败: ${error}`);
  } finally {
    loading.value = false;
  }
}

// 生成模拟子目录数据（用于演示）
function generateMockSubdirectories(parentPath: string, count: number): DirectoryInfo[] {
  const subdirs: DirectoryInfo[] = [];
  for (let i = 0; i < count; i++) {
    subdirs.push({
      path: `${parentPath}\\subdir_${i}`,
      name: `子目录_${i}`,
      size: Math.floor(Math.random() * 1024 * 1024 * 100), // 随机大小，最大100MB
      fileCount: Math.floor(Math.random() * 1000), // 随机文件数
      subdirectories: [], // 初始为空，可以继续懒加载
      isExpanded: false
    });
  }
  return subdirs;
}

// 大数据优化：分批处理
async function processLargeDataset(items: DirectoryInfo[], batchSize: number = 100): Promise<DirectoryInfo[]> {
  const result: DirectoryInfo[] = [];
  
  for (let i = 0; i < items.length; i += batchSize) {
    const batch = items.slice(i, i + batchSize);
    
    // 使用 requestAnimationFrame 避免阻塞UI
    await new Promise(resolve => requestAnimationFrame(resolve));
    
    // 处理当前批次
    const processedBatch = batch.map(item => ({
      ...item,
      // 可以在这里添加额外的处理逻辑
      sizePercentage: getSizePercentage(item.size)
    }));
    
    result.push(...processedBatch);
  }
  
  return result;
}

// 获取内存使用情况（模拟）
function getMemoryUsage(): number {
  // 在实际应用中，这里应该使用性能API获取真实的内存使用情况
  // 这里使用模拟数据
  const cacheSize = expandedDirectoryCache.value.size;
  const loadedSize = loadedDirectories.value.size;
  const dataSize = directoryData.value.length;
  
  // 粗略估计内存使用（MB）
  return Math.round((cacheSize * 0.5 + loadedSize * 0.3 + dataSize * 0.1) * 100) / 100;
}

// 内存优化：清理不必要的数据
function optimizeMemoryUsage() {
  // 清理未使用的缓存
  const currentExpanded = new Set(expandedKeys.value);
  
  // 清理未展开的目录缓存
  for (const [path] of expandedDirectoryCache.value.entries()) {
    if (!currentExpanded.has(path)) {
      expandedDirectoryCache.value.delete(path);
    }
  }
  
  // 清理已加载目录记录
  for (const path of loadedDirectories.value) {
    if (!currentExpanded.has(path)) {
      loadedDirectories.value.delete(path);
    }
  }
  
  console.log('内存优化完成');
}

// 性能监控
function monitorPerformance() {
  const startTime = performance.now();
  
  return {
    end: () => {
      const endTime = performance.now();
      const duration = endTime - startTime;
      console.log(`操作耗时: ${duration.toFixed(2)}ms`);
      
      if (duration > 1000) {
        console.warn('操作耗时超过1秒，建议优化');
      }
    }
  };
}

// 组件挂载时初始化
onMounted(() => {
  // 不再自动扫描，等待用户手动操作
  ElMessage.info('请选择要扫描的目录');
  
  // 初始化表格高度
  nextTick(() => {
    calculateTableHeight();
    window.addEventListener('resize', calculateTableHeight);
  });
});

// 计算表格高度
function calculateTableHeight() {
  if (!treeContainer.value) return;
  
  const container = treeContainer.value;
  const containerHeight = container.clientHeight;
  const toolbarHeight = container.querySelector('.toolbar')?.clientHeight || 0;
  const controlsHeight = container.querySelector('.tree-controls')?.clientHeight || 0;
  const progressHeight = container.querySelector('.scan-progress')?.clientHeight || 0;
  
  // 预留一些边距
  const padding = 20;
  tableHeight.value = containerHeight - toolbarHeight - controlsHeight - progressHeight - padding;
}

// 组件卸载时清理资源
onUnmounted(() => {
  window.removeEventListener('resize', calculateTableHeight);
  
  // 清理所有缓存
  expandedDirectoryCache.value.clear();
  loadedDirectories.value.clear();
});
</script>

<style scoped>
.directory-tree {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.toolbar {
  padding: 10px;
  background-color: #f5f7fa;
  border-bottom: 1px solid #e4e7ed;
  display: flex;
  align-items: center;
}

.scan-progress {
  padding: 10px;
  background-color: #f0f9ff;
  border-bottom: 1px solid #b3d8ff;
}

.scan-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 5px;
  font-size: 12px;
  color: #606266;
}

.directory-name {
  display: flex;
  align-items: center;
  gap: 8px;
}

.name-text {
  font-weight: 500;
}

.size-cell {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

:deep(.el-table__row:hover) {
  background-color: #f5f7fa;
}

.tree-controls {
  padding: 10px;
  background-color: #f5f7fa;
  border-bottom: 1px solid #e4e7ed;
  display: flex;
  align-items: center;
  gap: 10px;
}

.tree-info {
  font-size: 14px;
  color: #606266;
}

/* 性能优化相关样式 */
.tree-container {
  flex: 1;
  overflow: hidden;
  position: relative;
}

/* 虚拟滚动优化 */
:deep(.el-table__body-wrapper) {
  overflow-y: auto;
  scrollbar-width: thin;
}

:deep(.el-table__body-wrapper::-webkit-scrollbar) {
  width: 6px;
}

:deep(.el-table__body-wrapper::-webkit-scrollbar-thumb) {
  background-color: #c0c4cc;
  border-radius: 3px;
}

/* 懒加载指示器 */
.lazy-loading {
  display: inline-block;
  width: 16px;
  height: 16px;
  border: 2px solid #f3f3f3;
  border-top: 2px solid #409eff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-left: 8px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* 性能控制面板 */
.performance-controls {
  padding: 8px 10px;
  background-color: #f0f9ff;
  border-bottom: 1px solid #b3d8ff;
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 12px;
}

.performance-info {
  color: #606266;
  font-size: 11px;
}

/* 大数据优化样式 */
.large-data-warning {
  background-color: #fff3cd;
  border: 1px solid #ffeaa7;
  color: #856404;
  padding: 8px 12px;
  border-radius: 4px;
  margin: 5px 0;
  font-size: 12px;
}

/* 内存优化提示 */
.memory-optimization-info {
  position: absolute;
  top: 10px;
  right: 10px;
  background-color: rgba(64, 158, 255, 0.1);
  color: #409eff;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  z-index: 100;
}

/* 响应式优化 */
@media (max-width: 768px) {
  .toolbar {
    flex-wrap: wrap;
    gap: 8px;
  }
  
  .toolbar > * {
    margin-right: 0 !important;
    margin-bottom: 8px;
  }
  
  .performance-controls {
    flex-wrap: wrap;
  }
}
</style>