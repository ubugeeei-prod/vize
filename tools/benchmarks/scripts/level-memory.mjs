#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { loadBudgets, loadRegistry, reconcile } from "./instruction-counts-lib.mjs";
import {
  MEMORY_ENVIRONMENT,
  NORMAL_ARGUMENTS,
  REPORT_DIRECTORY,
  sha256,
  validatePair,
} from "./level-memory-lib.mjs";
import {
  bindProbe,
  build,
  collectIdentities,
  collectNormal,
  preserveExistingReports,
  run,
  writeJson,
} from "./level-memory-runner.mjs";

const [baseArgument, headArgument, outArgument] = process.argv.slice(2);
assert.equal(process.argv.length, 5, "usage: level-memory.mjs BASE HEAD OUTPUT");
assert.equal(process.platform, "linux", "memory collection requires Linux");
assert.equal(process.arch, "x64", "memory collection requires x86_64");
assert.equal(process.env.GITHUB_ACTIONS, "true", "memory collection requires Actions");
assert.equal(
  process.env.MEMORY_CONFLICTING_MODES,
  "false",
  "memory mode cannot combine with other Criterion modes",
);
const roots = { base: fs.realpathSync(baseArgument), head: fs.realpathSync(headArgument) };
assert.notEqual(roots.base, roots.head, "base/head checkouts must be isolated");
const out = path.resolve(outArgument);
assert.ok(!fs.existsSync(out), "refusing to reuse output evidence");
fs.mkdirSync(out, { recursive: true });
const probes = path.join(path.dirname(out), "level-memory-probes");
assert.ok(!fs.existsSync(probes), "refusing to reuse probe directory");
fs.mkdirSync(probes);
const revisions = { base: process.env.MEMORY_BASE_SHA, head: process.env.MEMORY_HEAD_SHA };
for (const revision of Object.values(revisions))
  assert.match(revision ?? "", /^[a-f0-9]{40}$/, "exact revision required");
assert.notEqual(revisions.base, revisions.head, "baseline and candidate must differ");
assert.equal(revisions.head, process.env.GITHUB_SHA, "candidate must equal actual dispatch source");
const measurements = { base: [], head: [] };
const manifest = {
  schema_version: 1,
  source: revisions,
  recorded_run: `${process.env.GITHUB_SERVER_URL}/${process.env.GITHUB_REPOSITORY}/actions/runs/${process.env.GITHUB_RUN_ID}`,
  methodology: {
    profile: "ci-opt",
    target: "x86_64-unknown-linux-gnu",
    rust: "1.99.0",
    suites: 12,
    population: 104,
    repetitions: 3,
    arguments: NORMAL_ARGUMENTS,
    allocator: "existing-counting-mimalloc-normal-mode-reserve128m-purgeoff",
    environment: MEMORY_ENVIRONMENT,
    fixture_digest: "input-identity-v2",
    identities: "one-source-built-callgrind-pass-per-side;not-instruction-qualification",
    windows: "existing-routine-return-and-stage-return",
    instruction_preinit_bytes: 0,
    rss: "existing-process-baseline-growth;report-only-in-bench-compare",
    wall: "existing-budget-dependent-gate;quick-samples-not-a-speedup-claim",
  },
  measurements: {},
  gates: [],
  observations_complete: false,
  qualified: false,
};
writeJson(path.join(out, "measurement.json"), manifest);

function provenance(root, side) {
  const directory = path.join(out, side, "provenance");
  const head = run("git", ["rev-parse", "HEAD"], root, path.join(directory, "head")).stdout.trim();
  assert.equal(head, revisions[side], "checkout revision changed");
  const status = run(
    "git",
    ["status", "--porcelain", "--untracked-files=all"],
    root,
    path.join(directory, "status-before"),
  ).stdout;
  assert.equal(status, "", "source checkout must start clean");
  run("git", ["ls-tree", "-r", "--full-tree", "HEAD"], root, path.join(directory, "whole-tree"));
  preserveExistingReports(root, path.join(directory, "preexisting-reports"));
}

