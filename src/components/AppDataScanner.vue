<template>
  <div class="appdata-scanner">
    <!-- 标题和操作区域 -->
    <div class="scanner-header">
      <h2>AppData 空间分析</h2>
      <p class="description">扫描当前用户的应用程序数据目录，识别占用空间最大的应用</p>
      
      <div class="controls">
        <el-button 
          type="primary" 
          @click="startScan" 
          :loading="scanning"
          :disabled="scanning"
          icon="Search"
        >
          {{ scanning ? '扫描中...' : '开始扫描' }}
        </el-button>
        
        <el-button 
          @click="stopScan" 
          v-if="scanning"
          icon="Close"
        >
          停止扫描
        </el-button>
        
        <el-button 
          @click="refreshScan" 
          :disabled="scanning"
          icon="Refresh"
        >
          重新扫描
        </el-button>

        <!-- 目标盘符选择 -->
        <el-select 
          v-model="targetDrive" 
          placeholder="选择目标盘符"
          style="width: 150px; margin-left: 10px;"
          :disabled="!canMigrate"
        >
          <el-option
            v-for="drive in availableDrives"
            :key="drive"
            :label="drive"
            :value="drive"
          />
        </el-select>

        <el-button 
          type="warning"
          @click="migrateSelectedItems" 
          :disabled="!canMigrate || selectedItems.length === 0"
          icon="Upload"
        >
          迁移选中项
        </el-button>
      </div>
    </div>

    <!-- 扫描进度 -->
    <div class="scan-progress" v-if="scanning">
      <el-progress 
        :percentage="progress" 
        :status="progressStatus"
        :stroke-width="20"
      >
        <template #default="{ percentage }">
          <span class="progress-text">{{ Math.round(percentage) }}%</span>
        </template>
      </el-progress>
      
      <div class="progress-info">
        <p>正在扫描: {{ currentPath }}</p>
        <p>发现大项目: {{ largeItemsFound }} 个</p>
        <p>当前排序: {{ sortField === 'size' ? '按大小' : '按名称' }} {{ sortOrder === 'desc' ? '降序' : '升序' }}</p>
        <p v-if="estimatedTimeRemaining > 0">
          预计剩余时间: {{ formatTime(estimatedTimeRemaining) }}
        </p>
      </div>
    </div>

    <!-- 扫描结果 -->
    <div class="scan-results" v-if="scanResult && !scanning">
      <!-- 概览信息 -->
      <div class="overview-section">
        <el-card class="overview-card">
          <template #header>
            <div class="card-header">
              <span>扫描概览</span>
              <el-tag type="success" v-if="scanResult.success">扫描成功</el-tag>
              <el-tag type="danger" v-else>扫描失败</el-tag>
            </div>
          </template>
          
          <div class="overview-stats">
            <div class="stat-item">
              <span class="stat-label">总大小:</span>
              <span class="stat-value">{{ formatSize(scanResult.data?.totalSize || 0) }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">大项目数量:</span>
              <span class="stat-value">{{ scanResult.data?.largeItems.length || 0 }} 个</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">扫描耗时:</span>
              <span class="stat-value">{{ formatTime((scanResult.data?.scanTimeMs || 0) / 1000) }}</span>
            </div>
          </div>
        </el-card>
      </div>

      <!-- 目录大小分布 -->
      <div class="directory-distribution" v-if="scanResult.data">
        <el-card>
          <template #header>
            <span>目录大小分布</span>
          </template>
          
          <div class="distribution-stats">
            <div class="dist-item">
              <span class="dist-label">Local:</span>
              <span class="dist-value">{{ formatSize(scanResult.data.localSize) }}</span>
              <el-progress 
                :percentage="getPercentage(scanResult.data.localSize, scanResult.data.totalSize)"
                :stroke-width="6"
              />
            </div>
            <div class="dist-item">
              <span class="dist-label">LocalLow:</span>
              <span class="dist-value">{{ formatSize(scanResult.data.localLowSize) }}</span>
              <el-progress 
                :percentage="getPercentage(scanResult.data.localLowSize, scanResult.data.totalSize)"
                :stroke-width="6"
              />
            </div>
            <div class="dist-item">
              <span class="dist-label">Roaming:</span>
              <span class="dist-value">{{ formatSize(scanResult.data.roamingSize) }}</span>
              <el-progress 
                :percentage="getPercentage(scanResult.data.roamingSize, scanResult.data.totalSize)"
                :stroke-width="6"
              />
            </div>
          </div>
        </el-card>
      </div>

      <!-- 一级项目列表（按目录分组） -->
      <div class="first-level-items-section" v-if="scanResult.data && scanResult.data.firstLevelItems.length > 0">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>一级项目列表 (按目录分组)</span>
              <div class="header-controls">
                <el-checkbox v-model="showOnlyLarge" label="只显示>1GB" />
                <el-select v-model="sortField" style="width: 120px; margin-left: 10px;">
                  <el-option label="按大小排序" value="size" />
                  <el-option label="按名称排序" value="name" />
                </el-select>
                <el-select v-model="sortOrder" style="width: 100px; margin-left: 10px;">
                  <el-option label="降序" value="desc" />
                  <el-option label="升序" value="asc" />
                </el-select>
              </div>
            </div>
          </template>
          
          <!-- Local 目录 -->
          <div class="directory-group" v-if="groupedItems.Local && groupedItems.Local.length > 0">
            <h4 class="group-title">
              <el-icon><Folder /></el-icon>
              Local ({{ groupedItems.Local.length }} 项)
            </h4>
            <div class="items-list">
              <div 
                v-for="item in groupedItems.Local" 
                :key="item.path"
                class="item-row"
                @click="toggleItemSelection(item)"
                :class="{ 'selected': isItemSelected(item), 'large-item': item.isLarge }"
              >
                <el-checkbox
                  :model-value="isItemSelected(item)"
                  @click.stop
                  @change="toggleItemSelection(item)"
                />
                <div class="item-info">
                  <div class="item-name">{{ item.name }}</div>
                  <div class="item-path" :title="item.path">{{ item.path }}</div>
                </div>
                <div class="item-size">
                  <span class="size-value">{{ formatSize(item.size) }}</span>
                  <el-progress 
                    :percentage="item.sizePercentage"
                    :stroke-width="4"
                    :color="getSizeColor(item.size)"
                  />
                </div>
                <el-tag :type="item.itemType === 'directory' ? 'info' : 'warning'" size="small">
                  {{ item.itemType === 'directory' ? '文件夹' : '文件' }}
                </el-tag>
              </div>
            </div>
          </div>

          <!-- LocalLow 目录 -->
          <div class="directory-group" v-if="groupedItems.LocalLow && groupedItems.LocalLow.length > 0">
            <h4 class="group-title">
              <el-icon><Folder /></el-icon>
              LocalLow ({{ groupedItems.LocalLow.length }} 项)
            </h4>
            <div class="items-list">
              <div 
                v-for="item in groupedItems.LocalLow" 
                :key="item.path"
                class="item-row"
                @click="toggleItemSelection(item)"
                :class="{ 'selected': isItemSelected(item), 'large-item': item.isLarge }"
              >
                <el-checkbox
                  :model-value="isItemSelected(item)"
                  @click.stop
                  @change="toggleItemSelection(item)"
                />
                <div class="item-info">
                  <div class="item-name">{{ item.name }}</div>
                  <div class="item-path" :title="item.path">{{ item.path }}</div>
                </div>
                <div class="item-size">
                  <span class="size-value">{{ formatSize(item.size) }}</span>
                  <el-progress 
                    :percentage="item.sizePercentage"
                    :stroke-width="4"
                    :color="getSizeColor(item.size)"
                  />
                </div>
                <el-tag :type="item.itemType === 'directory' ? 'info' : 'warning'" size="small">
                  {{ item.itemType === 'directory' ? '文件夹' : '文件' }}
                </el-tag>
              </div>
            </div>
          </div>

          <!-- Roaming 目录 -->
          <div class="directory-group" v-if="groupedItems.Roaming && groupedItems.Roaming.length > 0">
            <h4 class="group-title">
              <el-icon><Folder /></el-icon>
              Roaming ({{ groupedItems.Roaming.length }} 项)
            </h4>
            <div class="items-list">
              <div 
                v-for="item in groupedItems.Roaming" 
                :key="item.path"
                class="item-row"
                @click="toggleItemSelection(item)"
                :class="{ 'selected': isItemSelected(item), 'large-item': item.isLarge }"
              >
                <el-checkbox
                  :model-value="isItemSelected(item)"
                  @click.stop
                  @change="toggleItemSelection(item)"
                />
                <div class="item-info">
                  <div class="item-name">{{ item.name }}</div>
                  <div class="item-path" :title="item.path">{{ item.path }}</div>
                </div>
                <div class="item-size">
                  <span class="size-value">{{ formatSize(item.size) }}</span>
                  <el-progress 
                    :percentage="item.sizePercentage"
                    :stroke-width="4"
                    :color="getSizeColor(item.size)"
                  />
                </div>
                <el-tag :type="item.itemType === 'directory' ? 'info' : 'warning'" size="small">
                  {{ item.itemType === 'directory' ? '文件夹' : '文件' }}
                </el-tag>
              </div>
            </div>
          </div>

          <!-- 空状态 -->
          <el-empty 
            v-if="Object.keys(groupedItems).length === 0" 
            description="没有找到符合条件的项目"
          />
        </el-card>
      </div>
    </div>

    <!-- 错误状态 -->
    <div class="error-state" v-if="scanResult && !scanResult.success">
      <el-result
        icon="error"
        title="扫描失败"
        :sub-title="scanResult.error"
      >
        <template #extra>
          <el-button type="primary" @click="startScan">重新扫描</el-button>
        </template>
      </el-result>
    </div>

    <!-- 初始状态 -->
    <div class="initial-state" v-if="!scanResult && !scanning">
      <el-result
        icon="info"
        title="开始扫描"
        sub-title="点击开始扫描按钮来分析您的AppData目录"
      >
        <template #extra>
          <el-button type="primary" @click="startScan" icon="Search">
            开始扫描
          </el-button>
        </template>
      </el-result>
    </div>

    <!-- 迁移确认对话框 -->
    <el-dialog
      v-model="migrationDialog.visible"
      title="确认迁移"
      width="500px"
    >
      <div v-if="migrationDialog.items.length > 0">
        <p>将迁移以下 {{ migrationDialog.items.length }} 个项目到 {{ targetDrive }}:</p>
        <el-scrollbar height="200px">
          <div v-for="item in migrationDialog.items" :key="item.path" class="migration-item">
            <span class="item-name">{{ item.name }}</span>
            <span class="item-size">{{ formatSize(item.size) }}</span>
          </div>
        </el-scrollbar>
        <p class="migration-warning">⚠️ 迁移过程将创建同名路径并移动文件，请确保目标盘符有足够空间。</p>
      </div>
      <template #footer>
        <el-button @click="migrationDialog.visible = false">取消</el-button>
        <el-button type="primary" @click="confirmMigration" :loading="migrationDialog.loading">
          确认迁移
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Folder } from '@element-plus/icons-vue';
import type { AppDataScanOptions, AppDataScanResult, AppDataFirstLevelItem, AppDataMigrationOptions } from '../types/appdata';
import { appDataAPI } from '../services/api';
import { formatFileSize, getSizeColor } from '../utils/formatters';

