import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

test("App check preparation and whole packets survive failed row uploads", () => {
  const action = parse(
    readRepoFile(".github", "actions", "app-e2e-row", "action.yml"),
  ) as {
    runs: {
      steps: Array<{
        name?: string;
        if?: string;
        env?: Record<string, string>;
        with?: Record<string, string>;
      }>;
    };
  };
  const run = action.runs.steps.find(
    (step) => step.name === "Run planned App E2E row",
  );
  const upload = action.runs.steps.find(
    (step) => step.name === "Upload App E2E row artifacts",
  );
  assert.ok(run && upload);
  const capture = run.env?.VIZE_APP_CHECK_CAPTURE_DIR;
  assert.ok(
    capture,
    "Nuxt preparation otherwise retains authority only in runner /tmp",
  );
  assert.ok(capture.startsWith("${{ github.workspace }}/"));
  const relative = capture.slice("${{ github.workspace }}/".length);
  assert.ok(!path.posix.normalize(relative).startsWith("../"));
  const roots = (upload.with?.path ?? "").split("\n").filter(Boolean);
  assert.ok(
    roots.some((root) => root.endsWith("/") && relative.startsWith(root)),
    "the whole capture tree must be covered by the existing artifact upload",
  );
  assert.equal(
    upload.if,
    "${{ cancelled() || failure() || steps.run.outcome == 'failure' }}",
  );
  assert.match(
    readRepoFile("tests", "_helpers", "nuxt-check-prepare.ts"),
    /process\.env\.VIZE_APP_CHECK_CAPTURE_DIR/,
  );
  assert.match(
    readRepoFile("tests", "_helpers", "app-check-packet.ts"),
    /process\.env\.VIZE_APP_CHECK_CAPTURE_DIR/,
  );
});
