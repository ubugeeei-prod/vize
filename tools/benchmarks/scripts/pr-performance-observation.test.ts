import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { advisoryComparison, collectObservation } from "./pr-performance-observation.ts";
import { verifyObservationBindings } from "./pr-performance-observation-binding.ts";
import {
  assertFileCoverage,
  checkPacket,
  qualifyCheck,
} from "./pr-performance-observation-check.ts";
import {
  parseGnuTime,
  observationEnvironment,
  runObservation,
  invocationSucceeded,
  sha256,
} from "./pr-performance-observation-evidence.ts";

void test("advisory regression retains the existing paired estimator without a budget failure", () => {
  const result = advisoryComparison([10, 20, 30], [20, 40, 60], null, "compile", "Compile", 5);
  assert.equal(result.availability, "observed");
  assert.equal(result.result?.rate, 2);
  assert.equal(result.result?.status, "regression");
  assert.equal(advisoryComparison([10], [1], "missing coverage", "check", "Check", 5).result, null);
  assert.equal(
    advisoryComparison([10], [0], null, "compile", "Compile", 5).availability,
    "unavailable",
  );
});

void test("check admission rejects omitted files and diagnostics hidden by Set deduplication", () => {
  const report = {
    files: [{ file: "A.vue", diagnostics: ["error:1:1 [TS2322] bad", "error:1:1 [TS2322] bad"] }],
    fileCount: 1,
    errorCount: 1,
    warningCount: 0,
  };
  assert.throws(() => checkPacket(JSON.stringify(report)), /identical duplicate/u);
  report.errorCount = 2;
  const packet = checkPacket(JSON.stringify(report));
  assert.equal(packet.files[0].diagnostics.length, 2);
  assert.throws(() => assertFileCoverage(packet, ["A.vue", "B.vue"]), /every timed SFC/u);
  assert.throws(
    () =>
      checkPacket(
        JSON.stringify({ ...report, files: [report.files[0], report.files[0]], fileCount: 2 }),
      ),
    /duplicate file/u,
  );
});

void test("GNU time RSS is bytes and missing or zero evidence stays unavailable", () => {
  assert.equal(parseGnuTime("\tMaximum resident set size (kbytes): 1234\n"), 1234 * 1024);
  assert.equal(parseGnuTime("Maximum resident set size (kbytes): 0"), null);
  assert.equal(parseGnuTime(""), null);
  assert.equal(observationEnvironment("max").RAYON_NUM_THREADS, undefined);
  assert.equal(observationEnvironment(1).RAYON_NUM_THREADS, "1");
  assert.equal(observationEnvironment(1).GITHUB_TOKEN, undefined);
});

