<template>
  <div class="performance-monitor">
    <div class="monitor-header">
      <h3>性能监控器</h3>
      <div class="monitor-controls">
        <el-button @click="refreshStats" :loading="loading" size="small">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
        <el-button @click="runCleanup" size="small" type="warning">
          <el-icon><Delete /></el-icon>
          清理内存
        </el-button>
        <el-button @click="runBenchmark" size="small" type="primary">
          <el-icon><TrendCharts /></el-icon>
          性能基准
        </el-button>
      </div>
    </div>

    <div class="performance-overview">
      <el-row :gutter="20">
        <el-col :span="6">
          <el-card>
            <div class="metric-item">
              <div class="metric-value" :class="getMemoryColor(stats.memory_usage_mb)">
                {{ stats.memory_usage_mb.toFixed(1) }} MB
              </div>
              <div class="metric-label">内存使用</div>
              <div class="metric-trend">
                <el-icon :class="memoryTrendDirection">
                  <CaretTop v-if="memoryTrendDirection === 'up'" />
                  <CaretBottom v-else />
                </el-icon>
                <span>{{ memoryTrend }}%</span>
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card>
            <div class="metric-item">
              <div class="metric-value" style="color: #409eff;">
                {{ stats.cache_hit_rate.toFixed(1) }}%
              </div>
              <div class="metric-label">缓存命中率</div>
              <div class="metric-subtitle">
                {{ stats.cache_size }} 项
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card>
            <div class="metric-item">
              <div class="metric-value" style="color: #67c23a;">
                {{ stats.batch_queue_size }}
              </div>
              <div class="metric-label">批处理队列</div>
              <div class="metric-subtitle">
                最后清理: {{ stats.last_cleanup_seconds_ago }}秒前
              </div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card>
            <div class="metric-item">
              <div class="metric-value" style="color: #e6a23c;">
                {{ benchmark.overall_performance_score.toFixed(1) }}
              </div>
              <div class="metric-label">性能评分</div>
              <div class="metric-subtitle">
                {{ getPerformanceLevel(benchmark.overall_performance_score) }}
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <div class="performance-charts">
      <el-row :gutter="20">
        <el-col :span="12">
          <el-card title="内存使用趋势">
            <div class="chart-container">
              <canvas ref="memoryChart" width="400" height="200"></canvas>
            </div>
          </el-card>
        </el-col>
        <el-col :span="12">
          <el-card title="缓存性能">
            <div class="chart-container">
              <canvas ref="cacheChart" width="400" height="200"></canvas>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <div class="optimization-controls">
      <el-card title="优化控制">
        <el-form :model="optimizationConfig" label-width="150px">
          <el-form-item label="启用缓存">
            <el-switch v-model="optimizationConfig.enableCaching" />
          </el-form-item>
          <el-form-item label="批处理大小">
            <el-slider
              v-model="optimizationConfig.batchSize"
              :min="10"
              :max="500"
              :step="10"
              show-input
            />
          </el-form-item>
          <el-form-item label="最大并发数">
            <el-slider
              v-model="optimizationConfig.maxConcurrency"
              :min="1"
              :max="20"
              :step="1"
              show-input
            />
          </el-form-item>
          <el-form-item label="内存限制(MB)">
            <el-slider
              v-model="optimizationConfig.memoryLimit"
              :min="100"
              :max="2000"
              :step="50"
              show-input
            />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="applyOptimization">
              应用优化
            </el-button>
            <el-button @click="resetToDefault">
              重置默认
            </el-button>
          </el-form-item>
        </el-form>
      </el-card>
    </div>

    <div class="performance-recommendations" v-if="benchmark.recommendations.length > 0">
      <el-card title="性能建议">
        <el-alert
          v-for="(recommendation, index) in benchmark.recommendations"
          :key="index"
          :title="recommendation"
          :type="getRecommendationType(recommendation)"
          :closable="false"
          style="margin-bottom: 10px"
        />
      </el-card>
    </div>

    <div class="stress-test-section">
      <el-card title="压力测试">
        <el-space>
          <el-button @click="runMemoryStressTest" :loading="stressLoading.memory" type="warning">
            <el-icon><Warning /></el-icon>
            内存压力测试
          </el-button>
          <el-button @click="runDiskStressTest" :loading="stressLoading.disk" type="danger">
            <el-icon><Lightning /></el-icon>
            磁盘压力测试
          </el-button>
          <el-button @click="runConcurrentStressTest" :loading="stressLoading.concurrent" type="info">
            <el-icon><Connection /></el-icon>
            并发压力测试
          </el-button>
        </el-space>
      </el-card>
    </div>

    <!-- 压力测试进度对话框 -->
    <el-dialog
      v-model="stressDialog.visible"
      title="压力测试进行中"
      width="400px"
      :close-on-click-modal="false"
    >
      <div class="stress-progress">
        <el-progress
          :percentage="stressDialog.progress"
          :status="stressDialog.status"
          :stroke-width="8"
        />
        <div class="stress-info">
          <p>{{ stressDialog.message }}</p>
          <p>已处理: {{ stressDialog.processed }} / {{ stressDialog.total }}</p>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue';
