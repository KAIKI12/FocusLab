export const TASKS_CHANGED_EVENT = "focuslab:data:tasks-changed";
export const INSPIRATIONS_CHANGED_EVENT = "focuslab:data:inspirations-changed";

const QUICK_ADD_PREFILL_KEY = "focuslab:quick-add-prefill";

export interface QuickAddPrefillPayload {
  quadrant?: string;
  title: string;
}

export function setQuickAddPrefill(payload: QuickAddPrefillPayload) {
  localStorage.setItem(QUICK_ADD_PREFILL_KEY, JSON.stringify(payload));
}

export function consumeQuickAddPrefill(): QuickAddPrefillPayload | null {
  const raw = localStorage.getItem(QUICK_ADD_PREFILL_KEY);
  if (!raw) return null;

  localStorage.removeItem(QUICK_ADD_PREFILL_KEY);

  try {
    const parsed = JSON.parse(raw) as Partial<QuickAddPrefillPayload>;
    const title = String(parsed.title ?? "").trim();
    if (!title) return null;
    const quadrant = typeof parsed.quadrant === "string" && parsed.quadrant.trim()
      ? parsed.quadrant
      : undefined;
    return { title, quadrant };
  } catch {
    return null;
  }
}
