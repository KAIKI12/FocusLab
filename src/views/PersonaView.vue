<script setup lang="ts">
/**
 * PersonaView · 科研人格图鉴 — 对齐 prototype/screens/persona-card.html + persona-hatch.html。
 *
 * 11 门 × 4 位置(TL/TR/BL/BR) = 44 张人格画像,图资源存在 public/personas/。
 * 文件名规则:`{order}_{gate}_{pos}_{name}.png`
 *
 * 每门 BR 位置为隐藏款:孵化到 Day 7 才解锁(未解锁时显示问号 + 锁定提示)。
 * 孵化起始日写在 settings KV(persona_hatch_start),首次访问写入今天。
 */

import { toPng } from "html-to-image";
import { computed, onMounted, ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import {
  DIM_HIGH,
  DIM_LABELS,
  DIM_LOW,
  HATCH_DAYS,
  PERSONA_META,
  stars,
  type BasePersonaMeta,
  type ComboPersonaMeta,
  type PersonaMeta,
} from "@/data/personas";

interface PersonaDef {
  id: string;
  gate: number;
  pos: "TL" | "TR" | "BL" | "BR";
  name: string;
  imageUrl: string;
  code: string;
}

// 完整 44 张清单(与 public/personas/ 下实际文件一一对应)
const PERSONA_FILES: string[] = [
  "03_1_TL_深夜幽灵.png",
  "04_1_TR_晨鸣鹤.png",
  "05_1_BL_黎明特攻队.png",
  "06_1_BR_六点蜜蜂.png",
  "07_2_TL_DDL炼金术士.png",
  "08_2_TR_火山爆发型选手.png",
  "09_2_BL_定时炸弹研究员.png",
  "10_2_BR_稳态水母.png",
  "11_3_TL_沉浸模式海龟.png",
  "12_3_TR_冬眠熊.png",
  "13_3_BL_树懒禅师.png",
  "14_3_BR_全天候鲨鱼.png",
  "15_4_TL_蜂巢建筑师.png",
  "16_4_TR_猎隼专注体.png",
  "17_4_BL_满月狼.png",
  "18_4_BR_凤凰协议.png",
  "19_5_TL_量子态研究员.png",
  "20_5_TR_学术僵尸.png",
  "21_5_BL_黑洞吸收者.png",
  "22_5_BR_筑巢松鼠.png",
  "23_6_TL_里程碑猎人.png",
  "24_6_TR_章鱼多线程.png",
  "25_6_BL_慢热变形者.png",
  "26_6_BR_实验室赌徒.png",
  "27_7_TL_俄罗斯套娃.png",
  "28_7_TR_数据幽灵.png",
  "29_7_BL_全能异常体.png",
  "30_7_BR_孤独星球.png",
  "31_8_TL_暗夜不死鸟.png",
  "32_8_TR_截止日期超导体.png",
  "33_8_BL_秩序编织者.png",
  "34_8_BR_行走的科研机器.png",
  "35_9_TL_隐形冠军.png",
  "36_9_TR_时间黑洞.png",
  "37_9_BL_永动机.png",
  "38_9_BR_薛定谔的作息.png",
  "39_10_TL_效率偏执狂.png",
  "40_10_TR_战略性摆烂.png",
  "41_10_BL_双面人.png",
  "42_10_BR_豪赌不死.png",
  "43_11_TL_早起但还是赶DDL.png",
  "44_11_TR_混沌之子.png",
  "45_11_BL_休眠火山.png",
  "46_11_BR_机器人嫌疑.png",
];

function parsePersona(file: string): PersonaDef | null {
  const m = file.match(/^\d+_(\d+)_([TB][LR])_(.+)\.png$/);
  if (!m) return null;
  const gate = Number(m[1]);
  const pos = m[2] as "TL" | "TR" | "BL" | "BR";
  const name = m[3];
  return {
    id: `${gate}-${pos}`,
    gate,
    pos,
    name,
    imageUrl: `/personas/${file}`,
    code: `G${gate}-${pos}`,
  };
}

const PERSONAS: PersonaDef[] = PERSONA_FILES.map(parsePersona).filter(
  (p): p is PersonaDef => p !== null,
);

const GATES = [
  { id: 1, emoji: "🌅", label: "时节门", hue: "linear-gradient(135deg, #1a0a2e, #6C5CE7)" },
  { id: 2, emoji: "⚡", label: "爆发门", hue: "linear-gradient(135deg, #2d1f00, #FAAD14)" },
  { id: 3, emoji: "🐢", label: "节奏门", hue: "linear-gradient(135deg, #0a1a2e, #7FB3FF)" },
  { id: 4, emoji: "🔥", label: "专注门", hue: "linear-gradient(135deg, #2d0a0a, #FF4757)" },
  { id: 5, emoji: "🌌", label: "玄学门", hue: "linear-gradient(135deg, #1a1a2e, #B87FFF)" },
  { id: 6, emoji: "🗺️", label: "战略门", hue: "linear-gradient(135deg, #0a2d1a, #27AE60)" },
  { id: 7, emoji: "🎭", label: "异常门", hue: "linear-gradient(135deg, #2d0a1a, #FF6B9D)" },
  { id: 8, emoji: "🏆", label: "精进门", hue: "linear-gradient(135deg, #2d1810, #FF6B6B)" },
  { id: 9, emoji: "♾️", label: "悖论门", hue: "linear-gradient(135deg, #0a2d2d, #4ECDC4)" },
  { id: 10, emoji: "⚖️", label: "两极门", hue: "linear-gradient(135deg, #2d2d0a, #FFD93D)" },
  { id: 11, emoji: "🌀", label: "混沌门", hue: "linear-gradient(135deg, #1a0a1a, #FF6B9D)" },
];

const activeGate = ref<number>(1);

const filteredPersonas = computed(() =>
  PERSONAS.filter((p) => p.gate === activeGate.value),
);

const activeGateInfo = computed(
  () => GATES.find((g) => g.id === activeGate.value) ?? GATES[0],
);

const selectedPersona = ref<PersonaDef | null>(filteredPersonas.value[0] ?? null);

// 点击门 tab 时切换选中 persona 到该门第一个
function selectGate(id: number) {
  activeGate.value = id;
  selectedPersona.value = PERSONAS.find((p) => p.gate === id) ?? null;
}

// ---------- 隐藏款 ----------
// 每门 BR 位置为隐藏款,孵化满 7 天解锁
const HIDDEN_UNLOCK_DAY = 7;

function isHidden(p: PersonaDef): boolean {
  return p.pos === "BR";
}

function isUnlocked(p: PersonaDef): boolean {
  if (!isHidden(p)) return true;
  return hatchDay.value >= HIDDEN_UNLOCK_DAY;
}

// ---------- 7 天孵化(接真实首启日期) ----------
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
  return Math.floor(ms / 86_400_000) + 1;
}

