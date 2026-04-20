<script setup lang="ts">
/**
 * BadgesView · 成就徽章墙 — 对齐 prototype/screens/persona-badges.html。
 * 6 分类 Tab + 4×N 网格 + 4 稀有度(铜/银/金/钻) + 点击详情。
 *
 * 解锁判定走 useBadgeEngine:引擎聚合 day_summaries + goals 的真实数据,
 * evaluateUnlocked 对每枚徽章按规则返回 bool。未接入数据源的徽章
 * (session preset / 时段分布 / AI 拆解计数 / 节日类)暂 return false。
 */

import { computed, onMounted, ref } from "vue";

import { useBadgeEngine, type BadgeStats } from "@/composables/useBadgeEngine";

interface BadgeDef {
  id: string;
  emoji: string;
  name: string;
  description: string;
  condition: string;
  rarity: "bronze" | "silver" | "gold" | "diamond";
  category: string;
  quote?: string;
}

interface Badge extends BadgeDef {
  unlocked: boolean;
  unlockedAt?: string;
}

const CATEGORIES = [
  { id: "all", label: "全部" },
  { id: "pomodoro", label: "🍅 番茄修行" },
  { id: "streak", label: "🔥 连续打卡" },
  { id: "settle", label: "📊 结算荣耀" },
  { id: "goal", label: "🎯 目标猎人" },
  { id: "time", label: "🌙 时段" },
  { id: "hidden", label: "🎭 彩蛋" },
];

