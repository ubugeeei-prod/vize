#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { parseTomlLite } from "../../support/compat/davinci/toml-lite.mjs";
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
  validateAllocatorSetup,
  ALLOCATOR_PROTOCOL,
  LIBC_DISPATCH,
  GUEST_ENVIRONMENT,
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
  ["vize_l1_to_l2", "l1_to_l2_storage"],
  ["vize_patina", "davinci_markup"],
  ["vize_musea", "davinci_art"],
].map(([pkg, bench]) => {
  // Target rename is bijective: probe ids, exact input bytes, windows and caps
  // stay fixed. Support either side until the move-only rename merges.
  if (pkg !== "vize_l1_to_l2") return [pkg, bench];
  const matches = [bench, "davinci_storage"].filter((name) =>
    fs.existsSync(path.join(root, "crates", pkg, "benches", `${name}.rs`)),
  );
  assert.equal(matches.length, 1, "level storage target rename must be bijective");
  return [pkg, matches[0]];
});

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
  const toolchain = parseTomlLite(fs.readFileSync(path.join(root, "rust-toolchain.toml"), "utf8"))
    .toolchain.channel;
  assert.ok(
    /^\d+\.\d+\.\d+$/.test(toolchain),
    "repository Rust toolchain must be an exact version",
  );
  const rustc = command("rustc", [`+${toolchain}`, "--version"]);
  assert.ok(
    rustc.startsWith(`rustc ${toolchain} `),
    "measurement must use the repository's pinned Rust",
  );
  const valgrind = command("valgrind", ["--version"]);
  assert.equal(valgrind, "valgrind-3.22.0", "use the pinned Ubuntu 24.04 Valgrind version");
  const report = {
    schema_version: 1,
    source_commit: command("git", ["rev-parse", "HEAD"]),
    recorded_run: `${process.env.GITHUB_SERVER_URL}/${process.env.GITHUB_REPOSITORY}/actions/runs/${process.env.GITHUB_RUN_ID}`,
    methodology: {
      allocator: ALLOCATOR_PROTOCOL,
      flags: "target-cpu=x86-64;instr-atstart=no;cache-sim=no;branch-sim=no",
      fixture_digest: "input-identity-v2",
      guest_context: "fixed-env-fixed-argv0-v1",
      libc: command("getconf", ["GNU_LIBC_VERSION"]),
      libc_dispatch: LIBC_DISPATCH,
      profile: "ci-opt",
      rustc,
      target: "x86_64-unknown-linux-gnu",
      valgrind,
      window_protocol: "callgrind-client-call-boundary-v1",
    },
    runs: [],
  };
  writeJson(path.join(out, "environment.json"), {
    run_attempt: Number(process.env.GITHUB_RUN_ATTEMPT),
    workflow: process.env.GITHUB_WORKFLOW,
    job: process.env.GITHUB_JOB,
    guest_environment: GUEST_ENVIRONMENT,
    uname: command("uname", ["-a"]),
    cpu: fs.readFileSync("/proc/cpuinfo", "utf8"),
    dpkg: command("dpkg-query", ["-W", "valgrind", "libc6"]),
  });
  // One optimized Cargo build, then three fresh executions of every suite.
  // Never run Cargo or Criterion sampling inside Valgrind.
  const cargoArgs = [
    `+${toolchain}`,
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
  const freshBuild = process.env.VIZE_INSTRUCTION_REBUILD === "1";
  if (freshBuild) {
    // Bootstrap cross-job proof must not reuse the same benchmark binaries.
    // Clear only the harness package; Cargo rebuilds its benchmark dependents
    // while retaining unrelated optimized dependencies in the sticky cache.
    command("cargo", [`+${toolchain}`, "clean", "--profile", "ci-opt", "-p", "davinci_harness"]);
  }
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
    const artifact = matches[0];
    if (freshBuild)
      assert.equal(artifact.fresh, false, `${pkg}/${bench}: benchmark binary was reused`);
    return { pkg, bench, source, executable: artifact.executable, fresh: artifact.fresh };
  });
  writeJson(
    path.join(out, "binaries.json"),
    binaries.map(({ pkg, bench, executable, fresh }) => ({
      pkg,
      bench,
      fresh,
      sha256: command("sha256sum", [executable]).split(" ")[0],
    })),
  );
  // Variable CI environment lengths change guest stack addresses. libc copy
  // routines branch on address aliasing even with the same binary and bytes.
  // Keep both environment and argv0 fixed, including across target renames.
  // Links stay outside the uploaded artifact so binaries do not bloat it.
  const probeDir = path.join(path.dirname(out), "instruction-probes");
  assert.ok(!fs.existsSync(probeDir), `refusing to reuse guest executable directory ${probeDir}`);
  fs.mkdirSync(probeDir);
  const probes = binaries.map((suite, index) => {
    const executable = path.join(probeDir, `probe-${String(index).padStart(2, "0")}`);
    fs.symlinkSync(suite.executable, executable);
    return { ...suite, executable };
  });
  for (let run = 1; run <= 3; run += 1) {
    const rows = {};
    const runDir = path.join(out, `run-${run}`);
    fs.mkdirSync(runDir);
    for (const suite of probes) {
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
          env: GUEST_ENVIRONMENT,
        },
      );
      fs.writeFileSync(`${prefix}.stderr`, measured.stderr ?? "");
      fs.writeFileSync(`${prefix}.stdout`, measured.stdout ?? "");
      assert.equal(
        measured.status,
        0,
        `Valgrind failed ${suite.pkg}/${suite.bench}: ${measured.stderr}`,
      );
      validateAllocatorSetup(measured.stderr);
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
  const modes = args.filter((arg) =>
    ["--collect", "--check", "--baseline", "--verify-budgets"].includes(arg),
  );
  assert.equal(modes.length, 1, "select exactly one --collect/--check/--baseline/--verify-budgets");
  const allowed = new Set([
    "--collect",
    "--check",
    "--baseline",
    "--verify-budgets",
    "--out",
    "--measurement",
    "--budgets",
    "--base-budgets",
    "--registry",
  ]);
  for (let index = 0; index < args.length; index += 1) {
    assert.ok(allowed.has(args[index]), `unknown option ${args[index]}`);
    if (!["--collect", "--check", "--baseline", "--verify-budgets"].includes(args[index]))
      index += 1;
  }
  const registry = loadRegistry(
    flag(args, "--registry", path.join(root, "docs/davinci/plan/budgets.toml")),
  );
  if (modes[0] === "--collect") {
    collect(flag(args, "--out", path.join(root, "target/instruction-counts")), registry);
  } else if (modes[0] === "--verify-budgets") {
    const budgets = loadBudgets(
      flag(args, "--budgets", path.join(root, "docs/davinci/plan/instruction-budgets.toml")),
      registry,
    );
    const base = flag(args, "--base-budgets");
    if (base) ratchetBudgets(budgets, loadBudgets(base));
    console.log(`instruction-counts: ${registry.size} pinned ceilings and ratchet verified`);
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
