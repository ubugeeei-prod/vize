import { before, describe, it } from "node:test";
import { requireVizeBin, vuefesApp } from "../../_helpers/apps.ts";
import { assertInspectorCompareBudgets } from "../../_helpers/inspector-parity.ts";

describe("vuefes-2025 inspector parity with Vue compiler", () => {
  before(requireVizeBin);

  it("tracks DOM and SSR compiler diff budgets for every app Vue file", () => {
    assertInspectorCompareBudgets(vuefesApp, [
      {
        target: "dom",
        changedFiles: 71,
        additions: 3_268,
        removals: 3_706,
        officialErrors: 0,
        vizeErrors: 0,
      },
      {
        target: "ssr",
        changedFiles: 71,
        additions: 7_575,
        removals: 10_005,
        officialErrors: 0,
        vizeErrors: 0,
      },
    ]);
  });
});
