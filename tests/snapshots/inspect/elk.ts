import { before, describe, it } from "node:test";
import { elkApp, requireVizeBin } from "../../_helpers/apps.ts";
import { assertInspectorCompareBudgets } from "../../_helpers/inspector-parity.ts";

describe("elk inspector parity with Vue compiler", () => {
  before(requireVizeBin);

  it("tracks DOM and SSR compiler diff budgets for every app Vue file", () => {
    assertInspectorCompareBudgets(elkApp, [
      {
        target: "dom",
        changedFiles: 256,
        additions: 11_220,
        removals: 14_198,
        officialErrors: 3,
        // Both compilers reject the same two unresolved #shared macro types;
        // Vue additionally rejects floating-vue's unresolved extends base.
        vizeErrors: 2,
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
