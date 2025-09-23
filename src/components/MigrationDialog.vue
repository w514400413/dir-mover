<template>
  <el-dialog
    v-model="dialogVisible"
    title="迁移目录"
    width="500px"
    :close-on-click-modal="false"
  >
    <div class="migration-content">
      <div class="info-section">
        <h4>源目录信息</h4>
        <div class="info-item">
          <span class="label">路径：</span>
          <span class="value">{{ sourceDirectory?.path }}</span>
        </div>
        <div class="info-item">
          <span class="label">大小：</span>
          <span class="value">{{ formatSize(sourceDirectory?.size || 0) }}</span>
        </div>
        <div class="info-item">
          <span class="label">文件数：</span>
          <span class="value">{{ sourceDirectory?.fileCount || 0 }}</span>
        </div>
      </div>

      <el-divider />

      <div class="target-section">
        <h4>目标位置</h4>
        <div class="path-input">
          <el-input
            v-model="targetPath"
            placeholder="请选择目标路径"
            readonly
          >
            <template #append>
              <el-button @click="selectTargetPath" :icon="FolderOpened">
                浏览
              </el-button>
            </template>
          </el-input>
        </div>
        
        <el-alert
          v-if="validationMessage"
          :title="validationMessage"
          :type="validationType"
          :closable="false"
          show-icon
          style="margin-top: 10px"
        />
      </div>

      <el-divider />

      <div class="options-section">
        <h4>迁移选项</h4>
        <el-checkbox v-model="createSymlink" :disabled="!canCreateSymlink">
          创建符号链接（保持原路径可用）
        </el-checkbox>
        <el-checkbox v-model="deleteSource">
          迁移完成后删除源目录
        </el-checkbox>
      </div>

      <!-- 迁移进度显示 -->
      <div v-if="migrationInProgress" class="progress-section">
        <div class="progress-header">
          <h4>迁移进度</h4>
          <span class="progress-time">{{ elapsedTime }}</span>
        </div>
        
        <el-progress
          :percentage="migrationProgress"
          :status="migrationStatus"
          :stroke-width="8"
          :show-text="true"
          :text-inside="true"
        />
        
        <div class="progress-details">
          <div class="progress-item">
            <span class="progress-label">当前操作:</span>
            <span class="progress-value">{{ currentOperation }}</span>
          </div>
          <div class="progress-item">
            <span class="progress-label">已处理:</span>
            <span class="progress-value">{{ processedItems }} / {{ totalItems }} 个项目</span>
          </div>
          <div class="progress-item">
            <span class="progress-label">传输速度:</span>
            <span class="progress-value">{{ transferSpeed }}</span>
          </div>
          <div class="progress-item">
            <span class="progress-label">预计剩余时间:</span>
            <span class="progress-value">{{ estimatedTimeRemaining }}</span>
          </div>
        </div>
        
        <div class="progress-info">
          <span>{{ migrationMessage }}</span>
          <el-button
            v-if="canCancel"
            type="text"
            size="small"
            @click="cancelMigration"
            :loading="cancelLoading"
          >
            取消迁移
          </el-button>
        </div>
      </div>

      <!-- 迁移统计信息 -->
      <div v-if="migrationCompleted" class="statistics-section">
        <h4>迁移统计</h4>
        <div class="statistics-grid">
          <div class="stat-item">
            <div class="stat-value">{{ formatSize(totalBytesTransferred) }}</div>
            <div class="stat-label">总传输量</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ totalFilesTransferred }}</div>
            <div class="stat-label">文件数</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ totalDirectoriesTransferred }}</div>
            <div class="stat-label">目录数</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ totalMigrationTime }}</div>
            <div class="stat-label">总耗时</div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="dialogVisible = false" :disabled="migrationInProgress">
          取消
        </el-button>
        <el-button
          type="primary"
          @click="confirmMigration"
          :loading="migrationInProgress"
          :disabled="!targetPath || !isValidTarget"
        >
          开始迁移
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { FolderOpened } from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import type { DirectoryInfo } from '../types/directory';
import { migrationAPI, systemAPI, utils } from '../services/api';

