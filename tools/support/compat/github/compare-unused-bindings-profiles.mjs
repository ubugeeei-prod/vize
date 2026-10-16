import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
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
      const args = ["test", ...(profile === "ci" ? ["--profile", "ci"] : []), ...selection];
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

if (import.meta.main) await main();
