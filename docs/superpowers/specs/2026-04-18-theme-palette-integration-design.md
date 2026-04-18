# Theme Palette Integration · Design Spec

- **Spec date**: 2026-04-18
- **Scope**: FocusLab prototype (`prototype/`)
- **Source of themes**: `FocusLab-主题预览.html`
- **Status**: approved by user, ready for implementation plan

## 1. 背景与目标

FocusLab 当前只有一套默认"宁静蓝"主题，外加一个独立的浅色/深色切换。仓库里已经有一份由用户准备好的 `FocusLab-主题预览.html`，里面给出 11 组完整的主题色板（每组均含 light 与 dark 两个变体）。

本次改动的目标：

1. 将 `FocusLab-主题预览.html` 中的 11 组主题**全部正式接入**产品
2. 保留现有默认蓝作为正式主题之一，共 **12 个色调主题**
3. 色调主题是**全局的**，一处切换，全原型页面立即同步
4. 明暗模式（light/dark）保持独立控制；色调主题与明暗模式**两轴正交**
5. 所有主题化配色（含功能色、象限色、表面、文本、描边等）都按对应主题同步变化，不仅改 primary

**非目标**：自定义调色板、主题导入/导出、基于时间自动切主题、扩展功能 token。

## 2. 总体设计原则

### 2.1 两轴正交

| 轴 | 属性 | 取值 |
|----|------|------|
| 明暗 | `html[data-theme]` | `light`（缺省） / `dark` |
| 色调 | `html[data-accent-theme]` | `default` / `claude` / `green` / `lavender` / `blue-classic` / `graphite` / `sakura` / `candy` / `milktea` / `amber` / `teal` / `slate`（共 12 个） |

- 12 色调 × 2 明暗 = 24 套完整变量值
- `default` 不写 data 属性（由 `:root` 与 `[data-theme="dark"]` 负责），降低属性开销
- 其余 11 主题以 `[data-accent-theme="X"]` 覆盖 light；`[data-theme="dark"][data-accent-theme="X"]` 覆盖 dark（双属性特异性高于单 `[data-theme="dark"]`）

### 2.2 全配色主题化

本次改动把全部主题化变量都纳入色调主题：

- **品牌**：primary / primary-light / primary-dark / primary-soft
- **功能色**：success / gold / warning / neutral（含对应 soft）
- **象限色**：q1 / q2 / q3 / q4（含对应 soft）
- **表面**：bg / bg-elevated / bg-subtle / bg-hover
- **文本**：text-primary / text-secondary / text-muted
- **描边**：border / border-strong / divider
- **聚焦**：shadow-focus

### 2.3 保留固定的识别色

以下颜色是"功能身份色"，与模式语义强绑定，**不随主题变化**：

| 颜色 | 用途 |
|------|------|
| `#8B5CF6` / `#A78BFA` / `#7B52D6` | 🌀 自由模式系列紫罗兰（pomodoro / main-today / floating-ball / settings 均引用） |
| `#FF7A5C` | 🍅 番茄计数点 |
| 悬浮球假桌面渐变 (`#6DA5FF / #B87FFF / #FF9AA2`) | 纯装饰，不是语义色 |

## 3. Token 拆分与派生规则

### 3.1 被主题化的变量清单（共 30 个）

**品牌（4）**

```
--color-primary            ← 预览 accent
--color-primary-light      ← 预览 accent-light
--color-primary-dark       ← 预览 accent-dark
--color-primary-soft       ← 派生：color-mix(in srgb, primary 16%, transparent)
```

**功能（7）**

```
--color-success / --color-success-soft   ← 预览 success / 派生 14%
--color-gold / --color-gold-soft         ← 预览 gold    / 派生 16%
--color-warning / --color-warning-soft   ← 预览 warning / 派生 14%
--color-neutral                          ← 预览 muted
```

**象限（8）**

```
--color-q1 / --color-q1-soft    ← 按主题色温手挑的红  + 派生 soft
--color-q2 / --color-q2-soft    ← = gold            + 对应 soft
--color-q3 / --color-q3-soft    ← 按主题色温手挑的橙  + 派生 soft
--color-q4 / --color-q4-soft    ← = success         + 对应 soft
```

**表面（4）**

```
--color-bg            ← 预览 bg-primary
--color-bg-elevated   ← 预览 bg-card
--color-bg-subtle     ← 预览 bg-sidebar
--color-bg-hover      ← 预览 bg-hover
```

**文本（3）**

