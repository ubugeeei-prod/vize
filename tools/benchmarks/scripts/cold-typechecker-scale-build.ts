/** Build custody: each immutable source/feature role owns a fresh Cargo target. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";

const hash = (path: string) => createHash("sha256").update(readFileSync(path)).digest("hex");
const flags = [
  "build",
  "--locked",
  "--profile",
  "ci-opt",
  "-p",
  "vize",
  "--message-format=json-render-diagnostics",
  "--config",
  'profile.ci-opt.debug="line-tables-only"',
  "--config",
  'profile.ci-opt.package.vize.strip="none"',
];

function output(executable: string, args: string[], cwd: string) {
  const result = spawnSync(executable, args, {
    cwd,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

export function buildColdScaleBinary(options: {
  sourceRoot: string;
  targetDir: string;
  binary: string;
  receipt: string;
  role: string;
  allocation: boolean;
  /** Test-only executable injection; Actions uses cargo from its pinned toolchain. */
  cargoExecutable?: string;
}) {
  const sourceRoot = resolve(options.sourceRoot);
  const targetDir = resolve(options.targetDir);
  const binary = resolve(options.binary);
  const receiptPath = resolve(options.receipt);
  assert.ok(!existsSync(targetDir), "each role requires a fresh, unshared Cargo target directory");
  assert.ok(!existsSync(binary) && !existsSync(receiptPath), "refuse to replace build evidence");
  const sourceSha = output("git", ["rev-parse", "HEAD"], sourceRoot);
  const treeSha = output("git", ["rev-parse", "HEAD^{tree}"], sourceRoot);
  assert.equal(
    output("git", ["status", "--porcelain", "--untracked-files=no"], sourceRoot),
    "",
    "only a clean immutable source may qualify a build",
  );
  mkdirSync(dirname(receiptPath), { recursive: true });
  const cargo = options.cargoExecutable ?? "cargo";
  const args = [...flags, ...(options.allocation ? ["--features", "profiling"] : [])];
  const startedAt = new Date().toISOString();
  const result = spawnSync(cargo, args, {
    cwd: sourceRoot,
    env: { ...process.env, CARGO_TARGET_DIR: targetDir },
    encoding: "utf8",
    timeout: 30 * 60_000,
    maxBuffer: 128 * 1024 * 1024,
  });
  writeFileSync(receiptPath + ".stdout.jsonl", result.stdout ?? "");
  writeFileSync(receiptPath + ".stderr.txt", result.stderr ?? "");
  process.stderr.write(result.stderr ?? "");
  assert.equal(result.status, 0, `build failed; see ${receiptPath}.stderr.txt and .stdout.jsonl`);
  const artifacts = (result.stdout ?? "")
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line))
    .filter(
      (row) =>
        row.reason === "compiler-artifact" &&
        row.target?.name === "vize" &&
        row.target.kind.includes("bin"),
    );
  assert.equal(artifacts.length, 1, "Cargo must identify the actual vize executable");
  const built = resolve(artifacts[0].executable);
  const features: string[] = artifacts[0].features ?? [];
  assert.equal(
    features.includes("profiling"),
    options.allocation,
    "Cargo feature role must match the measured binary",
  );
  assert.notEqual(
    artifacts[0].fresh,
    true,
    "the root executable must be built in its fresh target",
  );
  assert.equal(
    built,
    join(targetDir, "ci-opt", process.platform === "win32" ? "vize.exe" : "vize"),
  );
  assert.equal(
    output("git", ["rev-parse", "HEAD"], sourceRoot),
    sourceSha,
    "source changed during build",
  );
  assert.equal(output("git", ["status", "--porcelain", "--untracked-files=no"], sourceRoot), "");
  mkdirSync(dirname(binary), { recursive: true });
  copyFileSync(built, binary);
  chmodSync(binary, statSync(built).mode & 0o777);
  const receipt = {
    schemaVersion: 1,
    role: options.role,
    sourceSha,
    treeSha,
    manifests: Object.fromEntries(
      ["Cargo.toml", "Cargo.lock"].map((name) => [
        name,
        existsSync(join(sourceRoot, name)) ? hash(join(sourceRoot, name)) : null,
      ]),
    ),
    sourceRoot,
    targetDir,
    targetStartedEmpty: true,
    binary,
    binarySha256: hash(binary),
    binaryBytes: statSync(binary).size,
    cargoArtifact: {
      executable: built,
      fresh: artifacts[0].fresh ?? null,
      packageId: artifacts[0].package_id ?? null,
      features,
    },
    compilerEnvironment: Object.fromEntries(
      ["RUSTC", "RUSTC_WRAPPER", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"].map((key) => [
        key,
        process.env[key] ?? null,
      ]),
    ),
    allocation: options.allocation,
    cargo: { executable: cargo, args, version: output(cargo, ["--version"], sourceRoot) },
    rustc: output("rustc", ["-Vv"], sourceRoot),
    startedAt,
    finishedAt: new Date().toISOString(),
  };
  writeFileSync(receiptPath, JSON.stringify(receipt, null, 2) + "\n");
  console.log(JSON.stringify(receipt));
  return receipt;
}

