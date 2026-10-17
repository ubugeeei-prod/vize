import { before, describe, it } from "node:test";
import { elkApp, requireVizeBin } from "../../_helpers/apps.ts";
import {
  assertInspectorCompareBudgets,
  type InspectorDiffBudget,
} from "../../_helpers/inspector-parity.ts";

describe("elk inspector parity with Vue compiler", () => {
  before(requireVizeBin);

  const budgets: InspectorDiffBudget[] = [
    {
      target: "dom",
      changedFiles: 259,
      // The current pinned elk fixture has four fewer added lines and one
      // more removed line than the previous budget: three fewer diff lines.
      additions: 10_761,
      removals: 13_778,
      officialErrors: 3,
      vizeErrors: 0,
    },
    {
      target: "ssr",
      changedFiles: 259,
      additions: 9_201,
      removals: 25_188,
      officialErrors: 3,
      vizeErrors: 0,
    },
  ];
  for (const budget of budgets) {
    it(`tracks ${budget.target.toUpperCase()} compiler diff budgets for every app Vue file`, () => {
      assertInspectorCompareBudgets(elkApp, [budget], {
        fileCount: 259,
        pathsSha256: "d02a84ea90084e448e52b595e36d740296f72f8d6f16d8f71d2c35ee5a63e6f0",
      });
    });
  }
});
