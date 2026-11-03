import assert from "node:assert/strict";
import { test } from "node:test";

import { metadata } from "./support/level-stage-dependencies.ts";

test("the retired compatibility runtime cannot rejoin the workspace or its declared graph", () => {
  for (const pkg of metadata.packages) {
    assert.notEqual(pkg.name, "vize_level");
    for (const dependency of pkg.dependencies) {
      assert.notEqual(dependency.name, "vize_level", `${pkg.name}: ${dependency.kind}`);
    }
  }
});
