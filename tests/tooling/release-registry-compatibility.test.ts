import assert from "node:assert/strict";
import { test } from "node:test";

import { releaseRegistryFixture } from "./support/release-registry-fixture.ts";

test("raw direct Moon release retains its policy without a task freshness request", () => {
  const fixture = releaseRegistryFixture(37);
  try {
    delete fixture.env.VIZE_RELEASE_REGISTRY_REFRESH;
    const result = fixture.run(["minor", "-y"]);
    assert.equal(result.status, 83, result.stderr + result.stdout);
    assert.deepEqual(
      fixture.events().map((event) => event.kind),
      ["driver"],
    );
    assert.doesNotMatch(result.stdout + result.stderr, /registry stdout|registry stderr/);
    fixture.assertUnchanged();
  } finally {
    fixture.dispose();
  }
});

test("invalid task freshness markers fail before parser output or child dispatch", () => {
  const fixture = releaseRegistryFixture();
  try {
    fixture.env.VIZE_RELEASE_REGISTRY_REFRESH = "unexpected";
    const result = fixture.run(["minor", "-y"]);
    assert.equal(result.status, 1, result.stderr + result.stdout);
    assert.equal(result.stdout, "");
    assert.equal(result.stderr, "Invalid task registry freshness request\n");
    assert.deepEqual(fixture.events(), []);
    fixture.assertUnchanged();
  } finally {
    fixture.dispose();
  }
});
