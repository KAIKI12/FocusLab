/**
 * bubble-main · 悬浮球窗口入口。
 * 独立 Vue 实例,不走 router,只渲染 BubbleView。
 */

import { createApp } from "vue";
import { createPinia } from "pinia";

import BubbleView from "./views/BubbleView.vue";
import "./assets/styles/tokens.css";
import "./assets/styles/base.css";

const app = createApp(BubbleView);
app.use(createPinia());
app.mount("#app");