const hatchDay = computed<number>(() => {
  if (!hatchStart.value) return 1;
  return Math.max(diffDaysInclusive(hatchStart.value), 1);
});

const displayHatchDay = computed(() => Math.min(hatchDay.value, HIDDEN_UNLOCK_DAY));

function hatchState(day: number): "past" | "current" | "future" {
  const d = displayHatchDay.value;
  if (day < d) return "past";
  if (day === d) return "current";
  return "future";
}

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
    hatchStart.value = todayLocal();
  }
});

// ---------- 人格描述(每张独有 quote/hidden/dims 或 combo emojis/desc) ----------
const selectedMeta = computed<PersonaMeta | null>(() => {
  const p = selectedPersona.value;
  if (!p) return null;
  return PERSONA_META[p.name] ?? null;
});

function isBaseMeta(m: PersonaMeta | null): m is BasePersonaMeta {
  return m?.kind === "base";
}

function isComboMeta(m: PersonaMeta | null): m is ComboPersonaMeta {
  return m?.kind === "combo";
}

const baseMeta = computed<BasePersonaMeta | null>(() =>
  isBaseMeta(selectedMeta.value) ? selectedMeta.value : null,
);

const comboMeta = computed<ComboPersonaMeta | null>(() =>
  isComboMeta(selectedMeta.value) ? selectedMeta.value : null,
);

function dimDesc(v: number, i: number): string {
  if (v >= 4) return DIM_HIGH[i];
  if (v <= 2) return DIM_LOW[i];
  return "";
}

// ---------- 社交分享 ----------
const shareCardRef = ref<HTMLElement | null>(null);
const copyStatus = ref<"idle" | "copied" | "error">("idle");
const downloadStatus = ref<"idle" | "working" | "done" | "error">("idle");

