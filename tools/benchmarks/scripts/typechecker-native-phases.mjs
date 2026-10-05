#!/usr/bin/env node
/** Native phase evidence for actual CLI shards. No timing comparison or cache mode. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  statSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import { dirname, join, resolve } from "node:path";
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
import { MODES, selfTest as selfTestProtocol, writeJson } from "./type-snapshot-cli-protocol.mjs";
import { createPhaseRunner } from "./typechecker-native-phase-runner.mjs";
import { selfTestNativePhaseReport } from "./typechecker-native-phase-report.mjs";
import { profileNativeReferences } from "./typechecker-native-profile-corpus.mjs";
import { captureSourceCustody } from "./typechecker-native-source-custody.mjs";

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
    selfTestProtocol();
    selfTestCorpus();
    return;
  }
  assert.equal(
    argv.length,
    3,
    "usage: typechecker-native-phases.mjs VIZE_BIN PROJECTION_BIN OUTPUT",
  );
  const custody = captureSourceCustody({
    driverRoot: ROOT,
    sourceRoot: process.env.NATIVE_PHASE_SOURCE_ROOT,
  });
  const profiling = custody.nativeProfile !== "none";
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
    "typechecker-native-phase-runner.mjs",
    "typechecker-native-phase-forwarding.test.mjs",
    "typechecker-native-profile-corpus-fixture.mjs",
    ...["graph-archive", "profile-replay", "profile-corpus", "source-custody"].flatMap((name) => [
      "typechecker-native-" + name + ".mjs",
      "typechecker-native-" + name + ".test.mjs",
    ]),
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
    sourceSha: custody.source.sha,
    driverSha: custody.driver.sha,
    sourceCustody: custody,
    mainProductionSourceSha: custody.baseline.sha,
    sourceBaseline: {
      mainHeadSha: process.env.MAIN_HEAD_SHA ?? null,
      prBaseSha: process.env.PR_BASE_SHA ?? null,
      changedPaths: custody.changedPaths,
      productionMatchesBaseline: custody.productionMatchesBaseline,
    },
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
      nativeProfile: custody.nativeProfile,
      nativeMemberAuthority:
        "raw native listFilesOnly/config/member bytes; graph closure unclaimed",
      cliProgramsAuthority: "authored/configured CLI metadata, not native transitive membership",
      startupMs: null,
      programConstructionMs: null,
    },
    rows: [],
    freshness: [],
  };
  assert.equal(metadata.dependencies.runtime, "7.0.2", "unsupported native npm package version");
  assert.equal(metadata.versions.native, "Version 7.0.2", "unsupported native binary version");
  writeJson(join(directory, "provenance.json"), metadata);
  const { project, pair } = createPhaseRunner({ root: ROOT, directory, binaries, wrapper });
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
                    profiling,
                  ),
                );
              return gatePairs.get(cwd)[side];
            },
            plants.dirs,
            fullPlant.dir,
            checked[side].report,
          );
        metadata.rows.at(-1).plants = gates;
        if (profiling)
          metadata.rows.at(-1).nativeProfiles = profileNativeReferences({
            corpus,
            checked,
            gatePairs,
            fullPlant,
            mode,
            label,
            directory,
            project,
            pair,
          });
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
      // An integer-second starting timestamp is exactly representable by Date
      // and utimes. Verify nanoseconds so truncation cannot weaken this gate.
      utimesSync(file, 1_700_000_000, 1_700_000_000);
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
      const originalMtimeNs = statSync(file, { bigint: true }).mtimeNs;
      const errorText = readFileSync(file, "utf8");
      writeFileSync(file, errorText.replace('"bad"', "12345"));
      utimesSync(file, original.atime, original.mtime);
      const repairedMtimeNs = statSync(file, { bigint: true }).mtimeNs;
      assert.equal(repairedMtimeNs, originalMtimeNs, "mtime was not restored exactly");
      assert.equal(
        statSync(file).size,
        Buffer.byteLength(errorText),
        "repair changed source byte length",
      );
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
        mtimeNs: { before: String(originalMtimeNs), repaired: String(repairedMtimeNs) },
        sourceBytes: Buffer.byteLength(errorText),
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
    assert.deepEqual(
      captureSourceCustody({ driverRoot: ROOT, sourceRoot: process.env.NATIVE_PHASE_SOURCE_ROOT }),
      custody,
      "source custody changed during replay",
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
