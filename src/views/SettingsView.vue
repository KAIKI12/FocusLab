<script setup lang="ts">
/**
 * SettingsView · 设置页 — 对齐 prototype/settings/settings.html 8 分类。
 * 左侧导航 + 右侧面板。
 */

import { ref } from "vue";
import { useRouter } from "vue-router";

import { invokeCmd } from "@/composables/useTauriInvoke";
import { useAIStore } from "@/stores/useAIStore";
import { useTheme, type ThemeMode } from "@/composables/useTheme";
import { useUIStore } from "@/stores/useUIStore";

const { mode, accent, setMode, setAccent } = useTheme();
const ai = useAIStore();
const ui = useUIStore();
const router = useRouter();

const activeSection = ref("general");

const sections = [
  { id: "general", label: "通用", icon: "⚙️" },
  { id: "appearance", label: "外观", icon: "🎨" },
  { id: "pomodoro", label: "番茄钟", icon: "🍅" },
  { id: "notification", label: "通知", icon: "🔔" },
  { id: "ai", label: "AI 助手", icon: "✨" },
  { id: "privacy", label: "隐私", icon: "🔒" },
  { id: "data", label: "数据", icon: "💾" },
  { id: "about", label: "关于", icon: "ℹ️" },
];

// ---------- General ----------
const modes: ThemeMode[] = ["light", "dark", "auto"];

// ---------- Appearance ----------
const accents = [
  { id: "default", label: "默认蓝", color: "#4F8CFF" },
  { id: "claude", label: "奶油陶土", color: "#C4714F" },
  { id: "green", label: "护眼绿", color: "#5D8A6A" },
  { id: "lavender", label: "薰衣草紫", color: "#7B68AE" },
  { id: "blue-classic", label: "静谧蓝", color: "#5B7FBF" },
  { id: "graphite", label: "石墨灰", color: "#404040" },
  { id: "sakura", label: "樱花粉", color: "#D4717A" },
  { id: "candy", label: "糖果粉紫", color: "#A86CC1" },
  { id: "milktea", label: "奶茶棕粉", color: "#C47E7E" },
  { id: "amber", label: "琥珀橙", color: "#D48A3C" },
  { id: "teal", label: "水鸭青", color: "#2A8A8A" },
  { id: "slate", label: "石板蓝灰", color: "#475569" },
];

// ---------- AI ----------
const aiProvider = ref("compatible");
const aiBaseUrl = ref("");
const aiApiKey = ref("");
const aiModel = ref("gpt-4o-mini");
const aiTestResult = ref("");

async function onSaveAI() {
  try {
    await ai.configure(aiProvider.value, aiBaseUrl.value, aiApiKey.value, aiModel.value);
    aiTestResult.value = "✅ 已保存";
  } catch (e) { aiTestResult.value = `❌ 保存失败: ${e}`; }
}

async function onTestAI() {
  try {
    const result = await ai.testConnection();
    aiTestResult.value = `✅ ${result}`;
  } catch (e) { aiTestResult.value = `❌ 连接失败: ${e}`; }
}

// ---------- Data ----------
const exportResult = ref("");

async function exportTasksJson() {
  try {
    const ts = new Date().toISOString().slice(0, 10);
    const msg = await invokeCmd<string>("export_tasks_json", { path: `focuslab_tasks_${ts}.json` });
    exportResult.value = `✅ ${msg}`;
  } catch (e) { exportResult.value = `❌ ${e}`; }
}

async function exportSessionsCsv() {
  try {
    const ts = new Date().toISOString().slice(0, 10);
    const msg = await invokeCmd<string>("export_sessions_csv", { path: `focuslab_sessions_${ts}.csv` });
    exportResult.value = `✅ ${msg}`;
  } catch (e) { exportResult.value = `❌ ${e}`; }
}
</script>

