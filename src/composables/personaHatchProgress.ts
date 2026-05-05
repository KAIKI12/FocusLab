const HATCH_TOTAL_DAYS = 7;

function normalizeSettlementDays(settlementDays: number): number {
  if (!Number.isFinite(settlementDays) || settlementDays < 0) {
    return 0;
  }

  return Math.floor(settlementDays);
}

export function getDisplayHatchDay(settlementDays: number): number {
  const normalizedDays = normalizeSettlementDays(settlementDays);

  if (normalizedDays === 0) {
    return 1;
  }

  return Math.min(normalizedDays, HATCH_TOTAL_DAYS);
}

export function getRemainingHatchDays(settlementDays: number): number {
  const normalizedDays = normalizeSettlementDays(settlementDays);

  return Math.max(HATCH_TOTAL_DAYS - normalizedDays, 0);
}

export function isHatchComplete(settlementDays: number): boolean {
  return normalizeSettlementDays(settlementDays) >= HATCH_TOTAL_DAYS;
}
