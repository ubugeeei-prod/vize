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

import { spawn } from "node:child_process";
import { test } from "node:test";

/** How long the leaked child stays alive if nothing reaps it. */
export const LEAK_LIFETIME_MS = 60_000;

test("leaks a node descendant on purpose", () => {
  const child = spawn(process.execPath, ["-e", `setTimeout(() => {}, ${LEAK_LIFETIME_MS})`], {
    stdio: "ignore",
  });
  child.unref();
});
