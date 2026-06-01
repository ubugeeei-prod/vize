import { before, describe, it } from "node:test";
import { npmxApp, requireVizeBin } from "../../_helpers/apps.ts";
import { assertInspectorCompareBudgets } from "../../_helpers/inspector-parity.ts";

describe("npmx.dev inspector parity with Vue compiler", () => {
  before(requireVizeBin);

  it("tracks DOM and SSR compiler diff budgets for every app Vue file", () => {
    assertInspectorCompareBudgets(npmxApp, [
      {
        target: "dom",
        changedFiles: 134,
        additions: 12_208,
        removals: 16_337,
        officialErrors: 0,
        vizeErrors: 0,
      },
      {
        target: "ssr",
        changedFiles: 134,
        additions: 9_760,
        removals: 23_460,
        officialErrors: 0,
        vizeErrors: 0,
      },
    ]);
  });
});
