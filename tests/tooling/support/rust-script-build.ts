import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";

/**
 * Compile a rust-script command before a test times it. rust-script builds on
 * first use, and a cold build (crate downloads plus compile) can exceed a
 * timing budget that is meant to bound only the command's own behavior.
 * `argv` must make the command exit right after argument parsing.
 */
export function buildRustScript(script: string, argv: string[]) {
  const result = spawnSync("rust-script", [script, ...argv], { encoding: "utf8" });
  assert.equal(result.error, undefined, `${script}: rust-script did not start`);
  assert.doesNotMatch(
    result.stderr,
    /Could not execute cargo|could not compile|failed to get `/,
    `${script}: rust-script build failed\n${result.stderr}`,
  );
}
