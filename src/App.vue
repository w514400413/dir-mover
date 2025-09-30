<script setup lang="ts">
import { ref, onMounted } from "vue";
import { Refresh, InfoFilled, Check } from "@element-plus/icons-vue";
import DirectoryTree from "./components/DirectoryTree.vue";
import AppDataScanner from "./components/AppDataScanner.vue";

const activeTab = ref("disk");
const showWelcome = ref(true);

function refreshAll() {
  // 刷新所有数据
  console.log("刷新所有数据");
}

function handleTabChange(tabName: string) {
  activeTab.value = tabName;
  
  // UX-1: 首次使用体验优化
  if (tabName === 'appdata' && showWelcome.value) {
    showWelcome.value = false;
  }
}

function startAppDataAnalysis() {
  // 切换到AppData分析标签
  activeTab.value = 'appdata';
  showWelcome.value = false;
}

onMounted(() => {
  // UX-1: 自动检测是否首次使用，显示欢迎界面
  const hasSeenWelcome = localStorage.getItem('hasSeenAppDataWelcome');
  if (hasSeenWelcome) {
    showWelcome.value = false;
  }
  
  // UX-3: 键盘快捷键支持
  document.addEventListener('keydown', handleKeyboardShortcuts);
});

function handleKeyboardShortcuts(event: KeyboardEvent) {
  // Ctrl/Cmd + A: 快速切换到 AppData 分析
  if ((event.ctrlKey || event.metaKey) && event.key === 'a') {
    event.preventDefault();
    activeTab.value = 'appdata';
  }
  
  // Ctrl/Cmd + R: 刷新当前标签
  if ((event.ctrlKey || event.metaKey) && event.key === 'r') {
    event.preventDefault();
    refreshAll();
  }
  
  // F1: 显示帮助
  if (event.key === 'F1') {
    event.preventDefault();
    showWelcome.value = true;
  }
}

function closeWelcome() {
  showWelcome.value = false;
  localStorage.setItem('hasSeenAppDataWelcome', 'true');
}
</script>

<template>
  <div class="app-container">
    <!-- UX-1: 首次使用欢迎界面 -->
    <el-dialog
      v-model="showWelcome"
      title="欢迎使用 AppData 空间分析工具"
      width="600px"
      :close-on-click-modal="false"
      :close-on-press-escape="false"
      :show-close="false"
    >
      <div class="welcome-content">
        <div class="welcome-icon">
          <el-icon size="64" color="#409eff"><InfoFilled /></el-icon>
        </div>
        <h2>🎯 专注于 AppData 目录分析</h2>
        <p class="welcome-description">
          本工具专门用于分析 Windows 用户 AppData 目录的空间占用情况，帮助您快速识别占用空间最大的应用程序数据。
        </p>
        
        <div class="feature-highlights">
          <div class="feature-item">
            <el-icon><Check /></el-icon>
            <span>自动扫描 Local、LocalLow、Roaming 三个主要目录</span>
          </div>
          <div class="feature-item">
            <el-icon><Check /></el-icon>
            <span>智能识别大于 1GB 的大文件和文件夹</span>
          </div>
          <div class="feature-item">
            <el-icon><Check /></el-icon>
            <span>支持一键迁移到指定盘符，释放 C 盘空间</span>
          </div>
          <div class="feature-item">
            <el-icon><Check /></el-icon>
            <span>实时进度显示和动态排序</span>
          </div>
        </div>
        
        <div class="welcome-actions">
          <el-button type="primary" size="large" @click="startAppDataAnalysis">
            立即开始分析
          </el-button>
          <el-button @click="closeWelcome">稍后再说</el-button>
        </div>
      </div>
    </el-dialog>

    <el-header class="app-header" height="60px">
      <div class="header-content">
        <h1 class="app-title">C盘空间管理工具</h1>
        <div class="header-actions">
          <el-button type="primary" :icon="Refresh" @click="refreshAll">
            刷新全部
          </el-button>
          <!-- UX-3: 快速导航按钮 -->
          <el-button
            type="success"
            @click="startAppDataAnalysis"
            class="quick-appdata-btn"
          >
            快速分析 AppData
          </el-button>
        </div>
      </div>
    </el-header>

    <el-main class="app-main">
      <el-tabs v-model="activeTab" class="main-tabs" @tab-change="handleTabChange">
       <el-tab-pane label="磁盘分析" name="disk">
         <directory-tree />
       </el-tab-pane>
       <el-tab-pane label="AppData分析" name="appdata">
         <app-data-scanner />
       </el-tab-pane>
       <el-tab-pane label="系统信息" name="system">
         <div class="system-info">
           <h3>系统磁盘信息</h3>
           <p>磁盘信息功能开发中...</p>
         </div>
       </el-tab-pane>
       <el-tab-pane label="设置" name="settings">
         <div class="settings-panel">
           <h3>应用设置</h3>
           <p>设置功能开发中...</p>
         </div>
       </el-tab-pane>
     </el-tabs>
    </el-main>

    <el-footer class="app-footer">
      <div class="footer-content">
        <span>C盘空间管理工具 v1.0.0</span>
        <span>基于 Tauri + Vue + Element Plus</span>
        <!-- UX-3: 添加帮助链接 -->
        <el-link
          type="primary"
          @click="showWelcome = true"
          :underline="false"
        >
          使用帮助
        </el-link>
      </div>
    </el-footer>
  </div>