// 45 枚徽章静态定义(对齐原型 persona-badges.html)
// unlocked / unlockedAt 运行时由引擎判定
const BADGE_DEFS: BadgeDef[] = [
  // 🍅 番茄修行(7 枚)
  { id: "p1", emoji: "🌱", name: "初次专注", description: "完成第一个番茄钟", condition: "完成 1 个番茄", rarity: "bronze", category: "pomodoro", quote: "千里之行，始于足下" },
  { id: "p2", emoji: "🍅", name: "番茄农夫", description: "累计 50 个番茄钟", condition: "完成 50 个番茄", rarity: "silver", category: "pomodoro" },
  { id: "p3", emoji: "🏆", name: "番茄大师", description: "累计 500 个番茄钟", condition: "完成 500 个番茄", rarity: "gold", category: "pomodoro" },
  { id: "p4", emoji: "👑", name: "番茄帝王", description: "累计 2000 个番茄钟", condition: "完成 2000 个番茄", rarity: "diamond", category: "pomodoro" },
  { id: "p5", emoji: "⏱️", name: "深度潜水", description: "完成一个 90 分钟番茄", condition: "完成 immersive_90", rarity: "silver", category: "pomodoro" },
  { id: "p6", emoji: "🌊", name: "自由之风", description: "累计 10 次自由计时", condition: "自由模式 10 次", rarity: "bronze", category: "pomodoro" },
  { id: "p7", emoji: "🔥", name: "四连番茄", description: "单日完成 ≥4 个番茄钟", condition: "单日 ≥4 番茄", rarity: "gold", category: "pomodoro" },
  // 🔥 连续打卡(7 枚)
  { id: "s1", emoji: "📅", name: "三天打鱼", description: "连续使用 3 天", condition: "连续 3 天结算", rarity: "bronze", category: "streak" },
  { id: "s2", emoji: "🔥", name: "一周坚持", description: "连续使用 7 天", condition: "连续 7 天结算", rarity: "silver", category: "streak" },
  { id: "s3", emoji: "💪", name: "半月之约", description: "连续使用 15 天", condition: "连续 15 天结算", rarity: "silver", category: "streak" },
  { id: "s4", emoji: "💎", name: "月度勇者", description: "连续使用 30 天", condition: "连续 30 天结算", rarity: "gold", category: "streak" },
  { id: "s5", emoji: "🌟", name: "百日传说", description: "连续使用 100 天", condition: "连续 100 天结算", rarity: "diamond", category: "streak" },
  { id: "s6", emoji: "🎖️", name: "半年老兵", description: "连续使用 180 天", condition: "连续 180 天", rarity: "diamond", category: "streak" },
  { id: "s7", emoji: "🏛️", name: "年度传奇", description: "连续使用 365 天", condition: "连续 365 天", rarity: "diamond", category: "streak" },
  // 📊 结算荣耀(7 枚)
  { id: "g1", emoji: "✨", name: "首个 S 级", description: "获得第一个 S 级评价", condition: "获得 S 级", rarity: "silver", category: "settle" },
  { id: "g2", emoji: "💯", name: "五连 A", description: "连续 5 天 A 级或以上", condition: "连续 5 天 ≥A", rarity: "gold", category: "settle" },
  { id: "g3", emoji: "🏅", name: "月度 S 王", description: "单月获得 10 个 S 级", condition: "月内 10 个 S 级", rarity: "diamond", category: "settle" },
  { id: "g4", emoji: "📈", name: "稳步提升", description: "连续 3 天评级上升", condition: "连续 3 天评级 ↑", rarity: "silver", category: "settle" },
  { id: "g5", emoji: "🎯", name: "零中断", description: "某日结算中断次数为 0", condition: "当日 0 中断", rarity: "gold", category: "settle" },
  { id: "g6", emoji: "🌈", name: "满勤月", description: "整月每天都结算", condition: "月满勤", rarity: "diamond", category: "settle" },
  { id: "g7", emoji: "⭐", name: "首个 A 级", description: "获得第一个 A 级评价", condition: "获得 A 级", rarity: "bronze", category: "settle" },
  // 🎯 目标猎人(6 枚)
  { id: "t1", emoji: "🎯", name: "立下目标", description: "创建第一个长线目标", condition: "创建目标", rarity: "bronze", category: "goal" },
  { id: "t2", emoji: "🏔️", name: "里程碑达成", description: "完成第一个里程碑", condition: "完成里程碑", rarity: "silver", category: "goal" },
  { id: "t3", emoji: "🗻", name: "目标达成", description: "完成第一个长线目标", condition: "目标状态→completed", rarity: "gold", category: "goal" },
  { id: "t4", emoji: "🌍", name: "多线作战", description: "同时推进 3 个目标", condition: "3 个活跃目标", rarity: "silver", category: "goal" },
  { id: "t5", emoji: "🧩", name: "拆解大师", description: "使用 AI 拆解 10 个任务", condition: "AI 拆解 10 次", rarity: "silver", category: "goal" },
  { id: "t6", emoji: "💫", name: "目标收割者", description: "累计完成 5 个长线目标", condition: "完成 5 个目标", rarity: "diamond", category: "goal" },
  // 🌙 时段(6 枚)
  { id: "m1", emoji: "🌅", name: "早鸟", description: "6:00-8:00 完成番茄", condition: "清晨专注", rarity: "bronze", category: "time" },
  { id: "m2", emoji: "🦉", name: "夜猫子", description: "22:00-2:00 完成番茄", condition: "深夜专注", rarity: "bronze", category: "time" },
  { id: "m3", emoji: "🏃", name: "马拉松", description: "单日专注超过 8 小时", condition: "日专注 ≥8h", rarity: "gold", category: "time" },
  { id: "m4", emoji: "☀️", name: "黄金上午", description: "9-12 点完成 3 个番茄", condition: "上午 3 番茄", rarity: "silver", category: "time" },
  { id: "m5", emoji: "🌆", name: "夕阳冲刺", description: "17-19 点完成 2 个番茄", condition: "傍晚 2 番茄", rarity: "bronze", category: "time" },
  { id: "m6", emoji: "🌙", name: "午夜学者", description: "0:00-3:00 完成番茄", condition: "凌晨专注", rarity: "gold", category: "time" },
  // 🎭 彩蛋(12 枚)
  { id: "h1", emoji: "🎃", name: "万圣节", description: "在 10 月 31 日使用", condition: "万圣节当天", rarity: "gold", category: "hidden" },
  { id: "h2", emoji: "🎄", name: "圣诞快乐", description: "在 12 月 25 日使用", condition: "圣诞节当天", rarity: "gold", category: "hidden" },
  { id: "h3", emoji: "🎆", name: "新年快乐", description: "在 1 月 1 日使用", condition: "元旦当天", rarity: "gold", category: "hidden" },
  { id: "h4", emoji: "🐉", name: "龙年大吉", description: "在春节期间使用", condition: "春节期间", rarity: "diamond", category: "hidden" },
  { id: "h5", emoji: "🌙", name: "中秋团圆", description: "在中秋节使用", condition: "中秋当天", rarity: "gold", category: "hidden" },
  { id: "h6", emoji: "💘", name: "情人节", description: "在 2 月 14 日使用", condition: "情人节当天", rarity: "silver", category: "hidden" },
  { id: "h7", emoji: "🎓", name: "毕业季", description: "在 6-7 月使用", condition: "6 或 7 月", rarity: "bronze", category: "hidden" },
  { id: "h8", emoji: "🕐", name: "凌晨 3 点", description: "在凌晨 3:00 完成番茄", condition: "3:00 AM", rarity: "diamond", category: "hidden" },
  { id: "h9", emoji: "🔢", name: "666", description: "累计专注 666 分钟", condition: "总专注 666m", rarity: "silver", category: "hidden" },
  { id: "h10", emoji: "🎵", name: "1024", description: "累计 1024 个番茄", condition: "1024 番茄", rarity: "diamond", category: "hidden" },
  { id: "h11", emoji: "🐱", name: "猫咪日", description: "在 2 月 22 日使用", condition: "猫之日", rarity: "silver", category: "hidden" },
  { id: "h12", emoji: "❓", name: "???", description: "未知解锁条件", condition: "???", rarity: "diamond", category: "hidden" },
];

