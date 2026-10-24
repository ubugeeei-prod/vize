import assert from "node:assert/strict";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { parse as parseYaml } from "yaml";
import { patchTitleCall, repoRoot, runPolicy } from "./support/title-policy.ts";

test("the real sparse checkout can compile and execute the title policy with its local Cargo library", () => {
  const workflow = parseYaml(
    readFileSync(join(repoRoot, ".github/workflows/title-policy.yml"), "utf8"),
  ) as {
    jobs: Record<string, { steps: Array<{ uses?: string; with?: Record<string, string> }> }>;
  };
  const checkout = workflow.jobs["issue-pr-title-policy"]!.steps.find((step) =>
    step.uses?.startsWith("actions/checkout@"),
  )!;
  assert.equal(checkout.with?.["sparse-checkout-cone-mode"], false);
  const paths = checkout.with!["sparse-checkout"]!.trim().split(/\s+/);
  const root = mkdtempSync(join(tmpdir(), "vize-title-policy-sparse-"));
  try {
    for (const path of paths) {
      assert.match(path, /^(?:tools|tests|\.github)\//);
      assert(!path.includes("..") && !path.includes("*"), path);
      mkdirSync(dirname(join(root, path)), { recursive: true });
      cpSync(join(repoRoot, path), join(root, path), { recursive: true });
    }
    const payload = {
      action: "edited",
      pull_request: {
        number: 10418,
        title: "fix(check): preserve source layout",
        assignees: [{ login: "ubugeeei" }],
      },
    };
    const valid = runPolicy(payload, "pull_request_target", { cwd: root, force: true });
    assert.equal(valid.result.status, 0, valid.result.stderr);
    assert.deepEqual(valid.ghCalls, [patchTitleCall(10418, "fix(canon): preserve source layout")]);

    // A full-workspace test used to hide the missing sparse dependency.
    rmSync(join(root, "tools/support/Cargo.toml"));
    const incomplete = runPolicy(payload, "pull_request_target", { cwd: root, force: true });
    assert.notEqual(incomplete.result.status, 0);
    assert.deepEqual(incomplete.ghCalls, []);
    assert.match(incomplete.result.stderr, /no matching package named `vize_tool_support`/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
