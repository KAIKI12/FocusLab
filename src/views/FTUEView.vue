<script setup lang="ts">
/**
 * FTUEView · 5 步首次启动引导 — 对齐 prototype/onboarding/ftue.html。
 * Step 1: 欢迎 · Step 2: 基础偏好 · Step 3: AI 配置
 * Step 4: 第一步(目标或任务) · Step 5: 开始工作
 */

import { ref } from "vue";
import { useRouter } from "vue-router";

import { invokeCmd } from "@/composables/useTauriInvoke";
import { useAIStore } from "@/stores/useAIStore";
import { useGoalStore } from "@/stores/useGoalStore";
import { useTaskStore } from "@/stores/useTaskStore";
import { useTheme, type ThemeMode } from "@/composables/useTheme";

const router = useRouter();
const ai = useAIStore();
const goals = useGoalStore();
const tasks = useTaskStore();
const { setMode } = useTheme();

const step = ref(1);

// Step 2 偏好
const pomodoroPreset = ref<"classic_25" | "deep_45" | "immersive_90">("classic_25");
const themeMode = ref<ThemeMode>("light");
const notifyMode = ref<"on" | "focus" | "off">("on");

// Step 3 AI
const aiProvider = ref("compatible");
const aiBaseUrl = ref("");
const aiApiKey = ref("");
const aiModel = ref("gpt-4o-mini");
const skipAI = ref(false);

// Step 4 第一步
const firstGoal = ref("");
const firstTask = ref("");

function nextStep() {
  if (step.value < 5) step.value++;
}

function prevStep() {
  if (step.value > 1) step.value--;
}

async function onFinish() {
  // 应用偏好
  setMode(themeMode.value);

  // AI 配置
  if (!skipAI.value && aiApiKey.value) {
    try {
      await ai.configure(aiProvider.value, "openai", aiBaseUrl.value, aiApiKey.value, aiModel.value);
    } catch { /* 可选 */ }
  }

  // 第一个目标
  if (firstGoal.value.trim()) {
    try { await goals.createGoal({ name: firstGoal.value.trim() }); } catch { /* */ }
  }

  // 第一个任务
  if (firstTask.value.trim()) {
    try { await tasks.create({ name: firstTask.value.trim() }); } catch { /* */ }
  }

  // 标记 FTUE 完成
  try {
    await invokeCmd("update_timer_state", { patch: {} }); // placeholder
  } catch { /* */ }
  localStorage.setItem("fl-ftue-done", "true");
  router.push("/today");
}

function skipAll() {
  localStorage.setItem("fl-ftue-done", "true");
  router.push("/today");
}

const FEATURES = [
  { icon: "🍅", label: "专注计时", desc: "番茄钟 + 自由计时" },
  { icon: "📊", label: "数据洞察", desc: "热力图 · 趋势 · 周报" },
  { icon: "✨", label: "AI 助手", desc: "建议 · 拆解 · 鼓励" },
  { icon: "💛", label: "温和反馈", desc: "不用「失败」· 只有陪伴" },
];
</script>

