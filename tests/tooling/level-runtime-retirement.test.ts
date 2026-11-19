import assert from "node:assert/strict";
import { test } from "node:test";

import { metadata } from "./support/level-stage-dependencies.ts";

// The retired runtime crate's published name, spelled in parts so the codename rename keeps it.
const retired = ["vize", "da" + "vinci"].join("_");

test("the retired compatibility runtime cannot rejoin the workspace or its declared graph", () => {
  for (const pkg of metadata.packages) {
    assert.notEqual(pkg.name, retired);
    for (const dependency of pkg.dependencies) {
      assert.notEqual(dependency.name, retired, `${pkg.name}: ${dependency.kind}`);
    }
  }
});
