import { createApp } from "vue";
import { createPinia } from "pinia";

import ToolWindowHost from "./views/ToolWindowHost.vue";
import "./assets/styles/tokens.css";
import "./assets/styles/base.css";
import "./assets/styles/tailwind.css";

const app = createApp(ToolWindowHost);
app.use(createPinia());
app.mount("#app");
