import { before, describe, it } from "node:test";
import { requireVizeBin, vuefesApp } from "../../_helpers/apps.ts";
import {
  assertInspectorCompareBudgets,
  type InspectorDiffBudget,
} from "../../_helpers/inspector-parity.ts";

describe("vuefes-2025 inspector parity with Vue compiler", () => {
  before(requireVizeBin);

  const budgets: InspectorDiffBudget[] = [
    {
      target: "dom",
      changedFiles: 71,
      additions: 2_077,
      removals: 2_487,
      officialErrors: 2,
      vizeErrors: 0,
    },
    {
      target: "ssr",
      changedFiles: 71,
      additions: 3_397,
      removals: 7_400,
      officialErrors: 2,
      vizeErrors: 0,
    },
  ];
  for (const budget of budgets) {
    it(`tracks ${budget.target.toUpperCase()} compiler diff budgets for every app Vue file`, () => {
      assertInspectorCompareBudgets(vuefesApp, [budget], {
        fileCount: 71,
        pathsSha256: "ab1162e3967372fa4f651c1cd39d4ceb37b85de84d79769ff908f6a8935a8c72",
      });
    });
  }
});