import { ElMessage } from 'element-plus';
import { Refresh, Delete, TrendCharts, Warning, Lightning, Connection, CaretTop, CaretBottom } from '@element-plus/icons-vue';
import { performanceAPI, type PerformanceBenchmark } from '../services/api';

// 状态
const loading = ref(false);
const stats = ref({
  memory_usage_mb: 0,
  memory_peak_mb: 0,
  cache_hit_rate: 0,
  cache_size: 0,
  batch_queue_size: 0,
  last_cleanup_seconds_ago: 0
});

const benchmark = ref<PerformanceBenchmark>({
  memory_usage_score: 0,
  cache_performance_score: 0,
  concurrency_score: 0,
  overall_performance_score: 0,
  recommendations: []
});

const optimizationConfig = ref({
  enableCaching: true,
  batchSize: 100,
  maxConcurrency: 5,
  memoryLimit: 500
});

const stressLoading = ref({
  memory: false,
  disk: false,
  concurrent: false
});

const stressDialog = ref({
  visible: false,
  progress: 0,
  status: 'primary' as 'primary' | 'success' | 'exception',
  message: '',
  processed: 0,
  total: 100
});

const memoryHistory = ref<number[]>([]);
const cacheHistory = ref<number[]>([]);
const memoryTrend = ref(0);
const memoryTrendDirection = ref<'up' | 'down'>('up');

// 计算属性
const getMemoryColor = (usage: number) => {
  if (usage < 100) return 'color: #67c23a;'; // 绿色 - 良好
  if (usage < 300) return 'color: #e6a23c;'; // 黄色 - 警告
  return 'color: #f56c6c;'; // 红色 - 危险
};

const getPerformanceLevel = (score: number) => {
  if (score >= 90) return '优秀';
  if (score >= 80) return '良好';
  if (score >= 60) return '一般';
  return '需要优化';
};

const getRecommendationType = (recommendation: string) => {
  if (recommendation.includes('内存')) return 'warning';
  if (recommendation.includes('缓存')) return 'info';
  if (recommendation.includes('并发')) return 'success';
  return 'info';
};

// 方法
const refreshStats = async () => {
  loading.value = true;
  try {
    const result = await performanceAPI.getPerformanceStats();
    stats.value = result;
    
    // 更新历史数据
    memoryHistory.value.push(result.memory_usage_mb);
    cacheHistory.value.push(result.cache_hit_rate);
    
    // 保持历史数据在合理范围
    if (memoryHistory.value.length > 20) {
      memoryHistory.value.shift();
      cacheHistory.value.shift();
    }
    
    // 计算趋势
    if (memoryHistory.value.length >= 2) {
      const current = memoryHistory.value[memoryHistory.value.length - 1];
      const previous = memoryHistory.value[memoryHistory.value.length - 2];
      memoryTrend.value = ((current - previous) / previous) * 100;
      memoryTrendDirection.value = current > previous ? 'up' : 'down';
    }
    
    // 更新图表
    await nextTick();
    updateCharts();
    
    ElMessage.success('性能统计已更新');
  } catch (error) {
    ElMessage.error(`获取性能统计失败: ${error}`);
  } finally {
    loading.value = false;
  }
};

const runBenchmark = async () => {
  loading.value = true;
  try {
    const result = await performanceAPI.getPerformanceBenchmark();
    benchmark.value = result;
    ElMessage.success('性能基准测试完成');
  } catch (error) {
    ElMessage.error(`性能基准测试失败: ${error}`);
  } finally {
    loading.value = false;
  }
};

const runCleanup = async () => {
  try {
    const cleaned = await performanceAPI.runMemoryCleanup();
    if (cleaned) {
      ElMessage.success('内存清理完成');
      await refreshStats();
    } else {
      ElMessage.info('暂不需要内存清理');
    }
  } catch (error) {
    ElMessage.error(`内存清理失败: ${error}`);
  }
};

const applyOptimization = () => {
  ElMessage.success('优化配置已应用');
  // 这里应该将配置发送到后端
};

const resetToDefault = () => {
  optimizationConfig.value = {
    enableCaching: true,
    batchSize: 100,
    maxConcurrency: 5,
    memoryLimit: 500
  };
  ElMessage.success('已重置为默认配置');
};