/**
 * 按徽章 id 判定是否解锁。
 * 未列出的 id(p5/p6/t2/t5/mX/g5/h1-h8/h11-h12)暂无数据源,保持锁定,
 * 后续补齐 session 级统计 / 日期监听 / AI 拆解计数后再填进来。
 */
function evaluateUnlocked(id: string, s: BadgeStats): boolean {
  switch (id) {
    // 番茄修行
    case "p1": return s.totalPomodoros >= 1;
    case "p2": return s.totalPomodoros >= 50;
    case "p3": return s.totalPomodoros >= 500;
    case "p4": return s.totalPomodoros >= 2000;
    case "p7": return s.maxDayPomodoros >= 4;
    // 连续打卡
    case "s1": return s.maxStreakDays >= 3;
    case "s2": return s.maxStreakDays >= 7;
    case "s3": return s.maxStreakDays >= 15;
    case "s4": return s.maxStreakDays >= 30;
    case "s5": return s.maxStreakDays >= 100;
    case "s6": return s.maxStreakDays >= 180;
    case "s7": return s.maxStreakDays >= 365;
    // 结算荣耀
    case "g1": return s.hasSGrade;
    case "g2": return s.maxConsecutiveAOrAbove >= 5;
    case "g3": return s.maxMonthSCount >= 10;
    case "g4": return s.maxAscendingStreak >= 3;
    case "g6": return s.hasPerfectMonth;
    case "g7": return s.hasAGrade;
    // 目标猎人
    case "t1": return s.totalGoalCount >= 1;
    case "t3": return s.completedGoalCount >= 1;
    case "t4": return s.activeGoalCount >= 3;
    case "t6": return s.completedGoalCount >= 5;
    // 彩蛋(数值类)
    case "h9": return s.totalFocusMinutes >= 666;
    case "h10": return s.totalPomodoros >= 1024;
    default: return false;
  }
}

const { stats, unlockedAtMap, load } = useBadgeEngine();

const ALL_BADGES = computed<Badge[]>(() =>
  BADGE_DEFS.map((def) => ({
    ...def,
    unlocked: evaluateUnlocked(def.id, stats.value),
    unlockedAt: unlockedAtMap.value[def.id],
  })),
);