// 状态变量
const scanning = ref(false);
const scanResult = ref<AppDataScanResult | null>(null);
const progress = ref(0);
const progressStatus = ref<'success' | 'exception' | 'warning'>('success');
const currentPath = ref('');
const largeItemsFound = ref(0);
const estimatedTimeRemaining = ref(0);
const showOnlyLarge = ref(true);
const sortField = ref<'size' | 'name'>('size');
const sortOrder = ref<'asc' | 'desc'>('desc');
const selectedItems = ref<AppDataFirstLevelItem[]>([]);
const targetDrive = ref('');
const availableDrives = ref<string[]>([]);

// 迁移对话框
const migrationDialog = ref({
  visible: false,
  items: [] as AppDataFirstLevelItem[],
  loading: false
});

// 计算属性：是否可以迁移
const canMigrate = computed(() => {
  return scanResult.value?.success && scanResult.value.data && availableDrives.value.length > 0;
});

// 计算属性：按目录分组的一级项目
const groupedItems = computed(() => {
  if (!scanResult.value?.data?.firstLevelItems) return {};

  let items = [...scanResult.value.data.firstLevelItems];

  // 应用大小过滤
  if (showOnlyLarge.value) {
    items = items.filter(item => item.isLarge);
  }

  // 应用排序
  items.sort((a, b) => {
    let compareValue = 0;
    
    switch (sortField.value) {
      case 'size':
        compareValue = a.size - b.size;
        break;
      case 'name':
        compareValue = a.name.localeCompare(b.name);
        break;
    }

    return sortOrder.value === 'desc' ? -compareValue : compareValue;
  });

  // 按父目录类型分组
  const grouped: Record<string, AppDataFirstLevelItem[]> = {
    Local: [],
    LocalLow: [],
    Roaming: []
  };

  items.forEach(item => {
    if (grouped[item.parentType]) {
      grouped[item.parentType].push(item);
    }
  });

  return grouped;
});

