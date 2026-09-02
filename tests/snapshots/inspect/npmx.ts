import { before, describe, it } from "node:test";
import { npmxApp, requireVizeBin } from "../../_helpers/apps.ts";
import { assertInspectorCompareBudgets } from "../../_helpers/inspector-parity.ts";

describe("npmx.dev inspector parity with Vue compiler", () => {
  before(requireVizeBin);

  it("tracks DOM and SSR compiler diff budgets for every app Vue file", () => {
    assertInspectorCompareBudgets(npmxApp, [
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
    ]);
  });
});
