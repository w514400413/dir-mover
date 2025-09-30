<template>
  <div class="scan-progress-component">
    <!-- 主要进度条 -->
    <div class="progress-main">
      <el-progress
        :percentage="displayProgress"
        :status="progressStatus"
        :stroke-width="strokeWidth"
        :color="progressColor"
        :show-text="false"
      />
      
      <div class="progress-text-overlay">
        <span class="progress-percentage">{{ Math.round(displayProgress) }}%</span>
        <span class="progress-status">{{ statusText }}</span>
      </div>
    </div>

    <!-- 详细进度信息 -->
    <div class="progress-details">
      <!-- 当前路径 -->
      <div class="detail-item">
        <el-icon><Location /></el-icon>
        <span class="detail-label">当前路径:</span>
        <span class="detail-value" :title="currentPath">{{ truncatedPath }}</span>
      </div>

      <!-- 扫描统计 -->
      <div class="detail-stats">
        <div class="stat-item">
          <el-icon><Document /></el-icon>
          <span class="stat-label">已发现:</span>
          <span class="stat-value">{{ formatNumber(itemsFound) }} 个项目</span>
        </div>
        
        <div class="stat-item">
          <el-icon><CircleCheck /></el-icon>
          <span class="stat-label">大项目:</span>
          <span class="stat-value">{{ formatNumber(largeItemsFound) }} 个</span>
        </div>
        
        <div class="stat-item">
          <el-icon><Clock /></el-icon>
          <span class="stat-label">已用时:</span>
          <span class="stat-value">{{ elapsedTime }}</span>
        </div>
      </div>

      <!-- 预计剩余时间 -->
      <div v-if="estimatedTimeRemaining > 0" class="time-estimate">
        <el-icon><Timer /></el-icon>
        <span class="time-label">预计剩余:</span>
        <span class="time-value">{{ formatTime(estimatedTimeRemaining) }}</span>
      </div>

      <!-- 扫描速度 -->
      <div v-if="scanSpeed > 0" class="scan-speed">
        <el-icon><WindPower /></el-icon>
        <span class="speed-label">扫描速度:</span>
        <span class="speed-value">{{ formatNumber(scanSpeed) }} 项目/秒</span>
      </div>
    </div>

    <!-- 进度阶段指示器 -->
    <div class="progress-stages">
      <div 
        v-for="(stage, index) in scanStages" 
        :key="stage.key"
        class="stage-item"
        :class="{ 
          'stage-active': currentStage === stage.key,
          'stage-completed': isStageCompleted(stage.key),
          'stage-pending': isStagePending(stage.key)
        }"
      >
        <div class="stage-icon">
          <el-icon v-if="isStageCompleted(stage.key)"><CircleCheck /></el-icon>
          <el-icon v-else-if="currentStage === stage.key"><Loading /></el-icon>
          <el-icon v-else><MoreFilled /></el-icon>
        </div>
        <span class="stage-label">{{ stage.label }}</span>
        <div v-if="stage.count > 0" class="stage-count">{{ formatNumber(stage.count) }}</div>
      </div>
    </div>

    <!-- 实时事件流 -->
    <div v-if="showEventStream && recentEvents.length > 0" class="event-stream">
      <div class="event-header">
        <el-icon><Bell /></el-icon>
        <span class="event-title">扫描事件</span>
        <el-button size="small" text @click="clearEvents">清除</el-button>
      </div>
      
      <el-scrollbar max-height="150px">
        <div 
          v-for="event in recentEvents" 
          :key="event.id"
          class="event-item"
          :class="`event-${event.type}`"
        >
          <el-icon :class="getEventIcon(event.type)">
            <component :is="getEventIconComponent(event.type)" />
          </el-icon>
          <div class="event-content">
            <div class="event-message">{{ event.message }}</div>
            <div class="event-time">{{ formatEventTime(event.timestamp) }}</div>
          </div>
        </div>
      </el-scrollbar>
    </div>

    <!-- 性能指标 -->
    <div v-if="showPerformance && performanceMetrics" class="performance-metrics">
      <div class="metric-item">
        <span class="metric-label">CPU使用率</span>
        <el-progress 
          :percentage="performanceMetrics.cpuUsage" 
          :stroke-width="4"
          :color="getMetricColor(performanceMetrics.cpuUsage, 80)"
        />
      </div>
      
      <div class="metric-item">
        <span class="metric-label">内存使用</span>
        <el-progress 
          :percentage="performanceMetrics.memoryUsage" 
          :stroke-width="4"
          :color="getMetricColor(performanceMetrics.memoryUsage, 90)"
        />
      </div>
      
      <div class="metric-item">
        <span class="metric-label">缓存命中率</span>
        <span class="metric-value">{{ performanceMetrics.cacheHitRate.toFixed(1) }}%</span>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div v-if="showControls" class="progress-controls">
      <el-button 
        v-if="scanState === 'scanning'"
        size="small" 
        @click="$emit('pause')"
        :icon="VideoPause"
      >
        暂停
      </el-button>
      
      <el-button 
        v-if="scanState === 'paused'"
        size="small" 
        type="primary"
        @click="$emit('resume')"
        :icon="VideoPlay"
      >
        继续
      </el-button>
      
      <el-button 
        v-if="scanState === 'scanning' || scanState === 'paused'"
        size="small" 
        type="danger"
        @click="$emit('stop')"
        :icon="CircleClose"
      >
        停止
      </el-button>
      
      <el-button 
        v-if="scanState === 'completed'"
        size="small" 
        type="success"
        @click="$emit('reset')"
        :icon="RefreshRight"
      >
        重置
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import {
  Location,
  Document,
  CircleCheck,
  Clock,
  Timer,
  WindPower,
  MoreFilled,
  Loading,
  Bell,
  VideoPause,
  VideoPlay,
  CircleClose,
  RefreshRight
} from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';

