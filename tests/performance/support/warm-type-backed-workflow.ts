import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { finiteCut } from "./warm-type-backed-cut.ts";
import { assertUnchangedNonCargoLocks, sourcePair } from "./warm-type-backed-source-pair.ts";
import { qualifyCorsaGraph } from "./warm-type-backed-corsa-graph.ts";
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
  const driver = sourceIdentity(driverRoot);
  assert.ok(
    !fs.existsSync(before) && !fs.existsSync(output),
    "fresh owned directories are required",
  );
  fs.mkdirSync(output);
  fs.writeFileSync(
    path.join(output, "prepare-inputs.json"),
    `${JSON.stringify({ driver, inputs: Object.fromEntries(["SOURCE_SHA", "SOURCE_BASE_SHA", "ORIGINAL_CONTROL_SHA", "ORIGINAL_AFTER_SHA", "ORIGINAL_AFTER_TREE", "ORIGINAL_MANIFEST_SHA256"].map((key) => [key, process.env[key] ?? ""])) }, null, 2)}\n`,
  );
  let cut: ReturnType<typeof finiteCut>;
  let pair: ReturnType<typeof sourcePair>;
  try {
    pair = sourcePair(process.env, driver.revision, (args) => git(driverRoot, args));
    cut = finiteCut(output);
  } catch (error) {
    fs.writeFileSync(
      path.join(output, "prepare-failure.json"),
      `${JSON.stringify({ error: error instanceof Error ? error.stack : String(error) }, null, 2)}\n`,
    );
    throw error;
  }
  assert.equal(driver.dirty, "");
  if (cut) {
    assert.equal(driver.revision, process.env.GITHUB_SHA, "the dispatch driver must be literal");
    assert.equal(
      driver.revision,
      process.env.GITHUB_WORKFLOW_SHA,
      "the loaded workflow must match",
    );
  }
  const head = cut?.after ?? pair?.head ?? process.env.HEAD_SHA!;
  const base = cut?.control ?? pair?.base ?? process.env.BASE_SHA!;
  for (const sha of [head, base]) assert.match(sha, /^[a-f0-9]{40}$/u);
  if (!cut) assert.equal(driver.revision, head);
  const baseline = cut?.control ?? pair?.base ?? git(driverRoot, ["merge-base", base, head]);
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
  const harnessOnly = !cut && production.length === 0;
  if (harnessOnly) {
    for (const [file, digest] of Object.entries(inputAuthority())) {
      const original = spawnSync(
        "git",
        ["show", `${baseline}:tests/_fixtures/differential/lsp/warm-type-backed-requests/${file}`],
        { cwd: driverRoot },
      );
      assert.equal(original.status, 0, original.stderr.toString());
      assert.equal(
        sha256(original.stdout),
        digest,
        "harness qualification preserves every original input",
      );
    }
  } else assert.ok(production.length > 0);
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
  for (const [side, root] of [
    ["before", before],
    ["after", afterRoot],
    ["driver", driverRoot],
  ]) {
    const directory = path.join(output, "source-locks", side);
    fs.mkdirSync(directory, { recursive: true });
    for (const file of [
      "Cargo.toml",
      "Cargo.lock",
      "package.json",
      "pnpm-lock.yaml",
      "rust-toolchain.toml",
    ])
      fs.copyFileSync(path.join(root, file), path.join(directory, file));
  }
  if (pair) {
    assertUnchangedNonCargoLocks(sourceIdentity(before).locks, sourceIdentity(afterRoot).locks);
    const graph = qualifyCorsaGraph(
      fs.readFileSync(path.join(before, "Cargo.toml"), "utf8"),
      fs.readFileSync(path.join(afterRoot, "Cargo.toml"), "utf8"),
      fs.readFileSync(path.join(before, "Cargo.lock"), "utf8"),
      fs.readFileSync(path.join(afterRoot, "Cargo.lock"), "utf8"),
    );
    fs.writeFileSync(
      path.join(output, "source-pair-cargo-graph.json"),
      `${JSON.stringify(graph, null, 2)}\n`,
    );
  }
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
        authority: cut
          ? "root-frozen-release-cut"
          : pair
            ? pair.authority
            : harnessOnly
              ? "harness-only-qualification"
              : "owned-source-pr",
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
          [
            "session.ts",
            "session-process.ts",
            "launch.ts",
            "session-capture.ts",
            "published-launch.ts",
          ].map((name) => [
            name,
            sha256(fs.readFileSync(path.join(driverRoot, "tests/tooling/support/lsp", name))),
          ]),
        ),
        scope: cut
          ? "Literal published v0.433 and root-frozen release cut; complete changed Git-entry manifest, independent driver, identical fresh ci builds, original400 and one recorded runtime"
          : pair
            ? "Immutable ancestor/head source qualification only; six named Corsa registry-to-owned package identities with complete source locks and metadata, unchanged external graph/runtime/toolchain, identical shipping release recipe and original400 protocol. No release-cut, published-artifact or source-effect attribution credit."
            : harnessOnly
              ? "Same production source; closed reviewed harness-only delta and unchanged original inputs/locks. Qualification only, no performance or source-effect gain."
              : "Actual common ancestor and current source, only owned prepared-surface, readiness-ack, shared editor reference-options, document-batch transfer, streamed workspace-symbol producer, event navigation/rename ownership or exact authenticated path-host move delta or exact authenticated timing-observer host move delta or complete authenticated default-migration slice; one worker, identical release recipe, original400 inputs and current locked runtime",
      },
      null,
      2,
    ) + "\n",
  );
  if (!cut && !pair)
    assert.deepEqual(sourceIdentity(before).locks, sourceIdentity(afterRoot).locks);
  if (cut) fs.writeFileSync(path.join(output, "changed-tree-manifest.json"), cut.manifestBytes);
} else {
  throw new Error("usage: warm-type-backed-workflow.ts prepare");
}