<template>
  <section class="fl-settings-page">
    <!-- 左侧导航 -->
    <nav class="fl-set-nav">
      <button
        v-for="s in sections"
        :key="s.id"
        class="fl-set-nav-item"
        :class="{ 'is-active': activeSection === s.id }"
        @click="activeSection = s.id"
      >
        <span class="fl-set-nav-icon">{{ s.icon }}</span>
        <span>{{ s.label }}</span>
      </button>
    </nav>

    <!-- 右侧面板 -->
    <div class="fl-set-panel">
      <!-- 通用 -->
      <div v-if="activeSection === 'general'" class="fl-set-group">
        <h2>通用设置</h2>

        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">工作模式</div>
            <div class="fl-set-desc">标准模式功能完整，极简模式精简高效</div>
          </div>
          <div class="fl-mode-cards">
            <div class="fl-mode-card is-selected">
              <strong>标准</strong>
              <span>四象限 · AI · 目标</span>
            </div>
            <div class="fl-mode-card" @click="router.push('/minimal')">
              <strong>极简</strong>
              <span>任务 + 番茄钟</span>
            </div>
          </div>
        </div>

        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">明暗模式</div>
          </div>
          <div class="fl-set-control">
            <div class="fl-segmented">
              <button
                v-for="m in modes" :key="m"
                class="fl-seg-btn" :class="{ 'is-active': mode === m }"
                @click="setMode(m)"
              >
                {{ m === "light" ? "浅色" : m === "dark" ? "深色" : "系统" }}
              </button>
            </div>
          </div>
        </div>

        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">声效</div>
            <div class="fl-set-desc">番茄完成、休息结束等音效</div>
          </div>
          <div class="fl-set-control">
            <button class="fl-toggle" :class="{ 'is-on': ui.soundEnabled }" @click="ui.toggleSound()">
              <span class="fl-toggle-dot" />
            </button>
          </div>
        </div>
      </div>

      <!-- 外观 -->
      <div v-if="activeSection === 'appearance'" class="fl-set-group">
        <h2>外观</h2>

        <div class="fl-set-label" style="margin-bottom:var(--sp-3)">主题色调</div>
        <div class="fl-accent-grid">
          <button
            v-for="a in accents" :key="a.id"
            class="fl-accent-card" :class="{ 'is-selected': accent === a.id }"
            @click="setAccent(a.id)"
          >
            <span class="fl-accent-swatch" :style="{ background: a.color }" />
            <span class="fl-accent-name">{{ a.label }}</span>
            <span v-if="accent === a.id" class="fl-accent-check">✓</span>
          </button>
        </div>
      </div>

      <!-- 番茄钟 -->
      <div v-if="activeSection === 'pomodoro'" class="fl-set-group">
        <h2>番茄钟</h2>
        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">专注时段长度</div>
            <div class="fl-set-desc">建议从 25 分钟开始，逐步增加</div>
          </div>
          <div class="fl-set-control">
            <div class="fl-segmented">
              <button class="fl-seg-btn is-active">25m</button>
              <button class="fl-seg-btn">45m</button>
              <button class="fl-seg-btn">90m</button>
            </div>
          </div>
        </div>
        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">短休息时长</div>
          </div>
          <div class="fl-set-control">
            <div class="fl-segmented">
              <button class="fl-seg-btn is-active">5m</button>
              <button class="fl-seg-btn">10m</button>
            </div>
          </div>
        </div>
        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">长休息间隔</div>
            <div class="fl-set-desc">每隔 N 个番茄进入长休息</div>
          </div>
          <div class="fl-set-control">
            <span style="font-family:var(--font-mono);font-size:var(--fs-16)">4</span>
          </div>
        </div>
      </div>

      <!-- 通知 -->
      <div v-if="activeSection === 'notification'" class="fl-set-group">
        <h2>通知</h2>
        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">系统通知</div>
            <div class="fl-set-desc">番茄完成、休息结束时推送</div>
          </div>
          <div class="fl-set-control">
            <button class="fl-toggle is-on"><span class="fl-toggle-dot" /></button>
          </div>
        </div>
        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">日结算提醒</div>
            <div class="fl-set-desc">每天 22:00 提醒结算</div>
          </div>
          <div class="fl-set-control">
            <button class="fl-toggle is-on"><span class="fl-toggle-dot" /></button>
          </div>
        </div>
        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">到期任务提醒</div>
            <div class="fl-set-desc">到期前一天 20:00 提醒</div>
          </div>
          <div class="fl-set-control">
            <button class="fl-toggle is-on"><span class="fl-toggle-dot" /></button>
          </div>
        </div>
      </div>

      <!-- AI 助手 -->
      <div v-if="activeSection === 'ai'" class="fl-set-group">
        <h2>AI 助手</h2>
        <div class="fl-ai-form">
          <label class="fl-ai-field">
            <span>Provider</span>
            <select v-model="aiProvider" class="fl-ai-input">
              <option value="compatible">OpenAI 兼容</option>
              <option value="ollama">Ollama 本地</option>
            </select>
          </label>
          <label class="fl-ai-field">
            <span>Base URL</span>
            <input v-model="aiBaseUrl" class="fl-ai-input" type="text" :placeholder="aiProvider === 'ollama' ? 'http://localhost:11434' : 'https://api.openai.com'" />
          </label>
          <label class="fl-ai-field">
            <span>API Key</span>
            <input v-model="aiApiKey" class="fl-ai-input" type="password" placeholder="sk-..." />
          </label>
          <label class="fl-ai-field">
            <span>Model</span>
            <input v-model="aiModel" class="fl-ai-input" type="text" placeholder="gpt-4o-mini" />
          </label>
          <div class="fl-ai-actions">
            <button class="fl-set-btn" @click="onSaveAI">保存</button>
            <button class="fl-set-btn fl-set-btn-ghost" @click="onTestAI">测试连接</button>
          </div>
          <div v-if="aiTestResult" class="fl-ai-result">{{ aiTestResult }}</div>
        </div>
      </div>

      <!-- 隐私 -->
      <div v-if="activeSection === 'privacy'" class="fl-set-group">
        <h2>隐私</h2>
        <div class="fl-privacy-info">
          <p>所有数据存储在本地 SQLite 数据库，不上传到任何服务器。</p>
          <p>AI 功能需要发送任务名称到 AI 服务商（如 OpenAI），你可以随时关闭。</p>
        </div>
        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">匿名使用统计</div>
            <div class="fl-set-desc">帮助改善产品，不包含任务内容</div>
          </div>
          <div class="fl-set-control">
            <button class="fl-toggle"><span class="fl-toggle-dot" /></button>
          </div>
        </div>
      </div>

      <!-- 数据 -->
      <div v-if="activeSection === 'data'" class="fl-set-group">
        <h2>数据</h2>
        <div class="fl-data-cards">
          <div class="fl-data-card" @click="exportTasksJson">
            <span class="fl-data-icon">📋</span>
            <div>
              <strong>导出任务</strong>
              <span>JSON 格式</span>
            </div>
          </div>
          <div class="fl-data-card" @click="exportSessionsCsv">
            <span class="fl-data-icon">⏱️</span>
            <div>
              <strong>导出专注记录</strong>
              <span>CSV 格式</span>
            </div>
          </div>
        </div>
        <div v-if="exportResult" class="fl-ai-result" style="margin-top:var(--sp-3)">{{ exportResult }}</div>
      </div>

      <!-- 关于 -->
      <div v-if="activeSection === 'about'" class="fl-set-group">
        <h2>关于</h2>
        <div class="fl-about">
          <div class="fl-about-logo">FL</div>
          <div class="fl-about-name">FocusLab</div>
          <div class="fl-about-ver">v1.0.0</div>
          <p class="fl-about-desc">面向研究生的桌面专注伙伴</p>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.fl-settings-page {
  max-width: 900px;
  margin: 0 auto;
  display: grid;
  grid-template-columns: 200px 1fr;
  gap: var(--sp-6);
  min-height: 70vh;
}

