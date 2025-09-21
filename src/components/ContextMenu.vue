<template>
  <teleport to="body">
    <div
      v-if="visible"
      class="context-menu"
      :style="{ left: position.x + 'px', top: position.y + 'px' }"
      @click.stop
      @contextmenu.prevent
    >
      <div class="menu-item" @click="handleMigrate">
        <el-icon><FolderRemove /></el-icon>
        <span>迁移目录</span>
      </div>
      <div class="menu-item" @click="handleOpen">
        <el-icon><FolderOpened /></el-icon>
        <span>打开目录</span>
      </div>
      <div class="menu-item" @click="handleRefresh">
        <el-icon><Refresh /></el-icon>
        <span>刷新</span>
      </div>
      <div class="menu-divider"></div>
      <div class="menu-item" @click="handleProperties">
        <el-icon><InfoFilled /></el-icon>
        <span>属性</span>
      </div>
    </div>
  </teleport>
</template>

<script setup lang="ts">
import { onUnmounted } from 'vue';
import { FolderRemove, FolderOpened, Refresh, InfoFilled } from '@element-plus/icons-vue';
import type { DirectoryInfo } from '../types/directory';

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

// 处理迁移
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

// 处理属性
function handleProperties() {
  if (props.directory) {
    // 显示属性对话框
    console.log('显示属性:', props.directory);
    closeMenu();
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
if (props.visible) {
  document.addEventListener('click', handleClickOutside);
}

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
</style>