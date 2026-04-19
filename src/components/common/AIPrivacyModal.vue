<script setup lang="ts">
/**
 * AIPrivacyModal · AI 隐私声明弹窗。
 * 对齐 prototype/screens/modals.html §3 AI Privacy。
 * 首次启用 AI 时弹出,用户确认后写入 localStorage。
 */

import { ShieldCheck } from "lucide-vue-next";

defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: []; accepted: [] }>();

function onAccept() {
  localStorage.setItem("fl-ai-privacy-accepted", "1");
  emit("accepted");
}
</script>

<template>
  <Transition name="fl-fade">
    <div v-if="visible" class="fl-modal-mask" @click.self="emit('close')">
      <div class="fl-ap" role="dialog" aria-modal="true">
        <header class="fl-ap-head">
          <div class="fl-ap-icon">
            <ShieldCheck :size="32" />
          </div>
          <h2>AI 功能 · 数据透明声明</h2>
          <p class="fl-ap-sub">在你启用 AI 功能之前，我们想让你了解数据如何被使用。</p>
        </header>

        <div class="fl-ap-rules">
          <div class="fl-ap-rule fl-ap-allow">
            <span class="fl-ap-badge">✓ 会发送</span>
            <p>当前任务名、描述、完成时长、关联目标名称</p>
            <span class="fl-ap-note">仅用于生成建议和分析，不含个人身份信息</span>
          </div>
          <div class="fl-ap-rule fl-ap-deny">
            <span class="fl-ap-badge">✗ 绝不发送</span>
            <p>完整任务历史、其他任务详情、姓名/学校/邮箱等 PII</p>
            <span class="fl-ap-note">即使启用 AI，你的隐私数据也不会离开本地</span>
          </div>
          <div class="fl-ap-rule fl-ap-info">
            <span class="fl-ap-badge">🔑 API 密钥</span>
            <p>你的 API Key 仅存储在本地数据库，不经过任何中间服务器</p>
            <span class="fl-ap-note">所有请求直接从本机发送到你配置的 AI 提供商</span>
          </div>
        </div>

        <footer class="fl-ap-foot">
          <button class="fl-btn fl-btn-ghost" @click="emit('close')">暂不开启</button>
          <button class="fl-btn fl-btn-primary" @click="onAccept">我已了解 · 继续</button>
        </footer>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fl-modal-mask {
  position: fixed; inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 32%, transparent);
  display: grid; place-items: center;
  z-index: var(--z-modal); padding: var(--sp-4);
}

.fl-ap {
  width: min(440px, 100%);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-modal);
  overflow: hidden;
}

.fl-ap-head {
  padding: var(--sp-5); text-align: center;
}
.fl-ap-icon {
  width: 56px; height: 56px; border-radius: 50%;
  background: linear-gradient(135deg, var(--color-primary), color-mix(in srgb, var(--color-primary) 60%, #a78bfa));
  color: #fff; display: grid; place-items: center;
  margin: 0 auto var(--sp-3);
}
.fl-ap-head h2 { font-size: var(--fs-16); font-weight: var(--fw-semibold); margin: 0 0 var(--sp-1); }
.fl-ap-sub { font-size: var(--fs-12); color: var(--color-text-muted); margin: 0; }

.fl-ap-rules { padding: 0 var(--sp-5) var(--sp-4); display: flex; flex-direction: column; gap: var(--sp-3); }
.fl-ap-rule {
  padding: var(--sp-3); border-radius: var(--r-md);
  border: 1px solid var(--color-border);
}
.fl-ap-rule p { font-size: var(--fs-12); margin: var(--sp-1) 0; color: var(--color-text-primary); }
.fl-ap-badge {
  font-size: 11px; font-weight: var(--fw-semibold);
  padding: 2px 8px; border-radius: 10px;
}
.fl-ap-allow .fl-ap-badge { background: color-mix(in srgb, var(--color-success) 15%, transparent); color: var(--color-success); }
.fl-ap-deny .fl-ap-badge { background: color-mix(in srgb, var(--color-danger, #ef4444) 15%, transparent); color: var(--color-danger, #ef4444); }
.fl-ap-info .fl-ap-badge { background: color-mix(in srgb, var(--color-primary) 15%, transparent); color: var(--color-primary); }
.fl-ap-note { font-size: 11px; color: var(--color-text-muted); }

.fl-ap-foot {
  display: flex; justify-content: flex-end; gap: var(--sp-2);
  padding: var(--sp-3) var(--sp-5);
  border-top: 1px solid var(--color-border);
  background: var(--color-bg-subtle);
}
.fl-btn {
  padding: 8px 16px; border-radius: var(--r-md);
  font-size: var(--fs-12); font-weight: var(--fw-medium);
  border: 1px solid transparent; cursor: pointer;
}
.fl-btn-ghost { background: transparent; color: var(--color-text-secondary); border-color: var(--color-border); }
.fl-btn-ghost:hover { background: var(--color-bg-hover); }
.fl-btn-primary { background: var(--color-primary); color: var(--color-text-on-primary, #fff); }
.fl-btn-primary:hover { opacity: 0.9; }

.fl-fade-enter-active, .fl-fade-leave-active { transition: opacity var(--dur-base) var(--ease-smooth); }
.fl-fade-enter-from, .fl-fade-leave-to { opacity: 0; }
</style>
