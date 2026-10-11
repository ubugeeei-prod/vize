import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile, root } from "./support/github-workflows.ts";

test("full native workspace tests hydrate the actual pinned Element Plus gitlink first", () => {
  const job = parse(readRepoFile(".github", "workflows", "check.yml")).jobs["clippy-and-test"];
  assert.equal(
    job.if,
    "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}",
  );
  const fixture = "tests/_fixtures/_git/element-plus";
  const hydration = job.steps.findIndex((step: { run?: string }) =>
    step.run?.includes(`git submodule update --init --force --depth 1 -- ${fixture}`),
  );
  const native = job.steps.findIndex(
    (step: { uses?: string }) => step.uses === "./.github/actions/test-rust-workspace-differential",
  );
  assert.ok(hydration >= 0 && hydration < native, "hydrate in the same job before native tests");
  assert.equal(job.steps[hydration].if, undefined);
  assert.notEqual(job.steps[hydration]["continue-on-error"], true);
  const gitlink = execFileSync("git", ["ls-tree", "HEAD", fixture], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(gitlink, `160000 commit 20298a21ae54a5417955373b80130ae7c548a6bb\t${fixture}\n`);
  const oracle = readRepoFile(
    "crates",
    "vize_canon",
    "tests",
    "native_syntax_semantics",
    "element_plus.rs",
  );
  assert.match(oracle, /assert_eq!\(pin, "20298a21ae54a5417955373b80130ae7c548a6bb\\n"\)/);
});
