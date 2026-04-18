<script setup lang="ts">
/**
 * App · 主窗口布局。
 * - 左:Sidebar
 * - 右:路由视图(RouterView)
 * - 最外层挂 RecoveryDialog,受 useRecoveryStore 控制
 * - onMounted 触发崩溃恢复检查(Week 1b 起)
 */

import { onMounted } from "vue";

import Sidebar from "@/components/common/Sidebar.vue";
import RecoveryDialog from "@/components/recovery/RecoveryDialog.vue";
import SettlementDialog from "@/components/settlement/SettlementDialog.vue";
import BreakEndDialog from "@/components/timer/BreakEndDialog.vue";
import { useRecovery } from "@/composables/useRecovery";

const { checkOnMount } = useRecovery();

onMounted(() => {
  checkOnMount().catch((err) => {
    console.error("[recovery] checkOnMount failed", err);
  });
});
</script>

<template>
  <div class="fl-app">
    <Sidebar />
    <main class="fl-main">
      <RouterView />
    </main>
    <RecoveryDialog />
    <BreakEndDialog />
    <SettlementDialog />
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
