import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { basename, dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { performance } from "node:perf_hooks";

const suites = [
  ["fact_demand", "-p", "vize_l0", "--test", "fact_demand"],
  ["fact_spec", "-p", "vize_croquis", "--test", "fact_spec"],
  ["unused_setup_bindings", "-p", "vize_patina", "unused_setup_bindings"],
];
const detectors = [
  "a_producer_reading_outside_its_depends_is_refused_too",
  "an_undeclared_read_is_refused_exactly_even_when_computed",
];
const unitOutput = "target/unused-bindings-units";

// Cargo output hashes authenticate a bounded fingerprint directory. Unhashed
// final executables cannot be associated by crate name alone: report that gap.
export function collectUnitEvidence(output, target, context = {}) {
  mkdirSync(output, { recursive: true });
  const canonicalTarget = realpathSync(target);
  const inside = (path) => {
    const name = relative(canonicalTarget, realpathSync(path));
    return name !== ".." && !name.startsWith("../") && !isAbsolute(name);
  };
  const report = {
    status: "no compiler artifacts observed",
    context,
    logs: [],
    units: [],
    buildScripts: [],
    parseGaps: [],
  };
  for (const log of readdirSync(output)
    .filter((name) => name.endsWith(".jsonl"))
    .sort((a, b) => a.localeCompare(b))) {
    const lines = readFileSync(join(output, log), "utf8").split(/\r?\n/);
    report.logs.push(log);
    for (const [index, line] of lines.entries()) {
      let packet;
      try {
        packet = JSON.parse(line);
      } catch {
        if (line.trimStart().startsWith("{"))
          report.parseGaps.push({
            log,
            line: index + 1,
            reason: "incomplete JSON; raw line retained",
          });
        continue;
      } // libtest shares stdout.
      if (packet?.reason === "build-script-executed")
        report.buildScripts.push({ log, line: index + 1, packet });
      if (packet?.reason !== "compiler-artifact") continue;
      const unit = { log, line: index + 1, packet, fingerprints: [], gaps: [] };
      report.units.push(unit);
      const seen = new Set();
      for (const filename of packet.filenames) {
        if (!existsSync(filename) || !inside(filename)) {
          unit.gaps.push({ filename, reason: "output missing or outside target" });
          continue;
        }
        const parent = dirname(filename);
        const buildScript = basename(dirname(parent)) === "build";
        const hash = /-([a-f0-9]{16})(?:\.[^.]+)?$/.exec(
          buildScript ? basename(parent) : basename(filename),
        )?.[1];
        const profileRoot = buildScript ? dirname(dirname(parent)) : dirname(parent);
        if (!hash || (!buildScript && basename(parent) !== "deps")) {
          unit.gaps.push({ filename, reason: "no emitted unit hash; name matching refused" });
          continue;
        }
        const fingerprintRoot = join(profileRoot, ".fingerprint");
        if (seen.has(`${fingerprintRoot}/${hash}`)) continue;
        seen.add(`${fingerprintRoot}/${hash}`);
        if (!existsSync(fingerprintRoot) || !inside(fingerprintRoot)) {
          unit.gaps.push({ filename, hash, reason: "fingerprint root unavailable" });
          continue;
        }
        const matches = readdirSync(fingerprintRoot, { withFileTypes: true }).filter(
          (entry) => entry.isDirectory() && entry.name.endsWith(`-${hash}`),
        );
        if (matches.length !== 1) {
          unit.gaps.push({ filename, hash, reason: "missing or ambiguous fingerprint directory" });
          continue;
        }
        const directory = join(fingerprintRoot, matches[0].name);
        if (!inside(directory)) {
          unit.gaps.push({ filename, hash, reason: "fingerprint directory outside target" });
          continue;
        }
        const files = readdirSync(directory, { withFileTypes: true }).filter(
          (entry) => entry.isFile() && entry.name.endsWith(".json"),
        );
        if (files.length === 0)
          unit.gaps.push({ filename, hash, reason: "fingerprint JSON unavailable" });
        for (const file of files) {
          const source = join(directory, file.name);
          const sourcePath = relative(canonicalTarget, realpathSync(source));
          const name = sourcePath
            .split(sep)
            .filter((part) => part !== ".fingerprint")
            .join(sep);
          const destination = join(output, "fingerprints", name);
          mkdirSync(dirname(destination), { recursive: true });
          copyFileSync(source, destination);
          unit.fingerprints.push({
            filename,
            hash,
            source: sourcePath,
            path: join("fingerprints", name),
          });
        }
      }
    }
  }
  if (report.units.length > 0) report.status = "observed; inspect per-unit fingerprint gaps";
  writeFileSync(join(output, "units.json"), `${JSON.stringify(report, null, 2)}\n`);
  return report;
}

export function testResults(stdout) {
  const cases = [];
  const summaries = [];
  for (const line of stdout.split(/\r?\n/)) {
    const row = /^test (.+) \.\.\. (ok|ignored(?:, .*)?|FAILED)$/.exec(line);
    if (row) cases.push([row[1], row[2]]);
    const summary =
      /^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;/.exec(
        line,
      );
    if (summary) summaries.push(summary.slice(1));
  }
  assert.ok(summaries.length > 0, "Cargo produced no libtest summaries");
  assert.ok(
    cases.some((row) => row[1] === "ok"),
    "No selected passing tests",
  );
  const counts = summaries.reduce(
    (total, [status, passed, failed, ignored, measured]) => {
      assert.equal(status, "ok", "Failed libtest summary");
      assert.equal(Number(failed) + Number(measured), 0);
      total.passed += Number(passed);
      total.ignored += Number(ignored);
      return total;
    },
    { passed: 0, ignored: 0 },
  );
  assert.equal(cases.filter((row) => row[1] === "ok").length, counts.passed);
  assert.equal(cases.filter((row) => row[1].startsWith("ignored")).length, counts.ignored);
  assert.equal(cases.length, counts.passed + counts.ignored);
  const order = (left, right) => JSON.stringify(left).localeCompare(JSON.stringify(right));
  return { cases: cases.sort(order), summaries: summaries.sort(order) };
}

export function compareResults(before, after, suite) {
  assert.deepEqual(after, before, `${suite}: testcase/status or Cargo summary drift`);
  const required =
    suite === "fact_demand"
      ? detectors
      : suite === "fact_spec"
        ? ["the_corpus_shard_agrees_with_the_specs"]
        : [];
  for (const name of required)
    assert.ok(
      after.cases.some((row) => row[0] === name && row[1] === "ok"),
      `${name} must actually pass`,
    );
}

async function main() {
  const output = "target/unused-bindings-profiles";
  mkdirSync(output, { recursive: true });
  mkdirSync(unitOutput, { recursive: true });
  const commandOutput = (command, args) => execFileSync(command, args, { encoding: "utf8" }).trim();
  assert.ok(process.env.VIZE_DAVINCI_FACT_CORPUS, "The pinned corpus must be hydrated");
  const identity = () => ({
    source: commandOutput("git", ["rev-parse", "HEAD"]),
    rustc: commandOutput("rustc", ["-Vv"]),
    cargo: commandOutput("cargo", ["-V"]),
    corpus: process.env.VIZE_DAVINCI_FACT_CORPUS,
    submodules: commandOutput("git", [
      "submodule",
      "status",
      "--",
      ...process.env.VIZE_DAVINCI_FACT_CORPUS.split(","),
    ]),
    env: Object.fromEntries(
      [
        "RUSTUP_TOOLCHAIN",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTDOCFLAGS",
        "CARGO_BUILD_JOBS",
        "CARGO_TARGET_DIR",
      ].map((key) => [key, process.env[key] ?? null]),
    ),
  });
  const receipt = {
    before: identity(),
    order: "test first, ci second; one sample includes cache warming, not an all-CI claim",
    runs: [],
  };
  const save = () =>
    writeFileSync(`${output}/comparison.json`, `${JSON.stringify(receipt, null, 2)}\n`);
  save();
  assert.ok(
    receipt.before.submodules
      .split("\n")
      .every((line) => line.startsWith(" ") || /^[a-f0-9]/.test(line)),
    "Corpus submodules must be at their pinned commits",
  );
  if (process.env.GITHUB_SHA) assert.equal(receipt.before.source, process.env.GITHUB_SHA);
  for (const profile of ["test", "ci"]) {
    for (const [suite, ...selection] of suites) {
      const args = [
        "test",
        "--message-format=json-render-diagnostics",
        ...(profile === "ci" ? ["--profile", "ci"] : []),
        ...selection,
      ];
      const run = { suite, profile, command: ["cargo", ...args], exitCode: null, durationMs: null };
      receipt.runs.push(run);
      const started = performance.now();
      let stdout = "";
      let stderr = "";
      const child = spawn("cargo", args);
      child.stdout.setEncoding("utf8");
      child.stderr.setEncoding("utf8");
      child.stdout.on("data", (chunk) => {
        stdout += chunk;
        process.stdout.write(chunk);
      });
      child.stderr.on("data", (chunk) => {
        stderr += chunk;
        process.stderr.write(chunk);
      });
      run.exitCode = await new Promise((resolve) => {
        child.once("error", (error) => {
          stderr += String(error);
          resolve(-1);
        });
        child.once("close", resolve);
      });
      run.durationMs = performance.now() - started;
      writeFileSync(`${output}/${profile}-${suite}.stdout`, stdout);
      writeFileSync(`${output}/${profile}-${suite}.stderr`, stderr);
      writeFileSync(`${unitOutput}/${profile}-${suite}.jsonl`, stdout);
      save(); // Keep raw evidence before any result assertion can fail.
      assert.equal(run.exitCode, 0, `${run.command.join(" ")} failed`);
      run.results = testResults(stdout);
      save();
    }
  }
  receipt.after = identity();
  save();
  assert.deepEqual(receipt.after, receipt.before, "Source/compiler/corpus/environment drift");
  for (let index = 0; index < suites.length; index++)
    compareResults(
      receipt.runs[index].results,
      receipt.runs[index + suites.length].results,
      suites[index][0],
    );
  receipt.status = "passed";
  save();
}

if (import.meta.main) {
  if (process.argv[2] === "--collect-unit-evidence") {
    collectUnitEvidence(unitOutput, resolve("target"), {
      source: execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
      rustc: execFileSync("rustc", ["-Vv"], { encoding: "utf8" }).trim(),
      env: Object.fromEntries(
        [
          "RUSTUP_TOOLCHAIN",
          "RUSTFLAGS",
          "CARGO_ENCODED_RUSTFLAGS",
          "RUSTDOCFLAGS",
          "CARGO_BUILD_JOBS",
          "CARGO_TARGET_DIR",
        ].map((key) => [key, process.env[key] ?? null]),
      ),
    });
  } else await main();
}