export function verifyColdScaleBuildPair(basePath: string, headPath: string, repository: string) {
  const base = JSON.parse(readFileSync(basePath, "utf8"));
  const head = JSON.parse(readFileSync(headPath, "utf8"));
  assert.equal(base.schemaVersion, 1);
  assert.equal(head.schemaVersion, 1);
  assert.notEqual(base.sourceSha, head.sourceSha, "comparison source identities must differ");
  assert.notEqual(
    base.targetDir,
    head.targetDir,
    "comparison builds must not share a Cargo target",
  );
  assert.equal(base.targetStartedEmpty, true);
  assert.equal(head.targetStartedEmpty, true);
  assert.equal(base.allocation, head.allocation);
  assert.deepEqual(base.cargo.args, head.cargo.args, "equal compiler flags are required");
  assert.equal(base.rustc, head.rustc, "equal Rust toolchains are required");
  assert.equal(base.cargo.version, head.cargo.version);
  assert.deepEqual(
    base.compilerEnvironment,
    head.compilerEnvironment,
    "equal compiler environment is required",
  );
  for (const receipt of [base, head]) {
    assert.equal(hash(receipt.binary), receipt.binarySha256, "captured executable was replaced");
    assert.equal(statSync(receipt.binary).size, receipt.binaryBytes);
  }
  const productionDiff = output(
    "git",
    [
      "diff",
      "--name-only",
      base.sourceSha,
      head.sourceSha,
      "--",
      "crates",
      "davinci",
      "vendor",
      "Cargo.toml",
      "Cargo.lock",
      ".cargo",
    ],
    repository,
  );
  if (productionDiff)
    assert.notEqual(
      base.binarySha256,
      head.binarySha256,
      "production sources differ but the compared executable bytes are identical",
    );
  return {
    qualified: true,
    productionDiff: productionDiff.split("\n").filter(Boolean),
    base: { sourceSha: base.sourceSha, binarySha256: base.binarySha256 },
    head: { sourceSha: head.sourceSha, binarySha256: head.binarySha256 },
  };
}

/** A queued PR event's base SHA may be older than the actual test merge's base. */
export function selectColdScaleBaseline(options: {
  repository: string;
  event: string;
  requestedBase?: string;
  eventBase?: string;
  prHead?: string;
}) {
  const sourceSha = output("git", ["rev-parse", "HEAD"], options.repository);
  let baselineSha = options.requestedBase ?? "";
  if (options.event === "pull_request") {
    const commit = output("git", ["cat-file", "commit", "HEAD"], options.repository);
    const parents = [...commit.matchAll(/^parent ([a-f0-9]{40})$/gm)].map((match) => match[1]);
    assert.equal(parents.length, 2, "PR source must be the actual two-parent test merge");
    assert.equal(parents[1], options.prHead, "test merge second parent must match the PR head");
    baselineSha = parents[0];
  } else assert.equal(options.event, "workflow_dispatch");
  assert.match(baselineSha, /^[a-f0-9]{40}$/u, "baseline must be an immutable commit identity");
  assert.notEqual(baselineSha, sourceSha, "baseline and source must differ");
  return {
    sourceSha,
    baselineSha,
    strategy:
      options.event === "pull_request" ? "actual-test-merge-first-parent" : "explicit-dispatch",
    eventBaseSha: options.eventBase || null,
    prHeadSha: options.prHead || null,
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      "source-root": { type: "string" },
      "target-dir": { type: "string" },
      binary: { type: "string" },
      receipt: { type: "string" },
      role: { type: "string" },
      allocation: { type: "boolean", default: false },
      "base-receipt": { type: "string" },
      "head-receipt": { type: "string" },
      repository: { type: "string" },
      "select-baseline": { type: "boolean", default: false },
    },
  });
  if (values["select-baseline"]) {
    const selection = selectColdScaleBaseline({
      repository: values.repository ?? process.cwd(),
      event: process.env.BENCH_EVENT ?? "workflow_dispatch",
      requestedBase: process.env.BASE_SHA,
      eventBase: process.env.PR_EVENT_BASE_SHA,
      prHead: process.env.PR_HEAD_SHA,
    });
    if (values.receipt) {
      mkdirSync(dirname(values.receipt), { recursive: true });
      writeFileSync(values.receipt, JSON.stringify(selection, null, 2) + "\n");
    }
    console.log("BASE_SHA=" + selection.baselineSha);
  } else if (values["base-receipt"] && values["head-receipt"]) {
    const pair = verifyColdScaleBuildPair(
      values["base-receipt"],
      values["head-receipt"],
      values.repository ?? process.cwd(),
    );
    if (values.receipt) writeFileSync(values.receipt, JSON.stringify(pair, null, 2) + "\n");
    console.log(JSON.stringify(pair));
  } else {
    assert.ok(
      values["source-root"] &&
        values["target-dir"] &&
        values.binary &&
        values.receipt &&
        values.role,
    );
    buildColdScaleBinary({
      sourceRoot: values["source-root"],
      targetDir: values["target-dir"],
      binary: values.binary,
      receipt: values.receipt,
      role: values.role,
      allocation: values.allocation!,
    });
  }
}