// 方法：开始扫描
async function startScan() {
  try {
    scanning.value = true;
    progress.value = 0;
    scanResult.value = null;
    selectedItems.value = [];
    
    ElMessage.info('开始扫描AppData目录...');
    
    const options: AppDataScanOptions = {
      minSizeThreshold: 1024 * 1024 * 1024, // 1GB
      maxDepth: 2,
      sortOrder: sortOrder.value,
      showOnlyLarge: showOnlyLarge.value
    };
    
    // 使用流式扫描，实时接收事件
    const result = await appDataAPI.scanAppDataStreaming(options, (event) => {
      console.log('收到扫描事件:', event);
      
      if (event.type === 'DirectoryStarted') {
        // 更新当前扫描的目录
        currentPath.value = `正在扫描: ${event.name}`;
        ElMessage.info(`开始扫描 ${event.name} 目录...`);
      } else if (event.type === 'ItemFound') {
        // 实时添加项目到临时列表
        const newItem = event.item;
        largeItemsFound.value++;
        
        // 更新进度
        const currentProgress = Math.min((largeItemsFound.value / 50) * 100, 90); // 假设最多50个项目
        progress.value = Math.max(progress.value, currentProgress);
        
        // 实时更新显示（简化版本）
        if (!scanResult.value) {
          scanResult.value = {
            success: true,
            data: {
              localPath: '',
              localLowPath: '',
              roamingPath: '',
              localSize: 0,
              localLowSize: 0,
              roamingSize: 0,
              totalSize: 0,
              firstLevelItems: [],
              largeItems: [],
              scanTimeMs: 0
            },
            timestamp: Date.now()
          };
        }
        
        // 添加项目到列表
        if (scanResult.value.data) {
          scanResult.value.data.firstLevelItems.push(newItem);
          if (newItem.isLarge) {
            scanResult.value.data.largeItems.push(newItem);
          }
          scanResult.value.data.totalSize += newItem.size;
          
          // 根据父目录类型更新大小
          if (newItem.parentType === 'Local') {
            scanResult.value.data.localSize += newItem.size;
          } else if (newItem.parentType === 'LocalLow') {
            scanResult.value.data.localLowSize += newItem.size;
          } else if (newItem.parentType === 'Roaming') {
            scanResult.value.data.roamingSize += newItem.size;
          }
        }
      } else if (event.type === 'DirectoryCompleted') {
        ElMessage.success(`${event.name} 目录扫描完成，发现 ${event.itemCount} 个项目`);
      } else if (event.type === 'ScanCompleted') {
        // 扫描完成
        progress.value = 100;
        progressStatus.value = 'success';
        ElMessage.success(`AppData扫描完成！共发现 ${event.totalItems} 个项目`);
        
        // 获取可用盘符
        loadAvailableDrives();
      } else if (event.type === 'ScanError') {
        console.error('扫描错误:', event.error);
        ElMessage.error(`扫描错误: ${event.error}`);
      }
    });
    
    console.log('流式扫描完成，最终结果:', result);
    
    if (result) {
      scanResult.value = {
        success: true,
        data: result,
        timestamp: Date.now()
      };
      largeItemsFound.value = result.largeItems.length;
    }
  } catch (error) {
    console.error('扫描错误:', error);
    ElMessage.error(`扫描出错: ${error}`);
    progressStatus.value = 'exception';
    scanResult.value = {
      success: false,
      error: error instanceof Error ? error.message : String(error),
      timestamp: Date.now()
    };
  } finally {
    scanning.value = false;
  }
}

