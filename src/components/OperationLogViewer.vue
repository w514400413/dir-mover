<template>
  <div class="operation-log-viewer">
    <div class="log-header">
      <h3>操作日志</h3>
      <div class="log-controls">
        <el-button @click="refreshLogs" :loading="loading" size="small">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
        <el-button @click="exportLogs" size="small" type="primary">
          <el-icon><Download /></el-icon>
          导出
        </el-button>
        <el-button @click="showCleanupDialog" size="small" type="warning">
          <el-icon><Delete /></el-icon>
          清理
        </el-button>
      </div>
    </div>

    <div class="log-stats">
      <el-row :gutter="20">
        <el-col :span="6">
          <el-card>
            <div class="stat-item">
              <div class="stat-value">{{ statistics.total_operations }}</div>
              <div class="stat-label">总操作数</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card>
            <div class="stat-item">
              <div class="stat-value" style="color: #67c23a;">{{ statistics.completed_operations }}</div>
              <div class="stat-label">成功</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card>
            <div class="stat-item">
              <div class="stat-value" style="color: #f56c6c;">{{ statistics.failed_operations }}</div>
              <div class="stat-label">失败</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card>
            <div class="stat-item">
              <div class="stat-value" style="color: #e6a23c;">{{ statistics.cancelled_operations }}</div>
              <div class="stat-label">取消</div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <div class="log-filters">
      <el-form :inline="true" :model="filters">
        <el-form-item label="操作类型">
          <el-select v-model="filters.operationType" placeholder="全部" clearable size="small">
            <el-option label="扫描" value="Scan" />
            <el-option label="迁移" value="Migrate" />
            <el-option label="删除" value="Delete" />
            <el-option label="验证" value="Validate" />
            <el-option label="取消" value="Cancel" />
          </el-select>
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="filters.status" placeholder="全部" clearable size="small">
            <el-option label="成功" value="Completed" />
            <el-option label="失败" value="Failed" />
            <el-option label="取消" value="Cancelled" />
            <el-option label="进行中" value="InProgress" />
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button @click="applyFilters" type="primary" size="small">筛选</el-button>
          <el-button @click="clearFilters" size="small">重置</el-button>
        </el-form-item>
      </el-form>
    </div>

    <div class="log-table">
      <el-table
        :data="filteredLogs"
        style="width: 100%"
        :loading="loading"
        height="400"
      >
        <el-table-column prop="timestamp" label="时间" width="150">
          <template #default="{ row }">
            {{ formatDateTime(row.timestamp) }}
          </template>
        </el-table-column>
        <el-table-column prop="operation_type" label="操作类型" width="100">
          <template #default="{ row }">
            <el-tag :type="getOperationTypeTag(row.operation_type)">
              {{ getOperationTypeText(row.operation_type) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="status" label="状态" width="80">
          <template #default="{ row }">
            <el-tag :type="getStatusTag(row.status)">
              {{ getStatusText(row.status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="source_path" label="源路径" min-width="200" show-overflow-tooltip />
        <el-table-column prop="target_path" label="目标路径" min-width="200" show-overflow-tooltip />
        <el-table-column prop="details" label="详情" min-width="150" show-overflow-tooltip />
        <el-table-column prop="duration_ms" label="耗时" width="80">
          <template #default="{ row }">
            {{ formatDuration(row.duration_ms) }}
          </template>
        </el-table-column>
        <el-table-column prop="file_count" label="文件数" width="80" />
        <el-table-column prop="total_size" label="大小" width="100">
          <template #default="{ row }">
            {{ formatSize(row.total_size) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="100" fixed="right">
          <template #default="{ row }">
            <el-button @click="viewLogDetails(row)" size="small" type="text">
              详情
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <!-- 日志详情对话框 -->
    <el-dialog
      v-model="detailsDialog.visible"
      title="日志详情"
      width="600px"
    >
      <div v-if="detailsDialog.log" class="log-details">
        <el-descriptions :column="1" border>
          <el-descriptions-item label="操作ID">{{ detailsDialog.log.id }}</el-descriptions-item>
          <el-descriptions-item label="时间戳">{{ formatDateTime(detailsDialog.log.timestamp) }}</el-descriptions-item>
          <el-descriptions-item label="操作类型">
            <el-tag :type="getOperationTypeTag(detailsDialog.log.operation_type)">
              {{ getOperationTypeText(detailsDialog.log.operation_type) }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="getStatusTag(detailsDialog.log.status)">
              {{ getStatusText(detailsDialog.log.status) }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="用户">{{ detailsDialog.log.user }}</el-descriptions-item>
          <el-descriptions-item label="会话ID">{{ detailsDialog.log.session_id }}</el-descriptions-item>
          <el-descriptions-item label="源路径">{{ detailsDialog.log.source_path }}</el-descriptions-item>
          <el-descriptions-item label="目标路径">{{ detailsDialog.log.target_path || '无' }}</el-descriptions-item>
          <el-descriptions-item label="详情">{{ detailsDialog.log.details }}</el-descriptions-item>
          <el-descriptions-item label="错误信息" v-if="detailsDialog.log.error_message">
            <span style="color: #f56c6c;">{{ detailsDialog.log.error_message }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="耗时">{{ formatDuration(detailsDialog.log.duration_ms) }}</el-descriptions-item>
          <el-descriptions-item label="文件数">{{ detailsDialog.log.file_count || 0 }}</el-descriptions-item>
          <el-descriptions-item label="总大小">{{ formatSize(detailsDialog.log.total_size) }}</el-descriptions-item>
        </el-descriptions>
      </div>
    </el-dialog>

    <!-- 清理确认对话框 -->
    <el-dialog
      v-model="cleanupDialog.visible"
      title="清理旧日志"
      width="400px"
    >
      <p>确定要清理 {{ cleanupDialog.daysToKeep }} 天前的操作日志吗？</p>
      <p style="color: #f56c6c; font-size: 12px;">此操作不可恢复，请谨慎操作。</p>
      <template #footer>
        <el-button @click="cleanupDialog.visible = false">取消</el-button>
        <el-button type="warning" @click="confirmCleanup" :loading="cleanupDialog.loading">
          确认清理
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Refresh, Download, Delete } from '@element-plus/icons-vue';
import { operationAPI, type OperationLog, type OperationStatistics } from '../services/api';

// 状态
const loading = ref(false);
const logs = ref<OperationLog[]>([]);
const statistics = ref<OperationStatistics>({
  total_operations: 0,
  completed_operations: 0,
  failed_operations: 0,
  cancelled_operations: 0,
  total_bytes_transferred: 0,
  total_files_processed: 0,
  total_duration_ms: 0,
  average_duration_ms: 0
});

const filters = ref({
  operationType: '',
  status: ''
});

const detailsDialog = ref({
  visible: false,
  log: null as OperationLog | null
});

const cleanupDialog = ref({
  visible: false,
  daysToKeep: 30,
  loading: false
});

// 计算属性
const filteredLogs = computed(() => {
  let filtered = logs.value;

  if (filters.value.operationType) {
    filtered = filtered.filter(log => log.operation_type === filters.value.operationType);
  }

  if (filters.value.status) {
    filtered = filtered.filter(log => log.status === filters.value.status);
  }

  return filtered;
});

// 方法
const refreshLogs = async () => {
  loading.value = true;
  try {
    const [logsResult, statsResult] = await Promise.all([
      operationAPI.getOperationLogs(1000),
      operationAPI.getOperationStatistics()
    ]);
    
    logs.value = logsResult;
    statistics.value = statsResult;
    
    ElMessage.success('日志数据已更新');
  } catch (error) {
    ElMessage.error(`获取日志数据失败: ${error}`);
  } finally {
    loading.value = false;
  }
};

const exportLogs = async () => {
  try {
    const outputPath = prompt('请输入导出文件路径（例如：C:\\logs\\operations.csv）');
    if (outputPath) {
      const success = await operationAPI.exportOperationLogs(outputPath);
      if (success) {
        ElMessage.success(`日志已导出到: ${outputPath}`);
      }
    }
  } catch (error) {
    ElMessage.error(`导出日志失败: ${error}`);
  }
};

const showCleanupDialog = () => {
  cleanupDialog.value.visible = true;
};

const confirmCleanup = async () => {
  cleanupDialog.value.loading = true;
  try {
    const success = await operationAPI.cleanupOldOperationLogs(cleanupDialog.value.daysToKeep);
    if (success) {
      ElMessage.success('旧日志清理完成');
      await refreshLogs();
    }
  } catch (error) {
    ElMessage.error(`清理日志失败: ${error}`);
  } finally {
    cleanupDialog.value.loading = false;
    cleanupDialog.value.visible = false;
  }
};

const applyFilters = () => {
  // 过滤器通过计算属性自动应用
  ElMessage.success('筛选已应用');
};

const clearFilters = () => {
  filters.value.operationType = '';
  filters.value.status = '';
  ElMessage.success('筛选已重置');
};

const viewLogDetails = (log: OperationLog) => {
  detailsDialog.value.log = log;
  detailsDialog.value.visible = true;
};

// 格式化函数
const formatDateTime = (timestamp: string): string => {
  const date = new Date(timestamp);
  return date.toLocaleString('zh-CN');
};

const formatDuration = (durationMs?: number): string => {
  if (!durationMs) return '-';
  const seconds = Math.floor(durationMs / 1000);
  if (seconds < 60) return `${seconds}秒`;
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes}分${remainingSeconds}秒`;
};

const formatSize = (bytes?: number): string => {
  if (!bytes) return '-';
  return apiUtils.formatFileSize(bytes);
};

const getOperationTypeTag = (type: string): string => {
  const typeMap: Record<string, string> = {
    'Scan': 'info',
    'Migrate': 'primary',
    'Delete': 'danger',
    'CreateSymlink': 'warning',
    'Validate': 'info',
    'Cancel': 'warning',
    'Error': 'danger'
  };
  return typeMap[type] || 'info';
};

const getOperationTypeText = (type: string): string => {
  const typeMap: Record<string, string> = {
    'Scan': '扫描',
    'Migrate': '迁移',
    'Delete': '删除',
    'CreateSymlink': '创建链接',
    'Validate': '验证',
    'Cancel': '取消',
    'Error': '错误'
  };
  return typeMap[type] || type;
};

const getStatusTag = (status: string): string => {
  const statusMap: Record<string, string> = {
    'Completed': 'success',
    'Failed': 'danger',
    'Cancelled': 'warning',
    'InProgress': 'primary',
    'Started': 'info'
  };
  return statusMap[status] || 'info';
};

const getStatusText = (status: string): string => {
  const statusMap: Record<string, string> = {
    'Completed': '成功',
    'Failed': '失败',
    'Cancelled': '取消',
    'InProgress': '进行中',
    'Started': '开始'
  };
  return statusMap[status] || status;
};

// 工具函数
const utils = {
  formatFileSize: (bytes: number): string => {
    if (bytes === 0) return "0 B";
    
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB", "PB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }
};

// 修复导入的utils对象
import { utils as apiUtils } from '../services/api';

// 生命周期
onMounted(() => {
  refreshLogs();
});
</script>

<style scoped>
.operation-log-viewer {
  padding: 20px;
}

.log-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.log-header h3 {
  margin: 0;
  color: #303133;
}

.log-controls {
  display: flex;
  gap: 10px;
}

.log-stats {
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

.log-filters {
  margin-bottom: 20px;
  padding: 15px;
  background-color: #f5f7fa;
  border-radius: 4px;
}

.log-table {
  background-color: white;
  border-radius: 4px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.log-details {
  padding: 10px;
}
</style>