```
--color-text-primary     ← 预览 text-primary
--color-text-secondary   ← 预览 text-secondary
--color-text-muted       ← 取 muted；必要时轻度提亮
```

**描边（3）+ 阴影（1）**

```
--color-border         ← 预览 border
--color-border-strong  ← 派生：color-mix(border 70%, text-primary 30%)
--color-divider        ← 派生：color-mix(border 60%, bg-primary 40%)
--shadow-focus         ← 派生：0 0 0 3px color-mix(primary 30%-40%, transparent)
```

### 3.2 派生函数选型

统一使用现代浏览器支持的 `color-mix(in srgb, ...)` 派生 `*-soft`、`border-strong`、`divider`、`shadow-focus`。原型运行在 Electron / 现代浏览器下，无需 polyfill。

### 3.3 q1 / q3 的手挑策略

`FocusLab-主题预览.html` 每个主题并未给出独立的"红"与"橙"。按主题**色温**手工挑一组协调色，共 22 对（11 主题 × light/dark），结果写入 `tokens.css`：

- 暖色系主题（claude / milktea / amber / sakura）：q1 取主题偏暖红，q3 取主题 warning 基础上略降饱和
- 冷色系主题（green / lavender / blue-classic / teal / slate / candy）：q1 取主题偏冷红（带紫调或带灰调），q3 取偏橘但不过暖
- 中性主题（graphite）：q1 取中性红（#C86464 级别），q3 取中性琥珀

具体颜色值在实现阶段按主题逐一写入，不在本 spec 中预列表。

## 4. 设置页交互与布局

### 4.1 位置

`prototype/settings/settings.html` → `外观` 分区 → 原"主色"一行**彻底替换**为"主题色调"。放置顺序：`主题（浅/深/跟随系统）` → `主题色调` → `字号` → `减少动画`。

### 4.2 卡片数据（12 个）

| id | 标签 | 描述 |
|----|------|------|
| `default` | 🌊 默认蓝 | FocusLab 原版 |
| `claude` | ☁️ 奶油陶土 | 咖啡厅暖阳 |
| `green` | 🌿 现代护眼绿 | 清晨植物园 |
| `lavender` | 🪻 薰衣草紫 | 灵感工作室 |
| `blue-classic` | 🌊 静谧蓝 | 图书馆蓝天 |
| `graphite` | 🧊 极简石墨 | 无印良品 |
| `sakura` | 🌸 樱花粉 | 樱花树下 |
| `candy` | 🎀 糖果粉紫 | 现代粉紫 |
| `milktea` | 🧋 奶茶棕粉 | 奶茶店角落 |
| `amber` | 🍊 琥珀橙 | 秋日暖阳 |
| `teal` | 🦆 水鸭青 | 清澈湖水 |
| `slate` | 🪨 石板蓝灰 | VS Code 气质 |

### 4.3 卡片结构

```
┌─────────────────────┐
│ ● ● ●                │   ← 3 色条：accent / bg / text-primary
│ 🌊 默认蓝            │
│ FocusLab 原版        │
└─────────────────────┘
```

- 布局：3 列 grid，gap 与现有 mode-cards 一致
- 选中态：2px `--color-primary` 边框 + 右上角 ✓
- hover：border 变 `--color-primary-light`，轻微抬高
- 非选中：1px `--color-border`

### 4.4 交互脚本

```js
const accent = localStorage.getItem('fl-accent') || 'default';
applyAccent(accent);

function applyAccent(key) {
  if (key && key !== 'default') {
    document.documentElement.dataset.accentTheme = key;
  } else {
    delete document.documentElement.dataset.accentTheme;
  }
  document.querySelectorAll('.accent-card').forEach(c => {
    c.classList.toggle('is-selected', c.dataset.accent === key);
  });
}

cards.forEach(c => c.addEventListener('click', () => {
  const key = c.dataset.accent;
  localStorage.setItem('fl-accent', key);
  applyAccent(key);
}));
```

## 5. 全局生效机制与持久化

### 5.1 localStorage 键

| 键 | 含义 | 取值 |
|----|------|------|
| `fl-theme` | 明暗模式（已存在） | `light` / `dark` |
| `fl-accent` | 色调主题 | 12 个 id 之一 |

两个键都在 `fl-*` 命名空间下，与现有 `fl-pref-*` 区分。

### 5.2 注入时机

在每个页面 `<head>` 内尽早执行一段内联脚本：

