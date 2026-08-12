//! Negative control for the descendant guard (#4126).
//!
//! This phase passes its own assertion and still leaves a `node` process
//! running. It is the shape of leak the guard exists for: the child is spawned
//! into the phase's process group and then abandoned, so once the phase exits
//! the orphan is reparented to `init` and stops being anyone's descendant
//! while keeping the group it was forked into.
//!
//! It is never listed in `manifest.ts`; only
//! `tests/tooling/check-fixtures-supervisor.test.ts` runs it, to prove the
//! guard turns red.

import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import { test } from "node:test";

import { LEAK_LIFETIME_MS, LEAK_PID_FILE_ENV } from "./control-fixtures.ts";

test("leaks a node descendant on purpose", () => {
  const child = spawn(process.execPath, ["-e", `setTimeout(() => {}, ${LEAK_LIFETIME_MS})`], {
    stdio: "ignore",
  });
  child.unref();
  const pid = child.pid;
  assert.ok(pid != null, "the leaked child must have been forked");
  // Signal 0 checks existence without delivering anything, so the leak is
  // known to be live before the phase returns rather than merely requested.
  process.kill(pid, 0);
  const pidFile = process.env[LEAK_PID_FILE_ENV];
  if (pidFile != null && pidFile.length > 0) {
    fs.writeFileSync(pidFile, String(pid));
  }
});
