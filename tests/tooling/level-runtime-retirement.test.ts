import assert from "node:assert/strict";
import { test } from "node:test";

import { metadata } from "./support/davinci-stage-dependencies.ts";

test("the retired compatibility runtime cannot rejoin the workspace or its declared graph", () => {
  for (const pkg of metadata.packages) {
    assert.notEqual(pkg.name, "vize_davinci");
    for (const dependency of pkg.dependencies) {
      assert.notEqual(dependency.name, "vize_davinci", `${pkg.name}: ${dependency.kind}`);
    }
  }
});
