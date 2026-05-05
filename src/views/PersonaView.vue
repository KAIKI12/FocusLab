<script setup lang="ts">
/**
 * PersonaView · 科研人格图鉴 — 翻卡解锁机制。
 *
 * 12 门 · 46 型,全部默认锁定(卡背)。
 * 累计 7 个日结算后自动揭示用户匹配人格;之后按阶梯间隔获得翻卡机会(不累积):
 *   - 0-10 张: 每 3 天   - 11-25 张: 每 2 天   - 26+ 张: 每天
 */

import { toPng } from "html-to-image";
import { computed, nextTick, onMounted, ref, watch } from "vue";

import { getDisplayHatchDay, getRemainingHatchDays, isHatchComplete } from "@/composables/personaHatchProgress";
import { usePersonaFlipCard } from "@/composables/usePersonaFlipCard";
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
  pos?: "TL" | "TR" | "BL" | "BR";
  name: string;
  imageUrl: string;
  code: string;
}

const PERSONA_FILES: string[] = [
  "1_猫头鹰研究员.png",
  "2_实验室蝙蝠.png",
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
  if (m) {
    const gate = Number(m[1]);
    const pos = m[2] as "TL" | "TR" | "BL" | "BR";
    const name = m[3];
    return { id: `${gate}-${pos}`, gate, pos, name, imageUrl: `/personas/${file}`, code: `G${gate}-${pos}` };
  }
  const m2 = file.match(/^(\d+)_(.+)\.png$/);
  if (m2) {
    const order = m2[1];
    const name = m2[2];
    const meta = PERSONA_META[name];
    const code = meta && meta.kind === "base" ? meta.code : `N${order}`;
    return { id: `0-${order}`, gate: 0, name, imageUrl: `/personas/${file}`, code };
  }
  return null;
}

const PERSONAS: PersonaDef[] = PERSONA_FILES.map(parsePersona).filter(
  (p): p is PersonaDef => p !== null,
);