<template>
  <section class="fl-ftue">
    <!-- Stepper -->
    <div class="fl-stepper">
      <div
        v-for="i in 5"
        :key="i"
        class="fl-step-dot"
        :class="{ 'is-done': i < step, 'is-current': i === step }"
      />
    </div>

    <!-- Step 1: 欢迎 -->
    <div v-if="step === 1" class="fl-step-card">
      <div class="fl-welcome-logo">FL</div>
      <h1>欢迎来到 FocusLab</h1>
      <p class="fl-welcome-sub">面向研究生的桌面专注伙伴 · 2 分钟完成设置</p>

      <div class="fl-feature-grid">
        <div v-for="f in FEATURES" :key="f.label" class="fl-feature-item">
          <span class="fl-feature-icon">{{ f.icon }}</span>
          <div>
            <div class="fl-feature-label">{{ f.label }}</div>
            <div class="fl-feature-desc">{{ f.desc }}</div>
          </div>
        </div>
      </div>

      <div class="fl-step-actions">
        <button class="fl-btn-ghost" @click="skipAll">跳过全部</button>
        <button class="fl-btn-primary" @click="nextStep">开始设置</button>
      </div>
    </div>

    <!-- Step 2: 基础偏好 -->
    <div v-else-if="step === 2" class="fl-step-card">
      <h2>基础偏好</h2>

      <div class="fl-pref-group">
        <label class="fl-pref-label">专注时段长度</label>
        <div class="fl-pill-row">
          <button :class="{ 'is-active': pomodoroPreset === 'classic_25' }" @click="pomodoroPreset = 'classic_25'">25 分钟</button>
          <button :class="{ 'is-active': pomodoroPreset === 'deep_45' }" @click="pomodoroPreset = 'deep_45'">45 分钟</button>
          <button :class="{ 'is-active': pomodoroPreset === 'immersive_90' }" @click="pomodoroPreset = 'immersive_90'">90 分钟</button>
        </div>
      </div>

      <div class="fl-pref-group">
        <label class="fl-pref-label">外观</label>
        <div class="fl-pill-row">
          <button :class="{ 'is-active': themeMode === 'light' }" @click="themeMode = 'light'">浅色</button>
          <button :class="{ 'is-active': themeMode === 'dark' }" @click="themeMode = 'dark'">深色</button>
          <button :class="{ 'is-active': themeMode === 'auto' }" @click="themeMode = 'auto'">自动</button>
        </div>
      </div>

      <div class="fl-pref-group">
        <label class="fl-pref-label">工作模式</label>
        <div class="fl-pill-row">
          <button class="is-active">标准</button>
          <button @click="$router.push('/minimal')">极简</button>
        </div>
      </div>

      <div class="fl-pref-group">
        <label class="fl-pref-label">通知</label>
        <div class="fl-pill-row">
          <button :class="{ 'is-active': notifyMode === 'on' }" @click="notifyMode = 'on'">开启</button>
          <button :class="{ 'is-active': notifyMode === 'focus' }" @click="notifyMode = 'focus'">仅专注</button>
          <button :class="{ 'is-active': notifyMode === 'off' }" @click="notifyMode = 'off'">关闭</button>
        </div>
      </div>

      <div class="fl-step-actions">
        <button class="fl-btn-ghost" @click="prevStep">上一步</button>
        <button class="fl-btn-primary" @click="nextStep">下一步</button>
      </div>
    </div>

    <!-- Step 3: AI 配置 -->
    <div v-else-if="step === 3" class="fl-step-card">
      <h2>AI 助手 (可选)</h2>

      <div class="fl-privacy-banner">
        🔒 AI 功能需要将任务名称发送到 AI 服务商。FocusLab 不存储你的数据。
      </div>

      <!-- 提供商卡片选择 -->
      <div v-if="!skipAI" class="fl-provider-grid">
        <button
          class="fl-provider-card" :class="{ 'is-active': aiProvider === 'compatible' }"
          @click="aiProvider = 'compatible'"
        >
          <span class="fl-provider-icon">🌐</span>
          <strong>OpenAI 兼容</strong>
          <span>DeepSeek / 智谱 / 代理</span>
        </button>
        <button
          class="fl-provider-card" :class="{ 'is-active': aiProvider === 'openai' }"
          @click="aiProvider = 'openai'"
        >
          <span class="fl-provider-icon">🤖</span>
          <strong>OpenAI</strong>
          <span>GPT-4o / GPT-4o-mini</span>
        </button>
        <button
          class="fl-provider-card" :class="{ 'is-active': aiProvider === 'ollama' }"
          @click="aiProvider = 'ollama'"
        >
          <span class="fl-provider-icon">🏠</span>
          <strong>Ollama</strong>
          <span>本地模型</span>
        </button>
      </div>

      <label v-if="!skipAI" class="fl-input-field">
        <span>API Key</span>
        <input v-model="aiApiKey" type="password" placeholder="sk-..." />
      </label>
      <label v-if="!skipAI" class="fl-input-field">
        <span>Model</span>
        <input v-model="aiModel" type="text" placeholder="gpt-4o-mini" />
      </label>

      <label class="fl-skip-toggle">
        <input v-model="skipAI" type="checkbox" />
        <span>暂时跳过，以后再配</span>
      </label>

      <div class="fl-step-actions">
        <button class="fl-btn-ghost" @click="prevStep">上一步</button>
        <button class="fl-btn-primary" @click="nextStep">下一步</button>
      </div>
    </div>

    <!-- Step 4: 第一步 -->
    <div v-else-if="step === 4" class="fl-step-card">
      <h2>迈出第一步</h2>

      <label class="fl-input-field">
        <span>设置一个长线目标 (可选)</span>
        <input v-model="firstGoal" type="text" placeholder="例：发表 SCI 论文" maxlength="60" />
      </label>

      <label class="fl-input-field">
        <span>或者添加第一个任务</span>
        <input v-model="firstTask" type="text" placeholder="例：阅读文献 2 篇" maxlength="80" />
      </label>

      <div class="fl-step-actions">
        <button class="fl-btn-ghost" @click="prevStep">上一步</button>
        <button class="fl-btn-primary" @click="nextStep">下一步</button>
      </div>
    </div>

    <!-- Step 5: 开始 -->
    <div v-else class="fl-step-card">
      <h2>准备就绪!</h2>

      <div class="fl-summary-grid">
        <div class="fl-summary-item">
          <span>🍅</span>
          <span>专注时长</span>
          <strong>{{ { classic_25: '25m', deep_45: '45m', immersive_90: '90m' }[pomodoroPreset] }}</strong>
        </div>
        <div class="fl-summary-item">
          <span>🎨</span>
          <span>外观</span>
          <strong>{{ { light: '浅色', dark: '深色', auto: '自动' }[themeMode] }}</strong>
        </div>
        <div class="fl-summary-item">
          <span>✨</span>
          <span>AI</span>
          <strong>{{ skipAI ? '关闭' : '已配置' }}</strong>
        </div>
        <div class="fl-summary-item">
          <span>🔔</span>
          <span>通知</span>
          <strong>{{ { on: '开启', focus: '仅专注', off: '关闭' }[notifyMode] }}</strong>
        </div>
      </div>

      <div class="fl-step-actions">
        <button class="fl-btn-ghost" @click="prevStep">上一步</button>
        <button class="fl-btn-primary fl-btn-lg" @click="onFinish">
          🚀 开工了!
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.fl-ftue {
  max-width: 560px;
  margin: 0 auto;
  padding: var(--sp-8) var(--sp-4);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sp-6);
  min-height: 100vh;
}

