import { describe, expect, it } from "vitest";

import { activeLineIndex, interpolatePosition } from "./lyricsTiming";

const lines = [{ timeMs: 0 }, { timeMs: 1000 }, { timeMs: 2500 }];

describe("activeLineIndex", () => {
  it("returns -1 before the first line and follows timestamps", () => {
    expect(activeLineIndex([{ timeMs: 500 }], 0)).toBe(-1);
    expect(activeLineIndex(lines, 0)).toBe(0);
    expect(activeLineIndex(lines, 999)).toBe(0);
    expect(activeLineIndex(lines, 1000)).toBe(1);
    expect(activeLineIndex(lines, 2499)).toBe(1);
    expect(activeLineIndex(lines, 2500)).toBe(2);
  });

  it("applies metadata offset around boundaries", () => {
    expect(activeLineIndex(lines, 980, 0)).toBe(0);
    expect(activeLineIndex(lines, 980, 20)).toBe(1);
    expect(activeLineIndex(lines, 1020, -20)).toBe(1);
    expect(activeLineIndex(lines, 2600, -150)).toBe(1);
  });

  it("treats missing timestamps as zero and tolerates empty input", () => {
    expect(activeLineIndex([], 0)).toBe(-1);
    expect(activeLineIndex([{ timeMs: null }, { timeMs: 900 }], 100)).toBe(0);
  });
});

describe("interpolatePosition", () => {
  it("advances while playing and clamps to duration", () => {
    const anchor = { positionMs: 1000, playing: true, atMs: 5000 };
    expect(interpolatePosition(anchor, 5000, 0)).toBe(1000);
    expect(interpolatePosition(anchor, 5240, 0)).toBe(1240);
    expect(interpolatePosition(anchor, 9000, 3000)).toBe(3000);
  });

  it("freezes while paused and never goes negative", () => {
    const anchor = { positionMs: 1000, playing: false, atMs: 5000 };
    expect(interpolatePosition(anchor, 8000, 0)).toBe(1000);
    expect(interpolatePosition({ positionMs: 0, playing: true, atMs: 100 }, 50, 0)).toBe(0);
  });
});
