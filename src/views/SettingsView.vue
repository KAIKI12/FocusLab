<script setup lang="ts">
/**
 * SettingsView · 设置页 — 对齐 prototype/settings/settings.html 8 分类。
 * 左侧导航 + 右侧面板。
 */

import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";

import AIPrivacyModal from "@/components/common/AIPrivacyModal.vue";
import DangerConfirmModal from "@/components/common/DangerConfirmModal.vue";
import ExportModal from "@/components/common/ExportModal.vue";
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
  { id: "shortcuts", label: "快捷键", icon: "⌨️" },
  { id: "experiment", label: "实验功能", icon: "🧪" },
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

// ---------- Pomodoro ----------
const focusDuration = ref("25");
const shortBreak = ref("5");
const longBreakInterval = ref("4");

async function setFocusDuration(v: string) {
  focusDuration.value = v;
  await invokeCmd("set_setting", { key: "pomodoro_focus_minutes", value: v }).catch(() => {});
}
async function setShortBreak(v: string) {
  shortBreak.value = v;
  await invokeCmd("set_setting", { key: "pomodoro_short_break", value: v }).catch(() => {});
}
async function setLongBreakInterval(v: string) {
  longBreakInterval.value = v;
  await invokeCmd("set_setting", { key: "pomodoro_long_break_interval", value: v }).catch(() => {});
}

// ---------- Notification ----------
const notifySystem = ref(true);
const notifySettle = ref(true);
const notifyDue = ref(true);

async function toggleNotify(key: string, current: boolean) {
  const val = !current;
  if (key === "system") notifySystem.value = val;
  else if (key === "settle") notifySettle.value = val;
  else if (key === "due") notifyDue.value = val;
  await invokeCmd("set_setting", { key: `notify_${key}`, value: val ? "1" : "0" }).catch(() => {});
}

// ---------- Init ----------
onMounted(async () => {
  const load = async (key: string, fallback: string) => {
    try {
      const v = await invokeCmd<string | null>("get_setting", { key });
      return v ?? fallback;
    } catch { return fallback; }
  };
  focusDuration.value = await load("pomodoro_focus_minutes", "25");
  shortBreak.value = await load("pomodoro_short_break", "5");
  longBreakInterval.value = await load("pomodoro_long_break_interval", "4");
  notifySystem.value = (await load("notify_system", "1")) === "1";
  notifySettle.value = (await load("notify_settle", "1")) === "1";
  notifyDue.value = (await load("notify_due", "1")) === "1";
});

