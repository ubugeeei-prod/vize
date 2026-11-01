import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { SHIPPING_RECIPE } from "../lsp-initial-ready-pair/shipping-adapter.ts";
import {
  BOUND_PATHS,
  assertSources,
  capacity,
} from "../lsp-initial-ready-pair/readiness-parity.ts";

import {
  SOURCE_PATHS,
  type Obj,
  hash,
  record,
  json,
  save,
  fileHash,
  copyOwned,
  git,
  pathsBetween,
  immutableParent,
  clean,
  snapshot,
  run,
  succeeded,
  regularIdentity,
  cargoArtifact,
  resolveProgram,
  selectedTool,
  checkAmbientOverrides,
  checkOutcome,
  verifyCaptures,
} from "./source-custody.ts";
export { SOURCE_PATHS, checkAmbientOverrides, checkOutcome } from "./source-custody.ts";

export const BASE_SOURCE = "36d998e36e379cc8c12842bfbc42f56e9ae0473a";
export const CANDIDATE_SOURCE = "57149458f8da9d6636df1ad431ca44616e287e99";
export const PREVIOUS_CANDIDATE = "6c09437ff6a11f2905163fc2794903ab0552c210";
export const INTERMEDIATE_SOURCE = "3ac69d44ea8c12f365a36b6ffa67185a5a8646d6";
export const PREVIOUS_WORKFLOW = "0499c7bcf70222a9f3b21c41ffbb72bf2af20fe7";
export const ORDER = ["B", "C", "C", "B"] as const;
export type Role = (typeof ORDER)[number];
export type Mode = "memory";
export const WORKLOAD_FILES = [
  "controller.ts",
  "source-custody.ts",
  "custody-controls.test.ts",
  "workload.ts",
  "native-queries.ts",
  "fixtures.ts",
  "resources.ts",
  "controls.test.ts",
  "README.md",
] as const;
export const TOOLING_PATHS = [
  ".github/workflows/lsp-current-baseline.yml",
  ...["shipping-adapter.ts", "readiness-parity.ts"].map(
    (name) => `.github/actions/lsp-initial-ready-pair/${name}`,
  ),
  ...WORKLOAD_FILES.map((name) => `.github/actions/lsp-initial-ready-supplement/${name}`),
];
const SHA = /^[a-f0-9]{40}$/;
const equalPaths = (actual: readonly string[], expected: readonly string[]): void =>
  assert.deepEqual([...actual].sort(), [...expected].sort());
export function sourceFor(role: Role): string {
  return role === "B" ? BASE_SOURCE : CANDIDATE_SOURCE;
}
export function checkAncestry(
  parents: { candidate: string; previousCandidate: string; intermediate: string; workflow: string },
  sourcePaths: string[],
  toolingPaths: string[],
): void {
  assert.deepEqual(parents, {
    candidate: PREVIOUS_CANDIDATE,
    previousCandidate: INTERMEDIATE_SOURCE,
    intermediate: BASE_SOURCE,
    workflow: CANDIDATE_SOURCE,
  });
  equalPaths(sourcePaths, SOURCE_PATHS);
  equalPaths(toolingPaths, TOOLING_PATHS);
}
export function workloadPlan(): { mode: Mode; role: Role; attempt: number }[] {
  return ORDER.map((role, index) => ({ mode: "memory" as const, role, attempt: index + 1 }));
}
export function modeEnvironment(environment: NodeJS.ProcessEnv, mode: Mode): NodeJS.ProcessEnv {
  assert.equal(environment.VIZE_TRACE_EDITOR_PREPARATION, undefined);
  assert.equal(mode, "memory");
  return { ...environment, VIZE_TRACE_EDITOR_PREPARATION: "1" };
}

