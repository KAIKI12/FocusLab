<script setup lang="ts">
/**
 * ParticleEffect · 结算庆祝动画。
 * - grade="S": 金色粒子飘散(~3s, 25 粒)
 * - grade="A": 星星环绕(~2s, 12 粒)
 * 尊重 prefers-reduced-motion，此时不播放。
 */

import { onMounted, onUnmounted, ref } from "vue";

const props = withDefaults(defineProps<{ grade?: string }>(), { grade: "S" });

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

// S 级：金色渐变系
const GOLD_COLORS = ["#FAAD14", "#FFD700", "#FFC53D", "#F5A623", "#FFE58F"];
// A 级：浅金 + 白
const STAR_COLORS = ["#FFE58F", "#FFFBE6", "#FFF1B8", "#FFD700"];

function createParticles(cx: number, cy: number, count: number, colors: string[]): Particle[] {
  const particles: Particle[] = [];
  for (let i = 0; i < count; i++) {
    const angle = (Math.PI * 2 * i) / count + (Math.random() - 0.5) * 0.5;
    const speed = 1.5 + Math.random() * 4;
    particles.push({
      x: cx,
      y: cy,
      vx: Math.cos(angle) * speed,
      vy: Math.sin(angle) * speed - 1.5,
      size: 3 + Math.random() * 4,
      color: colors[Math.floor(Math.random() * colors.length)],
      alpha: 1,
      decay: 0.005 + Math.random() * 0.006, // ~3s lifetime
      rotation: Math.random() * Math.PI * 2,
      rotationSpeed: (Math.random() - 0.5) * 0.15,
    });
  }
  return particles;
}

onMounted(() => {
  // 尊重 prefers-reduced-motion
  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

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

  let particles: Particle[];

  if (props.grade === "S") {
    // S 级：中央 25 粒金色粒子飘散
    particles = createParticles(displayW * 0.5, displayH * 0.35, 25, GOLD_COLORS);
  } else {
    // A 级：12 粒浅金星星
    particles = createParticles(displayW * 0.5, displayH * 0.4, 12, STAR_COLORS);
  }

  function drawStar(ctx: CanvasRenderingContext2D, size: number) {
    const spikes = 5;
    const outer = size;
    const inner = size * 0.4;
    ctx.beginPath();
    for (let i = 0; i < spikes * 2; i++) {
      const r = i % 2 === 0 ? outer : inner;
      const a = (Math.PI * i) / spikes - Math.PI / 2;
      if (i === 0) ctx.moveTo(Math.cos(a) * r, Math.sin(a) * r);
      else ctx.lineTo(Math.cos(a) * r, Math.sin(a) * r);
    }
    ctx.closePath();
    ctx.fill();
  }

  function draw() {
    ctx!.clearRect(0, 0, displayW, displayH);

    for (const p of particles) {
      p.x += p.vx;
      p.y += p.vy;
      p.vy += 0.06; // gentle gravity
      p.alpha -= p.decay;
      p.rotation += p.rotationSpeed;

      if (p.alpha <= 0) continue;

      ctx!.save();
      ctx!.translate(p.x, p.y);
      ctx!.rotate(p.rotation);
      ctx!.globalAlpha = p.alpha;
      ctx!.fillStyle = p.color;

      if (props.grade === "S") {
        // S 级：矩形 confetti
        ctx!.fillRect(-p.size / 2, -p.size / 4, p.size, p.size / 2);
      } else {
        // A 级：星星形状
        drawStar(ctx!, p.size * 0.6);
      }

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
