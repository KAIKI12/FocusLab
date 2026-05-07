/**
 * useBubble · 悬浮球窗口的创建/销毁/状态管理。
 *
 * 修复:
 *  - 位置记忆(localStorage 持久化)
 *  - 边缘吸附(拖拽释放后自动贴边)
 *  - 展开方向(根据位置决定向上/向下展开)
 */

import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";

let bubble: WebviewWindow | null = null;

const POS_KEY = "fl-bubble-pos";
const BUBBLE_SIZE = 64;
const SCREEN_MARGIN = 16;

function clampPosition(pos: { x: number; y: number }): { x: number; y: number } {
  const maxX = Math.max(SCREEN_MARGIN, window.screen.width - BUBBLE_SIZE - SCREEN_MARGIN);
  const maxY = Math.max(SCREEN_MARGIN, window.screen.height - BUBBLE_SIZE - SCREEN_MARGIN);
  const x = Number.isFinite(pos.x) ? pos.x : maxX;
  const y = Number.isFinite(pos.y) ? pos.y : maxY;

  return {
    x: Math.max(SCREEN_MARGIN, Math.min(x, maxX)),
    y: Math.max(SCREEN_MARGIN, Math.min(y, maxY)),
  };
}

function loadPosition(): { x: number; y: number } {
  try {
    const saved = localStorage.getItem(POS_KEY);
    if (saved) {
      const pos = JSON.parse(saved);
      if (typeof pos.x === "number" && typeof pos.y === "number") return clampPosition(pos);
    }
  } catch { /* */ }
  // 默认右下角
  return clampPosition({ x: window.screen.width - BUBBLE_SIZE - 20, y: window.screen.height - BUBBLE_SIZE - 120 });
}

function savePosition(x: number, y: number) {
  try {
    localStorage.setItem(POS_KEY, JSON.stringify(clampPosition({ x, y })));
  } catch { /* */ }
}

export function useBubble() {
  async function open() {
    if (bubble) {
      try {
        const current = await bubble.outerPosition();
        const pos = clampPosition({ x: current.x, y: current.y });
        await bubble.setSize(new LogicalSize(BUBBLE_SIZE, BUBBLE_SIZE));
        await bubble.setPosition(new LogicalPosition(pos.x, pos.y));
        await bubble.show();
        await bubble.setFocus();
        return;
      } catch { bubble = null; }
    }

    const pos = loadPosition();

    bubble = new WebviewWindow("bubble", {
      url: "/bubble.html",
      title: "",
      width: BUBBLE_SIZE,
      height: BUBBLE_SIZE,
      decorations: false,
      alwaysOnTop: true,
      transparent: true,
      // 关掉 Windows DWM 方形阴影 — 否则透明窗口(圆形 body)四周会露出
      // 一圈方形阴影"小圆角";展开 Mini Panel 后同理,阴影按方形窗口投射,
      // 与卡片 border-radius 错位显得脏。圆形/卡片自身的视觉轮廓由 CSS 负责。
      shadow: false,
      skipTaskbar: true,
      resizable: false,
      x: pos.x,
      y: pos.y,
    });

    // 监听位置变化(拖拽结束后保存)
    bubble.once("tauri://created", () => {
      if (!bubble) return;
      // 定期保存位置(拖拽期间)
      let posInterval: ReturnType<typeof setInterval> | null = null;
      posInterval = setInterval(async () => {
        if (!bubble) {
          if (posInterval) clearInterval(posInterval);
          return;
        }
        try {
          const pos = await bubble.outerPosition();
          savePosition(pos.x, pos.y);
        } catch { /* window may be closed */ }
      }, 2000);
    });

    bubble.once("tauri://destroyed", () => {
      bubble = null;
    });
  }

  async function close() {
    if (bubble) {
      // 关闭前保存位置
      try {
        const pos = await bubble.outerPosition();
        savePosition(pos.x, pos.y);
      } catch { /* */ }
      try { await bubble.close(); } catch { /* */ }
      bubble = null;
    }
  }

  function isOpen() {
    return bubble !== null;
  }

  return { open, close, isOpen };
}
