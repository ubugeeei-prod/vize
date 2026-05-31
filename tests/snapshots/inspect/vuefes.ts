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
    ]);
  });
});
