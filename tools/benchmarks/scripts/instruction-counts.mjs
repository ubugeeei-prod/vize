#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import {
  baselineToml,
  checkMeasurement,
  fixtureDigest,
  loadBudgets,
  loadRegistry,
  parseCallgrind,
  parseIdentities,
  ratchetBudgets,
  reconcile,
  validateMeasurement,
} from "./instruction-counts-lib.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const suites = [
  ["davinci_harness", "selfcheck"],
  ["vize_armature", "davinci"],
  ["vize_croquis", "davinci"],
  ["vize_atelier_core", "davinci"],
  ["vize_atelier_dom", "davinci"],
  ["vize_atelier_vapor", "davinci"],
  ["vize_atelier_ssr", "davinci"],
  ["vize_davinci", "davinci"],
  ["vize_davinci", "davinci_fact"],
  ["vize_l1_to_l2", "davinci_storage"],
  ["vize_patina", "davinci_markup"],
  ["vize_musea", "davinci_art"],
];

function command(executable, args, options = {}) {
  const child = spawnSync(executable, args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 128 * 1024 * 1024,
    env: process.env,
    ...options,
  });
  if (child.status !== 0) {
    throw new Error(
      `${executable} failed (${child.status ?? child.error}):\n${child.stderr ?? ""}`,
    );
  }
  return child.stdout.trim();
}

function flag(args, name, fallback) {
  const index = args.indexOf(name);
  if (index < 0) return fallback;
  assert.ok(args[index + 1] && !args[index + 1].startsWith("--"), `${name} requires a value`);
  return path.resolve(args[index + 1]);
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}
function writeJson(file, value) {
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
}

function collect(out, registry) {
  assert.equal(process.platform, "linux", "instruction collection requires Linux");
  assert.equal(process.arch, "x64", "instruction collection requires x86_64");
  assert.equal(process.env.GITHUB_ACTIONS, "true", "baseline collection requires GitHub Actions");
  assert.ok(!fs.existsSync(out), `refusing to reuse measurement output directory ${out}`);
  fs.mkdirSync(out, { recursive: true });
  const rustc = command("rustc", ["--version"]);
  assert.ok(rustc.startsWith("rustc 1.98.0 "), "use rust-toolchain.toml's pinned Rust 1.98.0");
  const valgrind = command("valgrind", ["--version"]);
  assert.equal(valgrind, "valgrind-3.22.0", "use the pinned Ubuntu 24.04 Valgrind version");
  const report = {
    schema_version: 1,
    source_commit: command("git", ["rev-parse", "HEAD"]),
    recorded_run: `${process.env.GITHUB_SERVER_URL}/${process.env.GITHUB_REPOSITORY}/actions/runs/${process.env.GITHUB_RUN_ID}`,
    methodology: {
      allocator: "counting-mimalloc",
      flags: "target-cpu=x86-64;instr-atstart=no;cache-sim=no;branch-sim=no",
      libc: command("getconf", ["GNU_LIBC_VERSION"]),
      profile: "ci-opt",
      rustc,
      target: "x86_64-unknown-linux-gnu",
      valgrind,
      window_protocol: "callgrind-client-v1",
    },
    runs: [],
  };
  writeJson(path.join(out, "environment.json"), {
    uname: command("uname", ["-a"]),
    cpu: fs.readFileSync("/proc/cpuinfo", "utf8"),
    dpkg: command("dpkg-query", ["-W", "valgrind", "libc6"]),
  });
  // One optimized Cargo build, then three fresh executions of every suite.
  // Never run Cargo or Criterion sampling inside Valgrind.
  const cargoArgs = [
    "bench",
    "--locked",
    "--no-run",
    "--profile",
    "ci-opt",
    "--features",
    "davinci_harness/instruction-counts",
    "--message-format=json",
  ];
  for (const pkg of new Set(suites.map(([pkg]) => pkg))) cargoArgs.push("-p", pkg);
  for (const bench of new Set(suites.map(([, bench]) => bench))) cargoArgs.push("--bench", bench);
  const buildEnv = { ...process.env, RUSTFLAGS: "-C target-cpu=x86-64" };
  const build = command("cargo", cargoArgs, { env: buildEnv });
  const artifacts = build
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line))
    .filter(
      (row) =>
        row.reason === "compiler-artifact" && row.executable && row.target.kind.includes("bench"),
    );
  const binaries = suites.map(([pkg, bench]) => {
    const directory = pkg === "davinci_harness" ? "tools/benchmarks/crates" : "crates";
    const source = path.join(root, directory, pkg, "benches", `${bench}.rs`);
    const matches = artifacts.filter((row) => path.resolve(row.target.src_path) === source);
    assert.equal(matches.length, 1, `missing or duplicate build artifact ${pkg}/${bench}`);
    return { pkg, bench, source, executable: matches[0].executable };
  });
  for (let run = 1; run <= 3; run += 1) {
    const rows = {};
    const runDir = path.join(out, `run-${run}`);
    fs.mkdirSync(runDir);
    for (const suite of binaries) {
      const prefix = path.join(runDir, `${suite.pkg}-${suite.bench}.callgrind`);
      const measured = spawnSync(
        "valgrind",
        [
          "--tool=callgrind",
          "--instr-atstart=no",
          "--cache-sim=no",
          "--branch-sim=no",
          `--callgrind-out-file=${prefix}`,
          "--error-exitcode=97",
          suite.executable,
        ],
        {
          cwd: root,
          encoding: "utf8",
          maxBuffer: 64 * 1024 * 1024,
          env: { ...process.env, VIZE_INSTRUCTION_COUNTS: "1" },
        },
      );
      fs.writeFileSync(`${prefix}.stderr`, measured.stderr ?? "");
      fs.writeFileSync(`${prefix}.stdout`, measured.stdout ?? "");
      assert.equal(
        measured.status,
        0,
        `Valgrind failed ${suite.pkg}/${suite.bench}: ${measured.stderr}`,
      );
      const identities = parseIdentities(measured.stderr);
      const dumps = new Map();
      for (const file of fs
        .readdirSync(runDir)
        .filter(
          (file) =>
            file === path.basename(prefix) ||
            (file.startsWith(`${path.basename(prefix)}.`) && /\.\d+$/.test(file)),
        )) {
        const dump = parseCallgrind(fs.readFileSync(path.join(runDir, file), "utf8"));
        if (!dump) continue;
        assert.ok(!dumps.has(dump.bench_id), `duplicate dump ${dump.bench_id}`);
        dumps.set(dump.bench_id, dump.instructions);
      }
      reconcile(
        new Set(dumps.keys()),
        new Set(identities.keys()),
        `${suite.pkg}/${suite.bench} dumps`,
      );
      for (const [id, identity] of identities) {
        assert.ok(!Object.hasOwn(rows, id), `duplicate benchmark ${id}`);
        rows[id] = {
          fixture: identity.fixture,
          fixture_sha256: fixtureDigest(root, identity.fixture, suite.source),
          instructions: dumps.get(id),
          window: identity.window,
        };
      }
      console.log(
        `instruction-counts: run=${run} suite=${suite.pkg}/${suite.bench} measured=${dumps.size}`,
      );
    }
    reconcile(new Set(Object.keys(rows)), registry, `run ${run}`);
    report.runs.push(rows);
    writeJson(path.join(out, `run-${run}.json`), rows);
  }
  // Publish evidence even when deterministic equality fails, for diagnosis.
  writeJson(path.join(out, "measurement.json"), report);
  validateMeasurement(report, registry);
  fs.writeFileSync(path.join(out, "measured-budgets.toml"), baselineToml(report));
  console.log(`instruction-counts: ${registry.size} benchmarks identical across three executions`);
}

