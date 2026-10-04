#!/usr/bin/env node
/** Native phase evidence for actual CLI shards. No timing comparison or cache mode. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  statSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  assertBinariesUnchanged,
  fileSha256,
  hashInPlace,
  pinExecutable,
} from "./benchmark-binary.mjs";
import { packageVersion, resolveVuePackageDir } from "./check-gate-env.mjs";
import { gateVize, prepareMinimalPlants, prepareCorpusPlant } from "./check-gate-plants.mjs";
import { corpusManifest, prepareCliCorpora, selfTestCorpus } from "./type-snapshot-cli-corpus.mjs";
import {
  MODES,
  checkEnvironment,
  normalizeCliReport,
  writeJson,
  compareStrings,
} from "./type-snapshot-cli-protocol.mjs";
import { diagnosticFingerprint } from "./typecheck-command.mjs";
import { selfTestNativePhaseReport } from "./typechecker-native-phase-report.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const COPIED = {
  "type-snapshot-cli-corpus.mjs":
    "7d32655b382fe822cb04928fb4d64f81ab4c9179afdbb3bf9362ac18d34ec0aa",
  "type-snapshot-cli-leaf-corpus.mjs":
    "910957c9fdc61e8b480ed23a5521c074599dbc578eed0dba40c7e763a760b54c",
  "type-snapshot-cli-protocol.mjs":
    "60dfeeeccb3d3a9bf98a7ae4947d7f5cd041751cb506f1ad87d6f893e1c795a1",
};
const command = (binary, args, cwd, env = process.env) =>
  spawnSync(binary, args, {
    cwd,
    env,
    encoding: "utf8",
    timeout: 300_000,
    maxBuffer: 256 * 1024 * 1024,
  });
function successful(result, label) {
  assert.equal(result.error, undefined, `${label}: ${result.error?.message}`);
  assert.equal(result.status, 0, `${label}: ${result.stderr}`);
  return result.stdout.trim();
}

export function main(argv = process.argv.slice(2)) {
  if (argv.length === 1 && argv[0] === "--self-test") {
    selfTestNativePhaseReport();
    selfTestCorpus();
    return;
  }
  assert.equal(
    argv.length,
    3,
    "usage: typechecker-native-phases.mjs VIZE_BIN PROJECTION_BIN OUTPUT",
  );
  assert.match(process.env.SOURCE_SHA ?? "", /^[0-9a-f]{40}$/u);
  assert.equal(
    successful(command("git", ["rev-parse", "HEAD"], ROOT), "git HEAD"),
    process.env.SOURCE_SHA,
  );
  assert.equal(
    successful(command("git", ["diff", "--name-only", "HEAD"], ROOT), "clean source"),
    "",
  );
  const output = resolve(argv[2]);
  const directory = `${output}.samples`;
  assert(!existsSync(output) && !existsSync(directory), "output must be fresh");
  for (const part of ["work", "inputs", "raw", "projections", "virtual-ts", "native"])
    mkdirSync(join(directory, part), { recursive: true });
  const binaries = {
    vize: pinExecutable(realpathSync(resolve(argv[0])), join(directory, "work")),
    projection: pinExecutable(realpathSync(resolve(argv[1])), join(directory, "work")),
    native: hashInPlace(
      realpathSync(
        join(
          ROOT,
          "node_modules/@typescript",
          `typescript-${process.platform}-${process.arch}`,
          "lib",
          process.platform === "win32" ? "tsc.exe" : "tsc",
        ),
      ),
    ),
  };
  const vue = resolveVuePackageDir();
  assert(vue, "Vue dependency missing");
  const wrapper = join(ROOT, "tools/benchmarks/scripts/typechecker-native-phase-capture.mjs");
  assert(statSync(wrapper).mode & 0o111, "native wrapper must be committed executable");
  const helperNames = [
    ...Object.keys(COPIED),
    "generate.mjs",
    "check-gate-env.mjs",
    "check-gate-plants.mjs",
    "typecheck-command.mjs",
    "benchmark-binary.mjs",
    "typechecker-native-phases.mjs",
    "typechecker-native-phase-capture.mjs",
    "typechecker-native-phase-report.mjs",
  ];
  for (const [file, expected] of Object.entries(COPIED))
    assert.equal(
      fileSha256(join(ROOT, "tools/benchmarks/scripts", file)),
      expected,
      "neutral generator drift",
    );
  const metadata = {
    schemaVersion: 1,
    kind: "native-backend-phase-observation",
    sourceSha: process.env.SOURCE_SHA,
    mainProductionSourceSha: process.env.MAIN_SOURCE_SHA ?? null,
    generatedAt: new Date().toISOString(),
    runId: process.env.GITHUB_RUN_ID ?? null,
    runAttempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
    binaries,
    runner: {
      label: process.env.RUNNER_LABEL ?? "local",
      platform: process.platform,
      arch: process.arch,
      cpuCount: os.cpus().length,
      availableParallelism: os.availableParallelism(),
      node: process.version,
    },
    dependencies: {
      vue: packageVersion(vue),
      runtime: packageVersion(resolve(dirname(binaries.native.measuredPath), "..")),
      lockSha256: fileSha256(join(ROOT, "pnpm-lock.yaml")),
    },
    versions: Object.fromEntries(
      Object.entries(binaries)
        .filter(([key]) => key !== "projection")
        .map(([key, binary]) => [
          key,
          successful(command(binary.measuredPath, ["--version"], ROOT), `${key} version`),
        ]),
    ),
    provenance: {
      copiedNeutralProviderSha: "3712c7cb784848655050221538a6b9cfa8bb0dd9",
      scripts: Object.fromEntries(
        helperNames.map((file) => [file, fileSha256(join(ROOT, "tools/benchmarks/scripts", file))]),
      ),
    },
    protocol: {
      measurements: "untimed observation; no warm medians, speedup or cache claim",
      wrapper:
        "membership/hash before; original native argv; phase replay; membership/hash after; original output/status forwarded only after successful complete instrumentation",
      ordering:
        "direct native CLI then wrapped CLI; byte/code/map and ordered full diagnostics gates",
      states: [
        "empty semantic cache first use",
        "filesystem-warm fresh CLI without semantic state",
        "filesystem-warm fresh CLI with disk semantic state (not implemented)",
        "live persistent session (not measured)",
      ],
      limits:
        "phase values may overlap/aggregate parallel work; no startup or program-construction attribution from a residual",
      contention:
        "early wrappers replay while other original shard checks may still run; every wrapper wall/phase value is instrumentation only",
      instructionCeilingsChanged: false,
      incrementalCache: false,
    },
    rows: [],
    freshness: [],
  };
  writeJson(join(directory, "provenance.json"), metadata);
  let sequence = 0;
  function project(corpus, label) {
    const result = command(
      binaries.projection.measuredPath,
      [corpus.dir],
      ROOT,
      checkEnvironment(MODES[0]),
    );
    const stem = join(directory, "projections", label);
    writeFileSync(`${stem}.json`, result.stdout ?? "");
    writeFileSync(`${stem}.stderr.txt`, result.stderr ?? "");
    successful(result, "projection");
    assert.equal(result.stderr, "");
    const parsed = JSON.parse(result.stdout);
    const expected = corpusManifest(corpus.dir)
      .files.filter((file) => file.file.endsWith(".vue"))
      .map((file) => file.file);
    assert.deepEqual(
      parsed.files.map((file) => file.file),
      expected,
      "mapping probe omitted inputs",
    );
    return { text: result.stdout, sha256: fileSha256(`${stem}.json`) };
  }
  function run(corpus, mode, wrapped, label, captureVirtual = false) {
    const id = `${String(sequence++).padStart(3, "0")}-${label}-${wrapped ? "wrapped" : "direct"}`;
    const nativeDir = join(directory, "native", id);
    const settingsPath = join(directory, "raw", `${id}.settings.json`);
    writeJson(settingsPath, { directory: nativeDir, runtime: binaries.native.measuredPath });
    const expected = readdirSync(corpus.dir).filter((file) => file.endsWith(".vue"));
    const authoredInputs = corpusManifest(corpus.dir)
      .files.filter((file) => /\.(?:vue|[cm]?[jt]sx?)$/u.test(file.file))
      .map((file) => file.file);
    const virtualTargets = captureVirtual
      ? [...expected.map((file) => `${file}.virtual.ts`), "__vize_helpers.d.ts"]
      : [];
    for (const file of virtualTargets)
      assert(!existsSync(join(corpus.dir, file)), `unexpected old virtual output ${file}`);
    try {
      const args = [
        "check",
        ...(corpus.args ?? ["."]),
        "--quiet",
        "--format",
        "json",
        "--tsconfig",
        "tsconfig.json",
        "--corsa-path",
        wrapped ? wrapper : binaries.native.measuredPath,
        ...mode.args,
      ];
      if (captureVirtual)
        for (const file of [...expected, "__vize_helpers.d.ts"])
          args.push("--save-virtual-ts-for", file);
      const env = checkEnvironment(mode);
      env.NATIVE_PHASE_CONFIG = settingsPath;
      const result = command(binaries.vize.measuredPath, args, corpus.dir, env);
      writeFileSync(join(directory, "raw", `${id}.stdout.json`), result.stdout ?? "");
      writeFileSync(join(directory, "raw", `${id}.stderr.txt`), result.stderr ?? "");
      const record = {
        id,
        wrapped,
        command: [binaries.vize.measuredPath, ...args],
        cwd: corpus.dir,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        environment: {
          RAYON_NUM_THREADS: env.RAYON_NUM_THREADS ?? null,
          GOMAXPROCS: env.GOMAXPROCS ?? null,
        },
        excludedFromTimings: true,
      };
      writeJson(join(directory, "raw", `${id}.json`), record);
      assert.equal(result.error, undefined, result.error?.message);
      const normalized = normalizeCliReport(result, corpus.dir, expected);
      record.normalized = normalized;
      record.fingerprint = diagnosticFingerprint(normalized);
      if (captureVirtual) {
        record.virtualFiles = [];
        for (const file of virtualTargets) {
          const from = join(corpus.dir, file);
          assert(existsSync(from), `missing virtual output ${file}`);
          const target = join(directory, "virtual-ts", id, file);
          mkdirSync(dirname(target), { recursive: true });
          copyFileSync(from, target);
          record.virtualFiles.push({
            file,
            bytes: statSync(target).size,
            sha256: fileSha256(target),
          });
        }
      }
      if (wrapped) {
        assert(existsSync(nativeDir), "wrapper produced no native shard evidence");
        record.nativeShards = readdirSync(nativeDir)
          .filter((file) => /^\d+\.json$/u.test(file))
          .map((file) => ({
            record: relative(directory, join(nativeDir, file)),
            ...JSON.parse(readFileSync(join(nativeDir, file), "utf8")),
          }))
          .sort((a, b) => a.config.localeCompare(b.config, "en"));
        assert(record.nativeShards.length > 0, "no native shard receipt");
        if (captureVirtual)
          for (const file of record.virtualFiles) {
            const nativeName = file.file.replace(/\.virtual\.ts$/u, ".ts");
            const matches = record.nativeShards.flatMap((shard) =>
              shard.members.filter((member) => relative(shard.cwd, member.path) === nativeName),
            );
            assert(matches.length > 0, `saved output absent from native program: ${file.file}`);
            assert(
              matches.every(
                (member) => member.sha256 === file.sha256 && member.bytes === file.bytes,
              ),
              `saved output differs from native program bytes: ${file.file}`,
            );
          }
        const covered = new Set(
          record.nativeShards.flatMap((shard) =>
            shard.members
              .filter((file) => file.path.endsWith(".vue.ts"))
              .map((file) => relative(shard.cwd, file.path).replace(/\.ts$/u, "")),
          ),
        );
        assert.deepEqual(
          [...covered].sort(compareStrings),
          [...expected].sort(compareStrings),
          "actual native shards omitted Vue sources",
        );
        const programSources = new Set(normalized.programs.flatMap((program) => program.files));
        for (const file of authoredInputs) {
          assert(programSources.has(file), `frontend program omitted authored input: ${file}`);
          const nativeName = file.endsWith(".vue") ? `${file}.ts` : file;
          assert(
            record.nativeShards.some((shard) =>
              shard.members.some((member) => relative(shard.cwd, member.path) === nativeName),
            ),
            `native program omitted authored input: ${file}`,
          );
        }
      }
      writeJson(join(directory, "raw", `${id}.json`), record);
      return {
        ...record,
        report: JSON.parse(result.stdout),
        stdout: result.stdout,
        stderr: result.stderr,
      };
    } finally {
      for (const file of virtualTargets) rmSync(join(corpus.dir, file), { force: true });
    }
  }
  function pair(corpus, mode, label, virtual = false) {
    const manifest = corpusManifest(corpus.dir);
    const snapshot = join(directory, "inputs", label);
    mkdirSync(snapshot, { recursive: true });
    for (const file of manifest.files) {
      // Initial full source archives already retain unchanged corpus bytes.
      if (
        corpus.manifest.files.some(
          (original) => original.file === file.file && original.sha256 === file.sha256,
        )
      )
        continue;
      const target = join(snapshot, file.file);
      mkdirSync(dirname(target), { recursive: true });
      copyFileSync(join(corpus.dir, file.file), target);
    }
    writeJson(join(snapshot, "manifest.json"), manifest);
    const direct = run(corpus, mode, false, label, virtual);
    const wrapped = run(corpus, mode, true, label, virtual);
    assert.equal(
      wrapped.fingerprint,
      direct.fingerprint,
      `${label}: full ordered CLI report differs`,
    );
    if (virtual)
      assert.deepEqual(
        wrapped.virtualFiles,
        direct.virtualFiles,
        `${label}: actual virtual bytes differ`,
      );
    assert.equal(
      corpusManifest(corpus.dir).sha256,
      manifest.sha256,
      `${label}: probe changed fixture inputs`,
    );
    return { direct, wrapped };
  }
  try {
    const corpora = prepareCliCorpora(directory, vue);
    for (const corpus of corpora) {
      const before = project(corpus, `${corpus.id}-before`);
      for (const mode of MODES) {
        const label = `${corpus.id}-${mode.id}`;
        const checked = pair(corpus, mode, label, true);
        metadata.rows.push({
          corpus: corpus.id,
          mode: mode.id,
          inputManifest: corpus.manifest,
          directReceipt: checked.direct.id,
          wrappedReceipt: checked.wrapped.id,
          diagnosticFingerprint: checked.direct.fingerprint,
          virtualFiles: checked.direct.virtualFiles,
          nativeShards: checked.wrapped.nativeShards,
        });
        const plants = prepareMinimalPlants(join(directory, "work", `${label}-plants`), vue);
        const fullPlant = prepareCorpusPlant(corpus.dir, corpus.tsconfig);
        const gatePairs = new Map();
        const gates = {};
        for (const side of ["direct", "wrapped"])
          gates[side] = gateVize(
            (cwd) => {
              if (!gatePairs.has(cwd))
                gatePairs.set(
                  cwd,
                  pair(
                    { ...corpus, dir: cwd, args: cwd === fullPlant.dir ? corpus.args : ["."] },
                    mode,
                    `${label}-plant-${cwd === fullPlant.dir ? "corpus" : cwd.split(/[/\\]/u).at(-1)}`,
                  ),
                );
              return gatePairs.get(cwd)[side];
            },
            plants.dirs,
            fullPlant.dir,
            checked[side].report,
          );
        metadata.rows.at(-1).plants = gates;
      }
      const after = project(corpus, `${corpus.id}-after`);
      assert.equal(after.text, before.text, "code or complete mapping/link projection changed");
      assert.equal(
        corpusManifest(corpus.dir).sha256,
        corpus.manifest.sha256,
        "probe changed source inputs",
      );
      const file = join(corpus.dir, "__NativePhaseFreshness.vue");
      assert(!existsSync(file));
      writeFileSync(
        file,
        '<script setup lang="ts">const value: number = "bad";</script><template>{{ value }}</template>\n',
      );
      const errorPair = pair(corpus, MODES[0], `${corpus.id}-fresh-create`);
      assert(
        errorPair.direct.report.files.some(
          (entry) =>
            entry.file.endsWith("__NativePhaseFreshness.vue") &&
            entry.diagnostics.some((value) => value.includes("TS2322")),
        ),
        "fresh plant missing",
      );
      const original = statSync(file);
      const errorText = readFileSync(file, "utf8");
      writeFileSync(file, errorText.replace('"bad"', "12345"));
      utimesSync(file, original.atime, original.mtime);
      const repaired = pair(corpus, MODES[0], `${corpus.id}-fresh-same-length-repair`);
      assert(
        repaired.direct.report.files.some(
          (entry) =>
            entry.file.endsWith("__NativePhaseFreshness.vue") && entry.diagnostics.length === 0,
        ),
        "fresh repair not seen",
      );
      rmSync(file);
      const deleted = pair(corpus, MODES[0], `${corpus.id}-fresh-delete`);
      const baseline = metadata.rows.find((row) => row.corpus === corpus.id && row.mode === "max");
      assert.equal(
        deleted.direct.fingerprint,
        baseline.diagnosticFingerprint,
        "delete did not restore baseline",
      );
      metadata.freshness.push({
        corpus: corpus.id,
        create: errorPair.direct.id,
        repair: repaired.direct.id,
        delete: deleted.direct.id,
        sameLengthRestoredMtime: true,
        allDirectWrappedReportsEqual: true,
      });
      assert.equal(corpusManifest(corpus.dir).sha256, corpus.manifest.sha256);
    }
    assertBinariesUnchanged(binaries);
    for (const [name, binary] of Object.entries(binaries))
      assert.equal(
        fileSha256(binary.measuredPath),
        binary.sha256,
        `${name} measured binary changed`,
      );
    for (const [file, hash] of Object.entries(metadata.provenance.scripts))
      assert.equal(
        fileSha256(join(ROOT, "tools/benchmarks/scripts", file)),
        hash,
        `helper changed: ${file}`,
      );
    metadata.status = "complete";
    writeJson(output, metadata);
    console.log(
      `Native phase observation complete: ${metadata.rows.length} rows; no performance comparison.`,
    );
  } catch (error) {
    metadata.status = "failed";
    metadata.failure = error.stack ?? String(error);
    writeJson(join(directory, "failure.json"), metadata);
    throw error;
  }
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
