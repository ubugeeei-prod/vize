import { describe, expect, it } from "vite-plus/test";
import {
  buildSuppressionMap,
  filterSuppressedIssues,
  offsetToLineColumn,
  parseSuppressions,
} from "./utils";
import type { CrossFileIssue } from "./types";

function createIssue(file: string, line: number): CrossFileIssue {
  return {
    id: `${file}:${line}`,
    type: "reactivity",
    code: "cross-file",
    severity: "warning",
    message: "test issue",
    file,
    line,
    column: 1,
  };
}

describe("cross-file utils", () => {
  it("reuses the shared offset mapper", () => {
    expect(offsetToLineColumn("alpha\nbeta", 6)).toEqual({ line: 2, column: 1 });
  });

  it("maps forget comments to the next code line", () => {
    const source = [
      "// @vize forget: generated import",
      "// explanatory comment",
      "const generated = true",
      "const kept = true",
      "/* @vize forget: macro output */",
      "defineProps<{ id: string }>()",
    ].join("\n");

    expect([...parseSuppressions(source)]).toEqual([3, 6]);
  });

  it("filters only issues on suppressed lines", () => {
    const suppressionMap = buildSuppressionMap({
      "App.vue": "// @vize forget: generated\nconst generated = true",
    });
    const issues = [
      createIssue("App.vue", 2),
      createIssue("App.vue", 3),
      createIssue("Other.vue", 2),
    ];

    expect(filterSuppressedIssues(issues, suppressionMap)).toEqual([issues[1], issues[2]]);
  });
});
