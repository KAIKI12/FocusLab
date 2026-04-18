<script setup lang="ts">
/**
 * App · 主窗口布局。
 * - 左:Sidebar
 * - 右:路由视图(RouterView)
 * - 主题切换快捷键留给设置页;Week 1a 仅保证结构 + 主题生效
 */

import Sidebar from "@/components/common/Sidebar.vue";
</script>

<template>
  <div class="fl-app">
    <Sidebar />
    <main class="fl-main">
      <RouterView />
    </main>
  </div>
</template>

<style>
/* 全局根容器:占满视口,Flex 行布局 */
html,
body,
#app {
  height: 100%;
  margin: 0;
  background: var(--color-bg);
  color: var(--color-text-primary);
  font-family: var(--font-sans);
  font-size: var(--fs-14);
  line-height: var(--lh-base);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>

<style scoped>
.fl-app {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.fl-main {
  flex: 1;
  overflow: auto;
  padding: var(--sp-6) var(--sp-8);
}
</style>
