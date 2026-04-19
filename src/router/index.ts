/**
 * 路由表 — Phase 1 基础四视图。
 */

import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", redirect: "/today" },
    {
      path: "/ftue",
      name: "ftue",
      component: () => import("@/views/FTUEView.vue"),
      meta: { title: "欢迎", hideLayout: true },
    },
    {
      path: "/today",
      name: "today",
      component: () => import("@/views/TodayView.vue"),
      meta: { title: "今日" },
    },
    {
      path: "/goals",
      name: "goals",
      component: () => import("@/views/GoalsView.vue"),
      meta: { title: "长线目标" },
    },
    {
      path: "/stats",
      name: "stats",
      component: () => import("@/views/StatsView.vue"),
      meta: { title: "数据洞察" },
    },
    {
      path: "/calendar",
      name: "calendar",
      component: () => import("@/views/CalendarView.vue"),
      meta: { title: "日历" },
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("@/views/SettingsView.vue"),
      meta: { title: "设置" },
    },
    {
      path: "/parked",
      name: "parked",
      component: () => import("@/views/ParkedView.vue"),
      meta: { title: "搁置区" },
    },
  ],
});

export default router;
