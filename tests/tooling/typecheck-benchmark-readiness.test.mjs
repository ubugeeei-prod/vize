import assert from "node:assert/strict";
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";
import { test } from "node:test";

import {
  CORPUS_PLANT_FILE,
  MINIMAL_PLANTS,
} from "../../tools/benchmarks/scripts/check-gate-plants.mjs";
import {
  createTypecheckToolVariants,
  prepareTypecheckPackages,
} from "../../tools/benchmarks/scripts/compare-tools-typecheck.mjs";
import {
  normalizeTypecheckResult,
  runTypecheckCommand,
} from "../../tools/benchmarks/scripts/typecheck-command.mjs";
import {
  assertCorpusPlant,
  assertMinimalPlant,
  checkedMeasurement,
  gateTypecheckVariants,
} from "../../tools/benchmarks/scripts/typecheck-readiness.mjs";

function project(t) {
  const dir = realpathSync(mkdtempSync(join(tmpdir(), "typecheck-readiness-")));
  t.after(() => {
    for (const suffix of ["", "-readiness", "-gate-plant"])
      rmSync(`${dir}${suffix}`, { recursive: true, force: true });
  });
  writeFileSync(join(dir, "Base.vue"), "<script setup lang='ts'>const n: number = 'bad'</script>");
  writeFileSync(join(dir, "tsconfig.json"), JSON.stringify({ include: ["Base.vue"] }));
  return dir;
}

const baseDiagnostic = ["Base.vue", "error", "1", "30", "TS2322", "not a number"];
const baseline = { status: 1, diagnostics: [baseDiagnostic] };
const plain = (diagnostics = [baseDiagnostic]) => ({
  ms: 12.5,
  status: diagnostics.length ? 1 : 0,
  stdout: diagnostics
    .map(
      ([file, kind, line, col, code, message]) =>
        `${file}(${line},${col}): ${kind} ${code}: ${message}`,
    )
    .join("\n"),
  stderr: "",
});

function plantedRun(dir) {
  const plant = MINIMAL_PLANTS.find((p) => p.id === basename(dir));
  if (plant) {
    const [, line, col, code, message] = /^error:(\d+):(\d+) \[(\w+)\] (.*)$/u.exec(
      plant.expected.diagnostic,
    );
    return plain([["App.vue", "error", line, col, code, message]]);
  }
  return plain(
    existsSync(join(dir, CORPUS_PLANT_FILE))
      ? [
          baseDiagnostic,
          [CORPUS_PLANT_FILE, "error", "2", "7", "TS2322", "not a number"],
          [CORPUS_PLANT_FILE, "error", "7", "26", "TS2322", "not Booleanish"],
        ]
      : [baseDiagnostic],
  );
}

void test("every ranked lane proves minimal and timed-corpus work before measuring", (t) => {
  const dir = project(t);
  const calls = [];
  const variants = ["vue-tsc", "verter-tsc", "golar-typecheck", "golar-default"].map((id) => ({
    id,
    run: (cwd) => {
      calls.push([id, cwd]);
      return plantedRun(cwd);
    },
  }));
  const gated = gateTypecheckVariants(variants, dir, prepareTypecheckPackages);
  assert.equal(calls.length, 4 * 7);
  for (const lane of gated) {
    assert.deepEqual(
      lane.correctness.minimalPlants,
      MINIMAL_PLANTS.map((p) => p.id),
    );
    assert.equal(lane.correctness.corpusPlant, true);
    assert.equal(lane.correctness.diagnosticCount, 1);
    assert.match(lane.correctness.diagnosticFingerprint, /^[a-f0-9]{64}$/u);
    const callsBeforeMeasurement = calls.length;
    assert.equal(lane.measure(), 12.5);
    assert.equal(calls.length, callsBeforeMeasurement + 1);
    assert.equal(lane.measure(), 12.5);
    assert.equal(calls.length, callsBeforeMeasurement + 2);
  }
});

