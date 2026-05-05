<script setup lang="ts">
/**
 * SettingsView · 设置页 — 对齐 prototype/settings/settings.html 8 分类。
 * 左侧导航 + 右侧面板。
 */

import { onMounted, ref, type Ref } from "vue";
import { useRouter } from "vue-router";

import AIPrivacyModal from "@/components/common/AIPrivacyModal.vue";
import AIPayloadModal from "@/components/common/AIPayloadModal.vue";
import DangerConfirmModal from "@/components/common/DangerConfirmModal.vue";
import ExportModal from "@/components/common/ExportModal.vue";
import ShortcutSettingsPanel from "@/components/settings/ShortcutSettingsPanel.vue";
import { invokeCmd } from "@/composables/useTauriInvoke";
import { useAIStore } from "@/stores/useAIStore";
import { useAIProfileStore, type ChatProfile, type EmbeddingProfile } from "@/stores/useAIProfileStore";
import { detectShortcutPlatform } from "@/shortcuts/platform";
import { useShortcutStore } from "@/stores/useShortcutStore";
import { useTheme, type ThemeMode } from "@/composables/useTheme";
import { useUIStore } from "@/stores/useUIStore";
import { useChatStore } from "@/stores/useChatStore";

const { mode, accent, setMode, setAccent } = useTheme();
const ai = useAIStore();
const profileStore = useAIProfileStore();
const ui = useUIStore();
const chat = useChatStore();
const shortcutStore = useShortcutStore();
const router = useRouter();
const shortcutPlatform = detectShortcutPlatform(window.navigator.platform);

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
// A. 全局开关
const aiEnabled = ref(true);

// B/C. Chat / Embedding profile 编辑草稿 - profile 列表来自 profileStore
type ChatDraft = Omit<ChatProfile, "createdAt" | "updatedAt">;
type EmbeddingDraft = Omit<EmbeddingProfile, "createdAt" | "updatedAt">;
const blankChatDraft = (): ChatDraft => ({
  id: "",
  name: "",
  provider: "openai",
  apiFormat: "openai",
  baseUrl: "",
  apiKey: "",
  modelFast: "gpt-4o-mini",
  modelStrong: "gpt-4o-mini",
  selectedModels: [],
});
const blankEmbeddingDraft = (): EmbeddingDraft => ({
  id: "",
  name: "",
  baseUrl: "",
  apiKey: "",
  model: "text-embedding-3-small",
});
const chatDraft = ref<ChatDraft>(blankChatDraft());
const embeddingDraft = ref<EmbeddingDraft>(blankEmbeddingDraft());
const chatEditMode = ref<"new" | "edit" | null>(null);
const embeddingEditMode = ref<"new" | "edit" | null>(null);
const aiConnStatus = ref(""); // success | fail | ""
const aiConnCheckedAt = ref("");

// C. 语气风格
const aiTone = ref("academic");
const aiToneCustom = ref("");

// D. 强度
const aiIntensity = ref(3);

// 测试结果提示
const aiTestResult = ref("");

// 模型列表动态获取
const fetchedModels = ref<string[]>([]);
const modelFetching = ref(false);
const modelFetchError = ref("");

// Embedding 模型列表动态获取
const embeddingFetchedModels = ref<string[]>([]);
const embeddingModelFetching = ref(false);
const embeddingModelFetchError = ref("");

// Provider 预设表（base_url 默认值）
const providerPresets: Record<string, { baseUrl: string; models: string[]; format: string }> = {
  openai:     { baseUrl: "https://api.openai.com",          models: ["gpt-4o-mini", "gpt-4o", "gpt-4-turbo"], format: "openai" },
  claude:     { baseUrl: "https://api.anthropic.com",       models: ["claude-3-5-haiku-20241022", "claude-3-5-sonnet-20241022", "claude-opus-4-5"], format: "claude" },
  deepseek:   { baseUrl: "https://api.deepseek.com",        models: ["deepseek-chat", "deepseek-coder"], format: "openai" },
  moonshot:   { baseUrl: "https://api.moonshot.cn",         models: ["moonshot-v1-8k", "moonshot-v1-32k"], format: "openai" },
  qwen:       { baseUrl: "https://dashscope.aliyuncs.com/compatible-mode", models: ["qwen-turbo", "qwen-plus", "qwen-max"], format: "openai" },
  gemini:     { baseUrl: "https://generativelanguage.googleapis.com/v1beta/openai", models: ["gemini-2.0-flash", "gemini-1.5-pro"], format: "openai" },
  ollama:     { baseUrl: "http://localhost:11434",          models: ["llama3", "mistral", "qwen2.5"], format: "openai" },
  custom:     { baseUrl: "",                                models: [], format: "openai" },
};

function onProviderChange() {
  const preset = providerPresets[chatDraft.value.provider];
  if (preset) {
    if (chatDraft.value.provider !== "custom") {
      chatDraft.value.baseUrl = preset.baseUrl;
      chatDraft.value.apiFormat = preset.format;
      if (preset.models.length > 0) {
        chatDraft.value.modelFast = preset.models[0];
        chatDraft.value.modelStrong = preset.models[0];
      }
    }
  }
  fetchedModels.value = [];
  modelFetchError.value = "";
}

async function onFetchModels() {
  const baseUrl = chatDraft.value.baseUrl.trim();
  if (!baseUrl) { modelFetchError.value = "请先填写 Base URL"; return; }
  modelFetching.value = true;
  modelFetchError.value = "";
  try {
    const models = await ai.fetchModels(baseUrl, chatDraft.value.apiKey, chatDraft.value.apiFormat);
    fetchedModels.value = models;
    if (models.length > 0 && !models.includes(chatDraft.value.modelFast)) {
      chatDraft.value.modelFast = models[0];
    }
    if (models.length > 0 && !models.includes(chatDraft.value.modelStrong)) {
      chatDraft.value.modelStrong = chatDraft.value.modelFast;
    }
    await invokeCmd("set_setting", {
      key: `ai_fetched_models_${chatDraft.value.provider}`,
      value: JSON.stringify(models),
    }).catch(() => {});
  } catch (e) {
    modelFetchError.value = `获取失败: ${e}`;
  } finally {
    modelFetching.value = false;
  }
}