.fl-stepper {
  display: flex;
  gap: var(--sp-3);
}
.fl-step-dot {
  width: 10px; height: 10px; border-radius: 50%;
  background: var(--color-bg-hover); border: 1.5px solid var(--color-border);
  transition: all var(--dur-base);
}
.fl-step-dot.is-current { background: var(--color-primary); border-color: var(--color-primary); }
.fl-step-dot.is-done { background: var(--color-success); border-color: var(--color-success); }

.fl-step-card {
  width: 100%;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  padding: var(--sp-6);
  display: flex;
  flex-direction: column;
  gap: var(--sp-5);
  text-align: center;
}
.fl-step-card h1 { font-size: var(--fs-24); font-weight: var(--fw-semibold); margin: 0; }
.fl-step-card h2 { font-size: var(--fs-20, 20px); font-weight: var(--fw-semibold); margin: 0; }

.fl-welcome-logo {
  width: 72px; height: 72px; margin: 0 auto;
  border-radius: var(--r-lg);
  background: linear-gradient(135deg, var(--color-primary), var(--color-primary-light));
  color: #fff; font-weight: var(--fw-bold); font-size: 28px;
  display: grid; place-items: center;
  box-shadow: 0 8px 24px color-mix(in srgb, var(--color-primary) 30%, transparent);
}
.fl-welcome-sub { color: var(--color-text-secondary); font-size: var(--fs-14); margin: 0; }