/**
 * 扫描状态类型
 */
type ScanState = 'idle' | 'scanning' | 'paused' | 'completed' | 'error';

/**
 * 扫描阶段接口
 */
interface ScanStage {
  key: string;
  label: string;
  count: number;
  order: number;
}

/**
 * 扫描事件接口
 */
interface ScanEvent {
  id: string;
  type: 'info' | 'warning' | 'error' | 'success';
  message: string;
  timestamp: number;
}

/**
 * 性能指标接口
 */
interface PerformanceMetrics {
  cpuUsage: number;
  memoryUsage: number;
  cacheHitRate: number;
}

// Props 定义
interface Props {
  // 基本进度信息
  progress: number;
  currentPath: string;
  itemsFound: number;
  largeItemsFound: number;
  estimatedTimeRemaining?: number;
  scanSpeed?: number;
  
  // 状态控制
  scanState: ScanState;
  currentStage?: string;
  
  // 扫描阶段
  scanStages?: ScanStage[];
  
  // 性能指标
  performanceMetrics?: PerformanceMetrics;
  
  // 显示控制
  showEventStream?: boolean;
  showPerformance?: boolean;
  showControls?: boolean;
  showTimeEstimate?: boolean;
  
  // 样式配置
  strokeWidth?: number;
  compactMode?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  progress: 0,
  currentPath: '',
  itemsFound: 0,
  largeItemsFound: 0,
  estimatedTimeRemaining: 0,
  scanSpeed: 0,
  scanState: 'idle',
  currentStage: '',
  scanStages: () => [],
  performanceMetrics: undefined,
  showEventStream: true,
  showPerformance: false,
  showControls: true,
  showTimeEstimate: true,
  strokeWidth: 20,
  compactMode: false
});

// Emits 定义
const emit = defineEmits<{
  pause: [];
  resume: [];
  stop: [];
  reset: [];
}>();

// 状态变量
const startTime = ref<number>(0);
const elapsedTime = ref<string>('0:00');
const recentEvents = ref<ScanEvent[]>([]);
const eventCounter = ref<number>(0);

// 计算属性
const displayProgress = computed(() => {
  return Math.min(Math.max(props.progress, 0), 100);
});

const progressStatus = computed(() => {
  switch (props.scanState) {
    case 'scanning': return '';
    case 'paused': return 'warning';
    case 'completed': return 'success';
    case 'error': return 'exception';
    default: return '';
  }
});

const progressColor = computed(() => {
  if (props.scanState === 'error') return '#f56c6c';
  if (props.scanState === 'paused') return '#e6a23c';
  return '';
});

const statusText = computed(() => {
  switch (props.scanState) {
    case 'scanning': return '扫描中...';
    case 'paused': return '已暂停';
    case 'completed': return '扫描完成';
    case 'error': return '扫描失败';
    default: return '准备就绪';
  }
});

const truncatedPath = computed(() => {
  const maxLength = 50;
  if (props.currentPath.length <= maxLength) return props.currentPath;
  return '...' + props.currentPath.slice(-maxLength + 3);
});

// 方法：格式化数字
function formatNumber(num: number): string {
  if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
  if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
  return num.toString();
}

