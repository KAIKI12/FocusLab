<script setup lang="ts">
/**
 * ParticleEffect · S 级结算庆祝粒子动画。
 * 使用 Canvas 2D 渲染,mount 时自动播放 ~2 秒后自动停止。
 */

import { onMounted, onUnmounted, ref } from "vue";

const canvas = ref<HTMLCanvasElement | null>(null);
let raf = 0;

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  size: number;
  color: string;
  alpha: number;
  decay: number;
  rotation: number;
  rotationSpeed: number;
}

const COLORS = [
  "#FFD700", "#FF6B6B", "#4ECDC4", "#45B7D1",
  "#96CEB4", "#FFEAA7", "#DDA0DD", "#FF8C42",
];

function createParticles(cx: number, cy: number, count: number): Particle[] {
  const particles: Particle[] = [];
  for (let i = 0; i < count; i++) {
    const angle = (Math.PI * 2 * i) / count + (Math.random() - 0.5) * 0.5;
    const speed = 2 + Math.random() * 6;
    particles.push({
      x: cx,
      y: cy,
      vx: Math.cos(angle) * speed,
      vy: Math.sin(angle) * speed - 2,
      size: 3 + Math.random() * 5,
      color: COLORS[Math.floor(Math.random() * COLORS.length)],
      alpha: 1,
      decay: 0.008 + Math.random() * 0.008,
      rotation: Math.random() * Math.PI * 2,
      rotationSpeed: (Math.random() - 0.5) * 0.2,
    });
  }
  return particles;
}

onMounted(() => {
  const el = canvas.value;
  if (!el) return;
  const ctx = el.getContext("2d");
  if (!ctx) return;

  const w = el.offsetWidth * 2;
  const h = el.offsetHeight * 2;
  el.width = w;
  el.height = h;
  ctx.scale(2, 2);

  const displayW = el.offsetWidth;
  const displayH = el.offsetHeight;

  // 从顶部中央和两侧发射
  let particles = [
    ...createParticles(displayW * 0.5, displayH * 0.3, 40),
    ...createParticles(displayW * 0.2, displayH * 0.4, 20),
    ...createParticles(displayW * 0.8, displayH * 0.4, 20),
  ];

  function draw() {
    ctx!.clearRect(0, 0, displayW, displayH);

    for (const p of particles) {
      p.x += p.vx;
      p.y += p.vy;
      p.vy += 0.08; // gravity
      p.alpha -= p.decay;
      p.rotation += p.rotationSpeed;

      if (p.alpha <= 0) continue;

      ctx!.save();
      ctx!.translate(p.x, p.y);
      ctx!.rotate(p.rotation);
      ctx!.globalAlpha = p.alpha;
      ctx!.fillStyle = p.color;
      // 矩形 confetti
      ctx!.fillRect(-p.size / 2, -p.size / 4, p.size, p.size / 2);
      ctx!.restore();
    }

    particles = particles.filter((p) => p.alpha > 0);
    if (particles.length > 0) {
      raf = requestAnimationFrame(draw);
    }
  }

  raf = requestAnimationFrame(draw);
});

onUnmounted(() => {
  cancelAnimationFrame(raf);
});
</script>

<template>
  <canvas ref="canvas" class="fl-particles" />
</template>

<style scoped>
.fl-particles {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 1;
}
</style>
