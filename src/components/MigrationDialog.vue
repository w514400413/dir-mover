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

      <div v-if="migrationInProgress" class="progress-section">
        <el-progress
          :percentage="migrationProgress"
          :status="migrationStatus"
        />
        <div class="progress-info">
          <span>{{ migrationMessage }}</span>
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
import { ElMessage } from 'element-plus';
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
  set: (value) => emit('update:visible', value)
});

// 迁移状态
const targetPath = ref('');
const createSymlink = ref(true);
const deleteSource = ref(true);
const migrationInProgress = ref(false);
const migrationProgress = ref(0);
const migrationStatus = ref<'success' | 'exception' | 'warning'>('success');
const migrationMessage = ref('');

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
    const path = prompt(`请输入目标路径（例如：D:\\MovedFolders\\${props.sourceDirectory?.name || ''}）`);
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

// 确认迁移
async function confirmMigration() {
  if (!props.sourceDirectory || !targetPath.value) return;

  try {
    migrationInProgress.value = true;
    migrationProgress.value = 0;
    migrationMessage.value = '正在准备迁移...';
    migrationStatus.value = 'success';

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
      
      ElMessage.success('迁移成功！');
      dialogVisible.value = false;
    } else {
      migrationMessage.value = `迁移失败: ${result.message}`;
      migrationStatus.value = 'exception';
      ElMessage.error(`迁移失败: ${result.message}`);
    }
  } catch (error) {
    migrationMessage.value = `迁移失败: ${error}`;
    migrationStatus.value = 'exception';
    ElMessage.error(`迁移失败: ${error}`);
  } finally {
    migrationInProgress.value = false;
  }
}

// 监听目标路径变化
watch(targetPath, (newPath) => {
  if (newPath) {
    validateTargetPath(newPath);
  } else {
    validationMessage.value = '';
    isValidTarget.value = false;
  }
});

// 监听对话框显示状态
watch(dialogVisible, (visible) => {
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
</style>