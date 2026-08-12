//! Positive control for the descendant guard (#4126).
//!
//! The same shape as `leaked-child-fixture.ts`, except the child is awaited.
//! Without it the negative control would only prove the guard can be red, not
//! that it is quiet when a phase behaves — which is the difference between a
//! guard and a permanent failure.

import { spawnSync } from "node:child_process";
import assert from "node:assert/strict";
import { test } from "node:test";

test("spawns a node descendant and waits for it", () => {
  const result = spawnSync(process.execPath, ["-e", "process.exit(0)"], { stdio: "ignore" });
  assert.equal(result.status, 0);
});
