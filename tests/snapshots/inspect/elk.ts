import { before, describe, it } from "node:test";
import { elkApp, requireVizeBin } from "../../_helpers/apps.ts";
import { assertInspectorCompareBudgets } from "../../_helpers/inspector-parity.ts";

describe("elk inspector parity with Vue compiler", () => {
  before(requireVizeBin);

  it("tracks DOM and SSR compiler diff budgets for every app Vue file", () => {
    assertInspectorCompareBudgets(elkApp, [
      {
        target: "dom",
        changedFiles: 253,
        additions: 10_439,
        removals: 13_185,
        officialErrors: 3,
        vizeErrors: 0,
      },
      {
        target: "ssr",
        changedFiles: 253,
        additions: 9_131,
        removals: 24_456,
        officialErrors: 3,
        vizeErrors: 0,
      },
    ]);
  });
});
