import { createApp } from "vue";
import { createPinia } from "pinia";

import App from "./App.vue";
import router from "./router";

// 样式层:顺序重要 — tokens 提供 CSS 变量 → base 打底 → tailwind 工具类
import "./assets/styles/tokens.css";
import "./assets/styles/base.css";
import "./assets/styles/tailwind.css";

// vue-flow 样式必须放在 tailwind.css 之后,避免被 preflight 覆盖。
// 注意:这些 CSS 必须以"非 scoped"方式全局注入,vue-flow 内部 class 才能生效。
import "@vue-flow/core/dist/style.css";
import "@vue-flow/core/dist/theme-default.css";
import "@vue-flow/controls/dist/style.css";
import "@vue-flow/minimap/dist/style.css";

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.mount("#app");