// 方法：加载可用盘符
async function loadAvailableDrives() {
  try {
    const drives = await appDataAPI.getAvailableDrives();
    availableDrives.value = drives.filter(drive => drive !== 'C:\\'); // 排除系统盘
    if (availableDrives.value.length > 0 && !targetDrive.value) {
      targetDrive.value = availableDrives.value[0];
    }
  } catch (error) {
    console.error('加载可用盘符失败:', error);
    availableDrives.value = ['D:\\', 'E:\\', 'F:\\']; // 回退到默认
  }
}

// 方法：停止扫描
async function stopScan() {
  try {
    await ElMessageBox.confirm('确定要停止扫描吗？', '确认', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    });
    
    scanning.value = false;
    ElMessage.info('扫描已停止');
  } catch {
    // 用户取消
  }
}

// 方法：重新扫描
async function refreshScan() {
  await startScan();
}


// 方法：切换项目选择
function toggleItemSelection(item: AppDataFirstLevelItem) {
  const index = selectedItems.value.findIndex(selected => selected.path === item.path);
  if (index >= 0) {
    selectedItems.value.splice(index, 1);
  } else {
    selectedItems.value.push(item);
  }
}

// 方法：检查项目是否被选中
function isItemSelected(item: AppDataFirstLevelItem): boolean {
  return selectedItems.value.some(selected => selected.path === item.path);
}

