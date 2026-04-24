---
created: 2026-04-23
reason: quick-note.html AI 三版候选区改为横向三列布局，适配桌面应用宽屏阅读习惯
---

# quick-note.html · 横向三版布局改版

## 目标

保留上方原文输入区不变，将 AI 三版候选区从竖向堆叠改为**横向三列等分卡片**，每张卡片只显示标题 + 正文预览，点中某版后在三列下方展开统一操作条。

## 信息结构

```
[速记便签标题区]
[原文输入 textarea]
[inline-tip + 输入操作按钮]
─── AI 展开区（data-expanded="true"）───
[AI head: 标题 + 重新生成按钮]
[三列卡片横排]
  卡片A·任务导向  |  卡片B·笔记梳理  |  卡片C·行动清单
[picked-actions 操作条（选中后显示）]
  已选方案X  [继续编辑]  [保存笔记]  [创建任务]
[footer-actions: 收起 / 全部保存]
```

## CSS 改动清单

| 选择器 | 改动 | 原值 |
|--------|------|------|
| `.quick-note` | `width: min(1100px, 100%)` | `width: min(620px, 100%)` |
| `.candidate-grid` | `grid-template-columns: repeat(3, 1fr); gap: var(--sp-4); align-items: start` | 单列 `gap: var(--sp-3)` |
| `.candidate-body p` | 添加 `-webkit-line-clamp: 4` 截断 | 无截断 |
| `.candidate-actions` | **整体移除**（操作统一到 picked-actions） | 存在于每张卡片 |
| `.picked-actions`（新增） | `display: flex; justify-content: space-between; background: primary-soft; border: primary-25%` | 无 |
| `.stage` | `min-height` 改为 `auto; padding: var(--sp-8) var(--sp-6)` | `min-height: 860px` |

## HTML 改动清单

1. **移除** 三张 `<article class="candidate">` 内的 `<div class="candidate-actions">` 块（共 3 处）
2. **新增** `<div class="picked-actions" id="pickedActions" hidden>` 块，放在 `.candidate-grid` 之后：
   ```html
   <div class="picked-actions" id="pickedActions" hidden>
     <span class="picked-label" id="pickedLabel">已选方案 A · 偏任务导向</span>
     <div style="display:flex;gap:var(--sp-2)">
       <button class="btn btn-secondary btn-sm">继续编辑这版</button>
       <button class="btn btn-secondary btn-sm">保存笔记</button>
       <button class="btn btn-primary btn-sm">
         <span data-icon="plus" data-size="12"></span> 创建任务
       </button>
     </div>
   </div>
   ```
3. 将 `.candidate-head .meta` 中的 chips 保留（标识风格标签）
4. 保留 `.candidate.is-picked` 选中高亮样式（已有）

## JS 改动清单

在现有 `<script>` 末尾追加：

```js
// 横向三版：点击卡片切换选中 + 显示操作条
const pickedActions = document.getElementById('pickedActions');
const pickedLabel   = document.getElementById('pickedLabel');

document.querySelectorAll('.candidate').forEach(card => {
  card.addEventListener('click', () => {
    document.querySelectorAll('.candidate').forEach(c => c.classList.remove('is-picked'));
    card.classList.add('is-picked');
    const title = card.querySelector('.candidate-title').textContent.trim();
    pickedLabel.textContent = '已选 ' + title;
    pickedActions.hidden = false;
    if (window.renderIcons) window.renderIcons();
  });
});
```

## 不改动的部分

- `.showcase-wrap`、`.showcase-head`、`.control-row`、`.demo-grid`（侧边栏布局）
- `.side-rail` 说明卡片内容
- `.qn-head`、`.qn-body`、`.note-box`、`.input-actions`
- 主题切换脚本、icons.js 引入
- `toggleAiBtn` / `collapseAiBtn` 现有逻辑

## 验收标准

- [ ] AI 候选区三列横排，每列等宽
- [ ] 每张卡片正文超出 4 行后截断显示省略号
- [ ] 点击任意卡片：该卡高亮，下方操作条出现并显示对应方案名
- [ ] 未点击任何卡片时操作条隐藏
- [ ] 窗口收窄到 900px 以下时三列可降级为单列（media query）
- [ ] 暗色主题下样式正常
