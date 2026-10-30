import assert from "node:assert/strict";
import { test } from "node:test";
import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

test("instance-global qualification requires all three source-bound hosts by default", () => {
  const workflow = readRepoFile(".github", "workflows", "content-mapper-instance-globals.yml");
  const job = workflowJobBody(workflow, "real-beta-host");
  assert.match(
    workflow,
    /capture-only:\n\s+description: Retain discovery observations without qualification credit\n\s+required: false\n\s+type: boolean\n\s+default: false/u,
  );
  assert.match(
    job,
    /VIZE_TEST_GLOBALS_DISCOVERY: \$\{\{ github\.event_name == 'workflow_dispatch' && inputs\.capture-only && '1' \|\| '0' \}\}/u,
    "only an explicitly requested manual capture may use discovery mode",
  );
  assert.match(job, /VIZE_TEST_GLOBALS_HOST_MATRIX_REQUIRED: "1"/u);
  assert.match(job, /VIZE_LSP_REQUIRE_SOURCE_BUILD: "1"/u);
  assert.match(job, /VIZE_TEST_GLOBALS_REQUIRED: "1"/u);
  assert.match(job, /VIZE_TEST_BIN: \$\{\{ github\.workspace \}\}\/target\/ci\/vize/u);
  assert.match(
    job,
    /VIZE_TEST_GLOBALS_SOURCE_HOST: \$\{\{ runner\.temp \}\}\/instance-globals-source-host\/tsgo/u,
  );
  assert.match(
    job,
    /VIZE_TEST_GLOBALS_SOURCE_HOST_RECEIPT: \$\{\{ runner\.temp \}\}\/instance-globals-10617\/source-host\.json/u,
  );
  assert.match(job, /timeout-minutes: 25/u, "retain the original campaign resource bound");
  assert.match(job, /if: always\(\)/u, "retain actual packets after failed qualification");
  assert.match(
    job,
    /run: >-\n\s+vp node --test --test-concurrency=1\n\s+tests\/tooling\/content-mapper-instance-globals-packet-authority\.test\.ts\n\s+tests\/tooling\/content-mapper-instance-globals-10617\.test\.ts/u,
  );
});
