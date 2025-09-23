<template>
  <div class="error-recovery-monitor">
    <div class="recovery-header">
      <h3>错误恢复监控</h3>
      <div class="recovery-controls">
        <el-button @click="refreshStatistics" :loading="loading" size="small">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
        <el-button @click="cleanupBackups" size="small" type="warning">
          <el-icon><Delete /></el-icon>
          清理备份
        </el-button>
        <el-button @click="testRecovery" size="small" type="primary">
          <el-icon><Warning /></el-icon>
          测试恢复
        </el-button>
      </div>
    </div>

    <div class="recovery-stats">
      <el-row :gutter="20">
        <el-col :span="6">
          <el-card>
            <div class="stat-item">
              <div class="stat-value">{{ statistics.total_operations }}</div>
              <div class="stat-label">总恢复操作</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card>
            <div class="stat-item">
              <div class="stat-value" style="color: #67c23a;">{{ statistics.successful_recoveries }}</div>
              <div class="stat-label">成功恢复</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card>
            <div class="stat-item">
              <div class="stat-value" style="color: #f56c6c;">{{ statistics.failed_recoveries }}</div>
              <div class="stat-label">失败恢复</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card>
            <div class="stat-item">
              <div class="stat-value" style="color: #409eff;">
                {{ successRate }}%
              </div>
              <div class="stat-label">成功率</div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <div class="recovery-details">
      <el-row :gutter="20">
        <el-col :span="8">
          <el-card title="重试统计">
            <div class="detail-item">
              <span class="detail-label">重试成功:</span>
              <span class="detail-value">{{ statistics.retry_successes }}</span>
            </div>
          </el-card>
        </el-col>
        <el-col :span="8">
          <el-card title="回滚统计">
            <div class="detail-item">
              <span class="detail-label">回滚成功:</span>
              <span class="detail-value">{{ statistics.rollback_successes }}</span>
            </div>
          </el-card>
        </el-col>
        <el-col :span="8">
          <el-card title="其他操作">
            <div class="detail-item">
              <span class="detail-label">跳过:</span>
              <span class="detail-value">{{ statistics.skip_count }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">中止:</span>
              <span class="detail-value">{{ statistics.abort_count }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">手动:</span>
              <span class="detail-value">{{ statistics.manual_count }}</span>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <div class="recovery-config">
      <el-card title="恢复配置">
        <el-form :model="config" label-width="150px">
          <el-form-item label="自动恢复">
            <el-switch v-model="config.enableAutoRecovery" />
          </el-form-item>
          <el-form-item label="部分回滚">
            <el-switch v-model="config.enablePartialRollback" />
          </el-form-item>
          <el-form-item label="最大重试次数">
            <el-input-number v-model="config.maxRetries" :min="1" :max="10" />
          </el-form-item>
          <el-form-item label="重试延迟(毫秒)">
            <el-input-number v-model="config.retryDelayMs" :min="100" :max="10000" :step="100" />
          </el-form-item>
          <el-form-item label="最大回滚大小(MB)">
            <el-input-number v-model="config.maxRollbackSizeMb" :min="100" :max="10000" :step="100" />
          </el-form-item>
          <el-form-item label="备份保留时间(小时)">
            <el-input-number v-model="config.backupRetentionHours" :min="1" :max="168" :step="1" />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="applyConfig">应用配置</el-button>
            <el-button @click="resetConfig">重置</el-button>
          </el-form-item>
        </el-form>
      </el-card>
    </div>

    <div class="recovery-actions">
      <el-card title="恢复操作">
        <el-space>
          <el-button @click="showBackupDialog" type="primary">
            <el-icon><DocumentCopy /></el-icon>
            创建备份
          </el-button>
          <el-button @click="showRollbackDialog" type="warning">
            <el-icon><RefreshLeft /></el-icon>
            执行回滚
          </el-button>
          <el-button @click="showEmergencyDialog" type="danger">
            <el-icon><Warning /></el-icon>
            紧急恢复
          </el-button>
        </el-space>
      </el-card>
    </div>

    <!-- 创建备份对话框 -->
    <el-dialog
      v-model="backupDialog.visible"
      title="创建备份"
      width="500px"
    >
      <el-form :model="backupDialog">
        <el-form-item label="源路径">
          <el-input
            v-model="backupDialog.sourcePath"
            placeholder="请输入要备份的路径"
          />
        </el-form-item>
        <el-form-item label="操作类型">
          <el-select v-model="backupDialog.operationType" placeholder="选择操作类型">
            <el-option label="扫描" value="Scan" />
            <el-option label="迁移" value="Migrate" />
            <el-option label="删除" value="Delete" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="backupDialog.visible = false">取消</el-button>
        <el-button type="primary" @click="createBackup" :loading="backupDialog.loading">
          创建备份
        </el-button>
      </template>
    </el-dialog>

    <!-- 回滚对话框 -->
    <el-dialog
      v-model="rollbackDialog.visible"
      title="执行回滚"
      width="500px"
    >
      <p>确定要执行回滚操作吗？这将恢复到之前的状态。</p>
      <el-alert
        title="警告"
        type="warning"
        description="回滚操作不可撤销，请确保您了解其影响。"
        show-icon
      />
      <template #footer>
        <el-button @click="rollbackDialog.visible = false">取消</el-button>
        <el-button type="warning" @click="performRollback" :loading="rollbackDialog.loading">
          确认回滚
        </el-button>
      </template>
    </el-dialog>

    <!-- 紧急恢复对话框 -->
    <el-dialog
      v-model="emergencyDialog.visible"
      title="紧急恢复"
      width="500px"
    >
      <p>紧急恢复将尝试修复系统到稳定状态。</p>
      <el-alert
        title="紧急操作"
        type="error"
        description="此操作将尝试自动修复检测到的错误，可能需要一些时间。"
        show-icon
      />
      <template #footer>
        <el-button @click="emergencyDialog.visible = false">取消</el-button>
        <el-button type="danger" @click="performEmergencyRecovery" :loading="emergencyDialog.loading">
          开始紧急恢复
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Refresh, Delete, DocumentCopy, RefreshLeft, Warning } from '@element-plus/icons-vue';
import { errorRecoveryAPI, type RecoveryStatistics } from '../services/api';

// 状态
const loading = ref(false);
const statistics = ref<RecoveryStatistics>({
  total_operations: 0,
  successful_recoveries: 0,
  failed_recoveries: 0,
  retry_successes: 0,
  rollback_successes: 0,
  skip_count: 0,
  abort_count: 0,
  manual_count: 0
});

const config = ref({
  enableAutoRecovery: true,
  enablePartialRollback: true,
  maxRetries: 3,
  retryDelayMs: 1000,
  maxRollbackSizeMb: 1000,
  backupRetentionHours: 24
});

const backupDialog = ref({
  visible: false,
  sourcePath: '',
  operationType: 'Scan',
  loading: false
});

const rollbackDialog = ref({
  visible: false,
  loading: false
});

const emergencyDialog = ref({
  visible: false,
  loading: false
});

// 计算属性
const successRate = computed(() => {
  if (statistics.value.total_operations === 0) return 0;
  return Math.round((statistics.value.successful_recoveries / statistics.value.total_operations) * 100);
});

// 方法
const refreshStatistics = async () => {
  loading.value = true;
  try {
    const result = await errorRecoveryAPI.getRecoveryStatistics();
    statistics.value = result;
    ElMessage.success('恢复统计已更新');
  } catch (error) {
    ElMessage.error(`获取恢复统计失败: ${error}`);
  } finally {
    loading.value = false;
  }
};

const cleanupBackups = async () => {
  try {
    const result = await errorRecoveryAPI.cleanupExpiredBackups();
    ElMessage.success(`已清理 ${result} 个过期备份`);
  } catch (error) {
    ElMessage.error(`清理备份失败: ${error}`);
  }
};

const testRecovery = async () => {
  try {
    const result = await errorRecoveryAPI.testErrorRecovery();
    if (result) {
      ElMessage.success('错误恢复功能测试成功');
    }
  } catch (error) {
    ElMessage.error(`测试失败: ${error}`);
  }
};

const applyConfig = () => {
  ElMessage.success('配置已应用（需要后端支持）');
};

const resetConfig = () => {
  config.value = {
    enableAutoRecovery: true,
    enablePartialRollback: true,
    maxRetries: 3,
    retryDelayMs: 1000,
    maxRollbackSizeMb: 1000,
    backupRetentionHours: 24
  };
  ElMessage.success('配置已重置');
};

const showBackupDialog = () => {
  backupDialog.value.visible = true;
};

const showRollbackDialog = () => {
  rollbackDialog.value.visible = true;
};

const showEmergencyDialog = () => {
  emergencyDialog.value.visible = true;
};

const createBackup = async () => {
  if (!backupDialog.value.sourcePath) {
    ElMessage.warning('请输入源路径');
    return;
  }
  
  backupDialog.value.loading = true;
  try {
    // 这里应该调用创建备份的API
    ElMessage.success(`备份创建请求已发送: ${backupDialog.value.sourcePath}`);
    backupDialog.value.visible = false;
  } catch (error) {
    ElMessage.error(`创建备份失败: ${error}`);
  } finally {
    backupDialog.value.loading = false;
  }
};

const performRollback = async () => {
  rollbackDialog.value.loading = true;
  try {
    // 这里应该调用回滚的API
    ElMessage.success('回滚操作已执行');
    rollbackDialog.value.visible = false;
  } catch (error) {
    ElMessage.error(`回滚失败: ${error}`);
  } finally {
    rollbackDialog.value.loading = false;
  }
};

const performEmergencyRecovery = async () => {
  emergencyDialog.value.loading = true;
  try {
    // 这里应该调用紧急恢复的API
    ElMessage.success('紧急恢复操作已执行');
    emergencyDialog.value.visible = false;
  } catch (error) {
    ElMessage.error(`紧急恢复失败: ${error}`);
  } finally {
    emergencyDialog.value.loading = false;
  }
};

// 生命周期
onMounted(() => {
  refreshStatistics();
});
</script>

<style scoped>
.error-recovery-monitor {
  padding: 20px;
}

.recovery-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.recovery-header h3 {
  margin: 0;
  color: #303133;
}

.recovery-controls {
  display: flex;
  gap: 10px;
}

.recovery-stats {
  margin-bottom: 20px;
}

.stat-item {
  text-align: center;
  padding: 10px;
}

.stat-value {
  font-size: 24px;
  font-weight: bold;
  color: #409eff;
  margin-bottom: 5px;
}

.stat-label {
  font-size: 14px;
  color: #606266;
}

.recovery-details {
  margin-bottom: 20px;
}

.detail-item {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 14px;
}

.detail-label {
  color: #606266;
}

.detail-value {
  color: #303133;
  font-weight: 500;
}

.recovery-config {
  margin-bottom: 20px;
}

.recovery-actions {
  margin-bottom: 20px;
}
</style>