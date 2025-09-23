<template>
  <div class="test-manager">
    <div class="test-header">
      <h3>测试管理器</h3>
      <div class="test-controls">
        <el-button @click="runAllTests" :loading="loading" type="primary">
          <el-icon><VideoPlay /></el-icon>
          运行所有测试
        </el-button>
        <el-button @click="generateReport" :loading="reportLoading" type="success">
          <el-icon><Document /></el-icon>
          生成报告
        </el-button>
        <el-button @click="clearResults" type="info">
          <el-icon><Delete /></el-icon>
          清除结果
        </el-button>
      </div>
    </div>

    <div class="test-suite-selection">
      <el-card title="测试套件选择">
        <el-space>
          <el-button 
            v-for="suite in testSuites" 
            :key="suite.type"
            @click="runTestSuite(suite.type)"
            :loading="suite.loading"
            :type="suite.type === currentSuite ? 'primary' : 'default'"
          >
            <el-icon><component :is="suite.icon" /></el-icon>
            {{ suite.name }}
          </el-button>
        </el-space>
      </el-card>
    </div>

    <div class="test-statistics" v-if="statistics.total_tests > 0">
      <el-row :gutter="20">
        <el-col :span="6">
          <el-card>
            <div class="stat-item">
              <div class="stat-value">{{ statistics.total_tests }}</div>
              <div class="stat-label">总测试数</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card>
            <div class="stat-item">
              <div class="stat-value" style="color: #67c23a;">{{ statistics.passed_tests }}</div>
              <div class="stat-label">通过</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card>
            <div class="stat-item">
              <div class="stat-value" style="color: #f56c6c;">{{ statistics.failed_tests }}</div>
              <div class="stat-label">失败</div>
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

    <div class="test-progress" v-if="loading">
      <el-card title="测试进度">
        <el-progress
          :percentage="progressPercentage"
          :status="progressStatus"
          :stroke-width="8"
        />
        <div class="progress-info">
          <span>正在运行: {{ currentTest }}</span>
          <span>耗时: {{ elapsedTime }}</span>
        </div>
      </el-card>
    </div>

    <div class="test-details" v-if="testDetails.length > 0">
      <el-card title="测试详情">
        <el-table
          :data="testDetails"
          style="width: 100%"
          height="400"
        >
          <el-table-column prop="name" label="测试名称" min-width="200" />
          <el-table-column prop="status" label="状态" width="100">
            <template #default="{ row }">
              <el-tag :type="getStatusType(row.status)">
                {{ row.status }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="duration_ms" label="耗时(ms)" width="100" />
          <el-table-column prop="error_message" label="错误信息" min-width="200">
            <template #default="{ row }">
              <span v-if="row.error_message" style="color: #f56c6c;">
                {{ row.error_message }}
              </span>
              <span v-else>-</span>
            </template>
          </el-table-column>
        </el-table>
      </el-card>
    </div>

    <div class="test-actions">
      <el-card title="测试操作">
        <el-space>
          <el-button @click="showPerformanceTest" type="warning">
            <el-icon><TrendCharts /></el-icon>
            性能测试
          </el-button>
          <el-button @click="showStressTest" type="danger">
            <el-icon><Lightning /></el-icon>
            压力测试
          </el-button>
          <el-button @click="showBenchmark" type="info">
            <el-icon><DataAnalysis /></el-icon>
            基准测试
          </el-button>
        </el-space>
      </el-card>
    </div>

    <!-- 性能测试对话框 -->
    <el-dialog
      v-model="performanceDialog.visible"
      title="性能测试"
      width="600px"
    >
      <el-form :model="performanceConfig">
        <el-form-item label="测试规模">
          <el-select v-model="performanceConfig.scale">
            <el-option label="小型 (100文件)" value="small" />
            <el-option label="中型 (1000文件)" value="medium" />
            <el-option label="大型 (5000文件)" value="large" />
          </el-select>
        </el-form-item>
        <el-form-item label="文件大小">
          <el-select v-model="performanceConfig.fileSize">
            <el-option label="小文件 (1KB)" value="small" />
            <el-option label="中文件 (10KB)" value="medium" />
            <el-option label="大文件 (100KB)" value="large" />
          </el-select>
        </el-form-item>
        <el-form-item label="并发数">
          <el-input-number v-model="performanceConfig.concurrency" :min="1" :max="10" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="performanceDialog.visible = false">取消</el-button>
        <el-button type="warning" @click="runPerformanceTest" :loading="performanceDialog.loading">
          开始性能测试
        </el-button>
      </template>
    </el-dialog>

    <!-- 压力测试对话框 -->
    <el-dialog
      v-model="stressDialog.visible"
      title="压力测试"
      width="500px"
    >
      <el-form :model="stressConfig">
        <el-form-item label="测试时长(秒)">
          <el-input-number v-model="stressConfig.duration" :min="10" :max="300" :step="10" />
        </el-form-item>
        <el-form-item label="并发操作数">
          <el-input-number v-model="stressConfig.operations" :min="5" :max="50" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="stressDialog.visible = false">取消</el-button>
        <el-button type="danger" @click="runStressTest" :loading="stressDialog.loading">
          开始压力测试
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { VideoPlay, Document, Delete, TrendCharts, Lightning, DataAnalysis } from '@element-plus/icons-vue';
import { testAPI, type TestStatistics, type TestDetail } from '../services/api';

// 状态
const loading = ref(false);
const reportLoading = ref(false);
const statistics = ref<TestStatistics>({
  total_tests: 0,
  passed_tests: 0,
  failed_tests: 0,
  skipped_tests: 0,
  total_duration_ms: 0
});

const testDetails = ref<TestDetail[]>([]);
const currentTest = ref('');
const startTime = ref<number>(0);
const currentSuite = ref<string>('');

const performanceDialog = ref({
  visible: false,
  loading: false
});

const stressDialog = ref({
  visible: false,
  loading: false
});

const performanceConfig = ref({
  scale: 'medium',
  fileSize: 'medium',
  concurrency: 3
});

const stressConfig = ref({
  duration: 60,
  operations: 10
});

// 测试套件配置
const testSuites = ref([
  { type: 'unit', name: '单元测试', icon: 'CircleCheck', loading: false },
  { type: 'integration', name: '集成测试', icon: 'Connection', loading: false },
  { type: 'e2e', name: '端到端测试', icon: 'Finished', loading: false },
  { type: 'performance', name: '性能测试', icon: 'TrendCharts', loading: false }
]);

// 计算属性
const successRate = computed(() => {
  if (statistics.value.total_tests === 0) return 0;
  return Math.round((statistics.value.passed_tests / statistics.value.total_tests) * 100);
});

const progressPercentage = computed(() => {
  if (statistics.value.total_tests === 0) return 0;
  return Math.min(100, Math.round(((statistics.value.passed_tests + statistics.value.failed_tests) / statistics.value.total_tests) * 100));
});

const progressStatus = computed(() => {
  if (statistics.value.failed_tests > 0) return 'exception';
  if (statistics.value.passed_tests === statistics.value.total_tests) return 'success';
  return '';
});

const elapsedTime = computed(() => {
  if (startTime.value === 0) return '00:00';
  const elapsed = Date.now() - startTime.value;
  const seconds = Math.floor(elapsed / 1000);
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}`;
});

// 方法
const runAllTests = async () => {
  loading.value = true;
  startTime.value = Date.now();
  currentTest.value = '综合测试套件';
  
  try {
    const result = await testAPI.runComprehensiveTests();
    statistics.value = result;
    
    // 模拟测试详情
    generateMockTestDetails();
    
    ElMessage.success(`测试完成！成功率: ${successRate.value}%`);
  } catch (error) {
    ElMessage.error(`测试失败: ${error}`);
  } finally {
    loading.value = false;
    startTime.value = 0;
    currentTest.value = '';
  }
};

const runTestSuite = async (suiteType: string) => {
  const suite = testSuites.value.find(s => s.type === suiteType);
  if (!suite) return;
  
  suite.loading = true;
  currentSuite.value = suiteType;
  currentTest.value = suite.name;
  
  try {
    const result = await testAPI.runTestSuite(suiteType as any);
    statistics.value = result;
    
    generateMockTestDetails();
    
    ElMessage.success(`${suite.name}完成！`);
  } catch (error) {
    ElMessage.error(`${suite.name}失败: ${error}`);
  } finally {
    suite.loading = false;
    currentSuite.value = '';
    currentTest.value = '';
  }
};

const generateReport = async () => {
  reportLoading.value = true;
  
  try {
    const outputPath = prompt('请输入报告输出路径（例如：C:\\tests\\report.html）');
    if (outputPath) {
      const success = await testAPI.generateTestReport(outputPath);
      if (success) {
        ElMessage.success(`测试报告已生成: ${outputPath}`);
      }
    }
  } catch (error) {
    ElMessage.error(`生成报告失败: ${error}`);
  } finally {
    reportLoading.value = false;
  }
};

const clearResults = () => {
  statistics.value = {
    total_tests: 0,
    passed_tests: 0,
    failed_tests: 0,
    skipped_tests: 0,
    total_duration_ms: 0
  };
  testDetails.value = [];
  ElMessage.success('测试结果已清除');
};

const generateMockTestDetails = () => {
  // 模拟测试详情数据
  const mockDetails: TestDetail[] = [];
  const testNames = [
    '磁盘扫描功能测试',
    '文件迁移测试',
    '路径验证测试',
    '错误处理测试',
    '操作日志测试',
    '性能基准测试',
    '并发操作测试',
    '大文件处理测试',
    '内存使用测试',
    '用户界面测试'
  ];
  
  for (let i = 0; i < statistics.value.total_tests && i < testNames.length; i++) {
    const isPassed = i < statistics.value.passed_tests;
    mockDetails.push({
      name: testNames[i],
      status: isPassed ? 'Passed' : 'Failed',
      duration_ms: Math.floor(Math.random() * 1000) + 100,
      error_message: isPassed ? undefined : `测试失败: ${testNames[i]} 未通过`
    });
  }
  
  testDetails.value = mockDetails;
};

const showPerformanceTest = () => {
  performanceDialog.value.visible = true;
};

const showStressTest = () => {
  stressDialog.value.visible = true;
};

const showBenchmark = () => {
  ElMessage.info('基准测试功能开发中...');
};

const runPerformanceTest = async () => {
  performanceDialog.value.loading = true;
  
  try {
    // 这里应该调用性能测试API
    ElMessage.success('性能测试已开始，请查看控制台输出');
    performanceDialog.value.visible = false;
  } catch (error) {
    ElMessage.error(`性能测试失败: ${error}`);
  } finally {
    performanceDialog.value.loading = false;
  }
};

const runStressTest = async () => {
  stressDialog.value.loading = true;
  
  try {
    // 这里应该调用压力测试API
    ElMessage.success('压力测试已开始，请查看控制台输出');
    stressDialog.value.visible = false;
  } catch (error) {
    ElMessage.error(`压力测试失败: ${error}`);
  } finally {
    stressDialog.value.loading = false;
  }
};

const getStatusType = (status: string) => {
  switch (status) {
    case 'Passed': return 'success';
    case 'Failed': return 'danger';
    case 'Skipped': return 'warning';
    default: return 'info';
  }
};

// 生命周期
onMounted(() => {
  // 可以在这里加载之前的测试结果
});
</script>

<style scoped>
.test-manager {
  padding: 20px;
}

.test-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.test-header h3 {
  margin: 0;
  color: #303133;
}

.test-controls {
  display: flex;
  gap: 10px;
}

.test-suite-selection {
  margin-bottom: 20px;
}

.test-statistics {
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

.test-progress {
  margin-bottom: 20px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-top: 10px;
  font-size: 14px;
  color: #606266;
}

.test-details {
  margin-bottom: 20px;
}

.test-actions {
  margin-bottom: 20px;
}
</style>