// ---------- Data ----------
const exportResult = ref("");
const showExportModal = ref(false);
const showDangerModal = ref(false);
const showAIPrivacy = ref(false);
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
              <button v-for="v in ['25','45','90']" :key="v" class="fl-seg-btn" :class="{ 'is-active': focusDuration === v }" @click="setFocusDuration(v)">{{ v }}m</button>
            </div>
          </div>
        </div>
        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">短休息时长</div>
          </div>
          <div class="fl-set-control">
            <div class="fl-segmented">
              <button v-for="v in ['5','10']" :key="v" class="fl-seg-btn" :class="{ 'is-active': shortBreak === v }" @click="setShortBreak(v)">{{ v }}m</button>
            </div>
          </div>
        </div>
        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">长休息间隔</div>
            <div class="fl-set-desc">每隔 N 个番茄进入长休息</div>
          </div>
          <div class="fl-set-control">
            <div class="fl-segmented">
              <button v-for="v in ['3','4','5']" :key="v" class="fl-seg-btn" :class="{ 'is-active': longBreakInterval === v }" @click="setLongBreakInterval(v)">{{ v }}</button>
            </div>
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
            <button class="fl-toggle" :class="{ 'is-on': notifySystem }" @click="toggleNotify('system', notifySystem)"><span class="fl-toggle-dot" /></button>
          </div>
        </div>
        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">日结算提醒</div>
            <div class="fl-set-desc">每天 22:00 提醒结算</div>
          </div>
          <div class="fl-set-control">
            <button class="fl-toggle" :class="{ 'is-on': notifySettle }" @click="toggleNotify('settle', notifySettle)"><span class="fl-toggle-dot" /></button>
          </div>
        </div>
        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">到期任务提醒</div>
            <div class="fl-set-desc">到期前一天 20:00 提醒</div>
          </div>
          <div class="fl-set-control">
            <button class="fl-toggle" :class="{ 'is-on': notifyDue }" @click="toggleNotify('due', notifyDue)"><span class="fl-toggle-dot" /></button>
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
          <p>AI 功能需要发送任务名称到 AI 服务商（如 OpenAI），你可以随时关闭。
            <a href="#" style="color:var(--color-primary);font-size:11px" @click.prevent="showAIPrivacy = true">查看隐私声明</a>
          </p>
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

      <!-- 快捷键 -->
      <div v-if="activeSection === 'shortcuts'" class="fl-set-group">
        <h2>快捷键</h2>
        <div class="fl-sc-list">
          <div class="fl-sc-section">焦点/番茄钟</div>
          <div class="fl-sc-row"><span>暂停 / 继续</span><kbd>Space</kbd></div>
          <div class="fl-sc-row"><span>结束番茄钟</span><kbd>⌘⇧X</kbd></div>
          <div class="fl-sc-section">任务/计划</div>
          <div class="fl-sc-row"><span>快速添加任务</span><kbd>⌘N</kbd></div>
          <div class="fl-sc-row"><span>结束今天</span><kbd>⌘⇧E</kbd></div>
          <div class="fl-sc-section">视图/导航</div>
          <div class="fl-sc-row"><span>今日计划</span><kbd>⌘1</kbd></div>
          <div class="fl-sc-row"><span>长线目标</span><kbd>⌘2</kbd></div>
          <div class="fl-sc-row"><span>日历视图</span><kbd>⌘3</kbd></div>
          <div class="fl-sc-row"><span>数据分析</span><kbd>⌘4</kbd></div>
          <div class="fl-sc-section">设置/模式</div>
          <div class="fl-sc-row"><span>命令面板</span><kbd>⌘/</kbd></div>
          <div class="fl-sc-row"><span>切换主题</span><kbd>⌘⇧T</kbd></div>
          <div class="fl-sc-row"><span>打开设置</span><kbd>⌘,</kbd></div>
        </div>
      </div>

      <!-- 实验功能 -->
      <div v-if="activeSection === 'experiment'" class="fl-set-group">
        <h2>实验功能</h2>
        <p class="fl-exp-desc">以下功能为实验性质，默认关闭。开启后可在对应页面体验。</p>
        <div class="fl-exp-list">
          <div class="fl-exp-card">
            <div class="fl-exp-head">
              <span>🌤 心情打卡</span>
              <button class="fl-toggle is-on"><span class="fl-toggle-dot" /></button>
            </div>
            <div class="fl-exp-body">早晨意图档位 + 晚间情绪记录</div>
          </div>
          <div class="fl-exp-card">
            <div class="fl-exp-head">
              <span>🧬 科研人格图鉴</span>
              <button class="fl-toggle"><span class="fl-toggle-dot" /></button>
            </div>
            <div class="fl-exp-body">30 型人格 · 7 天孵化 · 社交分享</div>
          </div>
          <div class="fl-exp-card">
            <div class="fl-exp-head">
              <span>🏆 成就徽章</span>
              <button class="fl-toggle is-on"><span class="fl-toggle-dot" /></button>
            </div>
            <div class="fl-exp-body">45 枚徽章 · 4 稀有度 · 6 分类</div>
          </div>
        </div>
      </div>

      <!-- 数据 -->
      <div v-if="activeSection === 'data'" class="fl-set-group">
        <h2>数据</h2>
        <div class="fl-data-cards">
          <div class="fl-data-card" @click="showExportModal = true">
            <span class="fl-data-icon">📦</span>
            <div>
              <strong>导出数据</strong>
              <span>JSON / Markdown / CSV</span>
            </div>
          </div>
          <div class="fl-data-card fl-data-danger" @click="showDangerModal = true">
            <span class="fl-data-icon">⚠️</span>
            <div>
              <strong>重置所有数据</strong>
              <span>清除任务、目标、专注记录</span>
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

  <ExportModal :visible="showExportModal" @close="showExportModal = false" />
  <AIPrivacyModal :visible="showAIPrivacy" @close="showAIPrivacy = false" @accepted="showAIPrivacy = false" />
  <DangerConfirmModal
    :visible="showDangerModal"
    title="重置所有数据"
    description="此操作不可撤销，将永久删除以下所有数据"
    :items="[
      { label: '任务', count: '全部' },
      { label: '目标 & 里程碑', count: '全部' },
      { label: '专注记录', count: '全部' },
      { label: '日结算', count: '全部' },
    ]"
    confirm-text="RESET"
    @close="showDangerModal = false"
    @confirmed="showDangerModal = false"
  />
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
.fl-data-card.fl-data-danger:hover { border-color: #ef4444; background: color-mix(in srgb, #ef4444 8%, transparent); }
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

/* 快捷键 */
.fl-sc-list { display: flex; flex-direction: column; gap: 4px; }
.fl-sc-section { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--color-text-muted); font-weight: var(--fw-medium); margin-top: var(--sp-3); margin-bottom: var(--sp-1); }
.fl-sc-section:first-child { margin-top: 0; }
.fl-sc-row { display: flex; align-items: center; justify-content: space-between; padding: var(--sp-2) 0; font-size: var(--fs-14); border-bottom: 1px solid var(--color-border); }
.fl-sc-row span { color: var(--color-text-primary); }
.fl-sc-row kbd { padding: 2px 8px; border: 1px solid var(--color-border); border-radius: 4px; font-size: 11px; font-family: var(--font-mono, monospace); background: var(--color-bg-subtle); color: var(--color-text-muted); }

/* 实验功能 */
.fl-exp-desc { font-size: var(--fs-14); color: var(--color-text-secondary); margin: 0 0 var(--sp-3); }
.fl-exp-list { display: flex; flex-direction: column; gap: var(--sp-3); }
.fl-exp-card { border: 1px solid var(--color-border); border-radius: var(--r-md); overflow: hidden; }
.fl-exp-head { display: flex; align-items: center; justify-content: space-between; padding: var(--sp-3) var(--sp-4); font-size: var(--fs-14); font-weight: var(--fw-medium); }
.fl-exp-body { padding: 0 var(--sp-4) var(--sp-3); font-size: var(--fs-12); color: var(--color-text-muted); }
</style>
