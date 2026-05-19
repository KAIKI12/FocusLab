# Remove Inspiration 500-Character Limit Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove the 500-character hard cap from inspiration inputs while keeping lightweight character count feedback.

**Architecture:** The backend already accepts long inspiration content, so the change stays in the Vue input layer. We will cover the regression with a source-level component test, then remove the `maxlength` attributes and update the counter copy in both inspiration entry points.

**Tech Stack:** Vue 3, Vite, Vitest, Tauri

---

### Task 1: Guard against reintroducing the 500-character cap

**Files:**
- Modify: `src/components/common/QuickNoteModal.spec.ts`
- Test: `src/components/common/QuickNoteModal.spec.ts`

**Step 1: Write the failing test**

Add assertions that both `QuickNoteModal.vue` and `InspirationPanel.vue` no longer contain `maxlength="500"` or `/ 500` counter copy.

**Step 2: Run test to verify it fails**

Run: `pnpm test -- QuickNoteModal.spec.ts`
Expected: FAIL because both components still contain the 500-character limit.

**Step 3: Write minimal implementation**

Update the two Vue templates to remove the `maxlength` attributes and replace `/ 500` with a plain character count label.

**Step 4: Run test to verify it passes**

Run: `pnpm test -- QuickNoteModal.spec.ts`
Expected: PASS

### Task 2: Remove the hard cap in both inspiration entry points

**Files:**
- Modify: `src/components/common/QuickNoteModal.vue`
- Modify: `src/components/inspiration/InspirationPanel.vue`

**Step 1: Remove the textarea limit in quick note modal**

Delete the `maxlength` attribute from the quick note textarea and keep the existing save / AI behavior unchanged.

**Step 2: Remove the textarea limit in today inspiration panel**

Delete the `maxlength` attribute from the panel textarea and keep the current save shortcut unchanged.

**Step 3: Update counter copy**

Change both counters from `current / 500` to `current 字` so users still get feedback without reading it as a hard ceiling.

**Step 4: Run verification**

Run: `pnpm test -- QuickNoteModal.spec.ts`
Expected: PASS