.fl-feature-grid { display: grid; grid-template-columns: 1fr 1fr; gap: var(--sp-3); text-align: left; }
.fl-feature-item {
  display: flex; gap: var(--sp-3); padding: var(--sp-3);
  border-radius: var(--r-md); border: 1px solid var(--color-border);
  transition: all var(--dur-base);
}
.fl-feature-item:hover { background: var(--color-primary-soft); transform: translateY(-1px); }
.fl-feature-icon { font-size: 24px; flex-shrink: 0; }
.fl-feature-label { font-size: var(--fs-14); font-weight: var(--fw-medium); }
.fl-feature-desc { font-size: var(--fs-12); color: var(--color-text-muted); }

.fl-step-actions { display: flex; gap: var(--sp-3); justify-content: center; }
.fl-btn-primary {
  padding: var(--sp-3) var(--sp-6); border-radius: var(--r-md); border: none;
  background: var(--color-primary); color: #fff; font-size: var(--fs-14);
  font-weight: var(--fw-medium); cursor: pointer;
  transition: background var(--dur-fast);
}
.fl-btn-primary:hover { background: var(--color-primary-dark); }
.fl-btn-lg { padding: var(--sp-4) var(--sp-8); font-size: var(--fs-16); }
.fl-btn-ghost {
  padding: var(--sp-3) var(--sp-5); border-radius: var(--r-md);
  border: 1px solid var(--color-border); background: transparent;
  color: var(--color-text-secondary); font-size: var(--fs-14); cursor: pointer;
}

.fl-pref-group { display: flex; flex-direction: column; gap: var(--sp-2); text-align: left; }
.fl-pref-label { font-size: var(--fs-12); color: var(--color-text-secondary); font-weight: var(--fw-medium); }
.fl-pill-row { display: flex; gap: var(--sp-2); }
.fl-pill-row button {
  flex: 1; padding: var(--sp-2) var(--sp-3); border-radius: var(--r-md);
  border: 1px solid var(--color-border); background: transparent;
  color: var(--color-text-secondary); font-size: var(--fs-12); cursor: pointer;
  transition: all var(--dur-fast);
}
.fl-pill-row button.is-active {
  background: var(--color-primary-soft); color: var(--color-primary);
  border-color: var(--color-primary); font-weight: var(--fw-medium);
}

.fl-privacy-banner {
  padding: var(--sp-3); border-radius: var(--r-sm);
  background: color-mix(in srgb, var(--color-gold, #FAAD14) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--color-gold, #FAAD14) 25%, transparent);
  font-size: var(--fs-12); color: var(--color-text-secondary); text-align: left;
}

.fl-input-field {
  display: flex; flex-direction: column; gap: var(--sp-1); text-align: left;
}
.fl-input-field span { font-size: var(--fs-12); color: var(--color-text-secondary); }
.fl-input-field input {
  padding: var(--sp-2) var(--sp-3); border-radius: var(--r-md);
  border: 1px solid var(--color-border); background: var(--color-bg-subtle);
  color: var(--color-text-primary); font-size: var(--fs-14); outline: none;
}
.fl-input-field input:focus { border-color: var(--color-primary); }

.fl-skip-toggle {
  display: flex; align-items: center; gap: var(--sp-2);
  font-size: var(--fs-12); color: var(--color-text-secondary); cursor: pointer;
}

.fl-provider-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--sp-2); text-align: left; }
.fl-provider-card {
  display: flex; flex-direction: column; gap: 2px; padding: var(--sp-3);
  border: 1px solid var(--color-border); border-radius: var(--r-md);
  background: transparent; cursor: pointer; transition: all var(--dur-fast);
}
.fl-provider-card:hover { border-color: var(--color-primary); }
.fl-provider-card.is-active { border-color: var(--color-primary); background: var(--color-primary-soft); }
.fl-provider-icon { font-size: 20px; }
.fl-provider-card strong { font-size: var(--fs-12); }
.fl-provider-card span { font-size: 10px; color: var(--color-text-muted); }

.fl-summary-grid { display: grid; grid-template-columns: 1fr 1fr; gap: var(--sp-3); }
.fl-summary-item {
  display: flex; flex-direction: column; align-items: center; gap: var(--sp-1);
  padding: var(--sp-3); border-radius: var(--r-md);
  background: var(--color-bg-subtle); font-size: var(--fs-12);
}
.fl-summary-item strong { font-size: var(--fs-14); color: var(--color-text-primary); }
</style>
