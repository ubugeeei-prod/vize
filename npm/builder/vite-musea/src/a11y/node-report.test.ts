import assert from "node:assert/strict";
import test from "node:test";
import type { A11yResult, A11yViolation } from "../types/index.ts";
import { computeA11ySummary, generateA11yHtmlReport, generateA11yJsonReport } from "./report.ts";

const finding: A11yViolation = {
  id: "color-contrast",
  impact: "serious",
  description: "Contrast",
  helpUrl: "https://example.test/rule",
  nodes: 1,
  targets: [
    {
      target: [["iframe", "#label"]],
      html: '<span onclick="bad()">Label</span>',
      failureSummary: "Insufficient contrast <4.5:1",
      any: [
        {
          id: "color-contrast",
          impact: "serious",
          message: "Contrast",
          data: {
            fgColor: "#bbbbbb",
            bgColor: "#ffffff",
            contrastRatio: 1.91,
            expectedContrastRatio: "4.5:1",
          },
        },
      ],
      all: [],
      none: [],
    },
  ],
};
const result: A11yResult = {
  artPath: "Card.art.vue",
  variantName: "Default",
  violations: [finding],
  passes: 3,
  incomplete: 1,
  incompleteResults: [{ ...finding, id: "needs-manual-review", impact: null }],
};

void test("JSON retains all target, HTML, failure, and check data for violations and review", () => {
  const report = JSON.parse(generateA11yJsonReport([result]));
  assert.deepEqual(report.results[0].violations, result.violations);
  assert.deepEqual(report.results[0].incompleteResults, result.incompleteResults);
  assert.equal(report.results[0].incomplete, 1);
  assert.equal(report.summary.totalViolations, 1);
});

void test("HTML explains affected nodes and escapes selectors, markup, and check data", () => {
  const html = generateA11yHtmlReport([result], computeA11ySummary([result]));
  for (const detail of ["iframe", "#label", "#bbbbbb", "#ffffff", "1.91", "4.5:1", "needs review"])
    assert.ok(html.includes(detail), detail);
  assert.ok(html.includes("&lt;span onclick=&quot;bad()&quot;&gt;"));
  assert.ok(!html.includes('<span onclick="bad()">'));
  assert.ok(html.includes("Insufficient contrast &lt;4.5:1"));
});

void test("review-only results remain visible and older count-only reports still render", () => {
  const review = { ...result, violations: [] };
  const html = generateA11yHtmlReport([review], computeA11ySummary([review]));
  assert.ok(html.includes("needs-manual-review"));
  assert.ok(!html.includes("No accessibility violations found"));
  const legacy = {
    ...result,
    violations: [{ ...finding, targets: undefined }],
    incomplete: 0,
    incompleteResults: undefined,
  };
  assert.ok(
    generateA11yHtmlReport([legacy], computeA11ySummary([legacy])).includes("color-contrast"),
  );
});