// 方法：迁移选中项
async function migrateSelectedItems() {
  if (selectedItems.value.length === 0) {
    ElMessage.warning('请先选择要迁移的项目');
    return;
  }

  if (!targetDrive.value) {
    ElMessage.warning('请选择目标盘符');
    return;
  }

  // 只迁移大于1GB的项目
  const itemsToMigrate = selectedItems.value.filter(item => item.isLarge);
  
  if (itemsToMigrate.length === 0) {
    ElMessage.warning('选中的项目都小于1GB，无需迁移');
    return;
  }

  migrationDialog.value.items = itemsToMigrate;
  migrationDialog.value.visible = true;
}

// 方法：确认迁移
async function confirmMigration() {
  if (!targetDrive.value || migrationDialog.value.items.length === 0) return;

  migrationDialog.value.loading = true;
  
  try {
    const options: AppDataMigrationOptions = {
      sourceItems: migrationDialog.value.items.map(item => item.path),
      targetDrive: targetDrive.value,
      createSymlink: true,
      deleteSource: false
    };

    const result = await appDataAPI.migrateAppDataItems(options);
    
    if (result.success) {
      ElMessage.success(`迁移成功！迁移了 ${result.migratedItems} 个项目`);
      // 清除选中状态
      selectedItems.value = [];
      // 重新扫描以更新数据
      await refreshScan();
    } else {
      ElMessage.error(`迁移失败: ${result.message}`);
    }
  } catch (error) {
    console.error('迁移错误:', error);
    ElMessage.error(`迁移出错: ${error}`);
  } finally {
    migrationDialog.value.loading = false;
    migrationDialog.value.visible = false;
  }
}

// 方法：格式化文件大小
function formatSize(bytes: number): string {
  return formatFileSize(bytes);
}

// 方法：格式化时间
function formatTime(seconds: number): string {
  if (seconds < 60) {
    return `${Math.round(seconds)}秒`;
  } else {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = Math.round(seconds % 60);
    return `${minutes}分${remainingSeconds}秒`;
  }
}

