export type SourceHandleId = "s-top" | "s-right" | "s-bottom" | "s-left";
export type TargetHandleId = "t-top" | "t-right" | "t-bottom" | "t-left";

interface GraphPoint {
  x: number;
  y: number;
}

interface GraphRect {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

const AUTO_PAN_EDGE_THRESHOLD = 72;
const AUTO_PAN_STEP = 24;

export function getEdgeHandles(
  source: GraphPoint,
  target: GraphPoint,
): { sourceHandle: SourceHandleId; targetHandle: TargetHandleId } {
  const dx = target.x - source.x;
  const dy = target.y - source.y;

  if (Math.abs(dx) >= Math.abs(dy)) {
    return dx >= 0
      ? { sourceHandle: "s-right", targetHandle: "t-left" }
      : { sourceHandle: "s-left", targetHandle: "t-right" };
  }

  return dy >= 0
    ? { sourceHandle: "s-bottom", targetHandle: "t-top" }
    : { sourceHandle: "s-top", targetHandle: "t-bottom" };
}

export function getAutoPanDelta(
  point: GraphPoint,
  rect: GraphRect,
  threshold = AUTO_PAN_EDGE_THRESHOLD,
  step = AUTO_PAN_STEP,
): { x: number; y: number } {
  let x = 0;
  let y = 0;

  if (point.x <= rect.left + threshold) x = step;
  else if (point.x >= rect.right - threshold) x = -step;

  if (point.y <= rect.top + threshold) y = step;
  else if (point.y >= rect.bottom - threshold) y = -step;

  return { x, y };
}
