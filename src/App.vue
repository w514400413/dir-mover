<script setup lang="ts">
import { ref } from "vue";
import { Refresh } from "@element-plus/icons-vue";
import DirectoryTree from "./components/DirectoryTree.vue";

const activeTab = ref("disk");

function refreshAll() {
  // 刷新所有数据
  console.log("刷新所有数据");
}
</script>

<template>
  <div class="app-container">
    <el-header class="app-header">
      <div class="header-content">
        <h1 class="app-title">C盘空间管理工具</h1>
        <div class="header-actions">
          <el-button type="primary" :icon="Refresh" @click="refreshAll">
            刷新全部
          </el-button>
        </div>
      </div>
    </el-header>

    <el-main class="app-main">
      <el-tabs v-model="activeTab" class="main-tabs">
        <el-tab-pane label="磁盘分析" name="disk">
          <directory-tree />
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