// 方法：计算百分比
function getPercentage(size: number, total: number): number {
  if (total === 0) return 0;
  return Math.round((size / total) * 100);
}

// 监听排序字段变化，实时更新
watch([sortField, sortOrder], () => {
  if (scanning.value) {
    // 扫描过程中动态排序
    console.log(`排序更新: ${sortField.value} ${sortOrder.value}`);
  }
});

// 生命周期
onMounted(async () => {
  console.log('AppData扫描器已加载');
  // UX-1: 根据用户偏好决定是否自动开始扫描
  const autoScanEnabled = localStorage.getItem('autoScanEnabled') !== 'false';
  if (autoScanEnabled) {
    await startScan();
  } else {
    // 显示友好的初始状态
    ElMessage.info('点击"开始扫描"按钮来分析您的AppData目录');
  }
});
</script>

<style scoped>
.appdata-scanner {
  padding: 20px;
  max-width: 1400px;
  margin: 0 auto;
}

.scanner-header {
  text-align: center;
  margin-bottom: 30px;
}

.scanner-header h2 {
  margin: 0 0 10px 0;
  color: #303133;
}

.description {
  color: #606266;
  margin-bottom: 20px;
}

.controls {
  display: flex;
  justify-content: center;
  gap: 10px;
  flex-wrap: wrap;
}

.scan-progress {
  margin-bottom: 30px;
}

.progress-text {
  font-size: 14px;
  font-weight: bold;
}

.progress-info {
  margin-top: 15px;
  text-align: center;
}

.progress-info p {
  margin: 5px 0;
  color: #606266;
}

.scan-results {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.overview-section {
  width: 100%;
}

.overview-card {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.overview-stats {
  display: flex;
  justify-content: space-around;
  text-align: center;
}

.stat-item {
  display: flex;
  flex-direction: column;
}

.stat-label {
  font-size: 14px;
  opacity: 0.9;
  margin-bottom: 5px;
}

.stat-value {
  font-size: 24px;
  font-weight: bold;
}

.directory-distribution {
  width: 100%;
}

.distribution-stats {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.dist-item {
  display: flex;
  align-items: center;
  gap: 10px;
}

.dist-label {
  width: 80px;
  font-weight: bold;
  color: #606266;
}

.dist-value {
  width: 100px;
  text-align: right;
  font-weight: bold;
}

.first-level-items-section {
  width: 100%;
}

.directory-group {
  margin-bottom: 30px;
}

.directory-group:last-child {
  margin-bottom: 0;
}

.group-title {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #303133;
  margin-bottom: 15px;
  font-size: 16px;
}

.items-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.item-row {
  display: flex;
  align-items: center;
  padding: 12px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s ease;
  gap: 12px;
}

.item-row:hover {
  background-color: #f5f7fa;
  border-color: #409eff;
}

.item-row.selected {
  background-color: #ecf5ff;
  border-color: #409eff;
}

.item-row.large-item {
  border-left: 4px solid #e6a23c;
}

.item-info {
  flex: 1;
  min-width: 0;
}

.item-name {
  font-weight: bold;
  color: #303133;
  margin-bottom: 4px;
}

.item-path {
  color: #909399;
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-size {
  width: 150px;
  text-align: right;
}

.size-value {
  display: block;
  font-weight: bold;
  color: #303133;
  margin-bottom: 4px;
}

.error-state,
.initial-state {
  text-align: center;
  padding: 60px 0;
}

.migration-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid #e4e7ed;
}

.migration-item:last-child {
  border-bottom: none;
}

.migration-warning {
  color: #e6a23c;
  font-size: 12px;
  margin-top: 10px;
  padding: 10px;
  background-color: #fdf6ec;
  border-radius: 4px;
}

@media (max-width: 768px) {
  .controls {
    flex-direction: column;
    align-items: center;
  }
  
  .header-controls {
    flex-wrap: wrap;
  }
  
  .item-row {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
  
  .item-size {
    width: 100%;
    text-align: left;
  }
}
</style>