function toggleSelectedModel(m: string) {
  const idx = chatDraft.value.selectedModels.indexOf(m);
  if (idx >= 0) {
    chatDraft.value.selectedModels.splice(idx, 1);
  } else {
    chatDraft.value.selectedModels.push(m);
  }
}

async function onFetchEmbeddingModels() {
  const baseUrl = embeddingDraft.value.baseUrl.trim();
  if (!baseUrl) { embeddingModelFetchError.value = "请先填写 Base URL"; return; }
  embeddingModelFetching.value = true;
  embeddingModelFetchError.value = "";
  try {
    // embedding 走 OpenAI 兼容协议(/v1/models)
    const all = await ai.fetchModels(baseUrl, embeddingDraft.value.apiKey, "openai");
    // 过滤出 embedding 模型(名字含 embed/embedding)
    const filtered = all.filter((m) => /embed/i.test(m));
    const list = filtered.length > 0 ? filtered : all;
    embeddingFetchedModels.value = list;
    if (list.length > 0 && !list.includes(embeddingDraft.value.model)) {
      embeddingDraft.value.model = list[0];
    }
  } catch (e) {
    embeddingModelFetchError.value = `获取失败: ${e}`;
  } finally {
    embeddingModelFetching.value = false;
  }
}

// 语气风格列表
const toneOptions = [
  { id: "academic", icon: "📚", name: "学术导师", desc: "严谨分析，结构清晰" },
  { id: "coach",    icon: "💪", name: "运动教练", desc: "充满活力，积极鼓励" },
  { id: "zen",      icon: "🍃", name: "禅意伙伴", desc: "温和平静，正念引导" },
  { id: "minimal",  icon: "📊", name: "简洁数据", desc: "数字说话，去除废话" },
  { id: "cat",      icon: "🐱", name: "猫咪助手", desc: "卖萌风格，偶尔傲娇" },
  { id: "custom",   icon: "✏️", name: "自定义", desc: "用自己的提示词" },
];

// ---------- Chat profile 编辑/激活/删除 ----------
function startNewChatProfile() {
  chatDraft.value = blankChatDraft();
  chatEditMode.value = "new";
  fetchedModels.value = [];
}

function startEditChatProfile(p: ChatProfile) {
  chatDraft.value = {
    id: p.id, name: p.name, provider: p.provider, apiFormat: p.apiFormat,
    baseUrl: p.baseUrl, apiKey: p.apiKey,
    modelFast: p.modelFast, modelStrong: p.modelStrong,
    selectedModels: [...p.selectedModels],
  };
  chatEditMode.value = "edit";
  fetchedModels.value = [];
  invokeCmd<string | null>("get_setting", { key: `ai_fetched_models_${p.provider}` })
    .then((raw) => {
      if (!raw) return;
      try {
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed)) fetchedModels.value = parsed.filter((m): m is string => typeof m === "string");
      } catch {}
    })
    .catch(() => {});
}

function cancelChatEdit() {
  chatEditMode.value = null;
  chatDraft.value = blankChatDraft();
}

async function saveChatProfile() {
  if (!chatDraft.value.name.trim()) { aiTestResult.value = "❌ 请填写 Profile 名称"; return; }
  // modelStrong 为空时回填 modelFast
  if (!chatDraft.value.modelStrong.trim()) chatDraft.value.modelStrong = chatDraft.value.modelFast;
  try {
    if (chatEditMode.value === "new") {
      const id = await profileStore.createChat(chatDraft.value);
      // 第一条 profile 自动激活
      if (profileStore.chatProfiles.length === 1 || !profileStore.activeChatId) {
        await profileStore.activateChat(id);
      }
      aiTestResult.value = "✅ 已新建 chat profile";
    } else {
      await profileStore.updateChat({
        ...chatDraft.value,
        createdAt: "", updatedAt: "", // 后端忽略
      } as ChatProfile);
      aiTestResult.value = "✅ 已保存修改";
    }
    cancelChatEdit();
  } catch (e) {
    aiTestResult.value = `❌ 保存失败: ${e}`;
  }
}

async function activateChatProfile(id: string) {
  try {
    await profileStore.activateChat(id);
    aiTestResult.value = "✅ 已切换激活 profile";
  } catch (e) { aiTestResult.value = `❌ 激活失败: ${e}`; }
}

async function deleteChatProfile(id: string, name: string) {
  if (!confirm(`确定删除 "${name}" 这条 chat profile?`)) return;
  try {
    await profileStore.deleteChat(id);
    aiTestResult.value = "✅ 已删除";
  } catch (e) { aiTestResult.value = `❌ 删除失败: ${e}`; }
}

// ---------- Embedding profile ----------
function startNewEmbeddingProfile() {
  embeddingDraft.value = blankEmbeddingDraft();
  embeddingEditMode.value = "new";
  embeddingFetchedModels.value = [];
  embeddingModelFetchError.value = "";
}

function startEditEmbeddingProfile(p: EmbeddingProfile) {
  embeddingDraft.value = {
    id: p.id, name: p.name, baseUrl: p.baseUrl, apiKey: p.apiKey, model: p.model,
  };
  embeddingEditMode.value = "edit";
  embeddingFetchedModels.value = [];
  embeddingModelFetchError.value = "";
}

function cancelEmbeddingEdit() {
  embeddingEditMode.value = null;
  embeddingDraft.value = blankEmbeddingDraft();
  embeddingFetchedModels.value = [];
  embeddingModelFetchError.value = "";
}

