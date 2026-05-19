import { describe, expect, it } from "vitest";

import {
  getAutoPanDelta,
  getEdgeHandles,
} from "@/components/inspiration/graphGeometry";

describe("graphGeometry", () => {
  it("selects horizontal handles when nodes are farther apart on x axis", () => {
    expect(
      getEdgeHandles(
        { x: 120, y: 100 },
        { x: 360, y: 140 },
      ),
    ).toEqual({
      sourceHandle: "s-right",
      targetHandle: "t-left",
    });

    expect(
      getEdgeHandles(
        { x: 320, y: 180 },
        { x: 120, y: 160 },
      ),
    ).toEqual({
      sourceHandle: "s-left",
      targetHandle: "t-right",
    });
  });

  it("selects vertical handles when nodes are farther apart on y axis", () => {
    expect(
      getEdgeHandles(
        { x: 180, y: 120 },
        { x: 200, y: 360 },
      ),
    ).toEqual({
      sourceHandle: "s-bottom",
      targetHandle: "t-top",
    });

    expect(
      getEdgeHandles(
        { x: 180, y: 320 },
        { x: 150, y: 120 },
      ),
    ).toEqual({
      sourceHandle: "s-top",
      targetHandle: "t-bottom",
    });
  });

  it("pans viewport when dragging close to the stage edge", () => {
    const rect = { left: 0, top: 0, right: 1000, bottom: 700 };

    expect(
      getAutoPanDelta(
        { x: 985, y: 350 },
        rect,
      ),
    ).toEqual({ x: -24, y: 0 });

    expect(
      getAutoPanDelta(
        { x: 18, y: 680 },
        rect,
      ),
    ).toEqual({ x: 24, y: -24 });
  });

  it("does not pan when pointer stays within the safe zone", () => {
    const rect = { left: 0, top: 0, right: 1000, bottom: 700 };

    expect(
      getAutoPanDelta(
        { x: 500, y: 350 },
        rect,
      ),
    ).toEqual({ x: 0, y: 0 });
  });
});
