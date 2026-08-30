import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { fuzzResultPolicy } from "../../tools/fuzz/enforce-result.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const command = path.join(root, "tools/commands/ci/fuzz/enforce-result.rs");

test("fuzz result policy keeps PRs advisory and release evidence strict", () => {
  for (const outcome of ["failure", "cancelled", "skipped"]) {
    assert.deepEqual(fuzzResultPolicy("pull_request", outcome), {
      unsuccessful: true,
      releaseBlocking: false,
    });
    for (const eventName of ["schedule", "workflow_dispatch"]) {
      assert.deepEqual(fuzzResultPolicy(eventName, outcome), {
        unsuccessful: true,
        releaseBlocking: true,
      });
    }
  }
  assert.deepEqual(fuzzResultPolicy("workflow_dispatch", "success"), {
    unsuccessful: false,
    releaseBlocking: false,
  });
  assert.throws(() => fuzzResultPolicy("push", "failure"), /Unsupported fuzz event/);
  assert.throws(() => fuzzResultPolicy("schedule", ""), /Unsupported fuzz outcome/);
});

test("fuzz result command reports the target, event, outcome, and gate decision", () => {
  const advisory = spawnSync("rust-script", [command, "pull_request", "sfc_parse", "failure"], {
    encoding: "utf8",
  });
  assert.equal(advisory.status, 0);
  assert.match(advisory.stderr, /warning.*sfc_parse.*failure.*pull_request/i);

  const blocking = spawnSync(
    "rust-script",
    [command, "workflow_dispatch", "template_lexer", "failure"],
    { encoding: "utf8" },
  );
  assert.equal(blocking.status, 1);
  assert.match(blocking.stderr, /error.*template_lexer.*failure.*workflow_dispatch/i);
});

test("fuzz result command rejects malformed invocation", () => {
  const result = spawnSync("rust-script", [command, "pull_request", "sfc_parse"], {
    encoding: "utf8",
  });

  assert.equal(result.status, 1);
  assert.match(result.stderr, /Usage: rust-script tools\/commands\/ci\/fuzz\/enforce-result\.rs/);
});
