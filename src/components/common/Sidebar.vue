<script setup lang="ts">
/**
 * Sidebar · 最小可用的侧边导航。
 * - 4 条主路由:今日 / 长线目标 / 数据洞察 / 设置
 * - 可折叠(交互在 Week 2 细化),当前只展示图标列
 * - 配色 / 字号 / 间距全部走 CSS 变量 + Tailwind 语义类
 */

import {
  CalendarCheck,
  ChevronLeft,
  ChevronRight,
  Settings,
  Target,
  TrendingUp,
} from "lucide-vue-next";
import { useRoute } from "vue-router";

import { useUIStore } from "@/stores/useUIStore";

const ui = useUIStore();
const route = useRoute();

const nav = [
  { to: "/today", label: "今日", icon: CalendarCheck },
  { to: "/goals", label: "长线目标", icon: Target },
  { to: "/stats", label: "数据洞察", icon: TrendingUp },
  { to: "/settings", label: "设置", icon: Settings },
];
</script>

<template>
  <aside
    class="fl-sidebar"
    :class="{ 'is-collapsed': ui.sidebarCollapsed }"
    :aria-label="'主导航'"
  >
    <div class="fl-sidebar-head">
      <div class="fl-logo" aria-hidden="true">FL</div>
      <span v-if="!ui.sidebarCollapsed" class="fl-brand">FocusLab</span>
    </div>

    <nav class="fl-nav">
      <router-link
        v-for="item in nav"
        :key="item.to"
        :to="item.to"
        class="fl-nav-item"
        :class="{ 'is-active': route.path.startsWith(item.to) }"
      >
        <component :is="item.icon" :size="18" />
        <span v-if="!ui.sidebarCollapsed" class="fl-nav-label">
          {{ item.label }}
        </span>
      </router-link>
    </nav>

    <button
      class="fl-collapse"
      type="button"
      :aria-label="ui.sidebarCollapsed ? '展开侧边栏' : '收起侧边栏'"
      @click="ui.toggleSidebar"
    >
      <component
        :is="ui.sidebarCollapsed ? ChevronRight : ChevronLeft"
        :size="16"
      />
    </button>
  </aside>
</template>

<style scoped>
.fl-sidebar {
  display: flex;
  flex-direction: column;
  width: 200px;
  padding: var(--sp-5) var(--sp-3);
  background: var(--color-bg-elevated);
  border-right: 1px solid var(--color-border);
  transition: width var(--dur-base) var(--ease-smooth);
  position: relative;
}

.fl-sidebar.is-collapsed {
  width: 64px;
  padding: var(--sp-5) var(--sp-2);
}

.fl-sidebar-head {
  display: flex;
  align-items: center;
  gap: var(--sp-3);
  padding: 0 var(--sp-2) var(--sp-5);
}

.fl-logo {
  width: 36px;
  height: 36px;
  flex: 0 0 36px;
  border-radius: var(--r-md);
  display: grid;
  place-items: center;
  color: var(--color-text-on-primary);
  font-weight: var(--fw-bold);
  font-size: var(--fs-14);
  background: linear-gradient(135deg, var(--color-primary), var(--color-primary-light));
  box-shadow: 0 4px 12px color-mix(in srgb, var(--color-primary) 30%, transparent);
}

.fl-brand {
  font-size: var(--fs-16);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
  letter-spacing: -0.2px;
}

.fl-nav {
  display: flex;
  flex-direction: column;
  gap: var(--sp-1);
  flex: 1;
}

.fl-nav-item {
  display: flex;
  align-items: center;
  gap: var(--sp-3);
  padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-sm);
  color: var(--color-text-secondary);
  font-size: var(--fs-14);
  text-decoration: none;
  transition:
    background var(--dur-fast) var(--ease-smooth),
    color var(--dur-fast) var(--ease-smooth);
}

.fl-nav-item:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.fl-nav-item.is-active {
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
  font-weight: var(--fw-medium);
}

.fl-nav-label {
  line-height: 1;
}

.fl-collapse {
  position: absolute;
  bottom: var(--sp-4);
  right: calc(var(--sp-3) * -0.5);
  width: 24px;
  height: 24px;
  border-radius: var(--r-pill);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  color: var(--color-text-secondary);
  cursor: pointer;
  display: grid;
  place-items: center;
  transition: color var(--dur-fast) var(--ease-smooth);
}

.fl-collapse:hover {
  color: var(--color-primary);
}
</style>
