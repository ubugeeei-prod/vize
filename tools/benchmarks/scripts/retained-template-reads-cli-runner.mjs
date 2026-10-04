/** Exact binary/dependency provenance and durable raw process evidence. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  appendFileSync,
  existsSync,
  readdirSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import { dirname, join, relative, resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { fileSha256, hashInPlace, pinExecutable } from "./benchmark-binary.mjs";
import { packageVersion, resolveVuePackageDir } from "./check-gate-env.mjs";
import { diagnosticFingerprint } from "./typecheck-command.mjs";
import {
  BUILD,
  PROTOCOL,
  SIDES,
  checkEnvironment,
  normalizeCliReport,
  profileReads,
  writeJson,
} from "./retained-template-reads-cli-protocol.mjs";

export function commandOutput(binary, args, cwd = process.cwd()) {
  const result = spawnSync(binary, args, { cwd, encoding: "utf8", timeout: 30_000 });
  assert.equal(result.error, undefined, `${binary}: ${result.error?.message}`);
  assert.equal(result.status, 0, `${binary}: ${result.stderr}`);
  return (result.stdout || result.stderr).trim();
}

export function prepareRun(baseInput, headInput, directory, root) {
  const workRoot = join(directory, "work");
  const binarySources = {
    base: realpathSync(resolve(baseInput)),
    head: realpathSync(resolve(headInput)),
  };
  const checkout = { base: resolve(dirname(binarySources.base), "../.."), head: root };
  const metadata = {
    schemaVersion: 1,
    kind: "vize-check-retained-template-reads-paired",
    generatedAt: new Date().toISOString(),
    baseSha: process.env.BASE_SHA,
    headSha: process.env.HEAD_SHA,
    build: BUILD,
    runner: {
      label: process.env.RUNNER_LABEL ?? "local",
      cpuCount: os.cpus().length,
      availableParallelism: os.availableParallelism(),
      cpuModel: os.cpus()[0]?.model ?? "unknown",
      platform: process.platform,
      arch: process.arch,
      node: process.version,
    },
    provenance: {
      runId: process.env.GITHUB_RUN_ID ?? null,
      runAttempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
      repository: process.env.GITHUB_REPOSITORY ?? null,
      rustflags: process.env.RUSTFLAGS ?? "",
      scripts: Object.fromEntries(
        [
          "retained-template-reads-cli-paired.mjs",
          "retained-template-reads-cli-protocol.mjs",
          "retained-template-reads-cli-corpus.mjs",
          "retained-template-reads-cli-runner.mjs",
          "retained-template-reads-cli-leaf-corpus.mjs",
          "generate.mjs",
          "check-gate-env.mjs",
          "check-gate-plants.mjs",
          "typecheck-command.mjs",
          "benchmark-binary.mjs",
        ].map((file) => [file, fileSha256(join(root, "tools/benchmarks/scripts", file))]),
      ),
    },
    protocol: PROTOCOL,
  };
  writeJson(join(directory, "provenance.json"), metadata);
  for (const side of SIDES) {
    assert.equal(
      commandOutput("git", ["rev-parse", "HEAD"], checkout[side]),
      metadata[`${side}Sha`],
    );
    assert.equal(
      commandOutput("git", ["diff", "--name-only", "HEAD"], checkout[side]),
      "",
      `${side} checkout has tracked edits`,
    );
  }
  for (const [helper, sha256] of Object.entries(metadata.provenance.scripts))
    assert.equal(
      fileSha256(join(checkout.base, "tools/benchmarks/scripts", helper)),
      sha256,
      `${helper}: base and head benchmark helper sources differ`,
    );
  const projectionSource = "crates/vize_canon/examples/retained_template_reads_projection.rs";
  metadata.provenance.projectionSourceSha256 = fileSha256(join(root, projectionSource));
  assert.match(metadata.provenance.projectionSourceSha256 ?? "", /^[0-9a-f]{64}$/u);
  assert.equal(
    fileSha256(join(checkout.base, projectionSource)),
    metadata.provenance.projectionSourceSha256,
    "base and head projection probe sources differ",
  );
  const runtimePath = realpathSync(
    join(
      root,
      "node_modules/@typescript",
      `typescript-${process.platform}-${process.arch}`,
      "lib",
      process.platform === "win32" ? "tsc.exe" : "tsc",
    ),
  );
  const vuePackageDir = resolveVuePackageDir();
  assert(vuePackageDir, "head dependencies must contain Vue");
  const binaries = {
    base: pinExecutable(binarySources.base, workRoot),
    head: pinExecutable(binarySources.head, workRoot),
    typescript: hashInPlace(runtimePath),
    baseProjection: pinExecutable(realpathSync(process.env.BASE_PROJECTION), workRoot),
    headProjection: pinExecutable(realpathSync(process.env.HEAD_PROJECTION), workRoot),
  };
  assert.match(binaries.typescript.sha256 ?? "", /^[0-9a-f]{64}$/u);
  metadata.binaries = binaries;
  metadata.dependencies = {
    vueVersion: packageVersion(vuePackageDir),
    vuePackageDir: realpathSync(vuePackageDir),
    runtimePackageVersion: packageVersion(resolve(dirname(runtimePath), "..")),
    headLockSha256: fileSha256(join(root, "pnpm-lock.yaml")),
    cargoLockSha256: Object.fromEntries(
      SIDES.map((side) => [side, fileSha256(join(checkout[side], "Cargo.lock"))]),
    ),
  };
  writeJson(join(directory, "provenance.json"), metadata);
  return { metadata, binaries, binarySources, runtimePath, vuePackageDir };
}

export function compareProjections(directory, binaries, corpus) {
  const captured = {};
  const files = {};
  for (const side of SIDES) {
    const result = spawnSync(binaries[`${side}Projection`].measuredPath, [corpus.dir], {
      encoding: "utf8",
      timeout: 300_000,
      maxBuffer: 256 * 1024 * 1024,
      env: checkEnvironment({ rayon: null }),
    });
    const stem = join(directory, "projections", `${corpus.id}-${side}`);
    writeFileSync(`${stem}.json`, result.stdout ?? "");
    writeFileSync(`${stem}.stderr.txt`, result.stderr ?? "");
    assert.equal(result.error, undefined, `${side} projection: ${result.error?.message}`);
    assert.equal(result.status, 0, `${side} projection: ${result.stderr}`);
    assert.equal(result.stderr, "", "unexpected projection stderr");
    const projection = JSON.parse(result.stdout);
    assert(Array.isArray(projection.files) && Array.isArray(projection.edgeCases));
    assert(projection.edgeCases.length > 0, "projection semantic edge cases are missing");
    const expected = corpus.manifest.files
      .filter((file) => file.file.endsWith(".vue"))
      .map((file) => file.file);
    assert.deepEqual(
      projection.files.map((file) => file.file),
      expected,
      "projection omitted Vue inputs",
    );
    captured[side] = result.stdout;
    files[side] = { file: relative(directory, `${stem}.json`), sha256: fileSha256(`${stem}.json`) };
  }
  assert(
    captured.head === captured.base,
    `${corpus.id}: generated code, mappings or semantic links differ`,
  );
  return { ...files, equal: true };
}

export function createRunner(directory, binaries, runtimePath) {
  let sequence = 0;
  return function run(side, mode, corpus, cwd, phase, profile = false) {
    const id = `${String(sequence++).padStart(3, "0")}-${corpus.id}-${mode.id}-${phase}-${side}`;
    const args = [
      "check",
      ...corpus.args,
      "--quiet",
      "--format",
      "json",
      "--tsconfig",
      "tsconfig.json",
      "--corsa-path",
      runtimePath,
      ...mode.args,
    ];
    const profileFile = join(directory, "profiles", `${id}.json`);
    if (profile) args.push("--profile-json", profileFile);
    const env = checkEnvironment(mode);
    const start = performance.now();
    const result = spawnSync(binaries[side].measuredPath, args, {
      cwd,
      encoding: "utf8",
      timeout: 300_000,
      maxBuffer: 64 * 1024 * 1024,
      env,
    });
    const record = {
      id,
      side,
      corpus: corpus.id,
      mode: mode.id,
      phase,
      profiling: profile,
      ms: profile ? undefined : performance.now() - start,
      status: result.status,
      signal: result.signal,
      command: [binaries[side].measuredPath, ...args],
      cwd,
      environment: {
        RAYON_NUM_THREADS: env.RAYON_NUM_THREADS ?? null,
        GOMAXPROCS: env.GOMAXPROCS ?? null,
        VIZE_BENCH: "1",
      },
    };
    // Write process evidence before validating, so a failed cold/gate run remains inspectable.
    writeFileSync(join(directory, "raw", `${id}.stdout.json`), result.stdout ?? "");
    writeFileSync(join(directory, "raw", `${id}.stderr.txt`), result.stderr ?? "");
    writeJson(join(directory, "raw", `${id}.json`), record);
    assert.equal(result.error, undefined, `${id}: ${result.error?.message}`);
    const expected = readdirSync(cwd).filter((file) => file.endsWith(".vue"));
    const normalized = normalizeCliReport(result, cwd, expected);
    if (corpus.id === "shared-leaf501-default" && cwd === corpus.dir) {
      assert.equal(normalized.errorCount, 1, "shared-leaf assignment plant count changed");
      assert.equal(normalized.warningCount, 0, "shared-leaf warnings changed");
      assert.equal(normalized.fileCount, 502, "shared-leaf check omitted source inputs");
      const planted = normalized.files.find((file) => file.file === "Planted.vue");
      assert.equal(planted?.diagnostics.length, 1, "shared-leaf assignment plant is missing");
      assert(
        planted.diagnostics[0].includes("[TS2322]"),
        "shared-leaf assignment plant was missed",
      );
      assert(
        normalized.files.some((file) => file.file === "shared.ts"),
        "shared leaf was not reported",
      );
      assert(
        normalized.programs.some((program) => program.files.includes("shared.ts")),
        "shared leaf is absent from effective programs",
      );
    }
    record.fingerprint = diagnosticFingerprint(normalized);
    record.normalized = normalized;
    if (profile) {
      assert(existsSync(profileFile), "CLI did not write its requested profile");
      const captured = JSON.parse(readFileSync(profileFile, "utf8"));
      assert.equal(captured.command, "check");
      record.templateReads = profileReads(captured, side, corpus.expectedRetainedReads);
      record.profileFile = relative(directory, profileFile);
      record.profileSha256 = fileSha256(profileFile);
    }
    writeJson(join(directory, "raw", `${id}.json`), record);
    appendFileSync(
      join(directory, "runs.ndjson"),
      `${JSON.stringify({ ...record, normalized: undefined })}\n`,
    );
    return {
      ...record,
      stdout: result.stdout,
      stderr: result.stderr,
      report: JSON.parse(result.stdout),
    };
  };
}
