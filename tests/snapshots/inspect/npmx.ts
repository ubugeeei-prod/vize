import { before, describe, it } from "node:test";
import { npmxApp, requireVizeBin } from "../../_helpers/apps.ts";
import {
  assertInspectorCompareBudgets,
  type InspectorDiffBudget,
} from "../../_helpers/inspector-parity.ts";

describe("npmx.dev inspector parity with Vue compiler", () => {
  before(requireVizeBin);

  const budgets: InspectorDiffBudget[] = [
    {
      target: "dom",
      changedFiles: 220,
      additions: 17_122,
      removals: 23_081,
      officialErrors: 0,
      vizeErrors: 0,
    },
    {
      target: "ssr",
      changedFiles: 220,
      additions: 14_510,
      removals: 36_627,
      officialErrors: 0,
      vizeErrors: 0,
    },
  ];
  for (const budget of budgets) {
    it(`tracks ${budget.target.toUpperCase()} compiler diff budgets for every app Vue file`, () => {
      assertInspectorCompareBudgets(npmxApp, [budget], {
        fileCount: 220,
        pathsSha256: "1414645911bc91961dd8bd248a08e6cb2eec188695ae8cdbb5ae1672cb3f4bec",
      });
    });
  }
});