</template>

<style scoped>
.app-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.app-header {
  background-color: #409eff;
  color: white;
  padding: 0;
  height: 60px !important;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 100%;
  padding: 0 20px;
}

.app-title {
  margin: 0;
  font-size: 20px;
  font-weight: 500;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.quick-appdata-btn {
  background: linear-gradient(135deg, #67c23a 0%, #409eff 100%);
  border: none;
  font-weight: bold;
}

.app-main {
  flex: 1;
  padding: 0;
  background-color: #f5f7fa;
}

.main-tabs {
  height: 100%;
}

.main-tabs :deep(.el-tabs__content) {
  height: calc(100% - 55px);
}

.main-tabs :deep(.el-tab-pane) {
  height: 100%;
}

.system-info, .settings-panel {
  padding: 20px;
  background: white;
  border-radius: 4px;
  margin: 10px;
}

.app-footer {
  background-color: #f5f7fa;
  border-top: 1px solid #e4e7ed;
  padding: 0;
  height: 40px !important;
}

.footer-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 100%;
  padding: 0 20px;
  font-size: 12px;
  color: #909399;
}

/* UX-1: 欢迎界面样式 */
.welcome-content {
  text-align: center;
  padding: 20px;
}

.welcome-icon {
  margin-bottom: 20px;
}

.welcome-content h2 {
  color: #303133;
  margin-bottom: 15px;
  font-size: 24px;
}

.welcome-description {
  color: #606266;
  font-size: 16px;
  line-height: 1.6;
  margin-bottom: 25px;
}

.feature-highlights {
  background: #f5f7fa;
  border-radius: 8px;
  padding: 20px;
  margin: 20px 0;
}

.feature-item {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
  color: #303133;
}

.feature-item:last-child {
  margin-bottom: 0;
}

.feature-item .el-icon {
  color: #67c23a;
  font-size: 18px;
}

.welcome-actions {
  display: flex;
  gap: 15px;
  justify-content: center;
  margin-top: 25px;
}

.welcome-actions .el-button {
  min-width: 120px;
}

/* UX-3: 响应式设计增强 */
@media (max-width: 768px) {
  .header-actions {
    flex-direction: column;
    gap: 8px;
  }
  
  .quick-appdata-btn {
    font-size: 12px;
    padding: 8px 12px;
  }
  
  .footer-content {
    flex-direction: column;
    gap: 5px;
    text-align: center;
  }
}

/* 标签页样式优化 */
:deep(.el-tabs__item) {
  font-weight: 500;
}

:deep(.el-tabs__item.is-active) {
  color: #409eff;
}

:deep(.el-tabs__active-bar) {
  background-color: #409eff;
}
</style>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}

</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>