void test("a successful no-op and a startup error cannot produce a ranked lane", (t) => {
  const dir = project(t);
  for (const result of [
    plain([]),
    { ...plain([]), status: 1, stderr: "Error [ERR_MODULE_NOT_FOUND]: golar" },
  ]) {
    assert.throws(
      () => gateTypecheckVariants([{ id: "broken", run: () => result }], dir, () => {}),
      /refusing to publish/u,
    );
  }
});

void test("a measured diagnostic disappearance, move or replacement fails even at the same count", (t) => {
  const dir = project(t);
  for (const diagnostics of [
    [],
    [[...baseDiagnostic.slice(0, 2), "2", ...baseDiagnostic.slice(3)]],
    [[...baseDiagnostic.slice(0, 4), "TS2339", "different"]],
  ]) {
    const measure = checkedMeasurement(
      { id: "changed", run: () => plain(diagnostics) },
      dir,
      baseline,
    );
    assert.throws(measure, /diagnostics changed/u);
  }
});

void test("planted diagnostics cannot substitute for changed baseline work", () => {
  const additions = ["2", "7"].map((line) => [
    CORPUS_PLANT_FILE,
    "error",
    line,
    "1",
    "TS2322",
    "bad",
  ]);
  assert.throws(
    () => assertCorpusPlant(baseline, { status: 1, diagnostics: additions }),
    /baseline diagnostics/u,
  );
  assert.throws(
    () => assertMinimalPlant({ status: 1, diagnostics: [baseDiagnostic] }, MINIMAL_PLANTS[0]),
    /missed/u,
  );
});

void test("native reports validate their file diagnostics, counts and exit status", (t) => {
  const dir = project(t);
  const report = {
    files: [{ file: "Base.vue", diagnostics: ["error:1:30 [TS2322] not a number"] }],
    errorCount: 1,
    warningCount: 0,
  };
  const result = { ...plain(), stdout: JSON.stringify(report) };
  assert.deepEqual(normalizeTypecheckResult(result, dir, "json"), baseline);
  assert.throws(() => normalizeTypecheckResult({ ...result, status: 2 }, dir, "json"), /exited/u);
  assert.equal(normalizeTypecheckResult({ ...plain(), status: 2 }, dir).status, 2);
  assert.throws(
    () => normalizeTypecheckResult({ ...result, status: 0 }, dir, "json"),
    /exit status/u,
  );
  assert.throws(
    () =>
      normalizeTypecheckResult(
        { ...result, stdout: JSON.stringify({ ...report, errorCount: 0 }) },
        dir,
        "json",
      ),
    /count/u,
  );
  assert.throws(() => normalizeTypecheckResult({ ...result, stdout: "not json" }, dir, "json"));
});

void test("signal deaths, compiler errors without diagnostics and invalid durations fail closed", (t) => {
  const dir = project(t);
  for (const result of [
    { ...plain(), status: null },
    { ...plain(), status: 3 },
    { ...plain([]), status: 1 },
  ]) {
    assert.throws(() => normalizeTypecheckResult(result, dir));
  }
  for (const ms of [0, -1, NaN, Infinity]) {
    assert.throws(
      checkedMeasurement({ id: "duration", run: () => ({ ...plain(), ms }) }, dir, baseline),
      /duration/u,
    );
  }
});

void test("the runner retains diagnostics and rejects missing executables", (t) => {
  const dir = project(t);
  const result = runTypecheckCommand(
    process.execPath,
    ["-e", "console.log('checked'); process.exitCode = 1"],
    { cwd: dir },
  );
  assert.equal(result.status, 1);
  assert.match(result.stdout, /checked/u);
  assert.ok(result.ms > 0);
  assert.throws(() => runTypecheckCommand(join(dir, "missing"), [], { cwd: dir }), /ENOENT/u);
});

