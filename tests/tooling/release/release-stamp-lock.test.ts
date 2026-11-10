import assert from "node:assert/strict";
import { test } from "node:test";

import { stampCargoLock } from "../../../tools/support/release/release/stamp.ts";

test("the release stamp updates every workspace-version example crate", () => {
  const examples = [
    "poc-angular",
    "poc-astro",
    "poc-coffeescript",
    "poc-moonbit",
    "poc-python",
    "poc-solid",
    "poc-support",
    "poc-svelte",
  ];
  const packages = [...examples, "serde"]
    .map((name) => `[[package]]\nname = "${name}"\nversion = "0.474.0"`)
    .join("\n\n");

  const stamped = stampCargoLock(packages, "0.474.0", "0.731.0");
  for (const name of examples) {
    assert.match(stamped, new RegExp(`name = "${name}"\\nversion = "0\\.731\\.0"`, "u"));
  }
  assert.match(stamped, /name = "serde"\nversion = "0\.474\.0"/u);
});