async function main(): Promise<void> {
  const repo = fs.realpathSync(process.cwd());
  const workflow = process.env.GITHUB_SHA ?? "";
  assert.equal(workflow, process.env.EXPECTED_WORKFLOW);
  assert.match(workflow, SHA);
  assert.equal(process.env.REQUESTED_CANDIDATE, CANDIDATE_SOURCE);
  assert.equal(process.env.RUNNER_OS, "Linux");
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  assert.equal(process.env.NODE_OPTIONS, "--disable-warning=DEP0040 --disable-warning=DEP0169");
  assert.equal(process.env.RUSTFLAGS ?? "", "");
  assert.equal(process.env.CARGO_ENCODED_RUSTFLAGS ?? "", "");
  assert.equal(process.env.CARGO_INCREMENTAL, "0");
  assert.equal(process.env.CARGO_BUILD_JOBS, "16");
  assert.equal(process.env.RAYON_NUM_THREADS, "8");
  clean(repo, workflow);
  checkAmbientOverrides(process.env);
  assert.equal(process.env.VIZE_TEST_DISABLE_TSGO, undefined);
  for (const name of [
    "RUSTC",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "CARGO_BUILD_RUSTC",
    "CARGO_BUILD_RUSTC_WRAPPER",
    "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
  ])
    assert.equal(process.env[name], undefined, `alternate compiler selection ${name} refused`);
  const parents = {
    candidate: immutableParent(repo, CANDIDATE_SOURCE),
    previousCandidate: immutableParent(repo, PREVIOUS_CANDIDATE),
    intermediate: immutableParent(repo, INTERMEDIATE_SOURCE),
    workflow: immutableParent(repo, workflow),
  };
  checkAncestry(
    parents,
    pathsBetween(repo, BASE_SOURCE, CANDIDATE_SOURCE),
    pathsBetween(repo, CANDIDATE_SOURCE, workflow),
  );
  assertSources(repo);
  for (const file of BOUND_PATHS.filter(
    (file) => !(SOURCE_PATHS as readonly string[]).includes(file),
  ))
    assert.equal(
      git(repo, "rev-parse", `${BASE_SOURCE}:${file}`),
      git(repo, "rev-parse", `${CANDIDATE_SOURCE}:${file}`),
    );
  const temp = fs.realpathSync(process.env.RUNNER_TEMP ?? "");
  const suffix = `${process.env.GITHUB_RUN_ID}-${process.env.GITHUB_RUN_ATTEMPT}`;
  const evidence = path.join(temp, `closed-context-memory-${suffix}`);
  fs.mkdirSync(evidence, { mode: 0o700 });
  const images = path.join(temp, `closed-context-memory-images-${suffix}`);
  fs.mkdirSync(images, { mode: 0o700 });
  const helperDirectory = path.dirname(fileURLToPath(import.meta.url));
  const helperHashes: Obj[] = [];
  for (const name of WORKLOAD_FILES) {
    const expected = spawnSync(
      "git",
      ["show", `${workflow}:.github/actions/lsp-initial-ready-supplement/${name}`],
      { cwd: repo, maxBuffer: 8 * 1024 * 1024 },
    );
    assert.equal(expected.status, 0);
    assert.equal(fileHash(path.join(helperDirectory, name)), hash(expected.stdout));
    helperHashes.push({ name, sha256: hash(expected.stdout) });
  }
  for (const name of ["shipping-adapter.ts", "readiness-parity.ts"]) {
    const expected = spawnSync(
      "git",
      ["show", `${workflow}:.github/actions/lsp-initial-ready-pair/${name}`],
      { cwd: repo, maxBuffer: 8 * 1024 * 1024 },
    );
    assert.equal(expected.status, 0);
    assert.equal(
      fileHash(path.join(helperDirectory, "../lsp-initial-ready-pair", name)),
      hash(expected.stdout),
    );
    if (name === "shipping-adapter.ts")
      assert.equal(
        hash(expected.stdout),
        "c8344518162a8b2f57c670b2820371ee56a47f0256da01414e1826c46c8bf240",
      );
    helperHashes.push({ name, sha256: hash(expected.stdout) });
  }
  const workflowBlob = spawnSync(
    "git",
    ["show", `${workflow}:.github/workflows/lsp-current-baseline.yml`],
    { cwd: repo, maxBuffer: 8 * 1024 * 1024 },
  );
  assert.equal(workflowBlob.status, 0);
  assert.equal(fileHash(path.join(helperDirectory, "../workflow.yml")), hash(workflowBlob.stdout));
  helperHashes.push({ name: "workflow.yml", sha256: hash(workflowBlob.stdout) });
  save(path.join(evidence, "helper-hashes.json"), helperHashes);
  const stageDir = path.join(repo, "target/ci");
  for (const part of [path.join(repo, "target"), stageDir]) {
    if (fs.existsSync(part))
      assert.ok(fs.lstatSync(part).isDirectory() && !fs.lstatSync(part).isSymbolicLink());
    else fs.mkdirSync(part);
  }
  const stage = path.join(stageDir, "vize");
  const receipt = `${stage}.differential-build.json`;
  assert.ok(!fs.existsSync(stage) && !fs.existsSync(receipt));
  const native = path.join(repo, "node_modules/@typescript/typescript-linux-x64/lib/tsc");
  const nativeIdentity = regularIdentity(native);
  assert.deepEqual(fs.readFileSync(native).subarray(0, 4), Buffer.from([127, 69, 76, 70]));
  const nativeManifest = record(
    json(path.join(path.dirname(fs.realpathSync(native)), "../package.json")),
  );
  assert.equal(nativeManifest.name, "@typescript/typescript-linux-x64");
  assert.equal(nativeManifest.version, "7.0.2");
  const env = {
    ...process.env,
    CORSA_PATH: native,
    VIZE_LSP_BIN: stage,
    VIZE_LSP_REQUIRE_SOURCE_BUILD: "1",
    VIZE_LSP_BUILD_RECIPE: SHIPPING_RECIPE,
  };
  for (const [label, executable, args] of [
    ["node-version", process.execPath, ["--version"]],
    ["vp-version", "vp", ["--version"]],
    ["rustc-version", "rustc", ["-vV"]],
    ["cargo-version", "cargo", ["--version"]],
  ] as const)
    succeeded(run(repo, evidence, label, executable, [...args], process.env));
  const toolPaths = {
    node: process.execPath,
    vpLauncher: resolveProgram(repo, "vp"),
    rustc: selectedTool(repo, evidence, "rustc"),
    cargo: selectedTool(repo, evidence, "cargo"),
  };
  const toolIdentity = (): Obj =>
    Object.fromEntries(
      Object.entries(toolPaths).map(([name, file]) => [name, regularIdentity(file)]),
    );
  const resources = capacity(repo, "authority", 120, path.join(evidence, "capacity.json"));
  const authority = {
    workflow,
    parents,
    base: BASE_SOURCE,
    candidate: CANDIDATE_SOURCE,
    sourcePaths: SOURCE_PATHS,
    toolingPaths: TOOLING_PATHS,
    recipe: SHIPPING_RECIPE,
    plan: workloadPlan(),
    resources,
    tools: toolIdentity(),
    native: nativeIdentity,
    controlledEnvironment: {
      CARGO_INCREMENTAL: "0",
      CARGO_BUILD_JOBS: "16",
      RAYON_NUM_THREADS: "8",
      NODE_OPTIONS: process.env.NODE_OPTIONS,
      rustflagsSha256: hash(process.env.RUSTFLAGS ?? ""),
      encodedRustflagsSha256: hash(process.env.CARGO_ENCODED_RUSTFLAGS ?? ""),
    },
    modePolicy: {
      memory: "trace ON; four matched B/C/C/B cohorts; fixed untimed PSS; no latency headline",
      initialization: "same releaseClosedProjectContexts:true JSON for both roles",
      reusedAuthority: PREVIOUS_WORKFLOW,
      old0499: "FAILED clean-input cohort; not reclassified or used as this comparison",
    },
    cacheScope: {
      application: "fresh process per closed workload case; no setup warm native query",
      osPageCache: "UNKNOWN; no purge",
      profiling: false,
      pssDuringTiming: false,
    },
    platform: { arch: os.arch(), release: os.release(), cpus: os.cpus().map(({ model }) => model) },
  };
  save(path.join(evidence, "authority.json"), authority);
  const campaign: Obj = { ...authority, attempts: [], complete: false, allPassed: false };
  const originals = new Map<
    Role,
    { image: string; digest: string; receipt: string; source: string }
  >();
  let primaryFailure: string | null = null;
  try {
    succeeded(run(repo, evidence, "native-version", native, ["--version"], env));
    assert.equal(
      fs.readFileSync(path.join(evidence, "native-version.stdout"), "utf8"),
      "Version 7.0.2\n",
    );
    assert.equal(fs.statSync(path.join(evidence, "native-version.stderr")).size, 0);
    for (const role of ["B", "C"] as const) {
      const source = sourceFor(role);
      git(repo, "checkout", "--detach", source);
      clean(repo, source);
      const dir = path.join(evidence, `build-${role}`);
      fs.mkdirSync(dir);
      const target = path.join(temp, `initial-ready-supplement-build-${suffix}-${role}`);
      assert.ok(!fs.existsSync(target));
      const cargoEnv = { ...env, CARGO_TARGET_DIR: target, CARGO_INCREMENTAL: "0" };
      capacity(repo, `build-${role}`, 60, path.join(dir, "capacity.json"));
      const sourceBefore = snapshot(repo);
      save(path.join(dir, "before-build.json"), {
        ...sourceBefore,
        target,
        cliAbsent: !fs.existsSync(path.join(target, "release/vize")),
        recipe: SHIPPING_RECIPE,
      });
      succeeded(
        run(
          repo,
          dir,
          "cargo-build",
          "cargo",
          ["build", "--release", "-p", "vize", "--message-format=json-render-diagnostics"],
          cargoEnv,
        ),
      );
      const actual = path.join(target, "release/vize");
      assert.deepEqual(fs.readFileSync(actual).subarray(0, 4), Buffer.from([127, 69, 76, 70]));
      const artifact = cargoArtifact(path.join(dir, "cargo-build.stdout"), actual);
      const image = path.join(images, `${role}-first-vize`);
      fs.copyFileSync(actual, image, fs.constants.COPYFILE_EXCL);
      fs.chmodSync(image, 0o500);
      const digest = fileHash(image);
      copyOwned(image, stage);
      succeeded(
        run(
          repo,
          dir,
          "build-receipt",
          "vp",
          [
            "exec",
            "node",
            "--input-type=module",
            "-e",
            "const m = await import('./tests/differential/build-receipt.ts'); console.log(m.writeBuildReceipt(process.cwd(), m.SHIPPING_BUILD_RECIPE));",
          ],
          env,
        ),
      );
      const receiptText = fs.readFileSync(receipt, "utf8");
      const build = record(JSON.parse(receiptText));
      assert.equal(build.recipe, SHIPPING_RECIPE);
      assert.equal(build.sourceRevision, source);
      assert.equal(build.binarySha256, digest);
      succeeded(run(repo, dir, "first-version", stage, ["--version"], env));
      assert.equal(
        fs.readFileSync(path.join(dir, "first-version.stdout"), "utf8"),
        `${build.cliVersion}\n`,
      );
      assert.equal(fs.statSync(path.join(dir, "first-version.stderr")).size, 0);
      save(path.join(dir, "first-image.json"), {
        source,
        imageSha256: digest,
        cargoArtifact: artifact,
        actualArtifact: regularIdentity(actual),
        stagedImage: regularIdentity(stage),
        buildReceipt: build,
        recipe: SHIPPING_RECIPE,
        actualArguments: [
          "build",
          "--release",
          "-p",
          "vize",
          "--message-format=json-render-diagnostics",
        ],
        metadataOnlyArgument: "--message-format=json-render-diagnostics",
        libraryTests: "not repeated; this supplemental workload uses the FIRST release executable",
      });
      assert.deepEqual(snapshot(repo), sourceBefore);
      originals.set(role, { image, digest, receipt: receiptText, source });
    }
    const attempts = campaign.attempts as Obj[];
    for (const [index, step] of workloadPlan().entries()) {
      const original = originals.get(step.role)!;
      git(repo, "checkout", "--detach", original.source);
      clean(repo, original.source);
      const dir = path.join(evidence, `${index + 1}-${step.mode}-${step.role}`);
      fs.mkdirSync(dir);
      copyOwned(original.image, stage);
      fs.writeFileSync(receipt, original.receipt);
      capacity(repo, `workload-${index + 1}`, 10, path.join(dir, "capacity.json"));
      const before = {
        ...snapshot(repo),
        imageSha256: fileHash(stage),
        native: regularIdentity(native),
        tools: toolIdentity(),
        receiptSha256: fileHash(receipt),
      };
      assert.equal(before.imageSha256, original.digest);
      assert.deepEqual(before.native, nativeIdentity);
      assert.deepEqual(before.tools, authority.tools);
      save(path.join(dir, "before.json"), before);
      const output = path.join(dir, "workload");
      fs.mkdirSync(output, { mode: 0o700 });
      const args = [
        "exec",
        "node",
        path.join(helperDirectory, "workload.ts"),
        "--mode",
        step.mode,
        "--repo",
        repo,
        "--output",
        output,
        "--source",
        original.source,
        "--attempt",
        String(step.attempt),
      ];
      const modeEnv = modeEnvironment(env, step.mode);
      const exit = run(repo, dir, "workload", "vp", args, modeEnv);
      const errors: string[] = [];
      try {
        succeeded(exit);
        const outcome = json(path.join(output, "outcome.json"));
        checkOutcome(outcome, step.mode, original.source, step.attempt);
        verifyCaptures(output, outcome, original.digest);
      } catch (error) {
        errors.push(error instanceof Error ? error.message : String(error));
      }
      const after = {
        ...snapshot(repo),
        imageSha256: fileHash(stage),
        native: regularIdentity(native),
        tools: toolIdentity(),
        receiptSha256: fileHash(receipt),
      };
      save(path.join(dir, "after.json"), after);
      assert.deepEqual(
        after,
        before,
        "source/provider/first release identity changed during workload",
      );
      attempts.push({
        ...step,
        index,
        source: original.source,
        imageSha256: original.digest,
        exit,
        errors,
        passed: errors.length === 0,
        outcomeSha256: fs.existsSync(path.join(output, "outcome.json"))
          ? fileHash(path.join(output, "outcome.json"))
          : null,
      });
    }
    campaign.complete = attempts.length === workloadPlan().length;
    campaign.allPassed = campaign.complete && attempts.every((row) => row.passed === true);
    assert.equal(
      campaign.allPassed,
      true,
      "all supplemental attempts must pass; original errors are retained",
    );
  } catch (error) {
    primaryFailure = error instanceof Error ? (error.stack ?? error.message) : String(error);
    throw error;
  } finally {
    campaign.failure = primaryFailure;
    save(path.join(evidence, "campaign.json"), campaign);
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  main().catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