async function saveEmbeddingProfile() {
  if (!embeddingDraft.value.name.trim()) { aiTestResult.value = "❌ 请填写 Profile 名称"; return; }
  try {
    if (embeddingEditMode.value === "new") {
      const id = await profileStore.createEmbedding(embeddingDraft.value);
      if (profileStore.embeddingProfiles.length === 1 || !profileStore.activeEmbeddingId) {
        await profileStore.activateEmbedding(id);
      }
      aiTestResult.value = "✅ 已新建 embedding profile";
    } else {
      await profileStore.updateEmbedding({
        ...embeddingDraft.value, createdAt: "", updatedAt: "",
      } as EmbeddingProfile);
      aiTestResult.value = "✅ 已保存修改";
    }
    cancelEmbeddingEdit();
  } catch (e) { aiTestResult.value = `❌ 保存失败: ${e}`; }
}

async function activateEmbeddingProfile(id: string) {
  try {
    await profileStore.activateEmbedding(id);
    aiTestResult.value = "✅ 已切换激活 embedding profile";
  } catch (e) { aiTestResult.value = `❌ 激活失败: ${e}`; }
}

async function deleteEmbeddingProfile(id: string, name: string) {
  if (!confirm(`确定删除 "${name}" 这条 embedding profile?`)) return;
  try {
    await profileStore.deleteEmbedding(id);
    aiTestResult.value = "✅ 已删除";
  } catch (e) { aiTestResult.value = `❌ 删除失败: ${e}`; }
}

async function onTestAI() {
  aiTestResult.value = "⏳ 测试中…";
  try {
    const result = await ai.testConnection();
    aiTestResult.value = `✅ ${result}`;
    aiConnStatus.value = "success";
  } catch (e) {
    aiTestResult.value = `❌ 连接失败: ${e}`;
    aiConnStatus.value = "fail";
  }
}

