/**
 * useBubble · 悬浮球窗口的创建/销毁/状态管理。
 *
 * 从主窗口调用 open() 创建 72×72 透明无边框 always-on-top 圆形窗口。
 * bubble 窗口内部可通过 getCurrentWindow().close() 自行关闭。
 */

import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

let bubble: WebviewWindow | null = null;

export function useBubble() {
  async function open() {
    // 已存在则聚焦
    if (bubble) {
      try {
        await bubble.setFocus();
        return;
      } catch {
        bubble = null;
      }
    }

    bubble = new WebviewWindow("bubble", {
      url: "/bubble.html",
      title: "",
      width: 72,
      height: 72,
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
      } catch {
        /* already closed */
      }
      bubble = null;
    }
  }

  function isOpen() {
    return bubble !== null;
  }

  return { open, close, isOpen };
}