function shareText(): string {
  const p = selectedPersona.value;
  if (!p || !isUnlocked(p)) return "";
  return `我在 FocusLab 匹配到了「${p.name}」· ${p.code} · ${activeGateInfo.value.label} — focuslab.app`;
}

async function copyShareText() {
  const text = shareText();
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    copyStatus.value = "copied";
  } catch {
    copyStatus.value = "error";
  }
  setTimeout(() => { copyStatus.value = "idle"; }, 1800);
}

async function downloadShareImage() {
  const el = shareCardRef.value;
  const p = selectedPersona.value;
  if (!el || !p || !isUnlocked(p)) return;
  downloadStatus.value = "working";
  try {
    const dataUrl = await toPng(el, {
      pixelRatio: 2,
      cacheBust: true,
      backgroundColor: "transparent",
    });
    const a = document.createElement("a");
    a.href = dataUrl;
    a.download = `focuslab-persona-${p.code}-${p.name}.png`;
    a.click();
    downloadStatus.value = "done";
  } catch (e) {
    console.error("[persona] export image failed", e);
    downloadStatus.value = "error";
  }
  setTimeout(() => { downloadStatus.value = "idle"; }, 1800);
}
</script>

<template>
  <section class="fl-persona">
    <header>
      <h1>🧬 科研人格图鉴</h1>
      <p class="fl-persona-sub">11 门 · 44 型 · 基于你的使用轨迹逐步解锁</p>
    </header>

    <!-- 7 天孵化时间轴 -->
    <div class="fl-hatch">
      <div class="fl-hatch-head">
        <h2>🥚 科研人格孵化中…</h2>
        <p>7 天数据积累,解锁你的专属科研人格</p>
      </div>
      <div class="fl-hatch-tl">
        <div
          v-for="d in HATCH_DAYS"
          :key="d.day"
          class="fl-hatch-node"
          :class="['is-' + hatchState(d.day)]"
        >
          <span class="fl-hatch-day-label">Day {{ d.day }}</span>
          <span class="fl-hatch-dot">
            {{ hatchState(d.day) === "past" ? "✓" : d.day }}
          </span>
          <div class="fl-hatch-card">
            <div class="fl-hatch-emoji">{{ d.emoji }}</div>
            <div class="fl-hatch-title">{{ d.title }}</div>
            <div class="fl-hatch-desc">{{ d.desc }}</div>
            <div class="fl-hatch-progress">
              <div class="fl-hatch-bar">
                <div
                  class="fl-hatch-fill"
                  :class="{ 'is-done': d.day === 7 && hatchState(7) !== 'future' }"
                  :style="{ width: hatchState(d.day) === 'future' ? '0%' : `${d.pct}%` }"
                />
              </div>
              <span class="fl-hatch-progress-label">{{ d.label }}</span>
            </div>
          </div>
        </div>
      </div>
      <div class="fl-hatch-hint">
        <template v-if="hatchDay >= HIDDEN_UNLOCK_DAY">
          🎉 已完成孵化 · 11 个隐藏款(每门 BR)已解锁
        </template>
        <template v-else>
          再使用 {{ HIDDEN_UNLOCK_DAY - hatchDay }} 天解锁全部隐藏款
        </template>
      </div>
    </div>

    <!-- 门别 Tab -->
    <div class="fl-gate-tabs">
      <button
        v-for="g in GATES" :key="g.id"
        class="fl-gate-tab"
        :class="{ 'is-active': activeGate === g.id }"
        @click="selectGate(g.id)"
      >
        {{ g.emoji }} {{ g.label }}
      </button>
    </div>

    <div class="fl-persona-main">
      <!-- Gallery(2×2 四位置) -->
      <div class="fl-gallery">
        <button
          v-for="p in filteredPersonas" :key="p.id"
          class="fl-gallery-item"
          :class="{
            'is-selected': selectedPersona?.id === p.id,
            'is-locked': !isUnlocked(p),
          }"
          @click="selectedPersona = p"
        >
          <template v-if="!isUnlocked(p)">
            <span class="fl-gi-lock">🔒</span>
            <span class="fl-gi-name">??? 隐藏款</span>
          </template>
          <template v-else>
            <img :src="p.imageUrl" :alt="p.name" class="fl-gi-image" loading="lazy" />
            <span class="fl-gi-name">{{ p.name }}</span>
          </template>
          <span class="fl-gi-pos">{{ p.pos }}</span>
        </button>
        <div v-if="!filteredPersonas.length" class="fl-gallery-empty">
          该门暂无人格数据
        </div>
      </div>

      <!-- 详情/分享卡 + 动作 -->
      <div v-if="selectedPersona" class="fl-share-col">
        <div ref="shareCardRef" class="fl-share-card" :style="{ background: activeGateInfo.hue }">
          <div class="fl-sc-header">FOCUSLAB · {{ activeGateInfo.label }}</div>

          <template v-if="!isUnlocked(selectedPersona)">
            <div class="fl-sc-lock-big">🔒</div>
            <div class="fl-sc-name">???</div>
            <div class="fl-sc-code">{{ selectedPersona.code }} · 隐藏款</div>
            <div class="fl-sc-quote">
              孵化满 {{ HIDDEN_UNLOCK_DAY }} 天解锁此人格<br />
              <small>还差 {{ Math.max(HIDDEN_UNLOCK_DAY - hatchDay, 0) }} 天</small>
            </div>
          </template>

          <template v-else>
            <img :src="selectedPersona.imageUrl" :alt="selectedPersona.name" class="fl-sc-image" crossorigin="anonymous" />
            <div class="fl-sc-name">{{ selectedPersona.name }}</div>
            <div class="fl-sc-code">{{ selectedPersona.code }} · {{ activeGateInfo.label }}</div>

            <template v-if="baseMeta">
              <div class="fl-sc-quote">“{{ baseMeta.quote }}”</div>
              <div class="fl-sc-hidden">🔍 {{ baseMeta.hidden }}</div>
              <div class="fl-sc-dims">
                <div v-for="(v, i) in baseMeta.dims" :key="i" class="fl-sc-dim-row">
                  <span class="fl-sc-dim-label">{{ DIM_LABELS[i] }}</span>
                  <span class="fl-sc-dim-stars">{{ stars(v) }}</span>
                  <span class="fl-sc-dim-desc">{{ dimDesc(v, i) }}</span>
                </div>
              </div>
            </template>

            <template v-else-if="comboMeta">
              <div class="fl-sc-combo-emojis">{{ comboMeta.emojis }}</div>
              <div class="fl-sc-quote">“{{ comboMeta.desc }}”</div>
              <div class="fl-sc-hidden">🏅 组合款称号</div>
            </template>

            <template v-else>
              <div class="fl-sc-quote">基于你的行为数据逐步匹配</div>
            </template>
          </template>

          <div class="fl-sc-footer">
            <span>focuslab.app</span>
            <div class="fl-sc-qr" />
          </div>
        </div>

        <!-- 社交分享动作(隐藏款不可分享) -->
        <div v-if="isUnlocked(selectedPersona)" class="fl-share-actions">
          <button
            class="fl-share-btn"
            :disabled="copyStatus !== 'idle'"
            @click="copyShareText"
          >
            {{ copyStatus === 'copied' ? '✓ 已复制' : copyStatus === 'error' ? '复制失败' : '📋 复制文案' }}
          </button>
          <button
            class="fl-share-btn"
            :disabled="downloadStatus === 'working'"
            @click="downloadShareImage"
          >
            {{ downloadStatus === 'working' ? '生成中…' : downloadStatus === 'done' ? '✓ 已保存' : downloadStatus === 'error' ? '导出失败' : '⬇ 下载图片' }}
          </button>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.fl-persona {
  max-width: 960px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--sp-5);
}
.fl-persona h1 { font-size: var(--fs-24); font-weight: var(--fw-semibold); margin: 0; }
.fl-persona-sub { font-size: var(--fs-14); color: var(--color-text-secondary); margin: var(--sp-1) 0 0; }

