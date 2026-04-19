<script setup lang="ts">
/**
 * App · 主窗口布局。
 * - FTUE 未完成时全屏展示引导,隐藏 Sidebar
 * - 正常模式:左 Sidebar + 右路由视图
 */

import { computed, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import Sidebar from "@/components/common/Sidebar.vue";
import CommandPalette from "@/components/common/CommandPalette.vue";
import RecoveryDialog from "@/components/recovery/RecoveryDialog.vue";
import SettlementDialog from "@/components/settlement/SettlementDialog.vue";
import BreakEndDialog from "@/components/timer/BreakEndDialog.vue";
import { useRecovery } from "@/composables/useRecovery";

const { checkOnMount } = useRecovery();
const route = useRoute();
const router = useRouter();

const hideLayout = computed(() => route.meta.hideLayout === true);

onMounted(() => {
  // FTUE 检查
  const ftueDone = localStorage.getItem("fl-ftue-done");
  if (!ftueDone && route.path !== "/ftue") {
    router.replace("/ftue");
  }

  checkOnMount().catch((err) => {
    console.error("[recovery] checkOnMount failed", err);
  });

  // 监听悬浮球的"打开主窗口"事件
  listen("bubble:open-main", async () => {
    const win = getCurrentWindow();
    await win.show();
    await win.unminimize();
    await win.setFocus();
  });
});
</script>

<template>
  <div class="fl-app" :class="{ 'is-fullscreen': hideLayout }">
    <Sidebar v-if="!hideLayout" />
    <main class="fl-main">
      <RouterView />
    </main>
    <template v-if="!hideLayout">
      <RecoveryDialog />
      <BreakEndDialog />
      <SettlementDialog />
      <CommandPalette />
    </template>
  </div>
</template>

<style>
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

.fl-app.is-fullscreen {
  flex-direction: column;
}

.fl-main {
  flex: 1;
  overflow: auto;
  padding: var(--sp-6) var(--sp-8);
}

.fl-app.is-fullscreen .fl-main {
  padding: 0;
}
</style>