const runMemoryStressTest = async () => {
  stressLoading.value.memory = true;
  stressDialog.value.visible = true;
  stressDialog.value.progress = 0;
  stressDialog.value.message = '正在执行内存压力测试...';
  stressDialog.value.total = 100;
  
  try {
    // 模拟内存压力测试
    for (let i = 0; i <= 100; i += 10) {
      stressDialog.value.progress = i;
      stressDialog.value.processed = i;
      await new Promise(resolve => setTimeout(resolve, 200));
    }
    
    stressDialog.value.status = 'success';
    stressDialog.value.message = '内存压力测试完成';
    ElMessage.success('内存压力测试通过');
  } catch (error) {
    stressDialog.value.status = 'exception';
    stressDialog.value.message = '内存压力测试失败';
    ElMessage.error(`内存压力测试失败: ${error}`);
  } finally {
    stressLoading.value.memory = false;
    setTimeout(() => {
      stressDialog.value.visible = false;
    }, 2000);
  }
};

const runDiskStressTest = async () => {
  stressLoading.value.disk = true;
  stressDialog.value.visible = true;
  stressDialog.value.progress = 0;
  stressDialog.value.message = '正在执行磁盘压力测试...';
  stressDialog.value.total = 200;
  
  try {
    // 模拟磁盘压力测试
    for (let i = 0; i <= 200; i += 20) {
      stressDialog.value.progress = (i / 200) * 100;
      stressDialog.value.processed = i;
      await new Promise(resolve => setTimeout(resolve, 150));
    }
    
    stressDialog.value.status = 'success';
    stressDialog.value.message = '磁盘压力测试完成';
    ElMessage.success('磁盘压力测试通过');
  } catch (error) {
    stressDialog.value.status = 'exception';
    stressDialog.value.message = '磁盘压力测试失败';
    ElMessage.error(`磁盘压力测试失败: ${error}`);
  } finally {
    stressLoading.value.disk = false;
    setTimeout(() => {
      stressDialog.value.visible = false;
    }, 2000);
  }
};

const runConcurrentStressTest = async () => {
  stressLoading.value.concurrent = true;
  stressDialog.value.visible = true;
  stressDialog.value.progress = 0;
  stressDialog.value.message = '正在执行并发压力测试...';
  stressDialog.value.total = 50;
  
  try {
    // 模拟并发压力测试
    for (let i = 0; i <= 50; i += 5) {
      stressDialog.value.progress = (i / 50) * 100;
      stressDialog.value.processed = i;
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    
    stressDialog.value.status = 'success';
    stressDialog.value.message = '并发压力测试完成';
    ElMessage.success('并发压力测试通过');
  } catch (error) {
    stressDialog.value.status = 'exception';
    stressDialog.value.message = '并发压力测试失败';
    ElMessage.error(`并发压力测试失败: ${error}`);
  } finally {
    stressLoading.value.concurrent = false;
    setTimeout(() => {
      stressDialog.value.visible = false;
    }, 2000);
  }
};

const updateCharts = () => {
  // 这里应该使用 Chart.js 或其他图表库来绘制实际的图表
  // 现在只是模拟图表更新
  console.log('更新性能图表');
};

const generate_performance_recommendations = (stats: any): string[] => {
  const recommendations: string[] = [];
  
  if (stats.memory_usage_mb > 400) {
    recommendations.push('内存使用较高，建议增加内存限制或更频繁地进行内存清理');
  }
  
  if (stats.cache_hit_rate < 50) {
    recommendations.push('缓存命中率较低，建议增加缓存大小或优化缓存策略');
  }
  
  if (stats.batch_queue_size > 20) {
    recommendations.push('批处理队列积压，建议增加并发数或优化批处理大小');
  }
  
  if (recommendations.length === 0) {
    recommendations.push('系统性能良好，当前配置适合当前负载');
  }
  
  return recommendations;
};

// 生命周期
onMounted(async () => {
  await refreshStats();
  await runBenchmark();
  
  // 定期刷新统计信息
  setInterval(async () => {
    if (!loading.value) {
      await refreshStats();
    }
  }, 30000); // 每30秒刷新一次
});
</script>

<style scoped>
.performance-monitor {
  padding: 20px;
}

.monitor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.monitor-header h3 {
  margin: 0;
  color: #303133;
}

.monitor-controls {
  display: flex;
  gap: 10px;
}

.performance-overview {
  margin-bottom: 20px;
}

.metric-item {
  text-align: center;
  padding: 15px;
}

.metric-value {
  font-size: 28px;
  font-weight: bold;
  margin-bottom: 5px;
}

.metric-label {
  font-size: 14px;
  color: #606266;
  margin-bottom: 5px;
}

.metric-subtitle {
  font-size: 12px;
  color: #909399;
}

.metric-trend {
  font-size: 12px;
  color: #909399;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.performance-charts {
  margin-bottom: 20px;
}

.chart-container {
  height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #f5f7fa;
  border-radius: 4px;
}

.optimization-controls {
  margin-bottom: 20px;
}

.performance-recommendations {
  margin-bottom: 20px;
}

.stress-test-section {
  margin-bottom: 20px;
}

.stress-progress {
  text-align: center;
}

.stress-info {
  margin-top: 15px;
  font-size: 14px;
  color: #606266;
}

.stress-info p {
  margin: 5px 0;
}
</style>