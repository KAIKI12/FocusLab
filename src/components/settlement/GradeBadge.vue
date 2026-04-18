<script setup lang="ts">
/**
 * GradeBadge · S/A/B/C 评级视觉徽章。
 * S: 金色  A: 辉光蓝  B: 蓝  C: 灰蓝(不用红色 — ADR-005)
 */

defineProps<{ grade: "S" | "A" | "B" | "C" }>();
</script>

<template>
  <span class="fl-grade" :data-grade="grade">{{ grade }}</span>
</template>

<style scoped>
.fl-grade {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  font-size: 20px;
  font-weight: 700;
  letter-spacing: -0.5px;
}
.fl-grade[data-grade="S"] {
  background: linear-gradient(135deg, #ffd700, #ffb347);
  color: #fff;
  box-shadow: 0 4px 16px rgba(255, 215, 0, 0.4);
}
.fl-grade[data-grade="A"] {
  background: linear-gradient(135deg, #4f8cff, #7aabff);
  color: #fff;
  box-shadow: 0 4px 12px rgba(79, 140, 255, 0.3);
}
.fl-grade[data-grade="B"] {
  background: color-mix(in srgb, var(--color-primary) 20%, var(--color-bg-elevated));
  color: var(--color-primary);
  border: 1.5px solid var(--color-primary);
}
.fl-grade[data-grade="C"] {
  background: var(--color-bg-subtle);
  color: var(--color-text-muted);
  border: 1.5px solid var(--color-border);
}
</style>
