/**
 * bubble-main · 悬浮球窗口入口。
 * 独立 Vue 实例,不走 router,只渲染 BubbleView。
 * 注意:不导入 base.css(会覆盖透明背景),只导入 tokens.css 获取 CSS 变量。
 */

import { createApp } from "vue";
import { createPinia } from "pinia";

import BubbleView from "./views/BubbleView.vue";
import "./assets/styles/tokens.css";

const app = createApp(BubbleView);
app.use(createPinia());
app.mount("#app");
