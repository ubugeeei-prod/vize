import { describe, expect, it } from "vite-plus/test";
import { formatNanos } from "./format";
import { summarizeRemarks } from "./remarks";

describe("formatNanos", () => {
  it("reads sub-microsecond work as under 1 µs and scales up to milliseconds", () => {
    expect([0, 999, 1_000, 5_000, 12_499, 999_499, 1_000_000, 2_345_678].map(formatNanos)).toEqual([
      "<1 µs",
      "<1 µs",
      "1 µs",
      "5 µs",
      "12 µs",
      "999 µs",
      "1.00 ms",
      "2.35 ms",
    ]);
  });
});

describe("summarizeRemarks", () => {
  it("counts applied and missed remarks", () => {
    const remark = (applied: boolean) => ({
      pass: "hoist-static",
      message: "m",
      applied,
      span: null,
    });
    expect(summarizeRemarks([])).toEqual({ applied: 0, missed: 0 });
    expect(summarizeRemarks([remark(true), remark(false), remark(false)])).toEqual({
      applied: 1,
      missed: 2,
    });
  });
});
