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

    <!-- 树形控制 -->
    <div class="tree-controls" v-if="directoryData.length > 0">
      <el-button @click="toggleAll" size="small">
        {{ allExpanded ? '折叠全部' : '展开全部' }}
      </el-button>
      <span class="tree-info">共 {{ totalItems }} 个项目</span>
    </div>

    <!-- 扫描进度 -->
    <div v-if="scanning" class="scan-progress">
      <el-progress :percentage="scanProgress" :status="scanStatus" />
      <div class="scan-info">
        <span>正在扫描: {{ currentScanPath }}</span>
        <el-button @click="stopScan" size="small" type="text">停止</el-button>
      </div>
    </div>

    <!-- 目录树表格 -->
    <el-table
      :data="filteredData"
      style="width: 100%"
      row-key="path"
      :tree-props="{ children: 'subdirectories', hasChildren: 'hasChildren' }"
      @row-contextmenu="handleRowContextMenu"
      v-loading="loading"
      :default-expand-all="false"
      :expand-row-keys="[]"
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
import { ref, computed, onMounted } from 'vue';
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

// 计算属性：总项目数
const totalItems = computed(() => {
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
  const totalSize = directoryData.value.reduce((sum, dir) => sum + dir.size, 0);
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
</style>