```html
<script>
  try {
    const t = localStorage.getItem('fl-theme');
    const a = localStorage.getItem('fl-accent');
    if (t) document.documentElement.dataset.theme = t;
    if (a && a !== 'default') document.documentElement.dataset.accentTheme = a;
  } catch (e) {}
</script>
```

- 位置：每页 `<head>` 里、在 `<link rel="stylesheet">` 加载后、可见内容渲染前
- 现有页底部的 `fl-theme` 恢复代码迁移进此 `<head>` 脚本，避免二次执行与首屏闪烁
- 同步（非 `async`）执行，容忍 localStorage 被禁的隐身模式

### 5.3 受影响的页面

实现阶段用 `Glob "prototype/**/*.html"` 枚举所有 `.html` 文件，统一注入相同 `<head>` 脚本。当前可见的至少包括：

- `prototype/index.html`
- `prototype/screens/main-today.html`
- `prototype/screens/pomodoro.html`
- `prototype/screens/floating-ball.html`
- `prototype/screens/modals.html`
- `prototype/settings/settings.html`
- 以及 `prototype/screens/` 下其它子页（mood-check、daily-settle、morning-guide、weekly-report 等，按实际扫描结果补全）

### 5.4 跨页同步

监听 `storage` 事件，在同一浏览器其它标签页写入时热应用：

```js
window.addEventListener('storage', (e) => {
  if (e.key === 'fl-accent' || e.key === 'fl-theme') {
    applyFromStorage();
  }
});
```

这段监听同样内联到每页 `<head>` 脚本底部，不走外链 JS。

### 5.5 容错

- `try/catch` 包裹 localStorage 访问，避免隐身模式报错
- 非法 `fl-accent` 值（不在 12 个中）→ 无对应变量块命中，页面回落到默认主题，不报错
- `default` 特殊处理：不写 `data-accent-theme` 属性，避免无效属性污染 DOM

## 6. 兼容性与验证

### 6.1 硬编码色处理

分三类：

1. **保持不动（识别色白名单）**
   - `#8B5CF6 / #A78BFA / #7B52D6` 🌀 自由模式紫罗兰
   - `#FF7A5C` 🍅 番茄计数点
   - 悬浮球桌面渐变
   - 其它已判定为纯装饰的色值

2. **改用 token**
   - 引用 `var(--color-*)` 替换能语义对应上的硬编码（例如 `#AD6800` 一类文字色替换为主题化 token）

3. **无 token 对应时保留硬编码**
   - 本次不扩展 token 集

实现阶段会：

- 用 `Grep "#[0-9A-Fa-f]{3,8}"` 扫 `prototype/**/*.html` 与 `.css`
- 对照白名单过滤识别色
- 其余按语义逐点替换

### 6.2 验证计划

1. **视觉验证**（人工过一遍）
   - 12 主题 × 2 明暗 = 24 组
   - 重点页面：`main-today / pomodoro / settings / modals / floating-ball`
   - 重点元素：主要按钮 / 象限 chip / 完成绿闪 / 🍅 / 🌀 / 评级徽章

2. **自动检查**
   - 计算 `var(--color-*)` 使用点数 vs. 硬编码色值使用点数，确认硬编码只剩白名单

3. **回归清单**
   - 焦点卡 🍅 ↔ 🌀 切换时 ring 颜色
   - pomodoro 4 状态 + 🌀 自由态
   - floating-ball 3 状态 + mini 面板 + 托盘菜单
   - settings 主题卡选中边框
   - modals 命令面板 / AI 隐私 / 日结算弹窗
   - 深色模式对比度与可读性

### 6.3 非功能要求

- 无性能倒退：所有 token 值都是 CSS 变量，不增加运行时开销
- 无额外依赖：不引入 JS 库、不引入 CSS 预处理器
- 不破坏既有 `fl-pref-*` 实验偏好

## 7. 交付物

- `prototype/assets/tokens.css` 新增 22 个主题块（11 主题 × light/dark）+ 注释说明
- `prototype/settings/settings.html` 外观分区：主色行替换为主题色调 12 卡 + 交互脚本
- 所有 `prototype/**/*.html` 的 `<head>` 注入统一主题初始化脚本 + storage 监听
- 所有可被主题化的硬编码色替换为 token
- `docs/05-交互设计与界面规范.md` 追加"主题色板"小节，引用本 spec

## 8. 后续（非本次范围）

- 主题预览页（`FocusLab-主题预览.html`）迁入原型导航
- 自定义主色调色板
- 按时间自动切主题 / 跟随系统强调色
- 主题导入导出
