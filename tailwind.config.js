/**
 * FocusLab · Tailwind 配置
 *
 * 策略: tokens.css 是样式 SSoT, Tailwind 做原子类工具层,颜色 / 间距 / 圆角 / 阴影
 * 全部引用 CSS 变量 — 主题切换靠切 `data-theme` / `data-accent-theme` 即可全栈联动。
 *
 * darkMode 用 selector 指到 tokens.css 的 `data-theme="dark"` 约定,不走默认的 `.dark` class。
 */

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,ts,js,tsx,jsx}",
  ],

  darkMode: ["selector", '[data-theme="dark"]'],

  theme: {
    extend: {
      colors: {
        // 品牌
        primary: "var(--color-primary)",
        "primary-light": "var(--color-primary-light)",
        "primary-dark": "var(--color-primary-dark)",
        "primary-soft": "var(--color-primary-soft)",

        // 功能色
        success: "var(--color-success)",
        "success-soft": "var(--color-success-soft)",
        "success-text": "var(--color-success-text)",
        gold: "var(--color-gold)",
        "gold-soft": "var(--color-gold-soft)",
        "gold-text": "var(--color-gold-text)",
        warning: "var(--color-warning)",
        "warning-soft": "var(--color-warning-soft)",
        "warning-text": "var(--color-warning-text)",
        neutral: "var(--color-neutral)",

        // 四象限
        q1: "var(--color-q1)",
        "q1-soft": "var(--color-q1-soft)",
        "q1-text": "var(--color-q1-text)",
        q2: "var(--color-q2)",
        "q2-soft": "var(--color-q2-soft)",
        q3: "var(--color-q3)",
        "q3-soft": "var(--color-q3-soft)",
        "q3-text": "var(--color-q3-text)",
        q4: "var(--color-q4)",
        "q4-soft": "var(--color-q4-soft)",

        // 表面
        bg: "var(--color-bg)",
        "bg-elevated": "var(--color-bg-elevated)",
        "bg-subtle": "var(--color-bg-subtle)",
        "bg-hover": "var(--color-bg-hover)",

        // 文本
        "text-primary": "var(--color-text-primary)",
        "text-secondary": "var(--color-text-secondary)",
        "text-muted": "var(--color-text-muted)",
        "text-on-primary": "var(--color-text-on-primary)",

        // 描边 / 分割
        border: "var(--color-border)",
        "border-strong": "var(--color-border-strong)",
        divider: "var(--color-divider)",
      },

      fontFamily: {
        sans: "var(--font-sans)",
        mono: "var(--font-mono)",
      },

      fontSize: {
        "fl-12": "var(--fs-12)",
        "fl-14": "var(--fs-14)",
        "fl-16": "var(--fs-16)",
        "fl-20": "var(--fs-20)",
        "fl-24": "var(--fs-24)",
        "fl-32": "var(--fs-32)",
      },

      spacing: {
        "fl-1": "var(--sp-1)",
        "fl-2": "var(--sp-2)",
        "fl-3": "var(--sp-3)",
        "fl-4": "var(--sp-4)",
        "fl-5": "var(--sp-5)",
        "fl-6": "var(--sp-6)",
        "fl-8": "var(--sp-8)",
        "fl-10": "var(--sp-10)",
        "fl-12": "var(--sp-12)",
        "fl-16": "var(--sp-16)",
      },

      borderRadius: {
        "fl-xs": "var(--r-xs)",
        "fl-sm": "var(--r-sm)",
        "fl-md": "var(--r-md)",
        "fl-lg": "var(--r-lg)",
        "fl-pill": "var(--r-pill)",
      },

      boxShadow: {
        card: "var(--shadow-card)",
        float: "var(--shadow-float)",
        modal: "var(--shadow-modal)",
        focus: "var(--shadow-focus)",
      },

      transitionDuration: {
        fast: "var(--dur-fast)",
        base: "var(--dur-base)",
        slow: "var(--dur-slow)",
        emph: "var(--dur-emph)",
      },

      zIndex: {
        sidebar: "var(--z-sidebar)",
        sticky: "var(--z-sticky)",
        dropdown: "var(--z-dropdown)",
        modal: "var(--z-modal)",
        toast: "var(--z-toast)",
        tooltip: "var(--z-tooltip)",
      },
    },
  },

  plugins: [],
};
