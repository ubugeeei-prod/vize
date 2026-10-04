#!/usr/bin/env node
/**
 * Same-run, exact-SHA `vize check` comparison. Every process retains its full
 * diagnostics; existing minimal and full-corpus plants gate every side/mode.
 * Cold means the first check process per row, without filesystem cache eviction.
 * Profiling runs only after all unprofiled timing pairs and never enters medians.
 */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { assertBinariesUnchanged, fileSha256 } from "./benchmark-binary.mjs";
import { gateVize, prepareCorpusPlant, prepareMinimalPlants } from "./check-gate-plants.mjs";
import {
  corpusManifest,
  prepareCliCorpora,
  selfTestCorpus,
} from "./retained-template-reads-cli-corpus.mjs";
import {
  PAIRS,
  MODES,
  SIDES,
  WARMUPS,
  pairOrder,
  checkEnvironment,
  selfTest,
  summarizePairs,
  writeJson,
} from "./retained-template-reads-cli-protocol.mjs";
import {
  commandOutput,
  captureCliVirtualTs,
  createRunner,
  prepareRun,
} from "./retained-template-reads-cli-runner.mjs";

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
  return { ...files, equal: true, variant: "public-content-mapper-transform" };
}

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

export function main(argv = process.argv.slice(2)) {
  const [baseInput, headInput, outputInput] = argv;
  assert.equal(
    argv.length,
    3,
    "usage: retained-template-reads-cli-paired.mjs BASE_BIN HEAD_BIN OUTPUT (BASE_SHA and HEAD_SHA required)",
  );
  for (const key of ["BASE_SHA", "HEAD_SHA"])
    assert.match(process.env[key] ?? "", /^[0-9a-f]{40}$/u);
  const output = resolve(outputInput);
  const directory = `${output}.samples`;
  assert(
    !existsSync(directory) && !existsSync(output),
    "output already exists; use a fresh artifact path",
  );
  mkdirSync(directory, { recursive: true });
  for (const subdir of ["raw", "profiles", "projections", "virtual-ts", "inputs", "work"])
    mkdirSync(join(directory, subdir));
  const workRoot = join(directory, "work");
  let metadata = { baseSha: process.env.BASE_SHA, headSha: process.env.HEAD_SHA };
  try {
    const prepared = prepareRun(baseInput, headInput, directory, ROOT);
    const { binaries, binarySources, runtimePath, vuePackageDir } = prepared;
    metadata = prepared.metadata;
    const corpora = prepareCliCorpora(directory, vuePackageDir);
    for (const corpus of corpora)
      corpus.projectionParity = compareProjections(directory, binaries, corpus);
    const modes = MODES;
    const rows = [];
    const run = createRunner(directory, binaries, runtimePath);
    for (const [corpusIndex, corpus] of corpora.entries()) {
      const manifest = corpus.manifest;
      for (const [modeIndex, mode] of modes.entries()) {
        const cold = {};
        for (const side of pairOrder(corpusIndex + modeIndex))
          cold[side] = run(side, mode, corpus, corpus.dir, "cold");
        assert.equal(
          cold.head.fingerprint,
          cold.base.fingerprint,
          `${corpus.id}/${mode.id}: cold diagnostics/programs differ`,
        );
        const readiness = {};
        const gateFingerprints = new Map();
        const plants = prepareMinimalPlants(
          join(workRoot, `${corpus.id}-${mode.id}`),
          vuePackageDir,
        );
        const corpusPlant = prepareCorpusPlant(corpus.dir, corpus.tsconfig);
        try {
          for (const side of SIDES)
            readiness[side] = gateVize(
              (cwd) => {
                const phase =
                  cwd === corpusPlant.dir ? "gate-corpus" : `gate-${relative(plants.root, cwd)}`;
                const result = run(side, mode, corpus, cwd, phase);
                const previous = gateFingerprints.get(phase);
                if (previous != null)
                  assert.equal(
                    result.fingerprint,
                    previous,
                    `${corpus.id}/${mode.id}/${phase}: planted diagnostics/programs differ`,
                  );
                gateFingerprints.set(phase, result.fingerprint);
                return result;
              },
              plants.dirs,
              corpusPlant.dir,
              cold[side].report,
            );
        } finally {
          plants.cleanup();
          corpusPlant.cleanup();
        }
        function paired(phase, index) {
          const current = {};
          for (const side of pairOrder(index)) {
            current[side] = run(side, mode, corpus, corpus.dir, `${phase}-${index}`);
            assert.equal(
              current[side].fingerprint,
              cold.base.fingerprint,
              `${current[side].id}: diagnostics/programs changed`,
            );
          }
          return current;
        }
        for (let index = 0; index < WARMUPS; index++) paired("warmup", index);
        const samples = { base: [], head: [] };
        for (let index = 0; index < PAIRS; index++) {
          const pair = paired("timed", index);
          for (const side of SIDES) samples[side].push(pair[side]);
        }
        rows.push({
          id: `${corpus.id}-${mode.id}`,
          corpus: corpus.id,
          mode: mode.id,
          inputSha256: manifest.sha256,
          projectionParity: corpus.projectionParity,
          fingerprint: cold.base.fingerprint,
          diagnostics: {
            errors: cold.base.report.errorCount,
            warnings: cold.base.report.warningCount,
            files: cold.base.report.fileCount,
          },
          readiness,
          cold: Object.fromEntries(
            SIDES.map((side) => [side, { ms: cold[side].ms, sampleId: cold[side].id }]),
          ),
          coldHeadToBaseRatio: cold.head.ms / cold.base.ms,
          ...summarizePairs(samples),
        });
        const previousMode = rows.find((row) => row.corpus === corpus.id && row.mode !== mode.id);
        if (previousMode)
          assert.equal(
            previousMode.fingerprint,
            cold.base.fingerprint,
            "thread modes changed diagnostics/programs",
          );
      }
    }
    // Profile only after all four rows finish: profiling must not warm later timing rows.
    for (const corpus of corpora)
      for (const mode of modes)
        for (const side of SIDES) {
          const profile = run(side, mode, corpus, corpus.dir, "profile", true);
          const row = rows.find((entry) => entry.id === `${corpus.id}-${mode.id}`);
          assert.equal(
            profile.fingerprint,
            row.fingerprint,
            `${profile.id}: profiled diagnostics/programs differ`,
          );
          row.profiles ??= {};
          row.profiles[side] = {
            sampleId: profile.id,
            file: profile.profileFile,
            sha256: profile.profileSha256,
            templateReads: profile.templateReads,
          };
        }
    // Capture actual CLI generation only after all timed rows and profiles finish.
    for (const corpus of corpora)
      for (const mode of modes) {
        const row = rows.find((entry) => entry.id === `${corpus.id}-${mode.id}`);
        row.cliVirtualTsParity = captureCliVirtualTs(directory, run, corpus, mode, row.fingerprint);
        assert.equal(
          corpusManifest(corpus.dir).sha256,
          corpus.manifest.sha256,
          "virtual TS capture changed corpus inputs",
        );
        const previous = rows.find(
          (entry) =>
            entry.corpus === corpus.id && entry.mode !== mode.id && entry.cliVirtualTsParity,
        );
        if (previous)
          assert.equal(
            previous.cliVirtualTsParity.base.sha256,
            row.cliVirtualTsParity.base.sha256,
            "thread mode changed actual CLI generation",
          );
      }
    assertBinariesUnchanged(binaries);
    for (const corpus of corpora)
      assert.equal(
        corpusManifest(corpus.dir).sha256,
        corpus.manifest.sha256,
        "corpus changed during measurement",
      );
    metadata.versions = {
      base: commandOutput(binarySources.base, ["--version"]),
      head: commandOutput(binarySources.head, ["--version"]),
      typescript: commandOutput(runtimePath, ["--version"]),
      rustc: commandOutput("rustc", ["-Vv"]),
      cargo: commandOutput("cargo", ["--version"]),
    };
    writeJson(join(directory, "provenance.json"), metadata);
    writeJson(output, {
      ...metadata,
      corpora: corpora.map((corpus) => ({ id: corpus.id, ...corpus.manifest })),
      rows,
    });
    const markdown = [
      "| Corpus / mode | Cold base / head (ms) | Steady base / head median (ms) | Head/base median | Median paired head/base |",
      "| --- | ---: | ---: | ---: | ---: |",
      ...rows.map(
        (row) =>
          `| ${row.id} | ${row.cold.base.ms.toFixed(1)} / ${row.cold.head.ms.toFixed(1)} | ${row.baseMedianMs.toFixed(1)} / ${row.headMedianMs.toFixed(1)} | ${row.headToBaseMedianRatio.toFixed(3)} | ${row.medianPairedHeadToBaseRatio.toFixed(3)} |`,
      ),
      "",
      `Exact base \`${metadata.baseSha}\` and head \`${metadata.headSha}\`; ${WARMUPS} warmups and ${PAIRS} alternating fresh-process pairs.`,
      "Cold samples are each row's first check process; filesystem caches were not evicted. All plant gates and complete ordered diagnostics/program signatures passed. Public mapper code/mappings and actual CLI virtual TS (every SFC plus shared helpers) matched. Profiles and virtual TS captures are separate, untimed runs.",
      "",
    ];
    writeFileSync(`${output}.md`, markdown.join("\n"));
    process.stdout.write(markdown.join("\n"));
  } catch (error) {
    writeJson(join(directory, "failure.json"), {
      ...metadata,
      failure: error instanceof Error ? error.message : String(error),
    });
    throw error;
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    if (process.argv.length === 3 && process.argv[2] === "--self-test") {
      selfTest();
      selfTestCorpus();
    } else main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
