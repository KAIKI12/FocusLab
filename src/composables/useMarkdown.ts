/**
 * useMarkdown · 受信任 markdown 渲染。
 *
 * - marked 解析 GFM(表格、删除线、自动链接、代码围栏)
 * - DOMPurify 兜底净化(MVP 信任 AI 输出,但仍走净化避免边缘 case)
 * - 计算属性形式 + memo 缓存,流式增量调用不会爆 CPU
 *
 * 用法:
 *   const html = renderMarkdown(content)
 */

import DOMPurify from "dompurify";
import { marked } from "marked";

// 初始化一次。GFM 默认开启;关闭 breaks(单个 \n 不强转 <br>),
// 让段落之间走标准 <p> margin,避免出现多余空行。
marked.setOptions({
  breaks: false,
  gfm: true,
});

const cache = new Map<string, string>();
const MAX_CACHE = 256;
const CACHE_VERSION = "v2-no-breaks";

export function renderMarkdown(content: string): string {
  if (!content) return "";
  const key = CACHE_VERSION + "::" + content;
  const hit = cache.get(key);
  if (hit !== undefined) return hit;

  // marked 同步模式(无 async highlighter): parse 直接返回 string
  const raw = marked.parse(content, { async: false }) as string;
  const safe = DOMPurify.sanitize(raw, {
    USE_PROFILES: { html: true },
    ADD_ATTR: ["target", "rel"],
  });

  if (cache.size >= MAX_CACHE) {
    // 简单 LRU:删最早一个
    const first = cache.keys().next().value;
    if (first !== undefined) cache.delete(first);
  }
  cache.set(key, safe);
  return safe;
}
