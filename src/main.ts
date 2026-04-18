import { createApp } from "vue";
import { createPinia } from "pinia";

import App from "./App.vue";
import router from "./router";

// 样式层:顺序重要 — tokens 提供 CSS 变量 → base 打底 → tailwind 工具类
import "./assets/styles/tokens.css";
import "./assets/styles/base.css";
import "./assets/styles/tailwind.css";

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.mount("#app");