try {
  const args = process.argv.slice(2);
  const modes = args.filter((arg) => ["--collect", "--check", "--baseline"].includes(arg));
  assert.equal(modes.length, 1, "select exactly one --collect/--check/--baseline");
  const allowed = new Set([
    "--collect",
    "--check",
    "--baseline",
    "--out",
    "--measurement",
    "--budgets",
    "--base-budgets",
    "--registry",
  ]);
  for (let index = 0; index < args.length; index += 1) {
    assert.ok(allowed.has(args[index]), `unknown option ${args[index]}`);
    if (!["--collect", "--check", "--baseline"].includes(args[index])) index += 1;
  }
  const registry = loadRegistry(
    flag(args, "--registry", path.join(root, "docs/davinci/plan/budgets.toml")),
  );
  if (modes[0] === "--collect") {
    collect(flag(args, "--out", path.join(root, "target/instruction-counts")), registry);
  } else {
    const report = validateMeasurement(
      readJson(
        flag(args, "--measurement", path.join(root, "target/instruction-counts/measurement.json")),
      ),
      registry,
    );
    if (modes[0] === "--baseline") {
      fs.writeFileSync(
        flag(args, "--out", path.join(root, "docs/davinci/plan/instruction-budgets.toml")),
        baselineToml(report),
      );
    } else {
      const budgets = loadBudgets(
        flag(args, "--budgets", path.join(root, "docs/davinci/plan/instruction-budgets.toml")),
        registry,
      );
      const base = flag(args, "--base-budgets");
      // The base can have fewer rows when a new benchmark is introduced.
      if (base) ratchetBudgets(budgets, loadBudgets(base));
      checkMeasurement(report, budgets);
      console.log(`instruction-counts: ${registry.size} ceilings hold`);
    }
  }
} catch (error) {
  console.error(`instruction-counts: ${error.message}`);
  process.exitCode = 1;
}
