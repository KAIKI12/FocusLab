<script setup lang="ts">
/**
 * PersonaView · 科研人格图鉴 — 对齐 prototype/screens/persona-card.html + persona-hatch.html。
 * 8 门别 Tab + Gallery + 分享卡预览 + 7 天孵化进度。
 */

import { computed, onMounted, ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";

interface Persona {
  id: string;
  emoji: string;
  name: string;
  code: string;
  gate: string;
  quote: string;
  dims: { label: string; stars: number; desc: string }[];
}

const GATES = [
  { id: "night", emoji: "🌙", label: "夜型生物门", gradient: "linear-gradient(135deg, #1a0a2e, #6C5CE7)" },
  { id: "morning", emoji: "☀️", label: "晨型觉醒门", gradient: "linear-gradient(135deg, #2d1810, #FF6B6B)" },
  { id: "ddl", emoji: "⚡", label: "DDL 战士门", gradient: "linear-gradient(135deg, #2d1f00, #FAAD14)" },
  { id: "zen", emoji: "🧘", label: "佛系修仙门", gradient: "linear-gradient(135deg, #0a1a2e, #7FB3FF)" },
  { id: "sprint", emoji: "🔥", label: "猛冲门", gradient: "linear-gradient(135deg, #2d0a0a, #FF4757)" },
  { id: "restart", emoji: "💀", label: "崩溃重启门", gradient: "linear-gradient(135deg, #1a1a2e, #B87FFF)" },
  { id: "strategy", emoji: "🗺️", label: "战略门", gradient: "linear-gradient(135deg, #0a2d1a, #27AE60)" },
  { id: "chaos", emoji: "🎲", label: "混沌门", gradient: "linear-gradient(135deg, #2d0a1a, #FF6B9D)" },
];

const PERSONAS: Persona[] = [
  {
    id: "bat", emoji: "🦇", name: "实验室蝙蝠", code: "BATS", gate: "night",
    quote: "白天是用来充电的，晚上才是属于我的战场。",
    dims: [
      { label: "专注力", stars: 4, desc: "夜间极强" },
      { label: "规律性", stars: 2, desc: "昼夜颠倒" },
      { label: "爆发力", stars: 5, desc: "凌晨巅峰" },
      { label: "社交度", stars: 1, desc: "独行侠" },
      { label: "抗压性", stars: 3, desc: "自洽型" },
    ],
  },
  {
    id: "rooster", emoji: "🐓", name: "晨间公鸡", code: "ROST", gate: "morning",
    quote: "六点的朝阳和咖啡，是一天最好的开场白。",
    dims: [
      { label: "专注力", stars: 4, desc: "上午巅峰" },
      { label: "规律性", stars: 5, desc: "极度规律" },
      { label: "爆发力", stars: 3, desc: "稳定输出" },
      { label: "社交度", stars: 3, desc: "适度社交" },
      { label: "抗压性", stars: 4, desc: "从容型" },
    ],
  },
  {
    id: "ddl-warrior", emoji: "⚡", name: "DDL 战神", code: "DDLW", gate: "ddl",
    quote: "截止日期是第一生产力，这话不接受反驳。",
    dims: [
      { label: "专注力", stars: 5, desc: "临死线爆发" },
      { label: "规律性", stars: 1, desc: "混乱善良" },
      { label: "爆发力", stars: 5, desc: "核弹级" },
      { label: "社交度", stars: 2, desc: "忙到消失" },
      { label: "抗压性", stars: 5, desc: "刀尖跳舞" },
    ],
  },
  {
    id: "zen-master", emoji: "🧘", name: "佛系大师", code: "ZENM", gate: "zen",
    quote: "做得完做不完，都是缘。",
    dims: [
      { label: "专注力", stars: 3, desc: "随缘专注" },
      { label: "规律性", stars: 3, desc: "自然而然" },
      { label: "爆发力", stars: 2, desc: "不急不躁" },
      { label: "社交度", stars: 4, desc: "广结善缘" },
      { label: "抗压性", stars: 5, desc: "万事皆空" },
    ],
  },
  {
    id: "phoenix", emoji: "🔥", name: "浴火凤凰", code: "PHNX", gate: "sprint",
    quote: "要么不做，要做就做到极致。",
    dims: [
      { label: "专注力", stars: 5, desc: "沉浸忘我" },
      { label: "规律性", stars: 2, desc: "间歇爆发" },
      { label: "爆发力", stars: 5, desc: "惊人峰值" },
      { label: "社交度", stars: 1, desc: "闭关修炼" },
      { label: "抗压性", stars: 4, desc: "越战越勇" },
    ],
  },
];

const activeGate = ref("night");
const selectedPersona = ref<Persona | null>(PERSONAS[0]);

const filteredPersonas = computed(() =>
  PERSONAS.filter((p) => p.gate === activeGate.value),
);

const activeGateInfo = computed(() =>
  GATES.find((g) => g.id === activeGate.value)!,
);

// 7 天孵化 — 基于首次访问人格页日期推算;存储在 Tauri settings KV(key=persona_hatch_start)
const HATCH_STORAGE_KEY = "persona_hatch_start";
const hatchStart = ref<string | null>(null);

function todayLocal(): string {
  const d = new Date();
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const dd = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${dd}`;
}

function diffDaysInclusive(startStr: string): number {
  const start = new Date(`${startStr}T00:00:00`);
  const today = new Date(`${todayLocal()}T00:00:00`);
  const ms = today.getTime() - start.getTime();
  return Math.floor(ms / 86_400_000) + 1; // 起始日算 Day 1
}

const hatchDay = computed<number>(() => {
  if (!hatchStart.value) return 1; // 未加载完前按 Day 1 兜底(视觉上显示蛋壳)
  return Math.min(Math.max(diffDaysInclusive(hatchStart.value), 1), 7);
});

onMounted(async () => {
  try {
    const stored = await invokeCmd<string | null>("get_setting", {
      key: HATCH_STORAGE_KEY,
    });
    if (stored) {
      hatchStart.value = stored;
    } else {
      const today = todayLocal();
      await invokeCmd("set_setting", { key: HATCH_STORAGE_KEY, value: today });
      hatchStart.value = today;
    }
  } catch {
    // 后端不可用时降级为"今天就是 Day 1",不阻塞页面
    hatchStart.value = todayLocal();
  }
});

const HATCH_STAGES = [
  { day: 1, emoji: "🥚", label: "蛋壳期" },
  { day: 3, emoji: "🐣", label: "破壳期" },
  { day: 5, emoji: "🐥", label: "成长期" },
  { day: 7, emoji: "🦇", label: "成型!" },
];

function starStr(n: number): string {
  return "★".repeat(n) + "☆".repeat(5 - n);
}
</script>

<template>
  <section class="fl-persona">
    <header>
      <h1>🧬 科研人格图鉴</h1>
      <p class="fl-persona-sub">30 型 · 9 门 · 基于你的行为数据自动匹配</p>
    </header>

    <!-- 7 天孵化进度 -->
    <div class="fl-hatch">
      <div class="fl-hatch-title">人格孵化中…</div>
      <div class="fl-hatch-timeline">
        <div
          v-for="stage in HATCH_STAGES" :key="stage.day"
          class="fl-hatch-stage"
          :class="{ 'is-done': hatchDay >= stage.day, 'is-current': hatchDay >= stage.day && (HATCH_STAGES.find(s => s.day > stage.day)?.day ?? 999) > hatchDay }"
        >
          <span class="fl-hatch-emoji">{{ stage.emoji }}</span>
          <span class="fl-hatch-label">Day {{ stage.day }}</span>
        </div>
      </div>
      <div class="fl-hatch-hint">
        <template v-if="hatchDay >= 7">🎉 已完成孵化 · 完整人格已解锁</template>
        <template v-else>再使用 {{ 7 - hatchDay }} 天解锁完整人格</template>
      </div>
    </div>

    <!-- 门别 Tab -->
    <div class="fl-gate-tabs">
      <button
        v-for="g in GATES" :key="g.id"
        class="fl-gate-tab"
        :class="{ 'is-active': activeGate === g.id }"
        @click="activeGate = g.id; selectedPersona = filteredPersonas[0] ?? null"
      >
        {{ g.emoji }} {{ g.label }}
      </button>
    </div>

    <div class="fl-persona-main">
      <!-- Gallery -->
      <div class="fl-gallery">
        <button
          v-for="p in filteredPersonas" :key="p.id"
          class="fl-gallery-item"
          :class="{ 'is-selected': selectedPersona?.id === p.id }"
          @click="selectedPersona = p"
        >
          <span class="fl-gi-emoji">{{ p.emoji }}</span>
          <span class="fl-gi-name">{{ p.name }}</span>
        </button>
        <div v-if="!filteredPersonas.length" class="fl-gallery-empty">
          该门暂无人格数据
        </div>
      </div>

      <!-- 分享卡预览 -->
      <div v-if="selectedPersona" class="fl-share-card" :style="{ background: activeGateInfo.gradient }">
        <div class="fl-sc-header">FOCUSLAB · 科研人格</div>
        <div class="fl-sc-emoji">{{ selectedPersona.emoji }}</div>
        <div class="fl-sc-name">{{ selectedPersona.name }}</div>
        <div class="fl-sc-code">{{ selectedPersona.code }} · {{ activeGateInfo.label }}</div>
        <div class="fl-sc-quote">"{{ selectedPersona.quote }}"</div>
        <div class="fl-sc-dims">
          <div v-for="d in selectedPersona.dims" :key="d.label" class="fl-sc-dim">
            <span class="fl-sc-dim-label">{{ d.label }}</span>
            <span class="fl-sc-dim-stars">{{ starStr(d.stars) }}</span>
            <span class="fl-sc-dim-desc">{{ d.desc }}</span>
          </div>
        </div>
        <div class="fl-sc-footer">
          <span>focuslab.app</span>
          <div class="fl-sc-qr" />
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.fl-persona {
  max-width: 860px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--sp-5);
}
.fl-persona h1 { font-size: var(--fs-24); font-weight: var(--fw-semibold); margin: 0; }
.fl-persona-sub { font-size: var(--fs-14); color: var(--color-text-secondary); margin: var(--sp-1) 0 0; }

/* 孵化 */
.fl-hatch {
  padding: var(--sp-5); text-align: center;
  background: radial-gradient(circle at 50% 45%, color-mix(in srgb, var(--color-gold, #FAAD14) 15%, transparent), transparent 60%), var(--color-bg-elevated);
  border: 1px solid var(--color-border); border-radius: var(--r-lg);
}
.fl-hatch-title { font-size: var(--fs-16); font-weight: var(--fw-semibold); margin-bottom: var(--sp-4); }
.fl-hatch-timeline { display: flex; justify-content: center; gap: var(--sp-6); margin-bottom: var(--sp-3); }
.fl-hatch-stage { display: flex; flex-direction: column; align-items: center; gap: var(--sp-1); opacity: 0.35; }
.fl-hatch-stage.is-done { opacity: 0.65; }
.fl-hatch-stage.is-current { opacity: 1; }
.fl-hatch-emoji { font-size: 32px; transition: transform var(--dur-base); }
.fl-hatch-stage.is-current .fl-hatch-emoji { transform: scale(1.15); filter: drop-shadow(0 4px 8px rgba(250,173,20,0.4)); }
.fl-hatch-label { font-size: 11px; color: var(--color-text-muted); }
.fl-hatch-hint { font-size: var(--fs-12); color: var(--color-text-muted); }

/* 门别 Tab */
.fl-gate-tabs { display: flex; gap: var(--sp-2); overflow-x: auto; padding-bottom: 2px; }
.fl-gate-tab {
  padding: 6px 14px; border-radius: var(--r-pill);
  border: 1px solid var(--color-border); background: transparent;
  font-size: var(--fs-12); color: var(--color-text-secondary);
  cursor: pointer; white-space: nowrap; transition: all var(--dur-fast);
}
.fl-gate-tab:hover { border-color: var(--color-primary); }
.fl-gate-tab.is-active { background: var(--color-primary); color: #fff; border-color: var(--color-primary); }

/* Main layout */
.fl-persona-main { display: flex; gap: var(--sp-6); }
@media (max-width: 720px) { .fl-persona-main { flex-direction: column; } }

/* Gallery */
.fl-gallery { display: flex; flex-wrap: wrap; gap: var(--sp-3); align-content: flex-start; }
.fl-gallery-item {
  width: 80px; height: 80px; border-radius: var(--r-md);
  display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 4px;
  border: 1px solid var(--color-border); background: var(--color-bg-elevated);
  cursor: pointer; transition: all var(--dur-fast);
}
.fl-gallery-item:hover { border-color: var(--color-primary); transform: translateY(-2px); }
.fl-gallery-item.is-selected { border-color: var(--color-primary); background: var(--color-primary-soft); }
.fl-gi-emoji { font-size: 28px; }
.fl-gi-name { font-size: 10px; color: var(--color-text-muted); overflow: hidden; text-overflow: ellipsis; max-width: 68px; white-space: nowrap; }
.fl-gallery-empty { font-size: var(--fs-12); color: var(--color-text-muted); padding: var(--sp-4); }

/* 分享卡 */
.fl-share-card {
  width: 360px; min-height: 500px; flex-shrink: 0;
  border-radius: var(--r-lg); padding: 24px 28px 20px;
  color: #fff; display: flex; flex-direction: column; align-items: center;
  gap: var(--sp-3); box-shadow: var(--shadow-modal);
}
@media (max-width: 720px) { .fl-share-card { width: 100%; min-height: 440px; } }

.fl-sc-header { font-size: 11px; opacity: 0.7; letter-spacing: 1px; text-transform: uppercase; }
.fl-sc-emoji { font-size: 56px; filter: drop-shadow(0 2px 8px rgba(0,0,0,0.3)); }
.fl-sc-name { font-size: 24px; font-weight: var(--fw-bold); }
.fl-sc-code {
  font-size: 11px; font-family: var(--font-mono); letter-spacing: 1px;
  padding: 2px 10px; background: rgba(255,255,255,0.15); border-radius: var(--r-pill);
}
.fl-sc-quote { font-size: 14px; font-style: italic; text-align: center; line-height: 1.5; opacity: 0.9; max-width: 280px; }

.fl-sc-dims { width: 100%; display: flex; flex-direction: column; gap: 6px; margin-top: var(--sp-2); }
.fl-sc-dim { display: flex; align-items: center; gap: 4px; font-size: 12px; }
.fl-sc-dim-label { width: 52px; text-align: right; opacity: 0.6; }
.fl-sc-dim-stars { letter-spacing: 1px; }
.fl-sc-dim-desc { opacity: 0.5; font-size: 10px; }

.fl-sc-footer {
  display: flex; justify-content: space-between; align-items: center;
  width: 100%; margin-top: auto; padding-top: var(--sp-3);
  border-top: 1px solid rgba(255,255,255,0.15); font-size: 11px; opacity: 0.6;
}
.fl-sc-qr { width: 40px; height: 40px; border-radius: 6px; background: rgba(255,255,255,0.2); }
</style>
