import { before, describe, it } from "node:test";
import { elkApp, requireVizeBin } from "../../_helpers/apps.ts";
import { assertInspectorCompareBudgets } from "../../_helpers/inspector-parity.ts";

describe("elk inspector parity with Vue compiler", () => {
  before(requireVizeBin);

  it("tracks DOM and SSR compiler diff budgets for every app Vue file", () => {
    assertInspectorCompareBudgets(elkApp, [
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
    ]);
  });
});