/* ---------- 孵化 · 7 天剧情时间轴 ---------- */
.fl-hatch {
  padding: var(--sp-5);
  background: radial-gradient(circle at 50% 30%, color-mix(in srgb, var(--color-gold, #FAAD14) 15%, transparent), transparent 60%), var(--color-bg-elevated);
  border: 1px solid var(--color-border); border-radius: var(--r-lg);
}
.fl-hatch-head { text-align: center; margin-bottom: var(--sp-5); }
.fl-hatch-head h2 { font-size: var(--fs-20); font-weight: var(--fw-semibold); margin: 0 0 var(--sp-1); }
.fl-hatch-head p { font-size: var(--fs-12); color: var(--color-text-secondary); margin: 0; }

.fl-hatch-tl { position: relative; padding-left: 70px; margin: 0 auto; max-width: 560px; }
.fl-hatch-tl::before {
  content: ""; position: absolute;
  left: 39px; top: 24px; bottom: 24px;
  width: 2px; background: var(--color-border);
}

.fl-hatch-node { position: relative; margin-bottom: var(--sp-3); transition: opacity var(--dur-base); }
.fl-hatch-node.is-past, .fl-hatch-node.is-current { opacity: 1; }
.fl-hatch-node.is-future { opacity: 0.4; }

.fl-hatch-day-label {
  position: absolute; left: -70px; top: 18px;
  width: 40px; text-align: right;
  font-size: var(--fs-12); font-weight: var(--fw-semibold);
  color: var(--color-text-secondary);
}

.fl-hatch-dot {
  position: absolute; left: -35px; top: 12px;
  width: 34px; height: 34px;
  border-radius: 50%;
  display: grid; place-items: center;
  font-size: var(--fs-12); font-weight: var(--fw-semibold);
  color: #fff;
  background: var(--color-border-strong, var(--color-text-muted));
  z-index: 1;
  transition: all var(--dur-base) var(--ease-out);
}
.fl-hatch-node.is-past .fl-hatch-dot { background: var(--color-success, #52c41a); }
.fl-hatch-node.is-current .fl-hatch-dot {
  background: var(--color-primary);
  animation: flHatchPulse 2s infinite;
}
@keyframes flHatchPulse {
  0%, 100% { box-shadow: 0 0 0 0 color-mix(in srgb, var(--color-primary) 40%, transparent); }
  50%     { box-shadow: 0 0 0 8px color-mix(in srgb, var(--color-primary) 0%, transparent); }
}

.fl-hatch-card {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  padding: var(--sp-3) var(--sp-4);
  text-align: left;
}
.fl-hatch-node.is-current .fl-hatch-card {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-primary) 12%, transparent);
}
.fl-hatch-emoji { font-size: var(--fs-20); margin-bottom: var(--sp-1); }
.fl-hatch-title { font-size: var(--fs-14); font-weight: var(--fw-semibold); margin-bottom: 4px; color: var(--color-text-primary); }
.fl-hatch-desc { font-size: var(--fs-12); color: var(--color-text-secondary); line-height: 1.5; margin-bottom: var(--sp-2); }
.fl-hatch-progress { display: flex; align-items: center; gap: var(--sp-2); }
.fl-hatch-bar { flex: 1; height: 4px; background: var(--color-bg-subtle); border-radius: 2px; overflow: hidden; }
.fl-hatch-fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: 2px;
  transition: width var(--dur-slow) var(--ease-out);
}
.fl-hatch-fill.is-done { background: var(--color-success, #52c41a); }
.fl-hatch-progress-label { font-size: 11px; color: var(--color-text-muted); white-space: nowrap; }

.fl-hatch-hint {
  text-align: center;
  font-size: var(--fs-12);
  color: var(--color-text-muted);
  margin-top: var(--sp-4);
}

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

/* Gallery — 2×2 */
.fl-gallery {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--sp-3);
  align-content: flex-start;
  flex: 1;
  min-width: 0;
}
.fl-gallery-item {
  position: relative;
  aspect-ratio: 1;
  border-radius: var(--r-md);
  display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 4px;
  border: 1px solid var(--color-border); background: var(--color-bg-elevated);
  cursor: pointer; transition: all var(--dur-fast);
  padding: var(--sp-2);
  overflow: hidden;
}
.fl-gallery-item:hover { border-color: var(--color-primary); transform: translateY(-2px); }
.fl-gallery-item.is-selected { border-color: var(--color-primary); background: var(--color-primary-soft); }
.fl-gallery-item.is-locked {
  background: var(--color-bg-subtle);
  color: var(--color-text-muted);
}
.fl-gi-image {
  max-width: 88%;
  max-height: 78%;
  object-fit: contain;
  border-radius: var(--r-sm);
}
.fl-gi-lock { font-size: 36px; opacity: 0.5; }
.fl-gi-name {
  font-size: 11px;
  color: var(--color-text-secondary);
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  max-width: 100%;
}
.fl-gallery-item.is-selected .fl-gi-name { color: var(--color-primary); font-weight: var(--fw-medium); }
.fl-gi-pos {
  position: absolute;
  top: 6px; left: 6px;
  font-family: var(--font-mono);
  font-size: 9px;
  color: var(--color-text-muted);
  padding: 1px 5px;
  background: color-mix(in srgb, var(--color-bg-elevated) 80%, transparent);
  border-radius: var(--r-pill);
}
.fl-gallery-empty { font-size: var(--fs-12); color: var(--color-text-muted); padding: var(--sp-4); grid-column: 1/-1; }

/* 分享卡 */
.fl-share-col { display: flex; flex-direction: column; gap: var(--sp-3); flex-shrink: 0; }
@media (max-width: 720px) { .fl-share-col { width: 100%; } }

.fl-share-card {
  width: 360px; min-height: 560px;
  border-radius: var(--r-lg); padding: 24px 28px 20px;
  color: #fff; display: flex; flex-direction: column; align-items: center;
  gap: var(--sp-2); box-shadow: var(--shadow-modal);
}
@media (max-width: 720px) { .fl-share-card { width: 100%; min-height: 500px; } }

.fl-sc-header { font-size: 11px; opacity: 0.7; letter-spacing: 1px; text-transform: uppercase; }
.fl-sc-image {
  width: 160px; height: 160px; object-fit: contain;
  filter: drop-shadow(0 4px 12px rgba(0,0,0,0.4));
}
.fl-sc-lock-big { font-size: 80px; opacity: 0.55; filter: drop-shadow(0 2px 8px rgba(0,0,0,0.3)); }
.fl-sc-name { font-size: 22px; font-weight: var(--fw-bold); }
.fl-sc-code {
  font-size: 11px; font-family: var(--font-mono); letter-spacing: 1px;
  padding: 2px 10px; background: rgba(255,255,255,0.15); border-radius: var(--r-pill);
}
.fl-sc-quote {
  font-size: 13px; text-align: center; line-height: 1.5;
  opacity: 0.92; max-width: 300px; font-style: italic;
  margin: 2px auto;
}
.fl-sc-quote small { opacity: 0.7; font-size: 11px; }
.fl-sc-hidden {
  font-size: 11px; text-align: center; opacity: 0.7;
  max-width: 300px; line-height: 1.4; margin: 0 auto;
}
.fl-sc-dims {
  display: flex; flex-direction: column; gap: 3px;
  margin: 6px 0 0; width: 100%;
}
.fl-sc-dim-row { display: flex; align-items: center; gap: 6px; font-size: 11px; }
.fl-sc-dim-label { width: 56px; flex-shrink: 0; opacity: 0.7; text-align: right; }
.fl-sc-dim-stars { letter-spacing: 1px; font-size: 12px; opacity: 0.95; }
.fl-sc-dim-desc { opacity: 0.55; font-size: 10px; }
.fl-sc-combo-emojis {
  font-size: 28px; letter-spacing: 6px; text-align: center;
  margin: 2px 0; filter: drop-shadow(0 2px 6px rgba(0,0,0,0.25));
}

.fl-sc-footer {
  display: flex; justify-content: space-between; align-items: center;
  width: 100%; margin-top: auto; padding-top: var(--sp-3);
  border-top: 1px solid rgba(255,255,255,0.15); font-size: 11px; opacity: 0.6;
}
.fl-sc-qr { width: 40px; height: 40px; border-radius: 6px; background: rgba(255,255,255,0.2); }

/* 分享动作 */
.fl-share-actions { display: flex; gap: var(--sp-2); }
.fl-share-btn {
  flex: 1; padding: 10px 12px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  border-radius: var(--r-md);
  font-size: var(--fs-13); cursor: pointer;
  transition: all var(--dur-fast);
}
.fl-share-btn:hover:not(:disabled) { border-color: var(--color-primary); color: var(--color-primary); }
.fl-share-btn:disabled { opacity: 0.7; cursor: default; }
</style>