async function toggleAiEnabled() {
  aiEnabled.value = !aiEnabled.value;
  await invokeCmd("set_setting", { key: "ai_enabled", value: aiEnabled.value ? "1" : "0" }).catch(() => {});
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

// ---------- 🌀 自由模式偏好(v1.2.3,对齐 prototype/settings/settings.html:883) ----------
// 先持久化到 key-value,实际 timer 行为在后续 commit 按 key 读取
const freeDefaultMode = ref("pomodoro"); // "pomodoro" | "free" | "last"
const freeMinRecord = ref("3");          // 分钟,1-15
const freeSoftRemind = ref("90");        // 分钟,30-180
const freeSoftRemindOn = ref(true);
const freePauseAsBreak = ref(false);
const freeSwitchAutoEnd = ref(true);

async function setFreeDefault(v: string) {
  freeDefaultMode.value = v;
  await invokeCmd("set_setting", { key: "pomodoro_default_mode", value: v }).catch(() => {});
}
async function setFreeMinRecord(v: string) {
  freeMinRecord.value = v;
  await invokeCmd("set_setting", { key: "free_min_record_minutes", value: v }).catch(() => {});
}
async function setFreeSoftRemind(v: string) {
  freeSoftRemind.value = v;
  await invokeCmd("set_setting", { key: "free_soft_remind_minutes", value: v }).catch(() => {});
}
async function toggleFreeFlag(key: "soft_remind_on" | "pause_as_break" | "switch_auto_end") {
  if (key === "soft_remind_on") freeSoftRemindOn.value = !freeSoftRemindOn.value;
  else if (key === "pause_as_break") freePauseAsBreak.value = !freePauseAsBreak.value;
  else freeSwitchAutoEnd.value = !freeSwitchAutoEnd.value;
  const storeKey = {
    soft_remind_on: "free_soft_remind_enabled",
    pause_as_break: "free_pause_as_break",
    switch_auto_end: "free_switch_auto_end",
  }[key];
  const val = (
    key === "soft_remind_on" ? freeSoftRemindOn.value
    : key === "pause_as_break" ? freePauseAsBreak.value
    : freeSwitchAutoEnd.value
  ) ? "1" : "0";
  await invokeCmd("set_setting", { key: storeKey, value: val }).catch(() => {});
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

// ---------- Privacy / Experiment 开关持久化 ----------
// 仅做前端持久化；开关对应功能的实际控制由后续 commit 分别接入。
const statsEnabled = ref(false);   // privacy_anonymous_stats
const expMood = ref(true);         // exp_mood_checkin
const expPersona = ref(false);     // exp_persona
const expBadges = ref(true);       // exp_badges

type ToggleKey = "stats" | "mood" | "persona" | "badges";
const toggleConfig: Record<ToggleKey, { ref: Ref<boolean>; key: string }> = {
  stats:   { ref: statsEnabled, key: "privacy_anonymous_stats" },
  mood:    { ref: expMood,      key: "exp_mood_checkin" },
  persona: { ref: expPersona,   key: "exp_persona" },
  badges:  { ref: expBadges,    key: "exp_badges" },
};

// 先持久化成功再更新 UI — 失败时保持原状态,避免"看似切换实际未生效"
async function onToggleSetting(target: ToggleKey) {
  const { ref: r, key } = toggleConfig[target];
  const nextVal = !r.value;
  try {
    await invokeCmd("set_setting", { key, value: nextVal ? "1" : "0" });
    r.value = nextVal;
  } catch (e) {
    console.warn(`[settings] persist ${key} failed`, e);
  }
}

// ---------- Init ----------
onMounted(async () => {
  shortcutStore.setPlatform(shortcutPlatform);
  const load = async (key: string, fallback: string) => {
    try {
      const v = await invokeCmd<string | null>("get_setting", { key });
      return v ?? fallback;
    } catch { return fallback; }
  };
  focusDuration.value = await load("pomodoro_focus_minutes", "25");
  shortBreak.value = await load("pomodoro_short_break", "5");
  longBreakInterval.value = await load("pomodoro_long_break_interval", "4");
  freeDefaultMode.value = await load("pomodoro_default_mode", "pomodoro");
  freeMinRecord.value = await load("free_min_record_minutes", "3");
  freeSoftRemind.value = await load("free_soft_remind_minutes", "90");
  freeSoftRemindOn.value = (await load("free_soft_remind_enabled", "1")) === "1";
  freePauseAsBreak.value = (await load("free_pause_as_break", "0")) === "1";
  freeSwitchAutoEnd.value = (await load("free_switch_auto_end", "1")) === "1";
  notifySystem.value = (await load("notify_system", "1")) === "1";
  notifySettle.value = (await load("notify_settle", "1")) === "1";
  notifyDue.value = (await load("notify_due", "1")) === "1";

  aiEnabled.value = (await load("ai_enabled", "1")) === "1";
  aiTone.value = await load("ai_tone", "academic");
  aiToneCustom.value = await load("ai_tone_custom", "");
  aiIntensity.value = Number(await load("ai_tone_intensity", "3"));
  aiConnCheckedAt.value = await load("ai_connection_checked_at", "");
  // 加载 chat / embedding profile 池子 (Phase 3 重构)
  try { await profileStore.loadAll(); } catch (e) { console.warn("[settings] load AI profiles failed", e); }

  statsEnabled.value = (await load("privacy_anonymous_stats", "0")) === "1";
  expMood.value = (await load("exp_mood_checkin", "1")) === "1";
  expPersona.value = (await load("exp_persona", "0")) === "1";
  expBadges.value = (await load("exp_badges", "1")) === "1";
});

// ---------- Data ----------
const exportResult = ref("");
const showExportModal = ref(false);
const showDangerModal = ref(false);
const showClearChatModal = ref(false);
const showAIPrivacy = ref(false);
const showAIPayload = ref(false);

async function onClearChat() {
  showClearChatModal.value = false;
  await chat.clearAllConversations();
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

        <!-- 🌀 自由模式偏好(v1.2.3) -->
        <div class="fl-set-subhead">🌀 自由计时模式</div>

        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">默认计时模式</div>
            <div class="fl-set-desc">启动专注时默认使用哪种模式 · 也可在焦点卡 🍅/🌀 临时切换</div>
          </div>
          <div class="fl-set-control">
            <div class="fl-segmented">
              <button class="fl-seg-btn" :class="{ 'is-active': freeDefaultMode === 'pomodoro' }" @click="setFreeDefault('pomodoro')">🍅 番茄</button>
              <button class="fl-seg-btn" :class="{ 'is-active': freeDefaultMode === 'free' }" @click="setFreeDefault('free')">🌀 自由</button>
              <button class="fl-seg-btn" :class="{ 'is-active': freeDefaultMode === 'last' }" @click="setFreeDefault('last')">上次</button>
            </div>
          </div>
        </div>

        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">最小记录时长</div>
            <div class="fl-set-desc">自由模式下,短于此时长的会话不计入专注时长 · 避免误触污染数据</div>
          </div>
          <div class="fl-set-control">
            <div class="fl-slider-row">
              <input
                type="range" min="1" max="15" step="1"
                :value="freeMinRecord"
                @input="setFreeMinRecord(($event.target as HTMLInputElement).value)"
              />
              <span class="fl-slider-val">{{ freeMinRecord }}m</span>
            </div>
          </div>
        </div>

        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">超时柔提醒</div>
            <div class="fl-set-desc">自由模式连续专注超过此时长时,轻微呼吸提示可以休息 · 不强制打断</div>
          </div>
          <div class="fl-set-control fl-set-control-dual">
            <div class="fl-slider-row">
              <input
                type="range" min="30" max="180" step="10"
                :value="freeSoftRemind"
                :disabled="!freeSoftRemindOn"
                @input="setFreeSoftRemind(($event.target as HTMLInputElement).value)"
              />
              <span class="fl-slider-val">{{ freeSoftRemind }}m</span>
            </div>
            <button class="fl-toggle" :class="{ 'is-on': freeSoftRemindOn }" @click="toggleFreeFlag('soft_remind_on')"><span class="fl-toggle-dot" /></button>
          </div>
        </div>

        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">暂停计为中断</div>
            <div class="fl-set-desc">开启后,自由模式中按暂停会结束当前会话并记录 · 关闭则仅暂停计时,可恢复</div>
          </div>
          <div class="fl-set-control">
            <button class="fl-toggle" :class="{ 'is-on': freePauseAsBreak }" @click="toggleFreeFlag('pause_as_break')"><span class="fl-toggle-dot" /></button>
          </div>
        </div>

        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">切换任务自动结束当前会话</div>
            <div class="fl-set-desc">自由模式中切换任务时,先结束当前会话再开启新会话 · 保证每段专注归属清晰</div>
          </div>
          <div class="fl-set-control">
            <button class="fl-toggle" :class="{ 'is-on': freeSwitchAutoEnd }" @click="toggleFreeFlag('switch_auto_end')"><span class="fl-toggle-dot" /></button>
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

        <div class="fl-ai-banner">
          <div>
            <strong>AI 所有能力默认本地优先，只有调用外部模型时才会出网。</strong>
            <div class="fl-set-desc">你可以在这里统一控制开关、供应商协议、语气风格与输出强度。</div>
          </div>
          <button class="fl-set-btn fl-set-btn-ghost" @click="showAIPrivacy = true">查看隐私说明</button>
        </div>

        <div class="fl-set-subhead">A. 全局开关</div>
        <div class="fl-set-row">
          <div class="fl-set-info">
            <div class="fl-set-label">启用 AI 助手</div>
            <div class="fl-set-desc">关闭后，除“测试连接”外的 AI 功能将被统一拦截。</div>
          </div>
          <div class="fl-set-control">
            <button class="fl-toggle" :class="{ 'is-on': aiEnabled }" @click="toggleAiEnabled"><span class="fl-toggle-dot" /></button>
          </div>
        </div>

        <div class="fl-set-subhead">B. Chat Profile 池</div>
        <div class="fl-set-desc" style="margin-bottom:var(--sp-2)">保存多份 chat 厂商连接,激活的那条会被所有 AI 调用使用。</div>

        <div class="fl-profile-list">
          <div
            v-for="p in profileStore.chatProfiles"
            :key="p.id"
            class="fl-profile-card"
            :class="{ 'is-active': profileStore.activeChatId === p.id }"
          >
            <div class="fl-profile-card-head">
              <strong>{{ p.name }}</strong>
              <span v-if="profileStore.activeChatId === p.id" class="fl-profile-active-tag">当前激活</span>
              <span class="fl-profile-provider">{{ p.provider }}</span>
            </div>
            <div class="fl-profile-card-meta">
              <span>{{ p.baseUrl }}</span>
              <span>fast: {{ p.modelFast || "—" }}</span>
              <span v-if="p.modelStrong && p.modelStrong !== p.modelFast">strong: {{ p.modelStrong }}</span>
            </div>
            <div class="fl-profile-card-actions">
              <button v-if="profileStore.activeChatId !== p.id" class="fl-set-btn fl-set-btn-ghost" @click="activateChatProfile(p.id)">激活</button>
              <button class="fl-set-btn fl-set-btn-ghost" @click="startEditChatProfile(p)">编辑</button>
              <button class="fl-set-btn fl-set-btn-ghost fl-profile-delete" @click="deleteChatProfile(p.id, p.name)">删除</button>
            </div>
          </div>
          <button v-if="!chatEditMode" class="fl-set-btn fl-profile-add" @click="startNewChatProfile">+ 新建 chat profile</button>
        </div>

        <div v-if="chatEditMode" class="fl-profile-form">
          <div class="fl-profile-form-head">{{ chatEditMode === "new" ? "新建" : "编辑" }} Chat Profile</div>
          <div class="fl-ai-grid">
            <label class="fl-ai-field fl-ai-field-wide">
              <span>Profile 名称</span>
              <input v-model="chatDraft.name" class="fl-ai-input" type="text" placeholder="例如: 工作号 GPT / 实验室 DeepSeek" />
            </label>
            <label class="fl-ai-field">
              <span>供应商</span>
              <select v-model="chatDraft.provider" class="fl-ai-input" @change="onProviderChange">
                <option value="openai">OpenAI</option>
                <option value="claude">Claude / Anthropic</option>
                <option value="deepseek">DeepSeek</option>
                <option value="moonshot">Moonshot</option>
                <option value="qwen">通义千问</option>
                <option value="gemini">Gemini</option>
                <option value="ollama">Ollama 本地</option>
                <option value="custom">自定义供应商</option>
              </select>
            </label>
            <label v-if="chatDraft.provider === 'custom'" class="fl-ai-field">
              <span>协议格式</span>
              <select v-model="chatDraft.apiFormat" class="fl-ai-input">
                <option value="openai">OpenAI 格式</option>
                <option value="claude">Claude 格式</option>
              </select>
            </label>
            <label class="fl-ai-field" :class="{ 'fl-ai-field-wide': chatDraft.provider !== 'custom' }">
              <span>Base URL</span>
              <input v-model="chatDraft.baseUrl" class="fl-ai-input" type="text" :placeholder="providerPresets[chatDraft.provider]?.baseUrl || 'https://api.example.com'" />
            </label>
            <label v-if="chatDraft.provider !== 'ollama'" class="fl-ai-field fl-ai-field-wide">
              <span>API Key</span>
              <input v-model="chatDraft.apiKey" class="fl-ai-input" type="password" placeholder="sk-... / claude-..." />
            </label>
            <label class="fl-ai-field">
              <span>Fast Model · 用于轻量场景</span>
              <div class="fl-ai-model-row">
                <select v-if="fetchedModels.length || providerPresets[chatDraft.provider]?.models?.length" v-model="chatDraft.modelFast" class="fl-ai-input fl-ai-model-select">
                  <option v-for="m in (fetchedModels.length ? fetchedModels : providerPresets[chatDraft.provider]?.models ?? [])" :key="m" :value="m">{{ m }}</option>
                </select>
                <input v-else v-model="chatDraft.modelFast" class="fl-ai-input fl-ai-model-select" type="text" placeholder="如 gpt-4o-mini" />
                <button class="fl-set-btn fl-set-btn-ghost fl-ai-fetch-btn" :disabled="modelFetching" @click="onFetchModels">
                  {{ modelFetching ? "获取中…" : "获取模型" }}
                </button>
              </div>
              <div v-if="modelFetchError" class="fl-ai-fetch-error">{{ modelFetchError }}</div>
            </label>
            <label class="fl-ai-field">
              <span>Strong Model · 用于重量场景 (可选)</span>
              <select v-if="fetchedModels.length || providerPresets[chatDraft.provider]?.models?.length" v-model="chatDraft.modelStrong" class="fl-ai-input">
                <option v-for="m in (fetchedModels.length ? fetchedModels : providerPresets[chatDraft.provider]?.models ?? [])" :key="m" :value="m">{{ m }}</option>
              </select>
              <input v-else v-model="chatDraft.modelStrong" class="fl-ai-input" type="text" placeholder="留空则与 Fast 相同" />
            </label>
            <div v-if="fetchedModels.length" class="fl-ai-field">
              <span>聊天可选模型 · 勾选后出现在聊天下拉</span>
              <div class="fl-ai-model-checklist">
                <label v-for="m in fetchedModels" :key="m" class="fl-ai-model-check">
                  <input
                    type="checkbox"
                    :checked="chatDraft.selectedModels.includes(m)"
                    @change="toggleSelectedModel(m)"
                  />
                  <span>{{ m }}</span>
                </label>
              </div>
            </div>
          </div>
          <div class="fl-ai-actions">
            <button class="fl-set-btn" @click="saveChatProfile">{{ chatEditMode === "new" ? "新建" : "保存修改" }}</button>
            <button class="fl-set-btn fl-set-btn-ghost" @click="cancelChatEdit">取消</button>
          </div>
        </div>

        <div class="fl-ai-status-row">
          <div class="fl-ai-status" :class="[`is-${aiConnStatus || 'idle'}`]">
            <span class="dot" />
            <span>
              {{ aiConnStatus === 'success' ? '最近一次连接测试成功' : aiConnStatus === 'fail' ? '最近一次连接测试失败' : '尚未测试连接' }}
            </span>
          </div>
          <div v-if="aiConnCheckedAt" class="fl-ai-status-time">{{ aiConnCheckedAt }}</div>
        </div>

        <div class="fl-ai-actions">
          <button class="fl-set-btn fl-set-btn-ghost" @click="onTestAI">测试激活的 profile 连接</button>
          <button class="fl-set-btn fl-set-btn-ghost" @click="showAIPayload = true">查看发送的数据示例</button>
        </div>
        <div v-if="aiTestResult" class="fl-ai-result">{{ aiTestResult }}</div>

        <div class="fl-set-subhead">C. Embedding Profile 池</div>
        <div class="fl-set-desc" style="margin-bottom:var(--sp-2)">为灵感图谱语义索引提供 embedding,与 chat 完全独立,可单独配置厂商。</div>

        <div class="fl-profile-list">
          <div
            v-for="p in profileStore.embeddingProfiles"
            :key="p.id"
            class="fl-profile-card"
            :class="{ 'is-active': profileStore.activeEmbeddingId === p.id }"
          >
            <div class="fl-profile-card-head">
              <strong>{{ p.name }}</strong>
              <span v-if="profileStore.activeEmbeddingId === p.id" class="fl-profile-active-tag">当前激活</span>
            </div>
            <div class="fl-profile-card-meta">
              <span>{{ p.baseUrl }}</span>
              <span>{{ p.model || "—" }}</span>
            </div>
            <div class="fl-profile-card-actions">
              <button v-if="profileStore.activeEmbeddingId !== p.id" class="fl-set-btn fl-set-btn-ghost" @click="activateEmbeddingProfile(p.id)">激活</button>
              <button class="fl-set-btn fl-set-btn-ghost" @click="startEditEmbeddingProfile(p)">编辑</button>
              <button class="fl-set-btn fl-set-btn-ghost fl-profile-delete" @click="deleteEmbeddingProfile(p.id, p.name)">删除</button>
            </div>
          </div>
          <button v-if="!embeddingEditMode" class="fl-set-btn fl-profile-add" @click="startNewEmbeddingProfile">+ 新建 embedding profile</button>
        </div>

        <div v-if="embeddingEditMode" class="fl-profile-form">
          <div class="fl-profile-form-head">{{ embeddingEditMode === "new" ? "新建" : "编辑" }} Embedding Profile</div>
          <div class="fl-ai-grid">
            <label class="fl-ai-field fl-ai-field-wide">
              <span>Profile 名称</span>
              <input v-model="embeddingDraft.name" class="fl-ai-input" type="text" placeholder="例如: OpenAI Embedding" />
            </label>
            <label class="fl-ai-field fl-ai-field-wide">
              <span>Embedding Base URL</span>
              <input v-model="embeddingDraft.baseUrl" class="fl-ai-input" type="text" placeholder="https://api.openai.com" />
            </label>
            <label class="fl-ai-field">
              <span>Embedding API Key</span>
              <input v-model="embeddingDraft.apiKey" class="fl-ai-input" type="password" placeholder="sk-..." />
            </label>
            <label class="fl-ai-field">
              <span>Embedding Model</span>
              <div class="fl-ai-model-row">
                <select v-if="embeddingFetchedModels.length" v-model="embeddingDraft.model" class="fl-ai-input fl-ai-model-select">
                  <option v-for="m in embeddingFetchedModels" :key="m" :value="m">{{ m }}</option>
                </select>
                <input v-else v-model="embeddingDraft.model" class="fl-ai-input fl-ai-model-select" type="text" placeholder="text-embedding-3-small" />
                <button class="fl-set-btn fl-set-btn-ghost fl-ai-fetch-btn" :disabled="embeddingModelFetching" @click="onFetchEmbeddingModels">
                  {{ embeddingModelFetching ? "获取中…" : "获取模型" }}
                </button>
              </div>
              <div v-if="embeddingModelFetchError" class="fl-ai-fetch-error">{{ embeddingModelFetchError }}</div>
            </label>
          </div>
          <div class="fl-ai-actions">
            <button class="fl-set-btn" @click="saveEmbeddingProfile">{{ embeddingEditMode === "new" ? "新建" : "保存修改" }}</button>
            <button class="fl-set-btn fl-set-btn-ghost" @click="cancelEmbeddingEdit">取消</button>
          </div>
        </div>

        <div class="fl-set-subhead">D. 语气风格</div>
        <div class="fl-tone-grid">
          <button
            v-for="tone in toneOptions"
            :key="tone.id"
            class="fl-tone-card"
            :class="{ 'is-selected': aiTone === tone.id }"
            @click="aiTone = tone.id"
          >
            <div class="fl-tone-card__head">
              <span class="fl-tone-card__icon">{{ tone.icon }}</span>
              <strong>{{ tone.name }}</strong>
            </div>
            <div class="fl-tone-card__desc">{{ tone.desc }}</div>
          </button>
        </div>
        <label v-if="aiTone === 'custom'" class="fl-ai-field">
          <span>自定义风格提示词</span>
          <textarea v-model="aiToneCustom" class="fl-ai-input fl-ai-textarea" rows="4" placeholder="例如：像一个毒舌但关心我的导师，说话短句、直接指出问题、最后给一个具体行动建议" />
        </label>

        <div class="fl-set-subhead">D. 输出强度</div>
        <div class="fl-ai-intensity-card">
          <div class="fl-ai-intensity-head">
            <strong>当前强度：Lv.{{ aiIntensity }}</strong>
            <span>{{ aiIntensity <= 2 ? '更短更克制' : aiIntensity === 3 ? '平衡模式' : '更详细更鼓励' }}</span>
          </div>
          <input v-model="aiIntensity" class="fl-ai-intensity-slider" type="range" min="1" max="5" step="1" />
          <div class="fl-ai-intensity-scale">
            <span>少话 / 简洁</span>
            <span>多鼓励 / 详细</span>
          </div>
        </div>

        <div class="fl-set-subhead">E. 用量参考</div>
        <div class="fl-cost-grid">
          <div class="fl-cost-card">
            <strong>L1 · 基础能力</strong>
            <span>任务拆解 / 四象限判断 / 快速改写</span>
            <p>低消耗，高频使用，建议默认开放。</p>
          </div>
          <div class="fl-cost-card">
            <strong>L2 · 分析能力</strong>
            <span>每日建议 / 日结算叙事</span>
            <p>中消耗，中频使用，适合每天 1-3 次。</p>
          </div>
          <div class="fl-cost-card">
            <strong>L3 · 规划能力</strong>
            <span>周总结 / 长文本复盘</span>
            <p>高消耗，低频使用，建议每周集中触发。</p>
          </div>
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
            <button class="fl-toggle" :class="{ 'is-on': statsEnabled }" @click="onToggleSetting('stats')"><span class="fl-toggle-dot" /></button>
          </div>
        </div>
        <div class="fl-set-row fl-set-row-danger">
          <div class="fl-set-info">
            <div class="fl-set-label">清空所有 AI 对话</div>
            <div class="fl-set-desc">删除全部聊天记录与对话历史，不可撤销</div>
          </div>
          <div class="fl-set-control">
            <button class="fl-set-btn fl-set-btn-danger-outline" @click="showClearChatModal = true">清空</button>
          </div>
        </div>
      </div>

      <!-- 快捷键 -->
      <div v-if="activeSection === 'shortcuts'" class="fl-set-group">
        <h2>快捷键</h2>
        <ShortcutSettingsPanel />
      </div>

      <!-- 实验功能 -->
      <div v-if="activeSection === 'experiment'" class="fl-set-group">
        <h2>实验功能</h2>
        <p class="fl-exp-desc">以下功能为实验性质，默认关闭。开启后可在对应页面体验。</p>
        <div class="fl-exp-list">
          <div class="fl-exp-card">
            <div class="fl-exp-head">
              <span>🌤 心情打卡</span>
              <button class="fl-toggle" :class="{ 'is-on': expMood }" @click="onToggleSetting('mood')"><span class="fl-toggle-dot" /></button>
            </div>
            <div class="fl-exp-body">早晨意图档位 + 晚间情绪记录</div>
          </div>
          <div class="fl-exp-card">
            <div class="fl-exp-head">
              <span>🧬 科研人格图鉴</span>
              <button class="fl-toggle" :class="{ 'is-on': expPersona }" @click="onToggleSetting('persona')"><span class="fl-toggle-dot" /></button>
            </div>
            <div class="fl-exp-body">30 型人格 · 7 天孵化 · 社交分享</div>
          </div>
          <div class="fl-exp-card">
            <div class="fl-exp-head">
              <span>🏆 成就徽章</span>
              <button class="fl-toggle" :class="{ 'is-on': expBadges }" @click="onToggleSetting('badges')"><span class="fl-toggle-dot" /></button>
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
  <AIPayloadModal :visible="showAIPayload" @close="showAIPayload = false" />
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
  <DangerConfirmModal
    :visible="showClearChatModal"
    title="清空所有 AI 对话"
    description="此操作不可撤销，将永久删除全部聊天记录"
    :items="[{ label: '对话 & 消息', count: '全部' }]"
    confirm-text="DELETE"
    @close="showClearChatModal = false"
    @confirmed="onClearChat"
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

/* 子分区标题 */
.fl-set-subhead {
  font-size: var(--fs-12); font-weight: var(--fw-semibold);
  color: var(--color-text-secondary); letter-spacing: 0.3px;
  margin: var(--sp-5) 0 var(--sp-2);
  padding-top: var(--sp-4); border-top: 1px dashed var(--color-divider);
}

/* Slider */
.fl-slider-row {
  display: flex; align-items: center; gap: var(--sp-2);
}
.fl-slider-row input[type="range"] {
  width: 120px; height: 4px; accent-color: var(--color-primary);
  cursor: pointer;
}
.fl-slider-row input[type="range"]:disabled { opacity: 0.4; cursor: not-allowed; }
.fl-slider-val {
  font-family: var(--font-mono); font-size: var(--fs-12);
  color: var(--color-text-primary); min-width: 34px; text-align: right;
}
.fl-set-control-dual {
  display: flex; align-items: center; gap: var(--sp-3);
}

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

/* AI */
.fl-ai-banner {
  display: flex; align-items: center; justify-content: space-between; gap: var(--sp-3);
  padding: var(--sp-3) var(--sp-4); border: 1px solid var(--color-border);
  background: var(--color-bg-subtle); border-radius: var(--r-md);
}

/* AI Profile Pool */
.fl-profile-list { display: flex; flex-direction: column; gap: var(--sp-2); }
.fl-profile-card {
  border: 1px solid var(--color-border); border-radius: var(--r-md);
  padding: var(--sp-3); background: var(--color-bg-subtle);
  border-left: 3px solid transparent;
}
.fl-profile-card.is-active {
  border-left-color: var(--color-primary);
  background: var(--color-primary-soft);
}
.fl-profile-card-head {
  display: flex; align-items: center; gap: var(--sp-2);
  font-size: var(--fs-14); margin-bottom: 4px;
}
.fl-profile-active-tag {
  font-size: 10px; padding: 2px 6px; border-radius: 8px;
  background: var(--color-primary); color: #fff;
}
.fl-profile-provider {
  font-size: 11px; padding: 2px 6px; border-radius: 4px;
  background: var(--color-bg-elevated); color: var(--color-text-secondary);
  margin-left: auto;
}
.fl-profile-card-meta {
  display: flex; flex-wrap: wrap; gap: var(--sp-3);
  font-size: 11px; color: var(--color-text-muted);
  margin-bottom: var(--sp-2); font-family: var(--font-mono, monospace);
}
.fl-profile-card-actions { display: flex; gap: var(--sp-2); flex-wrap: wrap; }
.fl-profile-delete:hover { color: #ef4444; border-color: #ef4444; }
.fl-profile-add {
  border-style: dashed; color: var(--color-text-muted); padding: var(--sp-2) var(--sp-4);
}
.fl-profile-form {
  margin-top: var(--sp-3); padding: var(--sp-3); border: 1px solid var(--color-primary);
  border-radius: var(--r-md); background: var(--color-bg-elevated);
  display: flex; flex-direction: column; gap: var(--sp-3);
}
.fl-profile-form-head { font-weight: var(--fw-semibold); font-size: var(--fs-14); }

.fl-ai-grid {
  display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--sp-3);
}
.fl-ai-field { display: flex; flex-direction: column; gap: var(--sp-1); }
.fl-ai-field-wide { grid-column: 1 / -1; }
.fl-ai-field span { font-size: var(--fs-12); color: var(--color-text-muted); }
.fl-ai-input {
  padding: var(--sp-2) var(--sp-3); border: 1px solid var(--color-border);
  border-radius: var(--r-md); background: var(--color-bg-subtle);
  color: var(--color-text-primary); font-size: var(--fs-14); font-family: inherit; outline: none;
}
.fl-ai-input:focus { border-color: var(--color-primary); }
.fl-ai-model-row { display: flex; gap: var(--sp-2); align-items: center; }
.fl-ai-model-select { flex: 1; min-width: 0; }
.fl-ai-fetch-btn { white-space: nowrap; flex-shrink: 0; font-size: var(--fs-12); padding: var(--sp-2) var(--sp-3); }
.fl-ai-model-checklist {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  max-height: 160px;
  overflow-y: auto;
  padding: var(--sp-2);
  border: 1px solid var(--color-border);
  border-radius: var(--r-sm);
  background: var(--color-bg);
}
.fl-ai-model-check {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
  cursor: pointer;
  user-select: none;
}
.fl-ai-model-check input[type="checkbox"] {
  accent-color: var(--color-primary);
}
.fl-ai-fetch-error { font-size: var(--fs-12); color: var(--color-danger, #e53e3e); margin-top: var(--sp-1); }
.fl-ai-textarea { resize: vertical; min-height: 92px; }
.fl-ai-actions { display: flex; flex-wrap: wrap; gap: var(--sp-2); }
.fl-ai-result {
  font-size: var(--fs-12); color: var(--color-text-secondary);
  padding: var(--sp-2); background: var(--color-bg-subtle); border-radius: var(--r-sm);
}
.fl-ai-status-row {
  display: flex; align-items: center; justify-content: space-between; gap: var(--sp-3);
}
.fl-ai-status {
  display: inline-flex; align-items: center; gap: 8px;
  padding: 8px 10px; border-radius: var(--r-md); font-size: var(--fs-12);
  background: var(--color-bg-subtle); color: var(--color-text-secondary);
}
.fl-ai-status .dot {
  width: 8px; height: 8px; border-radius: 50%; background: var(--color-text-muted);
}
.fl-ai-status.is-success .dot { background: #22c55e; }
.fl-ai-status.is-fail .dot { background: #ef4444; }
.fl-ai-status.is-idle .dot { background: #94a3b8; }
.fl-ai-status-time { font-size: var(--fs-12); color: var(--color-text-muted); }
.fl-tone-grid {
  display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--sp-3);
}
.fl-tone-card {
  padding: var(--sp-3); border: 1px solid var(--color-border); border-radius: var(--r-md);
  background: transparent; text-align: left; cursor: pointer; transition: all var(--dur-fast);
}
.fl-tone-card:hover { border-color: var(--color-primary); background: var(--color-primary-soft); }
.fl-tone-card.is-selected { border-color: var(--color-primary); background: var(--color-primary-soft); box-shadow: var(--shadow-card); }
.fl-tone-card__head { display: flex; align-items: center; gap: var(--sp-2); margin-bottom: 6px; }
.fl-tone-card__icon { font-size: 18px; }
.fl-tone-card__desc { font-size: var(--fs-12); color: var(--color-text-muted); line-height: 1.5; }
.fl-ai-intensity-card {
  padding: var(--sp-4); border: 1px solid var(--color-border); border-radius: var(--r-md);
  background: var(--color-bg-subtle); display: flex; flex-direction: column; gap: var(--sp-3);
}
.fl-ai-intensity-head {
  display: flex; align-items: center; justify-content: space-between; gap: var(--sp-3);
  font-size: var(--fs-12); color: var(--color-text-secondary);
}
.fl-ai-intensity-slider { width: 100%; accent-color: var(--color-primary); }
.fl-ai-intensity-scale {
  display: flex; align-items: center; justify-content: space-between;
  font-size: var(--fs-12); color: var(--color-text-muted);
}
.fl-cost-grid {
  display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--sp-3);
}
.fl-cost-card {
  padding: var(--sp-3); border: 1px solid var(--color-border); border-radius: var(--r-md);
  background: var(--color-bg-subtle); display: flex; flex-direction: column; gap: 6px;
}
.fl-cost-card strong { font-size: var(--fs-14); }
.fl-cost-card span,
.fl-cost-card p {
  margin: 0; font-size: var(--fs-12); color: var(--color-text-muted); line-height: 1.5;
}

@media (max-width: 720px) {
  .fl-ai-banner,
  .fl-ai-status-row,
  .fl-ai-intensity-head { flex-direction: column; align-items: stretch; }
  .fl-ai-grid,
  .fl-tone-grid,
  .fl-cost-grid { grid-template-columns: 1fr; }
}

.fl-set-btn {
  padding: var(--sp-2) var(--sp-4); border-radius: var(--r-md);
  border: 1px solid var(--color-border); background: var(--color-bg-elevated);
  color: var(--color-text-primary); font-size: var(--fs-12); cursor: pointer;
}
.fl-set-btn:hover { border-color: var(--color-primary); color: var(--color-primary); }
.fl-set-btn-ghost { background: transparent; color: var(--color-text-secondary); }
.fl-set-btn-danger-outline { background: transparent; color: #ef4444; border-color: #ef4444; }
.fl-set-btn-danger-outline:hover { background: color-mix(in srgb, #ef4444 10%, transparent); }

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