// 方法：格式化时间
function formatTime(seconds: number): string {
  if (seconds < 60) return `${Math.round(seconds)}秒`;
  
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.round(seconds % 60);
  
  if (minutes < 60) {
    return `${minutes}分${remainingSeconds}秒`;
  }
  
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  return `${hours}小时${remainingMinutes}分`;
}

// 方法：格式化事件时间
function formatEventTime(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp;
  
  if (diff < 1000) return '刚刚';
  if (diff < 60000) return `${Math.floor(diff / 1000)}秒前`;
  if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`;
  
  return new Date(timestamp).toLocaleTimeString();
}

// 方法：获取事件图标
function getEventIcon(type: string): string {
  switch (type) {
    case 'info': return 'el-icon-info';
    case 'warning': return 'el-icon-warning';
    case 'error': return 'el-icon-circle-close';
    case 'success': return 'el-icon-circle-check';
    default: return 'el-icon-info';
  }
}

// 方法：获取事件图标组件
function getEventIconComponent(type: string) {
  switch (type) {
    case 'info': return 'InfoFilled';
    case 'warning': return 'WarningFilled';
    case 'error': return 'CircleCloseFilled';
    case 'success': return 'CircleCheckFilled';
    default: return 'InfoFilled';
  }
}

// 方法：获取指标颜色
function getMetricColor(value: number, threshold: number): string {
  if (value >= threshold) return '#f56c6c';
  if (value >= threshold * 0.8) return '#e6a23c';
  return '#67c23a';
}

// 方法：检查阶段状态
function isStageCompleted(stageKey: string): boolean {
  if (!props.scanStages.length) return false;
  
  const currentStageIndex = props.scanStages.findIndex(s => s.key === props.currentStage);
  const stageIndex = props.scanStages.findIndex(s => s.key === stageKey);
  
  return stageIndex < currentStageIndex;
}

function isStagePending(stageKey: string): boolean {
  if (!props.scanStages.length) return false;
  
  const currentStageIndex = props.scanStages.findIndex(s => s.key === props.currentStage);
  const stageIndex = props.scanStages.findIndex(s => s.key === stageKey);
  
  return stageIndex > currentStageIndex;
}

// 方法：添加扫描事件
function addScanEvent(type: 'info' | 'warning' | 'error' | 'success', message: string) {
  const event: ScanEvent = {
    id: `event_${++eventCounter.value}`,
    type,
    message,
    timestamp: Date.now()
  };
  
  recentEvents.value.unshift(event);
  
  // 保持事件数量在限制范围内
  if (recentEvents.value.length > 20) {
    recentEvents.value = recentEvents.value.slice(0, 20);
  }
}

// 方法：清除事件
function clearEvents() {
  recentEvents.value = [];
}

// 方法：更新运行时间
function updateElapsedTime() {
  if (startTime.value === 0) return;
  
  const now = Date.now();
  const elapsed = Math.floor((now - startTime.value) / 1000);
  elapsedTime.value = formatTime(elapsed);
}

// 生命周期
onMounted(() => {
  startTime.value = Date.now();
  
  // 添加初始事件
  addScanEvent('info', '扫描准备就绪');
  
  // 定时更新运行时间
  const timer = setInterval(updateElapsedTime, 1000);
  
  onUnmounted(() => {
    clearInterval(timer);
  });
});

// 监听扫描状态变化
watch(() => props.scanState, (newState, oldState) => {
  if (newState !== oldState) {
    let message = '';
    let type: 'info' | 'warning' | 'error' | 'success' = 'info';
    
    switch (newState) {
      case 'scanning':
        message = '扫描开始';
        type = 'info';
        if (startTime.value === 0) {
          startTime.value = Date.now();
        }
        break;
      case 'paused':
        message = '扫描已暂停';
        type = 'warning';
        break;
      case 'completed':
        message = '扫描完成';
        type = 'success';
        break;
      case 'error':
        message = '扫描出错';
        type = 'error';
        break;
    }
    
    if (message) {
      addScanEvent(type, message);
    }
  }
});

// 监听进度变化，添加相应事件
watch(() => props.progress, (newProgress, oldProgress) => {
  // 在重要进度节点添加事件
  const milestones = [25, 50, 75, 90];
  for (const milestone of milestones) {
    if (oldProgress < milestone && newProgress >= milestone) {
      addScanEvent('info', `扫描进度达到 ${milestone}%`);
      break;
    }
  }
});
</script>

<style scoped>
.scan-progress-component {
  padding: 20px;
  background: #ffffff;
  border-radius: 8px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
}

.progress-main {
  position: relative;
  margin-bottom: 20px;
}

.progress-text-overlay {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  pointer-events: none;
}

.progress-percentage {
  display: block;
  font-size: 24px;
  font-weight: bold;
  color: #303133;
  line-height: 1;
}

.progress-status {
  display: block;
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

.progress-details {
  margin-bottom: 20px;
}

.detail-item {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  font-size: 14px;
}

.detail-label {
  color: #606266;
  font-weight: 500;
  min-width: 70px;
}

.detail-value {
  color: #303133;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.detail-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 12px;
  margin-bottom: 12px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px;
  background: #f5f7fa;
  border-radius: 4px;
  font-size: 13px;
}

.stat-label {
  color: #606266;
}

.stat-value {
  color: #303133;
  font-weight: 500;
}

.time-estimate,
.scan-speed {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  font-size: 13px;
}

.time-label,
.speed-label {
  color: #606266;
}

.time-value,
.speed-value {
  color: #409eff;
  font-weight: 500;
}

.progress-stages {
  display: flex;
  justify-content: space-between;
  margin-bottom: 20px;
  padding: 16px;
  background: #f5f7fa;
  border-radius: 6px;
}

.stage-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  flex: 1;
  position: relative;
}

.stage-item:not(:last-child)::after {
  content: '';
  position: absolute;
  top: 20px;
  right: -50%;
  width: 100%;
  height: 2px;
  background: #e4e7ed;
  z-index: 1;
}

.stage-item.stage-completed::after {
  background: #67c23a;
}

.stage-icon {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f5f7fa;
  border: 2px solid #e4e7ed;
  z-index: 2;
  position: relative;
}

.stage-item.stage-completed .stage-icon {
  background: #67c23a;
  border-color: #67c23a;
  color: white;
}

.stage-item.stage-active .stage-icon {
  background: #409eff;
  border-color: #409eff;
  color: white;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { transform: scale(1); }
  50% { transform: scale(1.05); }
  100% { transform: scale(1); }
}

.stage-label {
  font-size: 12px;
  color: #606266;
  text-align: center;
}

.stage-item.stage-active .stage-label {
  color: #409eff;
  font-weight: 500;
}

.stage-count {
  font-size: 11px;
  color: #909399;
  background: #ffffff;
  padding: 2px 6px;
  border-radius: 10px;
  margin-top: 2px;
}

.event-stream {
  margin-bottom: 16px;
}

.event-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  padding-bottom: 8px;
  border-bottom: 1px solid #e4e7ed;
}

.event-title {
  flex: 1;
  font-size: 14px;
  font-weight: 500;
  color: #303133;
}

.event-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 8px 0;
  border-bottom: 1px solid #f0f2f5;
  font-size: 12px;
}

.event-item:last-child {
  border-bottom: none;
}

.event-item.event-info {
  color: #409eff;
}

.event-item.event-warning {
  color: #e6a23c;
}

.event-item.event-error {
  color: #f56c6c;
}

.event-item.event-success {
  color: #67c23a;
}

.event-content {
  flex: 1;
}

.event-message {
  margin-bottom: 2px;
  word-break: break-word;
}

.event-time {
  font-size: 11px;
  color: #909399;
}

.performance-metrics {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
  margin-bottom: 16px;
}

.metric-item {
  padding: 12px;
  background: #f5f7fa;
  border-radius: 4px;
}

.metric-label {
  display: block;
  font-size: 12px;
  color: #606266;
  margin-bottom: 6px;
}

.metric-value {
  font-size: 18px;
  font-weight: bold;
  color: #303133;
}

.progress-controls {
  display: flex;
  gap: 8px;
  justify-content: center;
}

/* 紧凑模式 */
.compact-mode .progress-details {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.compact-mode .detail-item {
  margin-bottom: 4px;
}

.compact-mode .detail-stats {
  grid-template-columns: 1fr 1fr;
  gap: 4px;
}

.compact-mode .stat-item {
  padding: 4px;
}

.compact-mode .progress-stages {
  padding: 8px;
}

.compact-mode .stage-icon {
  width: 30px;
  height: 30px;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .progress-stages {
    flex-direction: column;
    gap: 8px;
  }
  
  .stage-item:not(:last-child)::after {
    display: none;
  }
  
  .detail-stats {
    grid-template-columns: 1fr;
  }
  
  .performance-metrics {
    grid-template-columns: 1fr;
  }
}
</style>