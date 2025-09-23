<template>
  <teleport to="body">
    <div
      v-if="visible"
      class="context-menu"
      :style="{ left: position.x + 'px', top: position.y + 'px' }"
      @click.stop
      @contextmenu.prevent
    >
      <!-- 主要操作区域 -->
      <div class="menu-section">
        <div class="menu-header">
          <el-icon><Folder /></el-icon>
          <span class="directory-name">{{ directory?.name || '未知目录' }}</span>
        </div>
        
        <div class="menu-item" @click="handleMigrate" v-if="showMigration">
          <el-icon><FolderRemove /></el-icon>
          <span>迁移目录</span>
          <span class="shortcut-key">Ctrl+M</span>
        </div>
        
        <div class="menu-item" @click="handleOpen">
          <el-icon><FolderOpened /></el-icon>
          <span>打开目录</span>
          <span class="shortcut-key">Ctrl+O</span>
        </div>
        
        <div class="menu-item" @click="handleRefresh">
          <el-icon><Refresh /></el-icon>
          <span>刷新</span>
          <span class="shortcut-key">F5</span>
        </div>
      </div>

      <div class="menu-divider"></div>

      <!-- 快捷操作区域 -->
      <div class="menu-section">
        <div class="menu-title">快捷操作</div>
        
        <div class="menu-item" @click="handleCopyPath">
          <el-icon><CopyDocument /></el-icon>
          <span>复制路径</span>
        </div>
        
        <div class="menu-item" @click="handleCopyName">
          <el-icon><Document /></el-icon>
          <span>复制名称</span>
        </div>
        
        <div class="menu-item" @click="handleOpenTerminal">
          <el-icon><Monitor /></el-icon>
          <span>在此处打开终端</span>
        </div>
      </div>

      <div class="menu-divider"></div>

      <!-- 预览区域 -->
      <div class="menu-section" v-if="directory">
        <div class="menu-title">预览信息</div>
        <div class="preview-info">
          <div class="info-item">
            <span class="info-label">大小:</span>
            <span class="info-value">{{ formatSize(directory.size) }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">文件数:</span>
            <span class="info-value">{{ directory.fileCount }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">路径:</span>
            <span class="info-value" :title="directory.path">{{ truncatePath(directory.path) }}</span>
          </div>
        </div>
      </div>

      <div class="menu-divider"></div>

      <!-- 高级操作 -->
      <div class="menu-section">
        <div class="menu-item" @click="handlePreviewMigration" v-if="showMigration">
          <el-icon><View /></el-icon>
          <span>预览迁移</span>
        </div>
        
        <div class="menu-item" @click="handleCalculateSpace">
          <el-icon><PieChart /></el-icon>
          <span>计算空间占用</span>
        </div>
        
        <div class="menu-item" @click="handleProperties">
          <el-icon><InfoFilled /></el-icon>
          <span>详细属性</span>
        </div>
      </div>

      <!-- 确认对话框 -->
      <el-dialog
        v-model="confirmDialogVisible"
        title="操作确认"
        width="400px"
        :before-close="handleConfirmClose"
      >
        <div class="confirm-content">
          <el-icon :size="48" :class="confirmIconClass">
            <WarningFilled />
          </el-icon>
          <p class="confirm-message">{{ confirmMessage }}</p>
          <div class="confirm-details" v-if="confirmDetails">
            <p>{{ confirmDetails }}</p>
          </div>
        </div>
        <template #footer>
          <span class="dialog-footer">
            <el-button @click="handleConfirmCancel">取消</el-button>
            <el-button type="primary" @click="handleConfirmOk" :loading="confirmLoading">
              确认
            </el-button>
          </span>
        </template>
      </el-dialog>

      <!-- 预览对话框 -->
      <el-dialog
        v-model="previewDialogVisible"
        title="迁移预览"
        width="500px"
      >
        <div class="preview-content">
          <div class="preview-item">
            <span class="preview-label">源目录:</span>
            <span class="preview-value">{{ directory?.path }}</span>
          </div>
          <div class="preview-item">
            <span class="preview-label">大小:</span>
            <span class="preview-value">{{ formatSize(directory?.size || 0) }}</span>
          </div>
          <div class="preview-item">
            <span class="preview-label">预计时间:</span>
            <span class="preview-value">{{ estimateMigrationTime(directory?.size || 0) }}</span>
          </div>
          <div class="preview-item">
            <span class="preview-label">子目录数:</span>
            <span class="preview-value">{{ directory?.subdirectories?.length || 0 }}</span>
          </div>
        </div>
        <template #footer>
          <span class="dialog-footer">
            <el-button @click="previewDialogVisible = false">关闭</el-button>
            <el-button type="primary" @click="proceedWithMigration">
              继续迁移
            </el-button>
          </span>
        </template>
      </el-dialog>
    </div>
  </teleport>
</template>

<script setup lang="ts">
import { ref, onUnmounted, computed, watch } from 'vue';
import {
  FolderRemove,
  FolderOpened,
  Refresh,
  InfoFilled,
  Folder,
  CopyDocument,
  Document,
  Monitor,
  View,
  PieChart,
  WarningFilled
} from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import type { DirectoryInfo } from '../types/directory';
import { utils } from '../services/api';

interface Props {
  visible: boolean;
  position: { x: number; y: number };
  directory: DirectoryInfo | null;
}

interface Emits {
  (e: 'update:visible', value: boolean): void;
  (e: 'migrate', directory: DirectoryInfo): void;
  (e: 'open', path: string): void;
  (e: 'refresh'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// 状态变量
const confirmDialogVisible = ref(false);
const previewDialogVisible = ref(false);
const confirmMessage = ref('');
const confirmDetails = ref('');
const confirmLoading = ref(false);
const confirmAction = ref<'migrate' | 'delete' | null>(null);

// 计算属性
const showMigration = computed(() => {
  return props.directory && props.directory.size > 1024 * 1024 * 100; // 只显示大于100MB目录的迁移选项
});

// 格式化文件大小
function formatSize(bytes: number): string {
  return utils.formatFileSize(bytes);
}

// 截断路径显示
function truncatePath(path: string): string {
  if (path.length <= 50) return path;
  return '...' + path.slice(-47);
}

// 估计迁移时间
function estimateMigrationTime(size: number): string {
  // 假设迁移速度为 100MB/s
  const speed = 100 * 1024 * 1024; // 100MB per second
  const seconds = size / speed;
  
  if (seconds < 60) {
    return `${Math.ceil(seconds)}秒`;
  } else if (seconds < 3600) {
    return `${Math.ceil(seconds / 60)}分钟`;
  } else {
    return `${Math.ceil(seconds / 3600)}小时`;
  }
}

// 处理迁移（带确认）
function handleMigrate() {
  if (props.directory) {
    emit('migrate', props.directory);
    closeMenu();
  }
}

// 处理打开
function handleOpen() {
  if (props.directory) {
    emit('open', props.directory.path);
    closeMenu();
  }
}

// 处理刷新
function handleRefresh() {
  emit('refresh');
  closeMenu();
}

// 处理属性（增强版）
function handleProperties() {
  if (props.directory) {
    // 显示详细的属性信息
    const details = `
目录名称: ${props.directory.name}
完整路径: ${props.directory.path}
目录大小: ${formatSize(props.directory.size)}
文件数量: ${props.directory.fileCount}
子目录数: ${props.directory.subdirectories?.length || 0}
    `;
    
    showConfirmDialog(
      '目录属性',
      `${props.directory.name} 的详细信息`,
      details,
      null
    );
  }
}

// 快捷操作：复制路径
function handleCopyPath() {
  if (props.directory) {
    copyToClipboard(props.directory.path);
    ElMessage.success('路径已复制到剪贴板');
    closeMenu();
  }
}

// 快捷操作：复制名称
function handleCopyName() {
  if (props.directory) {
    copyToClipboard(props.directory.name);
    ElMessage.success('目录名称已复制到剪贴板');
    closeMenu();
  }
}

// 快捷操作：打开终端
async function handleOpenTerminal() {
  if (props.directory) {
    try {
      // 使用Tauri的shell插件打开终端
      const { openPath } = await import('@tauri-apps/plugin-opener');
      await openPath(props.directory.path);
      ElMessage.success('终端已打开');
    } catch (error) {
      ElMessage.error(`打开终端失败: ${error}`);
    }
    closeMenu();
  }
}

// 预览迁移
function handlePreviewMigration() {
  if (props.directory) {
    previewDialogVisible.value = true;
  }
}

// 计算空间占用
function handleCalculateSpace() {
  if (props.directory) {
    const spacePercentage = ((props.directory.size / (1024 * 1024 * 1024)) * 100).toFixed(1);
    ElMessage.info(`空间占用: ${formatSize(props.directory.size)} (${spacePercentage}% of 1GB)`);
    closeMenu();
  }
}

// 继续迁移
function proceedWithMigration() {
  if (props.directory) {
    emit('migrate', props.directory);
    closeMenu();
  }
}

// 显示确认对话框
function showConfirmDialog(title: string, message: string, details: string, action: 'migrate' | 'delete' | null) {
  confirmDialogVisible.value = true;
  confirmMessage.value = message;
  confirmDetails.value = details;
  confirmAction.value = action;
}

// 确认对话框处理
function handleConfirmOk() {
  confirmLoading.value = true;
  
  setTimeout(() => {
    confirmLoading.value = false;
    confirmDialogVisible.value = false;
    
    if (confirmAction.value === 'migrate') {
      proceedWithMigration();
    }
    
    // 重置状态
    confirmAction.value = null;
    confirmMessage.value = '';
    confirmDetails.value = '';
  }, 1000);
}

function handleConfirmCancel() {
  confirmDialogVisible.value = false;
  confirmAction.value = null;
}

function handleConfirmClose() {
  if (!confirmLoading.value) {
    confirmDialogVisible.value = false;
  }
}

// 计算确认图标类名
const confirmIconClass = computed(() => {
  switch (confirmAction.value) {
    case 'migrate':
      return 'warning-icon';
    case 'delete':
      return 'error-icon';
    default:
      return 'info-icon';
  }
});

// 复制到剪贴板
async function copyToClipboard(text: string) {
  try {
    if (navigator.clipboard) {
      await navigator.clipboard.writeText(text);
    } else {
      // 降级方案
      const textArea = document.createElement('textarea');
      textArea.value = text;
      document.body.appendChild(textArea);
      textArea.select();
      document.execCommand('copy');
      document.body.removeChild(textArea);
    }
  } catch (error) {
    console.error('复制失败:', error);
    ElMessage.error('复制失败，请手动复制');
  }
}

// 关闭菜单
function closeMenu() {
  emit('update:visible', false);
}

// 点击外部关闭菜单
function handleClickOutside(event: MouseEvent) {
  const target = event.target as HTMLElement;
  if (!target.closest('.context-menu')) {
    closeMenu();
  }
}

// 监听点击事件
watch(() => props.visible, (visible) => {
  if (visible) {
    document.addEventListener('click', handleClickOutside);
  } else {
    document.removeEventListener('click', handleClickOutside);
  }
});

// 清理事件监听
onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<style scoped>
.context-menu {
  position: fixed;
  background: white;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
  z-index: 2000;
  min-width: 180px;
  padding: 5px 0;
}

.menu-item {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  cursor: pointer;
  transition: background-color 0.2s;
  font-size: 14px;
  color: #606266;
}

.menu-item:hover {
  background-color: #f5f7fa;
  color: #409eff;
}

.menu-item .el-icon {
  margin-right: 8px;
  font-size: 16px;
}

.menu-divider {
  height: 1px;
  background-color: #e4e7ed;
  margin: 5px 0;
}

/* 增强的右键菜单样式 */
.menu-section {
  padding: 4px 0;
}

.menu-header {
  display: flex;
  align-items: center;
  padding: 10px 16px 8px;
  border-bottom: 1px solid #f0f0f0;
  margin-bottom: 4px;
}

.directory-name {
  font-weight: 500;
  color: #303133;
  margin-left: 8px;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.menu-title {
  padding: 6px 16px 4px;
  font-size: 12px;
  color: #909399;
  font-weight: 500;
  text-transform: uppercase;
}

.shortcut-key {
  margin-left: auto;
  font-size: 11px;
  color: #c0c4cc;
  background-color: #f4f4f5;
  padding: 2px 6px;
  border-radius: 3px;
}

.preview-info {
  padding: 8px 16px;
  background-color: #f8f9fa;
  border-radius: 4px;
  margin: 4px 16px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  margin-bottom: 4px;
  font-size: 12px;
}

.info-item:last-child {
  margin-bottom: 0;
}

.info-label {
  color: #606266;
  font-weight: 500;
}

.info-value {
  color: #303133;
  font-weight: 400;
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 确认对话框样式 */
.confirm-content {
  text-align: center;
  padding: 20px;
}

.confirm-message {
  margin: 16px 0;
  font-size: 16px;
  color: #303133;
}

.confirm-details {
  background-color: #f8f9fa;
  padding: 12px;
  border-radius: 4px;
  margin-top: 12px;
  font-size: 13px;
  color: #606266;
  white-space: pre-wrap;
  text-align: left;
}

.warning-icon {
  color: #e6a23c;
}

.error-icon {
  color: #f56c6c;
}

.info-icon {
  color: #409eff;
}

/* 预览对话框样式 */
.preview-content {
  padding: 16px;
}

.preview-item {
  display: flex;
  justify-content: space-between;
  margin-bottom: 12px;
  padding: 8px 0;
  border-bottom: 1px solid #f0f0f0;
}

.preview-item:last-child {
  border-bottom: none;
  margin-bottom: 0;
}

.preview-label {
  font-weight: 500;
  color: #606266;
  min-width: 80px;
}

.preview-value {
  color: #303133;
  text-align: right;
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 响应式设计 */
@media (max-width: 480px) {
  .context-menu {
    min-width: 160px;
    max-width: 90vw;
  }
  
  .directory-name {
    max-width: 150px;
  }
  
  .preview-value {
    max-width: 200px;
  }
}

/* 动画效果 */
.menu-item {
  transition: all 0.2s ease;
}

.menu-item:hover {
  background-color: #f0f7ff;
  color: #409eff;
  transform: translateX(2px);
}

/* 加载状态 */
.loading-spinner {
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
</style>