import { describe, expect, it } from "vite-plus/test";
import { buildLineDiff, getDiffStats } from "./diff";

describe("buildLineDiff", () => {
  it("creates a full unified line diff", () => {
    const diff = buildLineDiff("one\ntwo\nthree", "one\nTWO\nthree\nfour");

    expect(diff).toEqual([
      { kind: "same", leftLine: 1, rightLine: 1, text: "one" },
      { kind: "remove", leftLine: 2, rightLine: null, text: "two" },
      { kind: "add", leftLine: null, rightLine: 2, text: "TWO" },
      { kind: "same", leftLine: 3, rightLine: 3, text: "three" },
      { kind: "add", leftLine: null, rightLine: 4, text: "four" },
    ]);
    expect(getDiffStats(diff)).toEqual({
      additions: 2,
      removals: 1,
      unchanged: 2,
    });
  });
});