interface Props {
  visible: boolean;
  sourceDirectory: DirectoryInfo | null;
}

interface Emits {
  (e: 'update:visible', value: boolean): void;
  (e: 'confirmed', targetPath: string): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// 对话框状态
const dialogVisible = computed({
  get: () => props.visible,
  set: (value: boolean) => emit('update:visible', value)
});

// 迁移状态
const targetPath = ref('');
const createSymlink = ref(true);
const deleteSource = ref(true);
const migrationInProgress = ref(false);
const migrationProgress = ref(0);
const migrationStatus = ref<'success' | 'exception' | 'warning'>('success');
const migrationMessage = ref('');

// 增强的进度跟踪状态
const currentOperation = ref('准备迁移...');
const processedItems = ref(0);
const totalItems = ref(0);
const transferSpeed = ref('0 B/s');
const estimatedTimeRemaining = ref('计算中...');
const elapsedTime = ref('00:00');
const canCancel = ref(true);
const cancelLoading = ref(false);
const migrationCompleted = ref(false);
const totalBytesTransferred = ref(0);
const totalFilesTransferred = ref(0);
const totalDirectoriesTransferred = ref(0);
const totalMigrationTime = ref('00:00');
const migrationStartTime = ref<number>(0);
const progressUpdateInterval = ref<number | null>(null);

// 验证状态
const validationMessage = ref('');
const validationType = ref<'success' | 'error' | 'warning'>('error');
const isValidTarget = ref(false);
const canCreateSymlink = ref(true);

// 格式化文件大小
function formatSize(bytes: number): string {
  return utils.formatFileSize(bytes);
}

// 选择目标路径
async function selectTargetPath() {
  try {
    // 这里应该调用 Tauri 的文件选择对话框
    // 暂时使用简单的输入提示
    const path = prompt(`请输入目标路径（例如：D:\\\\MovedFolders\\\\${props.sourceDirectory?.name || ''}）`);
    if (path) {
      targetPath.value = path;
      await validateTargetPath(path);
    }
  } catch (error) {
    ElMessage.error(`选择路径失败: ${error}`);
  }
}

// 验证目标路径
async function validateTargetPath(path: string) {
  try {
    validationMessage.value = '正在验证路径...';
    validationType.value = 'warning';
    
    const result = await migrationAPI.validateMigrationPath(
      props.sourceDirectory?.path || '',
      path
    );
    
    if (result.valid) {
      validationMessage.value = result.message;
      validationType.value = 'success';
      isValidTarget.value = true;
    } else {
      validationMessage.value = result.message;
      validationType.value = 'error';
      isValidTarget.value = false;
    }
  } catch (error) {
    validationMessage.value = `验证失败: ${error}`;
    validationType.value = 'error';
    isValidTarget.value = false;
  }
}

// 确认迁移（增强版）
async function confirmMigration() {
  if (!props.sourceDirectory || !targetPath.value) return;

  try {
    // 初始化迁移状态
    migrationInProgress.value = true;
    migrationCompleted.value = false;
    migrationProgress.value = 0;
    migrationMessage.value = '正在准备迁移...';
    migrationStatus.value = 'success';
    migrationStartTime.value = Date.now();
    currentOperation.value = '正在初始化迁移任务...';
    processedItems.value = 0;
    totalItems.value = 100; // 初始估计
    transferSpeed.value = '0 B/s';
    estimatedTimeRemaining.value = '计算中...';
    elapsedTime.value = '00:00';
    canCancel.value = true;
    cancelLoading.value = false;

    // 开始时间更新定时器
    startProgressTimer();

    // 模拟进度更新（在实际应用中，这里应该监听真实的进度事件）
    simulateProgressUpdate();

    const result = await migrationAPI.migrateDirectory({
      sourcePath: props.sourceDirectory.path,
      targetPath: targetPath.value,
      createSymlink: createSymlink.value,
      deleteSource: deleteSource.value
    });

    if (result.success) {
      migrationProgress.value = 100;
      migrationMessage.value = '迁移完成！';
      migrationStatus.value = 'success';
      migrationCompleted.value = true;
      canCancel.value = false;
      
      // 计算总耗时
      const endTime = Date.now();
      const totalTime = Math.round((endTime - migrationStartTime.value) / 1000);
      totalMigrationTime.value = formatTime(totalTime);
      
      ElMessage.success('迁移成功！');
      
      // 延迟关闭对话框，让用户看到完成信息
      setTimeout(() => {
        dialogVisible.value = false;
      }, 2000);
    } else {
      migrationMessage.value = `迁移失败: ${result.message}`;
      migrationStatus.value = 'exception';
      canCancel.value = false;
      ElMessage.error(`迁移失败: ${result.message}`);
    }
  } catch (error) {
    migrationMessage.value = `迁移失败: ${error}`;
    migrationStatus.value = 'exception';
    canCancel.value = false;
    ElMessage.error(`迁移失败: ${error}`);
  } finally {
    migrationInProgress.value = false;
    stopProgressTimer();
  }
}

// 模拟进度更新（在实际应用中，应该监听真实的进度事件）
function simulateProgressUpdate() {
  let currentProgress = 0;
  const totalSteps = 100;
  const stepInterval = 100; // 每100ms更新一次
  
  const interval = setInterval(() => {
    if (!migrationInProgress.value) {
      clearInterval(interval);
      return;
    }
    
    currentProgress += Math.random() * 2; // 随机增长
    
    if (currentProgress >= 95) {
      currentProgress = 95; // 保持在95%，等待实际完成
    }
    
    migrationProgress.value = Math.min(currentProgress, 95);
    
    // 更新详细信息
    updateProgressDetails(currentProgress);
    
    if (currentProgress >= 95) {
      clearInterval(interval);
    }
  }, stepInterval);
}

// 更新进度详细信息
function updateProgressDetails(progress: number) {
  // 模拟不同的操作阶段
  if (progress < 20) {
    currentOperation.value = '正在扫描源目录结构...';
  } else if (progress < 40) {
    currentOperation.value = '正在创建目标目录...';
  } else if (progress < 60) {
    currentOperation.value = '正在复制文件...';
  } else if (progress < 80) {
    currentOperation.value = '正在创建符号链接...';
  } else if (progress < 95) {
    currentOperation.value = '正在清理源目录...';
  } else {
    currentOperation.value = '正在完成迁移...';
  }
  
  // 更新已处理项目数
  processedItems.value = Math.floor((progress / 100) * totalItems.value);
  
  // 更新传输速度（模拟）
  const speed = Math.random() * 50 * 1024 * 1024; // 0-50MB/s
  transferSpeed.value = utils.formatFileSize(Math.floor(speed)) + '/s';
  
  // 更新预计剩余时间
  if (progress > 0 && progress < 100) {
    const remainingProgress = 100 - progress;
    const estimatedSeconds = Math.ceil((remainingProgress / progress) * (Date.now() - migrationStartTime.value) / 1000);
    estimatedTimeRemaining.value = formatTime(estimatedSeconds);
  }
  
  // 更新总传输字节数（模拟）
  if (props.sourceDirectory) {
    totalBytesTransferred.value = Math.floor((progress / 100) * props.sourceDirectory.size);
  }
  
  // 更新文件和目录计数（模拟）
  totalFilesTransferred.value = Math.floor((progress / 100) * (props.sourceDirectory?.fileCount || 0));
  totalDirectoriesTransferred.value = Math.floor((progress / 100) * (props.sourceDirectory?.subdirectories?.length || 0));
}

// 开始进度定时器
function startProgressTimer() {
  progressUpdateInterval.value = setInterval(() => {
    if (!migrationInProgress.value) {
      stopProgressTimer();
      return;
    }
    
    const elapsed = Math.floor((Date.now() - migrationStartTime.value) / 1000);
    elapsedTime.value = formatTime(elapsed);
  }, 1000);
}

// 停止进度定时器
function stopProgressTimer() {
  if (progressUpdateInterval.value) {
    clearInterval(progressUpdateInterval.value);
    progressUpdateInterval.value = null;
  }
}

// 取消迁移
async function cancelMigration() {
  if (!migrationInProgress.value || !canCancel.value) return;
  
  try {
    cancelLoading.value = true;
    
    // 显示确认对话框
    const confirmed = await ElMessageBox.confirm(
      '确定要取消当前的迁移操作吗？',
      '取消确认',
      {
        confirmButtonText: '确定取消',
        cancelButtonText: '继续迁移',
        type: 'warning'
      }
    );
    
    if (confirmed) {
      // 在实际应用中，这里应该调用取消迁移的API
      migrationMessage.value = '正在取消迁移...';
      migrationStatus.value = 'warning';
      canCancel.value = false;
      
      // 模拟取消过程
      setTimeout(() => {
        migrationInProgress.value = false;
        migrationMessage.value = '迁移已取消';
        migrationStatus.value = 'warning';
        ElMessage.warning('迁移操作已取消');
        stopProgressTimer();
        cancelLoading.value = false;
      }, 1500);
    }
  } catch (error) {
    // 用户点击了取消按钮，继续迁移
    console.log('用户选择继续迁移');
  } finally {
    cancelLoading.value = false;
  }
}

// 格式化时间
function formatTime(seconds: number): string {
  if (seconds < 60) {
    return `${seconds}秒`;
  } else if (seconds < 3600) {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}分${remainingSeconds}秒`;
  } else {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remainingSeconds = seconds % 60;
    return `${hours}小时${minutes}分${remainingSeconds}秒`;
  }
}

// 监听目标路径变化
watch(targetPath, (newPath: string) => {
  if (newPath) {
    validateTargetPath(newPath);
  } else {
    validationMessage.value = '';
    isValidTarget.value = false;
  }
});

// 监听对话框显示状态
watch(dialogVisible, (visible: boolean) => {
  if (visible) {
    // 重置状态
    targetPath.value = '';
    createSymlink.value = true;
    deleteSource.value = true;
    migrationInProgress.value = false;
    migrationProgress.value = 0;
    validationMessage.value = '';
    isValidTarget.value = false;
  }
});
</script>

<style scoped>
.migration-content {
  padding: 0 10px;
}

.info-section, .target-section, .options-section {
  margin-bottom: 20px;
}

.info-section h4, .target-section h4, .options-section h4 {
  margin: 0 0 10px 0;
  color: #303133;
  font-size: 14px;
}

.info-item {
  display: flex;
  margin-bottom: 8px;
  font-size: 13px;
}

.info-item .label {
  color: #909399;
  width: 60px;
  flex-shrink: 0;
}

.info-item .value {
  color: #606266;
  flex: 1;
  word-break: break-all;
}

.path-input {
  margin-bottom: 10px;
}

.options-section .el-checkbox {
  display: block;
  margin-bottom: 10px;
}

.progress-section {
  margin-top: 20px;
}

.progress-info {
  margin-top: 10px;
  text-align: center;
  color: #606266;
  font-size: 13px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.progress-details {
  margin-top: 15px;
  background-color: #f5f7fa;
  padding: 12px;
  border-radius: 4px;
}

.progress-item {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 12px;
}

.progress-item:last-child {
  margin-bottom: 0;
}

.progress-label {
  color: #606266;
  font-weight: 500;
}

.progress-value {
  color: #303133;
  font-weight: 600;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.progress-time {
  color: #909399;
  font-size: 12px;
}

.statistics-section {
  margin-top: 20px;
  padding: 15px;
  background-color: #f0f9ff;
  border-radius: 4px;
  border: 1px solid #b3d8ff;
}

.statistics-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 15px;
  margin-top: 10px;
}

.stat-item {
  text-align: center;
  padding: 10px;
  background-color: white;
  border-radius: 4px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.stat-value {
  font-size: 18px;
  font-weight: 600;
  color: #409eff;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 12px;
  color: #606266;
}
</style>