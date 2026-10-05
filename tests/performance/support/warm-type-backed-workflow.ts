import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { finiteCut } from "./warm-type-backed-cut.ts";

import {
  driverRoot,
  git,
  inputAuthority,
  sha256,
  sourceIdentity,
} from "./warm-type-backed-source.ts";

const output = path.join(process.env.RUNNER_TEMP!, "warm-pair");
const before = path.join(process.env.RUNNER_TEMP!, "warm-before");
assert.equal(process.platform, "linux");
assert.ok(process.env.RUNNER_TEMP && process.env.GITHUB_WORKSPACE);
assert.equal(fs.realpathSync(process.env.GITHUB_WORKSPACE), driverRoot);

if (process.argv[2] === "prepare") {
  const cut = finiteCut();
  const driver = sourceIdentity(driverRoot);
  assert.equal(driver.dirty, "");
  if (cut) {
    assert.equal(driver.revision, process.env.GITHUB_SHA, "the dispatch driver must be literal");
    assert.equal(
      driver.revision,
      process.env.GITHUB_WORKFLOW_SHA,
      "the loaded workflow must match",
    );
  }
  const head = cut?.after ?? process.env.HEAD_SHA!;
  const base = cut?.control ?? process.env.BASE_SHA!;
  for (const sha of [head, base]) assert.match(sha, /^[a-f0-9]{40}$/u);
  if (!cut) assert.equal(driver.revision, head);
  const baseline = cut?.control ?? git(driverRoot, ["merge-base", base, head]);
  const afterRoot = cut ? path.join(process.env.RUNNER_TEMP!, "warm-after") : driverRoot;
  const production = git(driverRoot, [
    "diff",
    "--name-only",
    baseline,
    head,
    "--",
    "crates",
    "davinci",
    "vendor",
    "npm",
    "Cargo.toml",
    "Cargo.lock",
    ".cargo",
    ".config",
    "rust-toolchain.toml",
  ])
    .split("\n")
    .filter(Boolean);
  const allowed = new Set([
    "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context.rs",
    "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/build.rs",
    "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/cache.rs",
    "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/cache/catalog.rs",
    "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/cache/fingerprint.rs",
    "crates/vize_canon/src/corsa_bridge/vue_document.rs",
    "crates/vize_canon/src/corsa_bridge/vue_document/build.rs",
    "crates/vize_canon/src/corsa_bridge/vue_document/build/tests.rs",
    "crates/vize_canon/src/corsa_bridge/vue_document/types.rs",
  ]);
  assert.ok(production.length > 0);
  if (!cut) {
    for (const file of production)
      assert.ok(allowed.has(file), `unqualified production delta: ${file}`);
  }
  assert.ok(
    !fs.existsSync(before) && !fs.existsSync(output),
    "fresh owned directories are required",
  );
  const result = spawnSync("git", ["worktree", "add", "--detach", before, baseline], {
    cwd: driverRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  if (cut) {
    assert.ok(!fs.existsSync(afterRoot), "the literal cut needs its own fresh checkout");
    const after = spawnSync("git", ["worktree", "add", "--detach", afterRoot, head], {
      cwd: driverRoot,
      encoding: "utf8",
    });
    assert.equal(after.status, 0, after.stderr);
  }
  fs.mkdirSync(output);
  fs.writeFileSync(
    path.join(output, "workflow-source.json"),
    JSON.stringify(
      {
        head,
        base,
        baseline,
        production,
        afterRoot,
        driverSource: driver,
        authority: cut ? "root-frozen-release-cut" : "owned-source-pr",
        finiteCut: cut,
        originals: inputAuthority(),
        runner: {
          run: process.env.GITHUB_RUN_ID,
          attempt: process.env.GITHUB_RUN_ATTEMPT,
          workflow: process.env.GITHUB_WORKFLOW_SHA,
          node: process.version,
          platform: process.platform,
          arch: process.arch,
        },
        driver: Object.fromEntries(
          fs
            .readdirSync(path.dirname(new URL(import.meta.url).pathname))
            .filter((name) => name.startsWith("warm-type-backed-"))
            .toSorted()
            .map((name) => [name, sha256(fs.readFileSync(new URL(name, import.meta.url)))]),
        ),
        protocol: Object.fromEntries(
          ["session.ts", "session-process.ts", "launch.ts", "session-capture.ts"].map((name) => [
            name,
            sha256(fs.readFileSync(path.join(driverRoot, "tests/tooling/support/lsp", name))),
          ]),
        ),
        scope: cut
          ? "Literal published v0.433 and root-frozen release cut; complete changed Git-entry manifest, independent driver, identical fresh ci builds, original400 and one recorded runtime"
          : "Actual common ancestor and current source, only owned prepared-surface production delta; one worker, identical ci recipe, original400 inputs and current locked runtime",
      },
      null,
      2,
    ) + "\n",
  );
  if (!cut) assert.deepEqual(sourceIdentity(before).locks, sourceIdentity(afterRoot).locks);
  else fs.writeFileSync(path.join(output, "changed-tree-manifest.json"), cut.manifestBytes);
} else {
  throw new Error("usage: warm-type-backed-workflow.ts prepare");
}
