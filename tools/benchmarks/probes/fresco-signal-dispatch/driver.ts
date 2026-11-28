#!/usr/bin/env node
// Source-only recipe: never run implicitly. Use four fresh release builds on
// one Actions runner, preserve all evidence, and reject any measured growth.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

import {
  ids,
  type BenchId,
  type BuildLabel,
  type SourceFile,
  type InstructionCounts,
  type InstructionLibrary,
  type MeasurementToolchainLibrary,
  type ProbeCompilerArtifact,
  type NmSymbol,
  type SignalOracle,
  type BuildMeasurement,
  type Delta,
  type Qualification,
  type MeasurementReport,
  type EnvironmentReceipt,
} from "./types.ts";
import { command, sha, json, profile, dependencyIdentity } from "./support.ts";

function expectedCommitSha(name: "SIGNAL_PROBE_BASE_SHA" | "SIGNAL_PROBE_HEAD_SHA"): string {
  const value = process.env[name];
  assert.ok(typeof value === "string", `${name} must be a full immutable commit SHA`);
  assert.match(value, /^[0-9a-f]{40}$/, `${name} must be a full immutable commit SHA`);
  return value;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isBenchId(value: string): value is BenchId {
  return ids.some((id) => id === value);
}

function assertCompleteCounts(
  value: Partial<InstructionCounts>,
): asserts value is InstructionCounts {
  for (const id of ids) assert.equal(typeof value[id], "number", `missing instruction count ${id}`);
}

function assertSignalOracle(value: unknown): asserts value is SignalOracle {
  assert.ok(isRecord(value), "signal oracle must be a JSON object");
  assert.equal(value.oracle, "fresco-public-signal-dispatch-v1");
  const numericFields = [
    "simple_calls",
    "simple_signal",
    "simple_entry_errno",
    "simple_exit_errno",
    "siginfo_calls",
    "siginfo_signal",
    "siginfo_entry_errno",
    "siginfo_exit_errno",
    "siginfo_has_information",
    "ignored_exit_errno",
  ] as const;
  for (const field of numericFields) {
    assert.equal(typeof value[field], "number", `signal oracle ${field} must be numeric`);
  }
  assert.equal(value.mask_contains_sigusr1, true);
  assert.equal(value.terminal_backend_active, false);
}

function isProbeCompilerArtifact(value: unknown): value is ProbeCompilerArtifact {
  if (!isRecord(value) || value.reason !== "compiler-artifact" || !value.executable) return false;
  assert.ok(isRecord(value.target), "compiler artifact target must be an object");
  if (value.target.name !== "fresco_signal_dispatch_probe") return false;
  assert.equal(typeof value.executable, "string", "probe executable must be a path");
  assert.equal(typeof value.fresh, "boolean", "probe artifact must report freshness");
  return true;
}

function parseNmSymbol(line: string): NmSymbol {
  const fields = line.trim().split(/\s+/);
  const address = fields[0];
  const size = fields[1];
  const kind = fields[2];
  assert.ok(address && size && kind && fields.length >= 4, "malformed saved-action nm row");
  assert.match(size, /^[0-9a-f]+$/i);
  return {
    raw: line,
    address,
    bytes: Number.parseInt(size, 16),
    kind,
    name: fields.slice(3).join(" "),
  };
}

function buildMeasurement(report: MeasurementReport, label: BuildLabel): BuildMeasurement {
  const measurement = report.builds[label];
  assert.ok(measurement, `missing completed build ${label}`);
  return measurement;
}

function firstRun(measurement: BuildMeasurement): InstructionCounts {
  const run = measurement.runs[0];
  assert.ok(run, "completed build has no instruction run");
  return run;
}

function comparisonRows(base: InstructionCounts, head: InstructionCounts): Record<BenchId, Delta> {
  const row = (id: BenchId): Delta => ({
    base: base[id],
    head: head[id],
    delta: head[id] - base[id],
  });
  return {
    "fresco.signal.install": row("fresco.signal.install"),
    "fresco.signal.dispatch.simple": row("fresco.signal.dispatch.simple"),
    "fresco.signal.dispatch.siginfo": row("fresco.signal.dispatch.siginfo"),
    "fresco.signal.dispatch.ignore": row("fresco.signal.dispatch.ignore"),
  };
}

let parameters = process.argv.slice(2);
if (parameters.length === 0) {
  const root = process.env.SIGNAL_PROBE_ROOT;
  assert.ok(root, "SIGNAL_PROBE_ROOT is required when no paths are supplied");
  parameters = [
    root,
    path.join(root, "signal-base"),
    path.join(root, "signal-head"),
    path.join(root, "signal-evidence"),
  ];
}
assert.equal(parameters.length, 4, "supply either all four paths or SIGNAL_PROBE_ROOT");
const [driverArg, baseArg, headArg, outArg] = parameters;
assert.ok(
  driverArg && baseArg && headArg && outArg,
  "usage: node driver.ts DRIVER_REPO BASE_REPO HEAD_REPO NEW_OUTPUT_DIRECTORY",
);
const expectedBaseSha = expectedCommitSha("SIGNAL_PROBE_BASE_SHA");
const expectedHeadSha = expectedCommitSha("SIGNAL_PROBE_HEAD_SHA");
const driverRepo = path.resolve(driverArg);
const baseRepo = path.resolve(baseArg);
const headRepo = path.resolve(headArg);
const out = path.resolve(outArg);
const authored = path.dirname(fileURLToPath(import.meta.url));
const driverCommit = command("git", ["rev-parse", "HEAD"], { cwd: driverRepo }).trim();
assert.equal(
  driverCommit,
  process.env.GITHUB_SHA,
  "driver checkout does not match the Actions source",
);
const shared: InstructionLibrary = await import(
  pathToFileURL(path.join(driverRepo, "tools/benchmarks/scripts/instruction-counts-lib.mjs")).href
);
const toolchainLibrary: MeasurementToolchainLibrary = await import(
  pathToFileURL(path.join(driverRepo, "tools/benchmarks/scripts/measurement-toolchain.ts")).href
);
const { measurementToolchain } = toolchainLibrary;
const {
  GUEST_ENVIRONMENT,
  BUILD_SOURCE_DATE_EPOCH,
  LIBC_DISPATCH,
  parseCallgrind,
  parseIdentities,
  validateAllocatorSetup,
} = shared;
assert.equal(process.platform, "linux");
assert.equal(process.arch, "x64");
assert.equal(process.env.GITHUB_ACTIONS, "true", "run on Actions");
assert.ok(!fs.existsSync(out), "output directory must be fresh");
fs.mkdirSync(out, { recursive: true });
const toolchain = measurementToolchain(driverRepo);
const rustc = command("rustc", [`+${toolchain}`, "--version"]).trim();
assert.ok(rustc.startsWith(`rustc ${toolchain} `));
assert.equal(command("valgrind", ["--version"]).trim(), "valgrind-3.22.0");
const sourceDigests: Record<SourceFile, string> = {
  "src/main.rs": sha(path.join(authored, "src/main.rs")),
  "src/instruction.rs": sha(path.join(authored, "src/instruction.rs")),
  "Cargo.toml.in": sha(path.join(authored, "Cargo.toml.in")),
  "driver.ts": sha(path.join(authored, "driver.ts")),
  "types.ts": sha(path.join(authored, "types.ts")),
  "support.ts": sha(path.join(authored, "support.ts")),
};
assert.ok(
  !out.startsWith(`${authored}${path.sep}`),
  "output cannot be inside the authored source bundle",
);
fs.cpSync(authored, path.join(out, "authored"), { recursive: true });
const markerPath = "tools/benchmarks/crates/davinci_harness/src/instruction.rs";
for (const repo of [driverRepo, baseRepo, headRepo]) {
  assert.equal(
    sha(path.join(repo, markerPath)),
    sourceDigests["src/instruction.rs"],
    "client marker source differs from frozen authored source",
  );
  assert.equal(measurementToolchain(repo), toolchain);
  assert.equal(
    command("git", ["status", "--porcelain", "--untracked-files=no"], { cwd: repo }).trim(),
    "",
    "tracked source changes prevent commit attribution",
  );
  for (const name of ["release", "ci-opt"] as const) {
    assert.equal(
      profile(repo, name),
      profile(driverRepo, name),
      "release profile differs across pair",
    );
  }
}
assert.equal(
  command("git", ["rev-parse", "HEAD"], { cwd: baseRepo }).trim(),
  expectedBaseSha,
  "base checkout HEAD does not match the immutable expected base SHA",
);
assert.equal(
  command("git", ["rev-parse", "HEAD"], { cwd: headRepo }).trim(),
  expectedHeadSha,
  "head checkout HEAD does not match the immutable expected head SHA",
);
const environment: NodeJS.ProcessEnv = {
  ...process.env,
  RUSTFLAGS: "-C target-cpu=x86-64",
  SOURCE_DATE_EPOCH: BUILD_SOURCE_DATE_EPOCH,
};
const report: MeasurementReport = {
  schema_version: 1,
  supplemental_probe: "fresco-public-signal-dispatch-v1",
  source_digests: sourceDigests,
  driver_commit: driverCommit,
  base_commit: command("git", ["rev-parse", "HEAD"], { cwd: baseRepo }).trim(),
  head_commit: command("git", ["rev-parse", "HEAD"], { cwd: headRepo }).trim(),
  expected_base_commit: expectedBaseSha,
  expected_head_commit: expectedHeadSha,
  methodology: {
    rustc,
    profile: "release",
    target: "x86_64-unknown-linux-gnu",
    flags: "target-cpu=x86-64;instr-atstart=no;cache-sim=no;branch-sim=no",
    window_protocol: "callgrind-client-call-boundary-v1",
    guest_environment: GUEST_ENVIRONMENT,
    libc_dispatch: LIBC_DISPATCH,
    source_date_epoch: BUILD_SOURCE_DATE_EPOCH,
    valgrind: "valgrind-3.22.0",
    libc: command("getconf", ["GNU_LIBC_VERSION"]).trim(),
  },
  scope: {
    terminal_backend_active: false,
    retry_injection: false,
    nesting_injection: false,
    default_disposition: "native-separate-process-correctness-only",
    linked_context: "standalone-public-api-client-built-from-actual-product-source",
  },
  builds: {},
};
const environmentReceipt: EnvironmentReceipt = {
  cpuinfo: fs.readFileSync("/proc/cpuinfo", "utf8"),
  uname: command("uname", ["-a"]).trim(),
  dpkg: command("dpkg-query", ["-W", "valgrind", "libc6"]),
  run_id: process.env.GITHUB_RUN_ID,
  run_attempt: process.env.GITHUB_RUN_ATTEMPT,
};
json(path.join(out, "environment.json"), environmentReceipt);
fs.mkdirSync(path.join(out, "guest"));
const guest = path.join(out, "guest", "probe");
const buildSlot = path.join(out, "build-slot");
const pkg = path.join(buildSlot, "package");
const target = path.join(buildSlot, "target");
let expectedOracle: SignalOracle | undefined;
const buildPlan: ReadonlyArray<readonly [BuildLabel, string]> = [
  ["base-a", baseRepo],
  ["head-a", headRepo],
  ["base-b", baseRepo],
  ["head-b", headRepo],
];
try {
  for (const [label, repo] of buildPlan) {
    const dir = path.join(out, label);
    fs.mkdirSync(dir);
    // Every build uses the exact same package and target paths. Empty both
    // before resolving/building, so the independent A/A twins cannot reuse
    // artifacts or differ merely because their authored source path changed.
    fs.rmSync(pkg, { recursive: true, force: true });
    fs.rmSync(target, { recursive: true, force: true });
    assert.ok(!fs.existsSync(target), "fixed build target was not fully cleared");
    fs.mkdirSync(path.join(pkg, "src"), { recursive: true });
    for (const file of ["src/main.rs", "src/instruction.rs"] as const) {
      fs.copyFileSync(path.join(authored, file), path.join(pkg, file));
      assert.equal(sha(path.join(pkg, file)), sourceDigests[file]);
    }
    let manifest = fs.readFileSync(path.join(authored, "Cargo.toml.in"), "utf8");
    assert.ok(!repo.includes('"') && !repo.includes("\\"), "unsupported source path quoting");
    manifest = manifest.replaceAll("@REPO@", repo);
    manifest += `\n${profile(repo, "release")}\n\n${profile(repo, "ci-opt")}\n`;
    fs.writeFileSync(path.join(pkg, "Cargo.toml"), manifest);
    fs.copyFileSync(path.join(repo, "Cargo.lock"), path.join(pkg, "Cargo.lock"));
    const buildEnv: NodeJS.ProcessEnv = { ...environment, CARGO_TARGET_DIR: target };
    const metadata = command("cargo", [`+${toolchain}`, "metadata", "--format-version=1"], {
      cwd: pkg,
      env: buildEnv,
    });
    fs.writeFileSync(path.join(dir, "metadata.json"), metadata);
    const resolvedDependencyIdentity = dependencyIdentity(
      path.join(repo, "Cargo.lock"),
      path.join(pkg, "Cargo.lock"),
    );
    fs.cpSync(pkg, path.join(dir, "package"), { recursive: true });
    const build = command(
      "cargo",
      [`+${toolchain}`, "build", "--locked", "--release", "--message-format=json"],
      { cwd: pkg, env: buildEnv },
    );
    fs.writeFileSync(path.join(dir, "build.jsonl"), build);
    assert.equal(
      command("git", ["status", "--porcelain", "--untracked-files=no"], { cwd: repo }).trim(),
      "",
      "building the public client changed tracked product source",
    );
    const messages: unknown[] = build
      .trim()
      .split("\n")
      .filter(Boolean)
      .map((line): unknown => JSON.parse(line));
    const binaries = messages.filter(isProbeCompilerArtifact);
    assert.equal(binaries.length, 1);
    const artifact = binaries[0];
    assert.ok(artifact, "the probe compiler artifact is missing");
    assert.equal(artifact.fresh, false, "independent binary build reused");
    const builtBinary = artifact.executable;
    const binary = path.join(dir, "fresco_signal_dispatch_probe");
    fs.copyFileSync(builtBinary, binary);
    fs.chmodSync(binary, 0o755);
    assert.equal(sha(binary), sha(builtBinary), "retained ELF differs from actual Cargo output");
    if (fs.existsSync(guest)) fs.unlinkSync(guest);
    fs.symlinkSync(binary, guest);
    const nm = command("nm", ["-S", "--size-sort", "--demangle", binary]);
    fs.writeFileSync(path.join(dir, "symbols.txt"), nm);
    fs.writeFileSync(path.join(dir, "sections.txt"), command("size", ["-A", binary]));
    fs.writeFileSync(path.join(dir, "elf-symbols.txt"), command("readelf", ["-sW", binary]));
    const slots: NmSymbol[] = nm
      .split("\n")
      .filter((line) => line.includes("PREVIOUS_ACTIONS"))
      .map(parseNmSymbol);
    assert.ok(slots.length > 0, "saved-action storage symbols missing; cannot qualify its storage");
    const savedActionStaticBytes = slots.reduce((total, symbol) => total + symbol.bytes, 0);
    const nativeDefault = spawnSync(guest, ["--default-sighup"], {
      encoding: "utf8",
      cwd: driverRepo,
      env: GUEST_ENVIRONMENT,
    });
    fs.writeFileSync(path.join(dir, "default.stderr"), nativeDefault.stderr ?? "");
    fs.writeFileSync(path.join(dir, "default.stdout"), nativeDefault.stdout ?? "");
    assert.equal(
      nativeDefault.signal,
      "SIGHUP",
      "default action did not terminate with native SIGHUP",
    );
    assert.match(nativeDefault.stderr, /FRESCO_SIGNAL_DEFAULT_READY/);
    const runs: InstructionCounts[] = [];
    for (let run = 1; run <= 3; run++) {
      const runDir = path.join(dir, `run-${run}`);
      fs.mkdirSync(runDir);
      const prefix = path.join(runDir, "probe.callgrind");
      const measured = spawnSync(
        "valgrind",
        [
          "--tool=callgrind",
          "--instr-atstart=no",
          "--cache-sim=no",
          "--branch-sim=no",
          `--callgrind-out-file=${prefix}`,
          "--error-exitcode=97",
          guest,
        ],
        { cwd: driverRepo, env: GUEST_ENVIRONMENT, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
      );
      fs.writeFileSync(path.join(runDir, "stderr.txt"), measured.stderr ?? "");
      fs.writeFileSync(path.join(runDir, "stdout.txt"), measured.stdout ?? "");
      assert.equal(measured.status, 0, measured.stderr);
      validateAllocatorSetup(measured.stderr);
      const oracle: unknown = JSON.parse(measured.stdout.trim());
      assertSignalOracle(oracle);
      expectedOracle ??= oracle;
      assert.deepEqual(
        oracle,
        expectedOracle,
        "behavior oracle differs across builds or executions",
      );
      const identities = parseIdentities(measured.stderr);
      assert.deepEqual([...identities.keys()].sort(), [...ids].sort());
      const counts: Partial<InstructionCounts> = {};
      for (const file of fs
        .readdirSync(runDir)
        .filter((name) => /^probe\.callgrind(?:\.\d+)?$/.test(name))) {
        const row = parseCallgrind(fs.readFileSync(path.join(runDir, file), "utf8"));
        if (!row) continue;
        assert.ok(isBenchId(row.bench_id), `unexpected Callgrind window ${row.bench_id}`);
        assert.ok(!Object.hasOwn(counts, row.bench_id), "duplicate Callgrind window");
        counts[row.bench_id] = row.instructions;
      }
      assert.deepEqual(Object.keys(counts).sort(), [...ids].sort());
      assertCompleteCounts(counts);
      if (runs.length) assert.deepEqual(counts, runs[0], "fresh executions are not deterministic");
      runs.push(counts);
    }
    assert.ok(expectedOracle, "the completed build produced no behavior oracle");
    report.builds[label] = {
      source_commit: command("git", ["rev-parse", "HEAD"], { cwd: repo }).trim(),
      binary_sha256: sha(binary),
      binary,
      dependency_identity: resolvedDependencyIdentity,
      saved_action_static_bytes: savedActionStaticBytes,
      saved_action_symbols: slots.map((symbol) => symbol.raw),
      default_oracle: "terminated-by-SIGHUP",
      oracle: expectedOracle,
      runs,
    };
    json(path.join(out, "measurement.json"), report);
  }
  for (const side of ["base", "head"] as const) {
    const first = buildMeasurement(report, `${side}-a`);
    const second = buildMeasurement(report, `${side}-b`);
    assert.equal(
      first.binary_sha256,
      second.binary_sha256,
      `${side} independently rebuilt A/A ELF bytes differ`,
    );
    assert.deepEqual(
      firstRun(first),
      firstRun(second),
      `${side} independently rebuilt A/A instruction counts differ`,
    );
    assert.equal(first.saved_action_static_bytes, second.saved_action_static_bytes);
  }
  const base = buildMeasurement(report, "base-a");
  const head = buildMeasurement(report, "head-a");
  const comparison = comparisonRows(firstRun(base), firstRun(head));
  report.comparison = comparison;
  const staticBytes: Delta = {
    base: base.saved_action_static_bytes,
    head: head.saved_action_static_bytes,
    delta: head.saved_action_static_bytes - base.saved_action_static_bytes,
  };
  report.static_saved_action_bytes = staticBytes;
  const qualification: Qualification = {
    instruction_nonregression: Object.values(comparison).every((row) => row.delta <= 0),
    saved_action_static_storage_nonregression: staticBytes.delta <= 0,
    no_functional_difference_observed: true,
    caveat:
      "supplemental rows; existing shipped-corpus and native terminal/Miri checks still required",
  };
  report.qualification = qualification;
  json(path.join(out, "measurement.json"), report);
  console.log(
    JSON.stringify(
      {
        comparison: report.comparison,
        static_saved_action_bytes: report.static_saved_action_bytes,
        qualification: report.qualification,
      },
      null,
      2,
    ),
  );
  if (
    !qualification.instruction_nonregression ||
    !qualification.saved_action_static_storage_nonregression
  )
    process.exitCode = 1;
} catch (error) {
  report.error = error instanceof Error ? (error.stack ?? String(error)) : String(error);
  json(path.join(out, "measurement.json"), report);
  throw error;
}
