import { describe, expect, it } from "vite-plus/test";
import { formatArg, remarksAt, summarizeRemarks, type StageRemark } from "./remarks";

const remark = (
  kind: StageRemark["kind"],
  name: string,
  span: [number, number],
  args: StageRemark["args"] = [],
): StageRemark => ({
  stage: "l2",
  pass: "hoist-static",
  kind,
  name,
  span: { start: span[0], end: span[1] },
  args,
});

// The first remarks the compiler emits for the Todo board preset.
const REMARKS = [
  remark("applied", "static-props", [3, 383], [{ key: "tag", value: "section" }]),
  remark(
    "missed",
    "static-subtree",
    [3, 383],
    [
      { key: "tag", value: "section" },
      { key: "blocker", value: "child" },
      { key: "op", value: "ui.element" },
    ],
  ),
  remark("applied", "static-subtree", [31, 47], [{ key: "tag", value: "h1" }]),
  remark(
    "analysis",
    "depth",
    [31, 47],
    [
      { key: "levels", value: 3 },
      { key: "nested", value: true },
    ],
  ),
];

describe("remarks", () => {
  it("counts every kind", () => {
    expect(summarizeRemarks(REMARKS)).toEqual({ applied: 2, missed: 1, analysis: 1 });
    expect(summarizeRemarks([])).toEqual({ applied: 0, missed: 0, analysis: 0 });
  });

  it("formats typed arguments as key=value", () => {
    expect(REMARKS[1].args.map(formatArg)).toEqual([
      "tag=section",
      "blocker=child",
      "op=ui.element",
    ]);
    expect(REMARKS[3].args.map(formatArg)).toEqual(["levels=3", "nested=true"]);
  });

  it("selects the remarks about exactly one construct", () => {
    expect(
      remarksAt(REMARKS, { start: 3, end: 383 }).map(({ kind, name }) => [kind, name]),
    ).toEqual([
      ["applied", "static-props"],
      ["missed", "static-subtree"],
    ]);
    expect(remarksAt(REMARKS, { start: 3, end: 382 })).toEqual([]);
  });
});
