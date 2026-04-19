<script setup lang="ts">
/**
 * MoodCheck · 心情打卡组件 — 对齐 prototype/screens/mood-check.html。
 * mode="morning" → 早晨意图 5 档(🌙🌤☀️⚡🔥)
 * mode="evening" → 晚间情绪 5 档(😔😕😐🙂😍)
 */

import { ref, watch } from "vue";

const props = withDefaults(
  defineProps<{ visible: boolean; mode?: "morning" | "evening" }>(),
  { mode: "morning" },
);
const emit = defineEmits<{ close: []; select: [value: number] }>();

const selected = ref<number | null>(null);

const MORNING = [
  { value: 1, emoji: "🌙", label: "保养档", desc: "轻量任务为主" },
  { value: 2, emoji: "🌤", label: "温和档", desc: "正常节奏" },
  { value: 3, emoji: "☀️", label: "常规档", desc: "可以推进" },
  { value: 4, emoji: "⚡", label: "进阶档", desc: "状态不错" },
  { value: 5, emoji: "🔥", label: "冲刺档", desc: "全力以赴" },
];

const EVENING = [
  { value: 1, emoji: "😔", label: "疲惫", desc: "今天有点累" },
  { value: 2, emoji: "😕", label: "一般", desc: "不太满意" },
  { value: 3, emoji: "😐", label: "还行", desc: "中规中矩" },
  { value: 4, emoji: "🙂", label: "不错", desc: "有收获" },
  { value: 5, emoji: "😍", label: "很好", desc: "今天很棒" },
];

const options = props.mode === "morning" ? MORNING : EVENING;
const title = props.mode === "morning" ? "🌤 今天想给自己定个档?" : "🌙 今天过得怎么样?";

watch(
  () => props.visible,
  (v) => { if (v) selected.value = null; },
);

function onSelect(value: number) {
  selected.value = value;
  const today = new Date().toISOString().slice(0, 10);
  localStorage.setItem(`fl-mood-${props.mode}-${today}`, String(value));
  setTimeout(() => emit("select", value), 300);
}
</script>

<template>
  <Transition name="fl-fade">
    <div v-if="visible" class="fl-mood-mask" @click.self="emit('close')">
      <div class="fl-mood-card" :class="[`is-${mode}`]">
        <h3>{{ title }}</h3>
        <p class="fl-mood-hint">五档都很好，没有"正确答案"</p>

        <div class="fl-mood-row">
          <button
            v-for="o in options"
            :key="o.value"
            class="fl-mood-btn"
            :class="{ 'is-selected': selected === o.value }"
            @click="onSelect(o.value)"
          >
            <div class="fl-mood-slot">
              <span class="fl-mood-emoji">{{ o.emoji }}</span>
            </div>
            <div class="fl-mood-label-wrap">
              <span class="fl-mood-label">{{ o.label }}</span>
              <span class="fl-mood-desc">{{ o.desc }}</span>
            </div>
          </button>
        </div>

        <div class="fl-mood-foot">
          <button class="fl-mood-skip" @click="emit('close')">跳过</button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fl-mood-mask {
  position: fixed; inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 28%, transparent);
  display: grid; place-items: center;
  z-index: var(--z-modal); padding: var(--sp-4);
}

.fl-mood-card {
  width: min(480px, 100%);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg); box-shadow: var(--shadow-modal);
  padding: var(--sp-6); text-align: center;
}
.fl-mood-card.is-morning {
  background:
    radial-gradient(circle at 20% 0%, rgba(255,214,102,0.18), transparent 55%),
    radial-gradient(circle at 90% 100%, rgba(255,173,20,0.14), transparent 60%),
    var(--color-bg-elevated);
}
.fl-mood-card.is-evening {
  background:
    radial-gradient(circle at 80% 0%, color-mix(in srgb, var(--color-primary) 14%, transparent), transparent 55%),
    var(--color-bg-elevated);
}

.fl-mood-card h3 { font-size: var(--fs-16); font-weight: var(--fw-semibold); margin: 0 0 var(--sp-2); }
.fl-mood-hint { font-size: var(--fs-12); color: var(--color-text-muted); margin: 0 0 var(--sp-5); }

.fl-mood-row { display: flex; justify-content: center; gap: var(--sp-3); }

.fl-mood-btn {
  display: flex; flex-direction: column; align-items: center; gap: var(--sp-2);
  background: none; border: none; cursor: pointer; padding: 0;
}

.fl-mood-slot {
  width: 56px; height: 56px; border-radius: var(--r-lg);
  display: grid; place-items: center;
  background: rgba(255,255,255,0.6);
  border: 1px solid transparent;
  transition: all var(--dur-base) var(--ease-out);
}
.fl-mood-emoji {
  font-size: 30px; line-height: 1;
  filter: grayscale(1); opacity: 0.4;
  transition: all var(--dur-base) var(--ease-out);
}

.fl-mood-btn:hover .fl-mood-slot { transform: scale(1.08); background: rgba(255,255,255,0.9); }
.fl-mood-btn:hover .fl-mood-emoji { filter: grayscale(0.3); opacity: 0.75; }

.fl-mood-btn.is-selected .fl-mood-slot {
  background: #fff;
  border-color: var(--color-gold, #FAAD14);
  box-shadow: 0 2px 8px color-mix(in srgb, var(--color-gold, #FAAD14) 30%, transparent);
}
.fl-mood-btn.is-selected .fl-mood-emoji {
  filter: grayscale(0); opacity: 1;
  transform: scale(1.1);
}

.fl-mood-label-wrap {
  display: flex; flex-direction: column; gap: 1px;
  opacity: 0; transform: translateY(4px);
  transition: all var(--dur-base) var(--ease-out);
}
.fl-mood-btn.is-selected .fl-mood-label-wrap,
.fl-mood-btn:hover .fl-mood-label-wrap {
  opacity: 1; transform: translateY(0);
}
.fl-mood-label { font-size: var(--fs-12); font-weight: var(--fw-medium); color: var(--color-text-primary); }
.fl-mood-desc { font-size: 10px; color: var(--color-text-muted); }

.fl-mood-foot { margin-top: var(--sp-5); }
.fl-mood-skip {
  background: none; border: none; color: var(--color-text-muted);
  font-size: var(--fs-12); cursor: pointer;
}
.fl-mood-skip:hover { color: var(--color-text-primary); }

.fl-fade-enter-active, .fl-fade-leave-active { transition: opacity var(--dur-base) var(--ease-smooth); }
.fl-fade-enter-from, .fl-fade-leave-to { opacity: 0; }
</style>