void test("a hung command is unavailable with raw timeout evidence, never a fast sample", () => {
  const dir = mkdtempSync(join(tmpdir(), "vize-pr-observer-timeout-"));
  try {
    const run = runObservation({
      executable: process.execPath,
      args: ["-e", "setInterval(()=>{},1000)"],
      cwd: dir,
      env: observationEnvironment(1),
      rawDir: join(dir, "raw"),
      timeoutMs: 20,
    });
    assert.equal(invocationSucceeded(run, true), false);
    assert.ok(run.receipt.error || run.receipt.status === 124 || run.receipt.signal);
    assert.ok(readFileSync(join(dir, "raw", "invocation.json"), "utf8").includes("timeoutMs"));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

void test("check lanes without explicit backend publish no timing or memory win", () => {
  const dir = mkdtempSync(join(tmpdir(), "vize-pr-observer-admission-"));
  try {
    const input = join(dir, "input");
    mkdirSync(input);
    writeFileSync(join(input, "A.vue"), "<template><div /></template>\n");
    writeFileSync(join(input, "tsconfig.json"), '{"include":["*.vue"]}');
    const report = collectObservation({
      baseBin: process.execPath,
      headBin: process.execPath,
      baseSource: "a".repeat(40),
      headSource: "b".repeat(40),
      inputDir: input,
      out: join(dir, "out"),
      runs: 1,
      warmups: 0,
      thresholdPercent: 5,
      timeoutMs: 2000,
      tasks: "check",
      backend: null,
      vueDir: null,
      buildReceipts: null,
    });
    assert.equal(report.advisory, true);
    assert.equal(report.failOnRegression, false);
    assert.equal(report.methodology.coldQualified, false);
    assert.equal(report.results[0].comparison.availability, "unavailable");
    assert.equal(report.results[0].comparison.result, null);
    assert.deepEqual(report.results[0].baseRuns, []);
    assert.equal(report.results[0].rss.ratio, null);
    assert.ok(readFileSync(join(dir, "out", "summary.md"), "utf8").includes("n/a"));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

void test("full external-process admission catches invalid witnesses and requires repaired controls", () => {
  const dir = mkdtempSync(join(tmpdir(), "vize-pr-observer-witness-"));
  try {
    const input = join(dir, "input"),
      vueDir = join(dir, "vue");
    mkdirSync(input);
    mkdirSync(vueDir);
    writeFileSync(join(input, "A.vue"), "<template><div /></template>\n");
    writeFileSync(join(input, "tsconfig.json"), '{"include":["*.vue"]}');
    const fake = join(dir, "fake-checker.mjs");
    const plants = new URL("./check-gate-plants.mjs", import.meta.url).href;
    writeFileSync(
      fake,
      `#!${process.execPath}
import {readdirSync,readFileSync} from 'node:fs';
import {basename} from 'node:path';
import {MINIMAL_PLANTS,CORPUS_PLANT_DIAGNOSTICS} from ${JSON.stringify(plants)};
const files=readdirSync(process.cwd()).filter(f=>f.endsWith('.vue')).sort().map(file=>({file,diagnostics:[]}));
let errors=0;
for(const file of files){
 const source=readFileSync(file.file,'utf8');
 const plant=MINIMAL_PLANTS.find(p=>p.id===basename(process.cwd()));
 if(plant && file.file===plant.expected.file && /not-a-number|disabledFlag: string|clickHandler = 123|variant="danger"/.test(source))file.diagnostics=[plant.expected.diagnostic];
 if(file.file==='__CheckGatePlant.vue')file.diagnostics=CORPUS_PLANT_DIAGNOSTICS;
 errors+=file.diagnostics.length;
}
console.log(JSON.stringify({files,fileCount:files.length,errorCount:errors,warningCount:0}));process.exitCode=errors?1:0;
`,
      { mode: 0o755 },
    );
    const result = qualifyCheck({
      binary: fake,
      backend: fake,
      vueDir,
      inputDir: input,
      out: join(dir, "controls"),
      expectedFiles: ["A.vue"],
      threads: 1,
      args: ["check", ".", "--tsconfig", join(input, "tsconfig.json")],
      env: observationEnvironment(1),
      timeoutMs: 2000,
    });
    assert.equal(result.qualified, true, result.reason ?? undefined);
    assert.deepEqual(result.witnesses, {
      script: true,
      templateProp: true,
      templateEvent: true,
      componentProp: true,
      fullCorpus: true,
    });
    assert.equal(result.baseline?.fileCount, 1);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

void test("build labels bind actual identity and independently fresh targets", () => {
  const dir = mkdtempSync(join(tmpdir(), "vize-pr-observer-bindings-"));
  try {
    const binaries = {
      base: {
        path: "/tmp/base/vize",
        bytes: 10,
        sha256: "a".repeat(64),
        version: "v1",
        versionStatus: 0,
      },
      head: {
        path: "/tmp/head/vize",
        bytes: 20,
        sha256: "b".repeat(64),
        version: "v1",
        versionStatus: 0,
      },
    };
    const sources = { base: "a".repeat(40), head: "b".repeat(40) };
    const receipts = Object.fromEntries(
      (["base", "head"] as const).map((side) => [
        side,
        {
          schemaVersion: 1,
          role: side,
          sourceSha: sources[side],
          treeSha: "c".repeat(40),
          binary: binaries[side].path,
          binaryBytes: binaries[side].bytes,
          binarySha256: binaries[side].sha256,
          targetStartedEmpty: true,
          targetDir: `/tmp/${side}/target`,
          cargoArtifact: {
            executable: `/tmp/${side}/target/ci-opt/vize`,
            fresh: false,
            features: ["default"],
          },
          allocation: false,
          profile: { name: "ci-opt" },
          rustc: "rustc 1.99.1",
          cargo: { args: ["build", "--profile", "ci-opt"], version: "cargo 1.99.1" },
          compilerEnvironment: {},
        },
      ]),
    );
    const write = () => {
      for (const side of ["base", "head"])
        writeFileSync(join(dir, `${side}.json`), JSON.stringify(receipts[side]));
    };
    write();
    assert.equal(verifyObservationBindings(dir, sources, binaries).qualified, true);
    receipts.head.sourceSha = sources.base;
    write();
    assert.equal(verifyObservationBindings(dir, sources, binaries).qualified, false);
    receipts.head.sourceSha = sources.head;
    receipts.head.targetDir = receipts.base.targetDir;
    write();
    assert.equal(verifyObservationBindings(dir, sources, binaries).qualified, false);
    receipts.head.targetDir = "/tmp/head/target";
    receipts.head.cargoArtifact.fresh = true;
    write();
    assert.equal(verifyObservationBindings(dir, sources, binaries).qualified, false);
    receipts.head.cargoArtifact.fresh = false;
    receipts.head.binarySha256 = binaries.base.sha256;
    write();
    assert.equal(verifyObservationBindings(dir, sources, binaries).qualified, false);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

void test("a backend swapped during otherwise successful commands prevents a speed claim", () => {
  const dir = mkdtempSync(join(tmpdir(), "vize-pr-observer-seal-"));
  try {
    const input = join(dir, "input"),
      receipts = join(dir, "build");
    mkdirSync(input);
    mkdirSync(receipts);
    writeFileSync(join(input, "A.vue"), "<template><div /></template>\n");
    const backend = join(dir, "backend.mjs");
    writeFileSync(backend, `#!${process.execPath}\nconsole.log('backend 1');\n`, { mode: 0o755 });
    const sources = { base: "a".repeat(40), head: "b".repeat(40) };
    const binaries = { base: join(dir, "base.mjs"), head: join(dir, "head.mjs") };
    for (const side of ["base", "head"] as const) {
      writeFileSync(
        binaries[side],
        `#!${process.execPath}
import{writeFileSync}from'node:fs';
if(process.argv.includes('--version'))console.log('vize fake');
else writeFileSync(${JSON.stringify(backend)},'changed during measurement');
`,
        { mode: 0o755 },
      );
      writeFileSync(
        join(receipts, side + ".json"),
        JSON.stringify({
          schemaVersion: 1,
          role: side,
          sourceSha: sources[side],
          treeSha: "c".repeat(40),
          binary: binaries[side],
          binarySha256: sha256(binaries[side]),
          binaryBytes: statSync(binaries[side]).size,
          targetDir: join(dir, side + "-target"),
          targetStartedEmpty: true,
          cargoArtifact: {
            executable: join(dir, side + "-target", "ci-opt", "vize"),
            fresh: false,
            features: [],
          },
          allocation: false,
          profile: { name: "ci-opt" },
          rustc: "rustc test",
          cargo: { args: ["build"], version: "cargo test" },
          compilerEnvironment: {},
        }),
      );
    }
    const report = collectObservation({
      baseBin: binaries.base,
      headBin: binaries.head,
      baseSource: sources.base,
      headSource: sources.head,
      inputDir: input,
      out: join(dir, "out"),
      runs: 1,
      warmups: 0,
      thresholdPercent: 5,
      timeoutMs: 2000,
      tasks: "compile",
      backend,
      vueDir: null,
      buildReceipts: receipts,
    });
    assert.equal(report.identitiesUnchanged, false);
    assert.equal(report.results[0].comparison.result, null);
    assert.equal(report.results[0].comparison.availability, "unavailable");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