void test("Golar packages resolve from the generated project, not the benchmark script", (t) => {
  const dir = project(t);
  prepareTypecheckPackages(dir);
  prepareTypecheckPackages(dir);
  assert.match(
    readFileSync(join(dir, "node_modules/golar/package.json"), "utf8"),
    /"name": "golar"/u,
  );
  assert.match(
    readFileSync(join(dir, "node_modules/@golar/vue/package.json"), "utf8"),
    /"name": "@golar\/vue"/u,
  );
  assert.match(readFileSync(join(dir, "golar.config.ts"), "utf8"), /import "@golar\/vue"/u);
});

void test("source messages containing error identifiers are diagnostics, including continuations", (t) => {
  const dir = project(t);
  const output = {
    ...plain(),
    stdout:
      "Base.vue(1,30): error TS2322: Type 'ERR_MODULE_NOT_FOUND' is not assignable.\n  Property 'value' is missing.",
  };
  const report = normalizeTypecheckResult(output, dir);
  assert.match(report.diagnostics[0][5], /\n  Property 'value'/u);
  assert.throws(
    checkedMeasurement(
      {
        id: "continuation",
        run: () => ({ ...output, stdout: output.stdout.replace("'value'", "'other'") }),
      },
      dir,
      report,
    ),
    /changed/u,
  );
});

void test("all six lanes check the requested project and native lanes pin their backend", () => {
  const calls = [];
  const variants = createTypecheckToolVariants({
    fileCount: 20,
    vizeBin: "/bin/vize",
    corsaPath: "/bin/pinned-tsgo",
    resolveWorkspaceBin: (name) => `/bin/${name}`,
    runCommand: (bin, args, options) => {
      calls.push({ bin, args, options });
      return plain();
    },
  });
  assert.deepEqual(
    variants.map((v) => v.id),
    [
      "vue-tsc",
      "verter-tsc",
      "golar-typecheck",
      "golar-default",
      "vize-check-1t",
      "vize-check-max",
    ],
  );
  for (const variant of variants) variant.run("/project");
  assert.ok(calls.every((call) => call.options.cwd === "/project"));
  assert.equal(calls[1].options.env.VERTER_TSGO_BIN, "/bin/pinned-tsgo");
  for (const call of calls.slice(4)) {
    assert.equal(call.args[call.args.indexOf("--corsa-path") + 1], "/bin/pinned-tsgo");
    assert.equal(call.args[call.args.indexOf("--format") + 1], "json");
    assert.equal(call.args[call.args.indexOf("--tsconfig") + 1], "/project/tsconfig.json");
  }
  assert.equal(calls[4].options.env.RAYON_NUM_THREADS, "1");
  assert.ok(calls[4].args.includes("--servers"));
  assert.ok(!calls[5].args.includes("--servers"));
});

void test("native JSON lanes and path-hashed virtual diagnostics survive the corpus copy", (t) => {
  const dir = project(t);
  const run = (cwd) => {
    const result = plantedRun(cwd);
    result.stdout = result.stdout.replaceAll("Base.vue", `${basename(cwd)}.virtual.ts`);
    const report = normalizeTypecheckResult(result, cwd);
    const files = new Map();
    for (const [file, kind, line, column, code, message] of report.diagnostics) {
      if (!files.has(file)) files.set(file, []);
      files.get(file).push(`${kind}:${line}:${column} [${code}] ${message}`);
    }
    return {
      ...result,
      stdout: JSON.stringify({
        files: [...files].map(([file, diagnostics]) => ({ file, diagnostics })),
        errorCount: report.diagnostics.length,
        warningCount: 0,
      }),
    };
  };
  const [gated] = gateTypecheckVariants(
    [{ id: "vize-check-max", format: "json", run }],
    dir,
    prepareTypecheckPackages,
  );
  assert.notEqual(
    gated.correctness.diagnosticFingerprint,
    gated.correctness.corpusBaselineFingerprint,
  );
  assert.equal(gated.measure(), 12.5);
});