@media (max-width: 720px) {
  .fl-settings-page { grid-template-columns: 1fr; }
}

/* ---------- Nav ---------- */
.fl-set-nav {
  display: flex; flex-direction: column; gap: 2px;
}
.fl-set-nav-item {
  display: flex; align-items: center; gap: var(--sp-2);
  padding: var(--sp-2) var(--sp-3); border-radius: var(--r-sm);
  border: none; background: transparent; color: var(--color-text-secondary);
  font-size: var(--fs-14); cursor: pointer; text-align: left;
  transition: all var(--dur-fast);
}
.fl-set-nav-item:hover { background: var(--color-bg-hover); color: var(--color-text-primary); }
.fl-set-nav-item.is-active { background: var(--color-primary-soft); color: var(--color-primary-dark); font-weight: var(--fw-medium); }
.fl-set-nav-icon { font-size: 16px; width: 20px; text-align: center; }

/* ---------- Panel ---------- */
.fl-set-panel { flex: 1; }
.fl-set-group { display: flex; flex-direction: column; gap: var(--sp-4); }
.fl-set-group h2 { font-size: var(--fs-20, 20px); font-weight: var(--fw-semibold); margin: 0; }

.fl-set-row {
  display: flex; align-items: center; justify-content: space-between;
  gap: var(--sp-4); padding: var(--sp-3) 0;
  border-bottom: 1px solid var(--color-border);
}
.fl-set-info { flex: 1; }
.fl-set-label { font-size: var(--fs-14); font-weight: var(--fw-medium); }
.fl-set-desc { font-size: var(--fs-12); color: var(--color-text-muted); margin-top: 2px; }
.fl-set-control { flex-shrink: 0; }

/* Segmented */
.fl-segmented {
  display: inline-flex; gap: 2px; padding: 3px;
  background: var(--color-bg-subtle); border-radius: var(--r-md);
}
.fl-seg-btn {
  padding: var(--sp-1) var(--sp-3); border: none; background: transparent;
  border-radius: var(--r-sm); color: var(--color-text-secondary);
  font-size: var(--fs-12); cursor: pointer;
}
.fl-seg-btn.is-active { background: var(--color-bg-elevated); color: var(--color-text-primary); box-shadow: var(--shadow-card); }