try {
  run(
    "git",
    ["merge-base", "--is-ancestor", revisions.base, revisions.head],
    roots.head,
    path.join(out, "ancestor"),
  );
  const registryFiles = Object.fromEntries(
    Object.entries(roots).map(([side, root]) => [
      side,
      path.join(root, "docs/davinci/plan/budgets.toml"),
    ]),
  );
  const registry = loadRegistry(registryFiles.head);
  assert.equal(registry.size, 104, "the entire original104 registry is mandatory");
  reconcile(loadRegistry(registryFiles.base), registry, "baseline registry");
  assert.deepEqual(
    fs.readFileSync(registryFiles.head),
    fs.readFileSync(registryFiles.base),
    "original allocation/peak/wall caps must remain byte exact",
  );
  const budgets = {};
  for (const [side, root] of Object.entries(roots)) {
    const file = path.join(root, "docs/davinci/plan/instruction-budgets.toml");
    budgets[side] = loadBudgets(file, registry);
    fs.copyFileSync(file, path.join(out, `${side}-instruction-budgets.toml`));
    fs.copyFileSync(registryFiles[side], path.join(out, `${side}-budgets.toml`));
    provenance(root, side);
  }
  assert.deepEqual(
    budgets.head.instruction,
    budgets.base.instruction,
    "original instruction identities and ceilings must remain unchanged",
  );
  const version = run(
    "valgrind",
    ["--version"],
    roots.head,
    path.join(out, "valgrind"),
  ).stdout.trim();
  assert.equal(version, "valgrind-3.22.0", "pinned Ubuntu24.04 Valgrind required");
  for (const [name, args] of [
    ["uname", ["-a"]],
    ["getconf", ["GNU_LIBC_VERSION"]],
    ["dpkg-query", ["-W", "valgrind", "libc6"]],
  ]) {
    run(name, args, roots.head, path.join(out, "environment", name));
  }
  fs.copyFileSync("/proc/cpuinfo", path.join(out, "cpuinfo.txt"));
  // Both source graphs build before timed windows, using separate Cargo targets.
  const binaries = {};
  for (const [side, root] of Object.entries(roots)) {
    const target = path.join(roots.head, "target", `level-memory-${side}`);
    binaries[side] = build(root, target, path.join(out, side, "build"));
  }
  const identities = { base: [], head: [] };
  for (const side of ["base", "head"]) {
    const whole = new Map();
    for (const [index, suite] of binaries[side].entries()) {
      const probe = bindProbe(suite, index, probes);
      const rows = collectIdentities(
        roots[side],
        suite,
        probe,
        path.join(out, side, "identity", String(index)),
        budgets[side],
      );
      identities[side].push(rows);
      for (const [id, row] of rows) {
        assert.ok(!whole.has(id), `duplicate suite identity ${id}`);
        whole.set(id, row);
      }
    }
    reconcile(new Set(whole.keys()), registry, `${side} original104 identity population`);
    writeJson(path.join(out, side, "whole-identities.json"), Object.fromEntries(whole));
  }
  // Adjacent base/head repetitions share the same probe paths, argv0 and env.
  // Collect all624 complete normal reports before any budget verdict.
  for (let repetition = 1; repetition <= 3; repetition += 1) {
    for (const side of ["base", "head"]) {
      const root = roots[side];
      const directory = path.join(out, side, `run-${repetition}`);
      const whole = new Map();
      fs.mkdirSync(path.join(directory, "reports"), { recursive: true });
      for (const [index, suite] of binaries[side].entries()) {
        const probe = bindProbe(suite, index, probes);
        const rows = collectNormal(
          root,
          suite,
          probe,
          path.join(directory, `suite-${index}`),
          identities[side][index],
        );
        for (const [id, row] of rows) {
          assert.ok(!whole.has(id), `duplicate suite report ${id}`);
          whole.set(id, row);
          fs.copyFileSync(row.file, path.join(directory, "reports", `${id}.json`));
          assert.equal(
            sha256(fs.readFileSync(path.join(directory, "reports", `${id}.json`))),
            row.raw_sha256,
          );
        }
      }
      reconcile(new Set(whole.keys()), registry, `${side} run${repetition} whole population`);
      measurements[side].push(whole);
      writeJson(path.join(directory, "whole-rows.json"), Object.fromEntries(whole));
    }
    validatePair(measurements.base[repetition - 1], measurements.head[repetition - 1]);
  }
  for (const side of ["base", "head"]) {
    for (const suite of binaries[side]) {
      assert.equal(
        sha256(fs.readFileSync(suite.executable)),
        suite.sha256,
        "binary changed after all executions",
      );
      assert.equal(
        sha256(fs.readFileSync(suite.source)),
        suite.source_sha256,
        "provider source changed after all executions",
      );
    }
    const changed = run(
      "git",
      ["diff", "--name-only", "HEAD"],
      roots[side],
      path.join(out, side, "source-after"),
    )
      .stdout.split("\n")
      .filter(Boolean);
    assert.ok(
      changed.every((file) => file.startsWith(`${REPORT_DIRECTORY}/`)),
      "build changed tracked source outside report output",
    );
    manifest.measurements[side] = measurements[side].map((rows) => Object.fromEntries(rows));
  }
  manifest.observations_complete = true;
  writeJson(path.join(out, "measurement.json"), manifest);
  for (let repetition = 1; repetition <= 3; repetition += 1) {
    const baseline = path.join(out, "base", `run-${repetition}`, "reports");
    for (const side of ["base", "head"]) {
      let accepted = false;
      try {
        run(
          "rust-script",
          [
            path.join(roots.head, "tools/commands/davinci/bench-compare.rs"),
            "--results",
            path.join(out, side, `run-${repetition}`, "reports"),
            "--baseline",
            baseline,
            "--budgets",
            registryFiles.head,
          ],
          roots.head,
          path.join(out, "gates", `${side}-${repetition}`),
          { ...process.env, RUSTUP_TOOLCHAIN: "1.99.0" },
        );
        accepted = true;
      } catch (error) {
        manifest.gates.push({ side, repetition, accepted, error: error.message });
        continue;
      }
      manifest.gates.push({ side, repetition, accepted });
    }
  }
  manifest.qualified = manifest.gates.length === 6 && manifest.gates.every((gate) => gate.accepted);
  writeJson(path.join(out, "measurement.json"), manifest);
  assert.ok(
    manifest.qualified,
    "original absolute comparison failed; all624 observations and six verdicts retained",
  );
  console.log(
    "level-memory: all104 original benchmarks, three complete paired normal-mode runs, existing gates passed",
  );
} catch (error) {
  manifest.failure = { name: error.name, message: error.message };
  writeJson(path.join(out, "measurement.json"), manifest);
  throw error;
}