const activeCategory = ref<string>("all");
const selectedBadge = ref<Badge | null>(null);

const filteredBadges = computed<Badge[]>(() => {
  if (activeCategory.value === "all") return ALL_BADGES.value;
  return ALL_BADGES.value.filter((b) => b.category === activeCategory.value);
});

const unlockedCount = computed(() => ALL_BADGES.value.filter((b) => b.unlocked).length);

const RARITY_COLORS: Record<string, string> = {
  bronze: "#CD7F32",
  silver: "#C0C0C0",
  gold: "#FFD700",
  diamond: "#B87FFF",
};

const RARITY_LABELS: Record<string, string> = {
  bronze: "铜",
  silver: "银",
  gold: "金",
  diamond: "钻石",
};

function rarityCount(r: string): number {
  return ALL_BADGES.value.filter((b) => b.rarity === r && b.unlocked).length;
}

function toggleDetail(badge: Badge) {
  selectedBadge.value = selectedBadge.value?.id === badge.id ? null : badge;
}

onMounted(() => load(BADGE_DEFS.map((d) => d.id), evaluateUnlocked));
</script>

<template>
  <section class="fl-badges">
    <header>
      <h1>🏆 成就徽章</h1>
      <p class="fl-badges-sub">
        已解锁 {{ unlockedCount }} / {{ BADGE_DEFS.length }}
        <span v-if="!stats.loaded" class="fl-badges-loading"> · 加载中…</span>
      </p>
    </header>

    <!-- 统计条 -->
    <div class="fl-badge-stats">
      <span v-for="r in ['bronze','silver','gold','diamond']" :key="r" class="fl-badge-stat">
        <span class="fl-badge-stat-dot" :style="{ background: RARITY_COLORS[r] }" />
        {{ RARITY_LABELS[r] }} {{ rarityCount(r) }}
      </span>
    </div>

    <!-- 分类 Tab -->
    <div class="fl-badge-tabs">
      <button
        v-for="c in CATEGORIES" :key="c.id"
        class="fl-badge-tab" :class="{ 'is-active': activeCategory === c.id }"
        @click="activeCategory = c.id"
      >
        {{ c.label }}
      </button>
    </div>

    <!-- 徽章网格 -->
    <div class="fl-badge-grid">
      <div
        v-for="b in filteredBadges" :key="b.id"
        class="fl-badge-item"
        :class="{ 'is-locked': !b.unlocked, 'is-selected': selectedBadge?.id === b.id }"
        @click="toggleDetail(b)"
      >
        <div
          class="fl-badge-circle"
          :class="[`is-${b.rarity}`]"
          :style="b.unlocked ? { borderColor: RARITY_COLORS[b.rarity] } : {}"
        >
          <span v-if="b.unlocked">{{ b.emoji }}</span>
          <span v-else style="font-size:14px">🔒</span>
        </div>
        <span class="fl-badge-name">{{ b.unlocked ? b.name : '???' }}</span>
      </div>

      <!-- 详情卡 -->
      <Transition name="fl-slide">
        <div v-if="selectedBadge" class="fl-badge-detail">
          <span class="fl-detail-emoji">{{ selectedBadge.emoji }}</span>
          <div class="fl-detail-info">
            <div class="fl-detail-head">
              <strong>{{ selectedBadge.name }}</strong>
              <span class="fl-detail-rarity" :style="{ color: RARITY_COLORS[selectedBadge.rarity] }">
                {{ RARITY_LABELS[selectedBadge.rarity] }}
              </span>
            </div>
            <div class="fl-detail-desc">{{ selectedBadge.description }}</div>
            <div class="fl-detail-cond">条件: {{ selectedBadge.condition }}</div>
            <div v-if="selectedBadge.quote" class="fl-detail-quote">
              "{{ selectedBadge.quote }}"
            </div>
            <div v-if="selectedBadge.unlockedAt" class="fl-detail-date">
              解锁于 {{ selectedBadge.unlockedAt }}
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </section>
</template>