/* Toggle */
.fl-toggle {
  width: 40px; height: 22px; border-radius: 11px; border: none;
  background: var(--color-bg-hover); cursor: pointer; position: relative;
  transition: background var(--dur-fast);
}
.fl-toggle.is-on { background: var(--color-primary); }
.fl-toggle-dot {
  position: absolute; top: 2px; left: 2px;
  width: 18px; height: 18px; border-radius: 50%;
  background: #fff; box-shadow: 0 1px 3px rgba(0,0,0,0.15);
  transition: transform var(--dur-fast);
}
.fl-toggle.is-on .fl-toggle-dot { transform: translateX(18px); }

/* Mode cards */
.fl-mode-cards { display: flex; gap: var(--sp-2); }
.fl-mode-card {
  flex: 1; padding: var(--sp-3); border: 1px solid var(--color-border);
  border-radius: var(--r-md); cursor: pointer;
  display: flex; flex-direction: column; gap: 2px;
  transition: all var(--dur-fast);
}
.fl-mode-card:hover { border-color: var(--color-primary); }
.fl-mode-card.is-selected { border-color: var(--color-primary); background: var(--color-primary-soft); }
.fl-mode-card strong { font-size: var(--fs-14); }
.fl-mode-card span { font-size: var(--fs-12); color: var(--color-text-muted); }

/* Accent grid */
.fl-accent-grid {
  display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--sp-2);
}
.fl-accent-card {
  display: flex; align-items: center; gap: var(--sp-2);
  padding: var(--sp-2) var(--sp-3); border: 1px solid var(--color-border);
  border-radius: var(--r-md); background: transparent; cursor: pointer;
  transition: all var(--dur-fast); font-size: var(--fs-12);
}
.fl-accent-card:hover { border-color: var(--color-primary); }
.fl-accent-card.is-selected { border-color: var(--color-primary); border-width: 2px; }
.fl-accent-swatch { width: 16px; height: 16px; border-radius: 50%; flex-shrink: 0; }
.fl-accent-name { flex: 1; }
.fl-accent-check { color: var(--color-primary); font-weight: var(--fw-bold); }

/* AI form */
.fl-ai-form { display: flex; flex-direction: column; gap: var(--sp-3); }
.fl-ai-field { display: flex; flex-direction: column; gap: var(--sp-1); }
.fl-ai-field span { font-size: var(--fs-12); color: var(--color-text-muted); }
.fl-ai-input {
  padding: var(--sp-2) var(--sp-3); border: 1px solid var(--color-border);
  border-radius: var(--r-md); background: var(--color-bg-subtle);
  color: var(--color-text-primary); font-size: var(--fs-14); font-family: inherit; outline: none;
}
.fl-ai-input:focus { border-color: var(--color-primary); }
.fl-ai-actions { display: flex; gap: var(--sp-2); }
.fl-ai-result {
  font-size: var(--fs-12); color: var(--color-text-secondary);
  padding: var(--sp-2); background: var(--color-bg-subtle); border-radius: var(--r-sm);
}

.fl-set-btn {
  padding: var(--sp-2) var(--sp-4); border-radius: var(--r-md);
  border: 1px solid var(--color-border); background: var(--color-bg-elevated);
  color: var(--color-text-primary); font-size: var(--fs-12); cursor: pointer;
}
.fl-set-btn:hover { border-color: var(--color-primary); color: var(--color-primary); }
.fl-set-btn-ghost { background: transparent; color: var(--color-text-secondary); }

/* Privacy */
.fl-privacy-info { font-size: var(--fs-14); color: var(--color-text-secondary); line-height: 1.6; }
.fl-privacy-info p { margin: 0 0 var(--sp-2); }

/* Data */
.fl-data-cards { display: grid; grid-template-columns: 1fr 1fr; gap: var(--sp-3); }
.fl-data-card {
  display: flex; gap: var(--sp-3); align-items: center;
  padding: var(--sp-4); border: 1px solid var(--color-border);
  border-radius: var(--r-md); cursor: pointer;
  transition: all var(--dur-fast);
}
.fl-data-card:hover { border-color: var(--color-primary); background: var(--color-primary-soft); }
.fl-data-icon { font-size: 24px; }
.fl-data-card strong { font-size: var(--fs-14); display: block; }
.fl-data-card span { font-size: var(--fs-12); color: var(--color-text-muted); }

/* About */
.fl-about { display: flex; flex-direction: column; align-items: center; gap: var(--sp-2); padding: var(--sp-6); }
.fl-about-logo {
  width: 64px; height: 64px; border-radius: var(--r-lg);
  background: linear-gradient(135deg, var(--color-primary), var(--color-primary-light));
  color: #fff; font-weight: var(--fw-bold); font-size: 24px;
  display: grid; place-items: center;
}
.fl-about-name { font-size: var(--fs-20, 20px); font-weight: var(--fw-semibold); }
.fl-about-ver { font-size: var(--fs-12); color: var(--color-text-muted); font-family: var(--font-mono); }
.fl-about-desc { font-size: var(--fs-14); color: var(--color-text-secondary); margin: 0; }
</style>
