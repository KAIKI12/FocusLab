/**
 * useBubble · 从主窗口控制悬浮球窗口的创建和销毁。
 *
 * 使用 Tauri 2 的 WebviewWindow API 动态创建一个透明无边框 always-on-top 窗口。
 */

import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

let bubble: WebviewWindow | null = null;

export function useBubble() {
  async function open() {
    if (bubble) {
      try {
        await bubble.setFocus();
      } catch {
        bubble = null;
      }
      if (bubble) return;
    }

    bubble = new WebviewWindow("bubble", {
      url: "/bubble.html",
      title: "",
      width: 200,
      height: 52,
      decorations: false,
      alwaysOnTop: true,
      transparent: true,
      skipTaskbar: true,
      resizable: false,
      x: 100,
      y: 100,
    });

    bubble.once("tauri://destroyed", () => {
      bubble = null;
    });
  }

  async function close() {
    if (bubble) {
      try {
        await bubble.close();
      } catch { /* already closed */ }
      bubble = null;
    }
  }

  function isOpen() {
    return bubble !== null;
  }

  return { open, close, isOpen };
}