<style scoped>
.fl-badges {
  max-width: 720px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--sp-5);
}
.fl-badges h1 { font-size: var(--fs-24); font-weight: var(--fw-semibold); margin: 0; }
.fl-badges-sub { font-size: var(--fs-14); color: var(--color-text-secondary); margin: var(--sp-1) 0 0; }
.fl-badges-loading { color: var(--color-text-muted); font-size: var(--fs-12); }

.fl-badge-stats {
  display: flex; gap: var(--sp-4); font-size: var(--fs-12); color: var(--color-text-secondary);
}
.fl-badge-stat { display: flex; align-items: center; gap: var(--sp-1); }
.fl-badge-stat-dot { width: 10px; height: 10px; border-radius: 50%; }

.fl-badge-tabs {
  display: flex; gap: var(--sp-2); overflow-x: auto; padding-bottom: 2px;
}
.fl-badge-tab {
  padding: 6px 14px; border-radius: var(--r-pill);
  border: 1px solid var(--color-border); background: transparent;
  font-size: var(--fs-12); color: var(--color-text-secondary);
  cursor: pointer; white-space: nowrap;
  transition: all var(--dur-fast);
}
.fl-badge-tab:hover { border-color: var(--color-primary); }
.fl-badge-tab.is-active {
  background: var(--color-primary); color: #fff; border-color: var(--color-primary);
}

.fl-badge-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--sp-4);
}

.fl-badge-item {
  display: flex; flex-direction: column; align-items: center; gap: 6px;
  cursor: pointer; transition: transform var(--dur-fast);
}
.fl-badge-item:hover { transform: scale(1.05); }
.fl-badge-item.is-locked { opacity: 0.45; }

.fl-badge-circle {
  width: 64px; height: 64px; border-radius: 50%;
  display: grid; place-items: center;
  font-size: 28px;
  background: var(--color-bg-elevated);
  border: 3px solid var(--color-border);
  transition: all var(--dur-base);
}
.fl-badge-circle.is-gold { box-shadow: 0 0 8px rgba(255,215,0,0.35); }
.fl-badge-circle.is-diamond { animation: diamond-pulse 2s ease-in-out infinite; }
@keyframes diamond-pulse {
  0%, 100% { box-shadow: 0 0 8px rgba(184,127,255,0.4); }
  50% { box-shadow: 0 0 18px rgba(184,127,255,0.7); }
}

.fl-badge-name {
  font-size: var(--fs-12); color: var(--color-text-secondary);
  max-width: 72px; text-align: center;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}

/* 详情卡 */
.fl-badge-detail {
  grid-column: 1 / -1;
  display: flex; gap: var(--sp-4); align-items: flex-start;
  padding: var(--sp-4);
  background: var(--color-bg-elevated); border: 1px solid var(--color-border);
  border-radius: var(--r-md);
}
.fl-detail-emoji { font-size: 36px; flex-shrink: 0; }
.fl-detail-info { flex: 1; display: flex; flex-direction: column; gap: var(--sp-2); }
.fl-detail-head { display: flex; align-items: center; gap: var(--sp-2); }
.fl-detail-head strong { font-size: var(--fs-16); }
.fl-detail-rarity { font-size: var(--fs-12); font-weight: var(--fw-semibold); }
.fl-detail-desc { font-size: var(--fs-14); color: var(--color-text-secondary); }
.fl-detail-cond { font-size: var(--fs-12); color: var(--color-text-muted); }
.fl-detail-quote { font-size: var(--fs-14); font-style: italic; color: var(--color-text-secondary); }
.fl-detail-date { font-size: var(--fs-12); color: var(--color-text-muted); }

.fl-slide-enter-active { animation: detail-in var(--dur-base) ease; }
@keyframes detail-in { from { opacity: 0; transform: translateY(-8px); } to { opacity: 1; transform: translateY(0); } }
.fl-slide-leave-active { transition: opacity var(--dur-fast); }
.fl-slide-leave-to { opacity: 0; }
</style>