const GATES = [
  { id: 0, emoji: "🌙", label: "夜型门", hue: "linear-gradient(135deg, #0a0a1f, #5b3aa8)" },
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

const TOTAL_PERSONAS = PERSONAS.length;

const activeGate = ref<number>(0);
const filteredPersonas = computed(() => PERSONAS.filter((p) => p.gate === activeGate.value));
const activeGateInfo = computed(() => GATES.find((g) => g.id === activeGate.value) ?? GATES[0]);
const selectedPersona = ref<PersonaDef | null>(filteredPersonas.value[0] ?? null);

function selectGate(id: number) {
  activeGate.value = id;
  selectedPersona.value = PERSONAS.find((p) => p.gate === id) ?? null;
}

// ---------- 翻卡系统 ----------
const {
  revealedCount,
  matchedPersona,
  flipAvailable,
  daysUntilFlip,
  currentInterval,
  isRevealed,
  useFlipCard,
  init: initFlipCard,
} = usePersonaFlipCard();

function isCardRevealed(p: PersonaDef): boolean {
  return isRevealed(p.name);
}

function isMyPersona(p: PersonaDef): boolean {
  return p.name === matchedPersona.value;
}

// 翻卡确认弹窗
const flipModalTarget = ref<PersonaDef | null>(null);
const showFlipModal = ref(false);
const flippingCard = ref<string | null>(null);

function handleCardClick(p: PersonaDef) {
  if (isCardRevealed(p)) {
    selectedPersona.value = p;
    return;
  }
  if (hatchDay.value < HIDDEN_UNLOCK_DAY) {
    selectedPersona.value = p;
    return;
  }
  if (flipAvailable.value) {
    flipModalTarget.value = p;
    showFlipModal.value = true;
  } else {
    selectedPersona.value = p;
  }
}

async function confirmFlip() {
  const target = flipModalTarget.value;
  if (!target) return;
  showFlipModal.value = false;
  flippingCard.value = target.name;
  await useFlipCard(target.name);
  await nextTick();
  selectedPersona.value = target;
  setTimeout(() => { flippingCard.value = null; }, 700);
}

function cancelFlip() {
  showFlipModal.value = false;
  flipModalTarget.value = null;
}

interface PersonaHatchProgress {
  settlementDays: number;
  remainingDays: number;
}

const HIDDEN_UNLOCK_DAY = 7;
const settlementDays = ref(0);
const hatchDay = computed<number>(() => getDisplayHatchDay(settlementDays.value));
const remainingHatchDays = computed(() => getRemainingHatchDays(settlementDays.value));
const hatchComplete = computed(() => isHatchComplete(settlementDays.value));

function hatchState(day: number): "past" | "current" | "future" {
  const d = hatchDay.value;
  if (day < d) return "past";
  if (day === d) return "current";
  return "future";
}

const activeHatchDay = ref(1);
const activeHatchDayData = computed(
  () => HATCH_DAYS.find((d) => d.day === activeHatchDay.value) ?? HATCH_DAYS[0],
);
watch(hatchDay, (v) => { activeHatchDay.value = v; }, { immediate: true });

onMounted(async () => {
  try {
    const progress = await invokeCmd<PersonaHatchProgress>("get_persona_hatch_progress");
    settlementDays.value = progress.settlementDays;
  } catch {
    settlementDays.value = 0;
  }

  await initFlipCard(settlementDays.value);
});

// ---------- 人格描述 ----------
const selectedMeta = computed<PersonaMeta | null>(() => {
  const p = selectedPersona.value;
  if (!p) return null;
  return PERSONA_META[p.name] ?? null;
});

function isBaseMeta(m: PersonaMeta | null): m is BasePersonaMeta { return m?.kind === "base"; }
function isComboMeta(m: PersonaMeta | null): m is ComboPersonaMeta { return m?.kind === "combo"; }

const baseMeta = computed<BasePersonaMeta | null>(() => isBaseMeta(selectedMeta.value) ? selectedMeta.value : null);
const comboMeta = computed<ComboPersonaMeta | null>(() => isComboMeta(selectedMeta.value) ? selectedMeta.value : null);

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
  if (!p || !isCardRevealed(p)) return "";
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
  if (!el || !p || !isCardRevealed(p)) return;
  downloadStatus.value = "working";
  try {
    const dataUrl = await toPng(el, { pixelRatio: 2, cacheBust: true, backgroundColor: "transparent" });
    const a = document.createElement("a");
    a.href = dataUrl;
    a.download = `focuslab-persona-${p.name}.png`;
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
      <p class="fl-persona-sub">12 门 · 46 型 · 基于你的使用轨迹逐步解锁</p>
    </header>

    <!-- 累计日结算孵化时间轴 -->
    <div class="fl-hatch">
      <div class="fl-hatch-head">
        <template v-if="hatchComplete">
          <h2>🎉 孵化完成</h2>
          <p v-if="matchedPersona">你的科研人格已揭晓 — 每隔 {{ currentInterval }} 天可翻开一张新卡</p>
        </template>
        <template v-else>
          <h2>🥚 科研人格孵化中…</h2>
          <p>累计 7 个日结算后，解锁你的专属科研人格</p>
        </template>
      </div>

      <div class="fl-hatch-stepper">
        <div
          v-for="(d, i) in HATCH_DAYS" :key="d.day"
          class="fl-hatch-step" :class="['is-' + hatchState(d.day)]"
          @click="activeHatchDay = d.day"
        >
          <div v-if="i > 0" class="fl-hatch-line" :class="{ 'is-done': hatchState(d.day) !== 'future' }" />
          <span class="fl-hatch-dot">
            <template v-if="hatchState(d.day) === 'past'">✓</template>
            <template v-else>{{ d.emoji }}</template>
          </span>
          <span class="fl-hatch-step-label">Day {{ d.day }}</span>
        </div>
      </div>

      <div class="fl-hatch-detail">
        <div class="fl-hatch-card">
          <div class="fl-hatch-card-row">
            <span class="fl-hatch-detail-emoji">{{ activeHatchDayData.emoji }}</span>
            <div class="fl-hatch-card-text">
              <div class="fl-hatch-title">{{ activeHatchDayData.title }}</div>
              <div class="fl-hatch-desc">{{ activeHatchDayData.desc }}</div>
            </div>
          </div>
          <div class="fl-hatch-progress">
            <div class="fl-hatch-bar">
              <div
                class="fl-hatch-fill"
                :class="{ 'is-done': activeHatchDayData.day === 7 && hatchState(7) !== 'future' }"
                :style="{ width: hatchState(activeHatchDayData.day) === 'future' ? '0%' : `${activeHatchDayData.pct}%` }"
              />
            </div>
            <span class="fl-hatch-progress-label">{{ activeHatchDayData.label }}</span>
          </div>
        </div>
      </div>

      <!-- 翻卡状态指示器 -->
      <div v-if="hatchComplete" class="fl-flip-status">
        <div class="fl-flip-status-left">
          <span class="fl-flip-icon">🃏</span>
          <span v-if="flipAvailable" class="fl-flip-avail">翻卡机会: 可用</span>
          <span v-else class="fl-flip-wait">下次翻卡: {{ daysUntilFlip }} 天后</span>
        </div>
        <div class="fl-flip-status-right">
          已收集 {{ revealedCount }}/{{ TOTAL_PERSONAS }}
        </div>
      </div>

      <div v-else class="fl-hatch-hint">
        再完成 {{ remainingHatchDays }} 个日结算解锁你的专属人格
      </div>
    </div>

    <!-- 门别 Tab -->
    <div class="fl-gate-tabs">
      <button
        v-for="g in GATES" :key="g.id"
        class="fl-gate-tab" :class="{ 'is-active': activeGate === g.id }"
        @click="selectGate(g.id)"
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
          :class="{
            'is-selected': selectedPersona?.id === p.id,
            'is-locked': !isCardRevealed(p),
            'is-mine': isMyPersona(p),
            'is-flipping': flippingCard === p.name,
          }"
          @click="handleCardClick(p)"
        >
          <template v-if="!isCardRevealed(p)">
            <div class="fl-gi-card-back">
              <span class="fl-gi-lock">🔒</span>
              <span class="fl-gi-mystery">?</span>
            </div>
            <span class="fl-gi-name">未解锁</span>
          </template>
          <template v-else>
            <img :src="p.imageUrl" :alt="p.name" class="fl-gi-image" loading="lazy" />
            <span class="fl-gi-name">{{ p.name }}</span>
            <span v-if="isMyPersona(p)" class="fl-gi-mine">✦</span>
          </template>
          <span v-if="p.pos" class="fl-gi-pos">{{ p.pos }}</span>
        </button>
        <div v-if="!filteredPersonas.length" class="fl-gallery-empty">该门暂无人格数据</div>
      </div>

      <!-- 详情/分享卡 -->
      <div v-if="selectedPersona" class="fl-share-col">
        <div ref="shareCardRef" class="fl-share-card" :style="{ background: activeGateInfo.hue }">
          <div class="fl-sc-header">FOCUSLAB · {{ activeGateInfo.label }}</div>

          <template v-if="!isCardRevealed(selectedPersona)">
            <div class="fl-sc-lock-big">🔒</div>
            <div class="fl-sc-name">???</div>
            <div class="fl-sc-code">{{ selectedPersona.code }} · 未解锁</div>
            <div class="fl-sc-quote">
              <template v-if="!hatchComplete">
                孵化满 {{ HIDDEN_UNLOCK_DAY }} 个日结算后开始翻卡探索<br />
                <small>还差 {{ remainingHatchDays }} 个日结算</small>
              </template>
              <template v-else-if="flipAvailable">
                点击卡片使用翻卡机会揭示此人格
              </template>
              <template v-else>
                再等 {{ daysUntilFlip }} 天获得翻卡机会
              </template>
            </div>
          </template>

          <template v-else>
            <img :src="selectedPersona.imageUrl" :alt="selectedPersona.name" class="fl-sc-image" crossorigin="anonymous" />
            <div class="fl-sc-name">
              {{ selectedPersona.name }}
              <span v-if="isMyPersona(selectedPersona)" class="fl-sc-mine-tag">✦ 你的人格</span>
            </div>
            <div class="fl-sc-code">{{ selectedPersona.code }} · {{ activeGateInfo.label }}</div>

            <template v-if="baseMeta">
              <div class="fl-sc-quote">"{{ baseMeta.quote }}"</div>
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
              <div class="fl-sc-quote">"{{ comboMeta.desc }}"</div>
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

        <div v-if="isCardRevealed(selectedPersona)" class="fl-share-actions">
          <button class="fl-share-btn" :disabled="copyStatus !== 'idle'" @click="copyShareText">
            {{ copyStatus === 'copied' ? '✓ 已复制' : copyStatus === 'error' ? '复制失败' : '📋 复制文案' }}
          </button>
          <button class="fl-share-btn" :disabled="downloadStatus === 'working'" @click="downloadShareImage">
            {{ downloadStatus === 'working' ? '生成中…' : downloadStatus === 'done' ? '✓ 已保存' : downloadStatus === 'error' ? '导出失败' : '⬇ 下载图片' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 翻卡确认弹窗 -->
    <Teleport to="body">
      <Transition name="fl-modal">
        <div v-if="showFlipModal" class="fl-flip-overlay" @click.self="cancelFlip">
          <div class="fl-flip-modal">
            <div class="fl-flip-modal-icon">🃏</div>
            <div class="fl-flip-modal-title">确认翻开这张卡？</div>
            <div class="fl-flip-modal-info">
              <span>{{ activeGateInfo.emoji }} {{ activeGateInfo.label }}</span>
              <span v-if="flipModalTarget?.pos"> · {{ flipModalTarget.pos }}</span>
            </div>
            <div class="fl-flip-modal-hint">翻卡机会将消耗 1 次,下次机会 {{ currentInterval }} 天后</div>
            <div class="fl-flip-modal-actions">
              <button class="fl-flip-modal-btn is-cancel" @click="cancelFlip">取消</button>
              <button class="fl-flip-modal-btn is-confirm" @click="confirmFlip">翻开 🎴</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
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

/* ---------- 孵化 ---------- */
.fl-hatch {
  padding: var(--sp-5);
  background: radial-gradient(circle at 50% 30%, color-mix(in srgb, var(--color-gold, #FAAD14) 15%, transparent), transparent 60%), var(--color-bg-elevated);
  border: 1px solid var(--color-border); border-radius: var(--r-lg);
}
.fl-hatch-head { text-align: center; margin-bottom: var(--sp-4); }
.fl-hatch-head h2 { font-size: var(--fs-20); font-weight: var(--fw-semibold); margin: 0 0 var(--sp-1); }
.fl-hatch-head p { font-size: var(--fs-12); color: var(--color-text-secondary); margin: 0; }

.fl-hatch-stepper { display: flex; align-items: flex-start; justify-content: center; gap: 0; margin: 0 auto; max-width: 640px; }
.fl-hatch-step { display: flex; flex-direction: column; align-items: center; position: relative; flex: 1; cursor: pointer; padding-top: 2px; }
.fl-hatch-line { position: absolute; top: 19px; right: 50%; width: 100%; height: 2px; background: var(--color-border); z-index: 0; }
.fl-hatch-line.is-done { background: var(--color-success, #52c41a); }
.fl-hatch-dot {
  width: 36px; height: 36px; border-radius: 50%; display: grid; place-items: center;
  font-size: 14px; font-weight: var(--fw-semibold); color: #fff;
  background: var(--color-border-strong, var(--color-text-muted));
  z-index: 1; transition: all var(--dur-base) var(--ease-out); position: relative;
}
.fl-hatch-step.is-past .fl-hatch-dot { background: var(--color-success, #52c41a); font-size: 13px; }
.fl-hatch-step.is-current .fl-hatch-dot { background: var(--color-primary); animation: flHatchPulse 2s infinite; }
.fl-hatch-step.is-future .fl-hatch-dot { opacity: 0.4; }
@keyframes flHatchPulse {
  0%, 100% { box-shadow: 0 0 0 0 color-mix(in srgb, var(--color-primary) 40%, transparent); }
  50%      { box-shadow: 0 0 0 8px color-mix(in srgb, var(--color-primary) 0%, transparent); }
}
.fl-hatch-step-label { margin-top: 6px; font-size: 11px; font-weight: var(--fw-semibold); color: var(--color-text-secondary); white-space: nowrap; }
.fl-hatch-step.is-current .fl-hatch-step-label { color: var(--color-primary); }
.fl-hatch-step.is-future .fl-hatch-step-label { opacity: 0.4; }

.fl-hatch-detail { margin-top: var(--sp-4); max-width: 640px; margin-left: auto; margin-right: auto; }
.fl-hatch-card { background: var(--color-bg-elevated); border: 1px solid var(--color-border); border-radius: var(--r-md); padding: var(--sp-3) var(--sp-4); }
.fl-hatch-card-row { display: flex; align-items: flex-start; gap: var(--sp-3); }
.fl-hatch-detail-emoji { font-size: 28px; flex-shrink: 0; line-height: 1.2; }
.fl-hatch-card-text { flex: 1; min-width: 0; }
.fl-hatch-title { font-size: var(--fs-14); font-weight: var(--fw-semibold); margin-bottom: 4px; color: var(--color-text-primary); }
.fl-hatch-desc { font-size: var(--fs-12); color: var(--color-text-secondary); line-height: 1.5; margin-bottom: var(--sp-2); }
.fl-hatch-progress { display: flex; align-items: center; gap: var(--sp-2); margin-top: var(--sp-2); }
.fl-hatch-bar { flex: 1; height: 4px; background: var(--color-bg-subtle); border-radius: 2px; overflow: hidden; }
.fl-hatch-fill { height: 100%; background: var(--color-primary); border-radius: 2px; transition: width var(--dur-slow) var(--ease-out); }
.fl-hatch-fill.is-done { background: var(--color-success, #52c41a); }
.fl-hatch-progress-label { font-size: 11px; color: var(--color-text-muted); white-space: nowrap; }
.fl-hatch-hint { text-align: center; font-size: var(--fs-12); color: var(--color-text-muted); margin-top: var(--sp-4); }

/* ---------- 翻卡状态指示器 ---------- */
.fl-flip-status {
  display: flex; align-items: center; justify-content: space-between;
  margin-top: var(--sp-4); padding: var(--sp-3) var(--sp-4);
  background: var(--color-bg-subtle); border-radius: var(--r-md);
  font-size: var(--fs-13);
}
.fl-flip-status-left { display: flex; align-items: center; gap: var(--sp-2); }
.fl-flip-icon { font-size: 18px; }
.fl-flip-avail { color: var(--color-success, #52c41a); font-weight: var(--fw-semibold); }
.fl-flip-wait { color: var(--color-text-muted); }
.fl-flip-status-right { color: var(--color-text-secondary); font-size: var(--fs-12); }

/* ---------- 门别 Tab ---------- */
.fl-gate-tabs { display: flex; gap: var(--sp-2); overflow-x: auto; padding-bottom: 2px; }
.fl-gate-tab {
  padding: 6px 14px; border-radius: var(--r-pill);
  border: 1px solid var(--color-border); background: transparent;
  font-size: var(--fs-12); color: var(--color-text-secondary);
  cursor: pointer; white-space: nowrap; transition: all var(--dur-fast);
}
.fl-gate-tab:hover { border-color: var(--color-primary); }
.fl-gate-tab.is-active { background: var(--color-primary); color: #fff; border-color: var(--color-primary); }

/* ---------- Gallery ---------- */
.fl-persona-main { display: flex; gap: var(--sp-6); }
@media (max-width: 720px) { .fl-persona-main { flex-direction: column; } }

.fl-gallery { display: grid; grid-template-columns: 1fr 1fr; gap: var(--sp-3); align-content: flex-start; flex: 1; min-width: 0; }
.fl-gallery-item {
  position: relative; aspect-ratio: 1; border-radius: var(--r-md);
  display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 4px;
  border: 1px solid var(--color-border); background: var(--color-bg-elevated);
  cursor: pointer; transition: all var(--dur-fast); padding: var(--sp-2); overflow: hidden;
  perspective: 800px;
}
.fl-gallery-item:hover { border-color: var(--color-primary); transform: translateY(-2px); }
.fl-gallery-item.is-selected { border-color: var(--color-primary); background: var(--color-primary-soft); }
.fl-gallery-item.is-mine { border-color: var(--color-gold, #FAAD14); box-shadow: 0 0 0 1px var(--color-gold, #FAAD14); }
.fl-gallery-item.is-locked { background: var(--color-bg-subtle); color: var(--color-text-muted); }

.fl-gallery-item.is-flipping {
  animation: flFlipCard 700ms ease-in-out;
}
@keyframes flFlipCard {
  0%   { transform: rotateY(0deg) scale(1); }
  50%  { transform: rotateY(90deg) scale(1.05); }
  100% { transform: rotateY(0deg) scale(1); }
}

.fl-gi-card-back {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: 4px; width: 100%; height: 100%; position: relative;
}
.fl-gi-lock { font-size: 24px; opacity: 0.35; }
.fl-gi-mystery { font-size: 32px; font-weight: var(--fw-bold); opacity: 0.15; color: var(--color-text-primary); }
.fl-gi-image { max-width: 88%; max-height: 78%; object-fit: contain; border-radius: var(--r-sm); }
.fl-gi-name {
  font-size: 11px; color: var(--color-text-secondary);
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 100%;
}
.fl-gallery-item.is-selected .fl-gi-name { color: var(--color-primary); font-weight: var(--fw-medium); }
.fl-gi-mine {
  position: absolute; top: 6px; right: 6px;
  font-size: 14px; color: var(--color-gold, #FAAD14);
  filter: drop-shadow(0 1px 2px rgba(0,0,0,0.3));
}
.fl-gi-pos {
  position: absolute; top: 6px; left: 6px;
  font-family: var(--font-mono); font-size: 9px; color: var(--color-text-muted);
  padding: 1px 5px; background: color-mix(in srgb, var(--color-bg-elevated) 80%, transparent);
  border-radius: var(--r-pill);
}
.fl-gallery-empty { font-size: var(--fs-12); color: var(--color-text-muted); padding: var(--sp-4); grid-column: 1/-1; }

/* ---------- 分享卡 ---------- */
.fl-share-col { display: flex; flex-direction: column; gap: var(--sp-3); flex-shrink: 0; }
@media (max-width: 720px) { .fl-share-col { width: 100%; } }

.fl-share-card {
  width: 360px; min-height: 560px; border-radius: var(--r-lg); padding: 24px 28px 20px;
  color: #fff; display: flex; flex-direction: column; align-items: center;
  gap: var(--sp-2); box-shadow: var(--shadow-modal);
}
@media (max-width: 720px) { .fl-share-card { width: 100%; min-height: 500px; } }

.fl-sc-header { font-size: 11px; opacity: 0.7; letter-spacing: 1px; text-transform: uppercase; }
.fl-sc-image { width: 160px; height: 160px; object-fit: contain; filter: drop-shadow(0 4px 12px rgba(0,0,0,0.4)); }
.fl-sc-lock-big { font-size: 80px; opacity: 0.55; filter: drop-shadow(0 2px 8px rgba(0,0,0,0.3)); }
.fl-sc-name { font-size: 22px; font-weight: var(--fw-bold); display: flex; align-items: center; gap: var(--sp-2); }
.fl-sc-mine-tag {
  font-size: 11px; font-weight: var(--fw-medium);
  color: var(--color-gold, #FAAD14); background: rgba(250,173,20,0.15);
  padding: 2px 8px; border-radius: var(--r-pill);
}
.fl-sc-code {
  font-size: 11px; font-family: var(--font-mono); letter-spacing: 1px;
  padding: 2px 10px; background: rgba(255,255,255,0.15); border-radius: var(--r-pill);
}
.fl-sc-quote {
  font-size: 13px; text-align: center; line-height: 1.5;
  opacity: 0.92; max-width: 300px; font-style: italic; margin: 2px auto;
}
.fl-sc-quote small { opacity: 0.7; font-size: 11px; }
.fl-sc-hidden { font-size: 11px; text-align: center; opacity: 0.7; max-width: 300px; line-height: 1.4; margin: 0 auto; }
.fl-sc-dims { display: flex; flex-direction: column; gap: 3px; margin: 6px 0 0; width: 100%; }
.fl-sc-dim-row { display: flex; align-items: center; gap: 6px; font-size: 11px; }
.fl-sc-dim-label { width: 56px; flex-shrink: 0; opacity: 0.7; text-align: right; }
.fl-sc-dim-stars { letter-spacing: 1px; font-size: 12px; opacity: 0.95; }
.fl-sc-dim-desc { opacity: 0.55; font-size: 10px; }
.fl-sc-combo-emojis { font-size: 28px; letter-spacing: 6px; text-align: center; margin: 2px 0; filter: drop-shadow(0 2px 6px rgba(0,0,0,0.25)); }
.fl-sc-footer {
  display: flex; justify-content: space-between; align-items: center;
  width: 100%; margin-top: auto; padding-top: var(--sp-3);
  border-top: 1px solid rgba(255,255,255,0.15); font-size: 11px; opacity: 0.6;
}
.fl-sc-qr { width: 40px; height: 40px; border-radius: 6px; background: rgba(255,255,255,0.2); }

.fl-share-actions { display: flex; gap: var(--sp-2); }
.fl-share-btn {
  flex: 1; padding: 10px 12px; border: 1px solid var(--color-border);
  background: var(--color-bg-elevated); color: var(--color-text-primary);
  border-radius: var(--r-md); font-size: var(--fs-13); cursor: pointer;
  transition: all var(--dur-fast);
}
.fl-share-btn:hover:not(:disabled) { border-color: var(--color-primary); color: var(--color-primary); }
.fl-share-btn:disabled { opacity: 0.7; cursor: default; }

/* ---------- 翻卡确认弹窗 ---------- */
.fl-flip-overlay {
  position: fixed; inset: 0; z-index: 9999;
  background: rgba(0,0,0,0.5); backdrop-filter: blur(4px);
  display: grid; place-items: center;
}
.fl-flip-modal {
  background: var(--color-bg-elevated); border: 1px solid var(--color-border);
  border-radius: var(--r-lg); padding: var(--sp-6); max-width: 360px; width: 90%;
  display: flex; flex-direction: column; align-items: center; gap: var(--sp-3);
  box-shadow: var(--shadow-modal);
}
.fl-flip-modal-icon { font-size: 48px; }
.fl-flip-modal-title { font-size: var(--fs-18); font-weight: var(--fw-semibold); color: var(--color-text-primary); }
.fl-flip-modal-info { font-size: var(--fs-13); color: var(--color-text-secondary); }
.fl-flip-modal-hint { font-size: var(--fs-12); color: var(--color-text-muted); text-align: center; }
.fl-flip-modal-actions { display: flex; gap: var(--sp-3); width: 100%; margin-top: var(--sp-2); }
.fl-flip-modal-btn {
  flex: 1; padding: 10px 16px; border-radius: var(--r-md);
  font-size: var(--fs-14); cursor: pointer; transition: all var(--dur-fast);
  border: 1px solid var(--color-border);
}
.fl-flip-modal-btn.is-cancel {
  background: var(--color-bg-subtle); color: var(--color-text-secondary);
}
.fl-flip-modal-btn.is-cancel:hover { border-color: var(--color-text-muted); }
.fl-flip-modal-btn.is-confirm {
  background: var(--color-primary); color: #fff; border-color: var(--color-primary);
}
.fl-flip-modal-btn.is-confirm:hover { opacity: 0.9; }

/* 弹窗过渡 */
.fl-modal-enter-active, .fl-modal-leave-active { transition: opacity 200ms ease; }
.fl-modal-enter-active .fl-flip-modal, .fl-modal-leave-active .fl-flip-modal { transition: transform 200ms ease; }
.fl-modal-enter-from, .fl-modal-leave-to { opacity: 0; }
.fl-modal-enter-from .fl-flip-modal { transform: scale(0.95); }
.fl-modal-leave-to .fl-flip-modal { transform: scale(0.95); }
</style>
