/** Optimized, unprofiled source-operation timing; never a shipping CLI claim. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { parseArgs } from "node:util";
import {
  normalProbeFixtureFiles,
  normalProbeGroups,
  normalProbeSizes,
} from "./cli-probe-history-normal-fixture.ts";

const { values } = parseArgs({
  options: {
    out: { type: "string" },
    target: { type: "string" },
    fixture: { type: "string" },
  },
});
assert.ok(values.out && values.target && values.fixture);
const source = fs.realpathSync(process.cwd()),
  out = path.resolve(values.out),
  target = path.resolve(values.target),
  fixture = path.resolve(values.fixture);
for (const value of [out, target, fixture]) {
  assert.ok(!fs.existsSync(value), "task-owned paths must start absent");
  const relative = path.relative(source, value);
  assert.ok(relative === ".." || relative.startsWith(".." + path.sep) || path.isAbsolute(relative));
}
fs.mkdirSync(out, { recursive: true });
const hash = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
const run = (command: string, args: string[], env: Record<string, string> = {}) =>
  spawnSync(command, args, {
    cwd: source,
    env: { ...process.env, ...env, NO_COLOR: "1" },
    encoding: "utf8",
    timeout: 1_800_000,
    maxBuffer: 128 * 1024 * 1024,
    killSignal: "SIGKILL",
  });
const checked = (command: string, args: string[], env: Record<string, string> = {}) => {
  const r = run(command, args, env);
  assert.equal(r.error, undefined);
  assert.equal(r.signal, null);
  assert.equal(r.status, 0, r.stderr);
  return r.stdout.trim();
};
const git = (...args: string[]) => checked("git", args);
const sourceSha = git("rev-parse", "HEAD"),
  sourceTree = git("rev-parse", "HEAD^{tree}");
assert.equal(git("status", "--porcelain", "--untracked-files=all"), "");
const env = {
  CARGO_TARGET_DIR: target,
  CARGO_BUILD_JOBS: "4",
  CARGO_INCREMENTAL: "0",
  VIZE_PROBE_NORMAL_SOURCE_SHA: sourceSha,
};
const args = [
  "test",
  "--locked",
  "--profile",
  "ci-opt",
  "-p",
  "vize",
  "--lib",
  "--no-run",
  "--message-format=json-render-diagnostics",
  "--jobs",
  "4",
  "--config",
  "profile.ci-opt.opt-level=3",
  "--config",
  'profile.ci-opt.lto="thin"',
  "--config",
  "profile.ci-opt.codegen-units=16",
  "--config",
  "profile.ci-opt.debug=0",
  "--config",
  "profile.ci-opt.debug-assertions=false",
  "--config",
  "profile.ci-opt.overflow-checks=false",
  "--config",
  'profile.ci-opt.package.vize.strip="none"',
];
const safe =
  /^(?:RUSTFLAGS|CARGO_ENCODED_RUSTFLAGS|RUSTC|RUSTC_WRAPPER|RUSTC_WORKSPACE_WRAPPER|RUSTUP_TOOLCHAIN|CARGO_INCREMENTAL|CARGO_BUILD_JOBS|CARGO_TARGET_DIR|CARGO_BUILD_RUSTFLAGS|CARGO_BUILD_RUSTC|CARGO_BUILD_RUSTC_WRAPPER|VIZE_PROBE_NORMAL_SOURCE_SHA|CARGO_TARGET_[A-Z0-9_]+_(?:RUSTFLAGS|LINKER)|CARGO_PROFILE_[A-Z0-9_]+_(?:OPT_LEVEL|DEBUG|STRIP|LTO|CODEGEN_UNITS|PANIC|DEBUG_ASSERTIONS|OVERFLOW_CHECKS|INCREMENTAL))$/;
const request = {
  sourceSha,
  sourceTree,
  args,
  targetStartedAbsent: true,
  normalAllocator: "mimalloc::MiMalloc",
  host: {
    platform: process.platform,
    arch: process.arch,
    model: os.cpus()[0]?.model ?? null,
    availableParallelism: os.availableParallelism(),
    affinity: checked("taskset", ["-pc", String(process.pid)]),
    configuredRayonThreads: process.env.RAYON_NUM_THREADS ?? null,
    backendProcesses: 0,
  },
  sourceCleanBefore: true,
  compiler: { cargo: checked("cargo", ["-Vv"]), rustc: checked("rustc", ["-Vv"]) },
  environment: Object.fromEntries(
    Object.entries({ ...process.env, ...env }).filter(([key]) => safe.test(key)),
  ),
  files: Object.fromEntries(
    [
      "Cargo.toml",
      "Cargo.lock",
      ".cargo/config",
      ".cargo/config.toml",
      "crates/vize/Cargo.toml",
      "crates/vize/src/lib.rs",
      "crates/vize/src/main.rs",
      "crates/vize/src/commands/check/imports/resolution.rs",
      "crates/vize/src/commands/check/imports/registration/reachability.rs",
      "crates/vize/src/commands/check/imports/registration/reachability/tests/normal.rs",
    ]
      .filter((file) => fs.existsSync(file))
      .map((file) => [file, hash(fs.readFileSync(file))]),
  ),
};
for (const config of [".cargo/config", ".cargo/config.toml"])
  if (fs.existsSync(config)) git("ls-files", "--error-unmatch", "--", config);
const write = (name: string, value: unknown) =>
  fs.writeFileSync(path.join(out, name), JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
write("build-request.json", request);
const built = run("cargo", args, env);
fs.writeFileSync(path.join(out, "cargo.stdout.jsonl"), built.stdout ?? "");
fs.writeFileSync(path.join(out, "cargo.stderr.txt"), built.stderr ?? "");
assert.equal(built.error, undefined);
assert.equal(built.signal, null);
assert.equal(built.status, 0, built.stderr);
interface Artifact {
  reason: string;
  target: { name: string; kind: string[] };
  profile: {
    test: boolean;
    opt_level: string;
    debug_assertions: boolean;
    overflow_checks: boolean;
  };
  executable?: string;
  fresh: boolean;
  features: string[];
}
const artifacts = (built.stdout ?? "").split("\n").flatMap((line) => {
  try {
    const row = JSON.parse(line) as Artifact;
    return row.reason === "compiler-artifact" &&
      row.target.name === "vize" &&
      row.target.kind.includes("lib") &&
      row.profile.test &&
      row.executable
      ? [row]
      : [];
  } catch {
    return [];
  }
});
assert.equal(artifacts.length, 1);
const artifact = artifacts[0];
assert.equal(artifact.fresh, false);
assert.equal(artifact.profile.opt_level, "3");
assert.equal(artifact.profile.debug_assertions, false);
assert.equal(artifact.profile.overflow_checks, false);
assert.deepEqual(artifact.features.toSorted(), ["default", "glyph", "maestro"]);
assert.ok(artifact.executable);
const executable = fs.realpathSync(artifact.executable),
  relative = path.relative(fs.realpathSync(target), executable);
assert.ok(
  relative !== "" &&
    relative !== ".." &&
    !relative.startsWith(".." + path.sep) &&
    !path.isAbsolute(relative),
);
const binary = {
  path: executable,
  sha256: hash(fs.readFileSync(executable)),
  bytes: fs.statSync(executable).size,
};
write("build.json", {
  ...request,
  artifact,
  binary,
  cargoStdoutSha256: hash(built.stdout),
  cargoStderrSha256: hash(built.stderr),
});
const sizes = normalProbeSizes,
  groups = normalProbeGroups;
const files = normalProbeFixtureFiles(fixture);
for (const [file, text] of Object.entries(files)) {
  fs.mkdirSync(path.dirname(path.join(fixture, file)), { recursive: true });
  fs.writeFileSync(path.join(fixture, file), text, { flag: "wx" });
}
const physical = fs.realpathSync(fixture);
const seal = () =>
  Object.fromEntries(
    Object.keys(files)
      .sort()
      .map((file) => [
        file,
        {
          bytes: fs.statSync(path.join(physical, file)).size,
          sha256: hash(fs.readFileSync(path.join(physical, file))),
        },
      ]),
  );
const fixtureSeal = seal();
write("fixture-seal.json", { root: physical, files: fixtureSeal });
const prefix = "commands::check::imports::registration::reachability::tests::";
const names = [
  "untracked_probes_keep_typescript_precedence_and_omit_candidate_histories",
  "untracked_path_mode_retains_complete_alias_probe_histories",
  "untracked_graphs_keep_exact_work_cycles_and_fresh_missing_to_created_inputs",
  "tracked_graph_control_still_retains_missing_candidates_and_real_sources",
  "cross_program_alias_agreement_keeps_equal_winners_with_distinct_proof_paths_apart",
];
const carrier = prefix + "normal::normal_local_probe_histories";
const list = checked(executable, ["--list"]).split("\n");
for (const name of [...names.map((name) => prefix + name), carrier])
  assert.equal(list.filter((line) => line === name + ": test").length, 1);
for (const [index, name] of names.entries()) {
  const r = run(executable, ["--exact", prefix + name, "--nocapture", "--test-threads", "1"]);
  fs.writeFileSync(path.join(out, "law-" + index + ".stdout.txt"), r.stdout ?? "");
  fs.writeFileSync(path.join(out, "law-" + index + ".stderr.txt"), r.stderr ?? "");
  assert.equal(r.status, 0, r.stderr);
  assert.equal(r.signal, null);
  assert.equal(r.error, undefined);
  assert.match(r.stdout, /test result: ok\. 1 passed; 0 failed; 0 ignored;/u);
}
interface Window {
  role: string;
  group: string;
  imports: number;
  elapsedNs: number;
  files: number;
  bytes: number;
  edges: number;
  parses: number;
  scans: number;
  compiledSourceRevision: string;
  fixtureRootPhysical: string;
  normalOptimizedSourceOperation: boolean;
  profilingFeature: boolean;
}
const rows: Window[] = [];
for (const group of groups)
  for (const size of sizes)
    for (let pair = 0; pair < 5; pair++) {
      for (const role of pair % 2 === 0 ? ["legacy", "candidate"] : ["candidate", "legacy"]) {
        const name = group + "-" + size + "-" + pair + "-" + role;
        const output = path.join(out, name + ".json");
        const r = run(
          executable,
          ["--exact", carrier, "--ignored", "--nocapture", "--test-threads", "1"],
          {
            VIZE_PROBE_NORMAL_ROLE: role,
            VIZE_PROBE_NORMAL_GROUP: group,
            VIZE_PROBE_NORMAL_IMPORTS: String(size),
            VIZE_PROBE_FIXTURE: physical,
            VIZE_PROBE_NORMAL_OUTPUT: output,
          },
        );
        fs.writeFileSync(path.join(out, name + ".stdout.txt"), r.stdout ?? "");
        fs.writeFileSync(path.join(out, name + ".stderr.txt"), r.stderr ?? "");
        assert.equal(r.error, undefined);
        assert.equal(r.signal, null);
        assert.equal(r.status, 0, r.stderr);
        assert.match(r.stdout, /test result: ok\. 1 passed; 0 failed; 0 ignored;/u);
        const row = JSON.parse(fs.readFileSync(output, "utf8")) as Window;
        assert.equal(row.role, role);
        assert.equal(row.group, group);
        assert.equal(row.imports, size);
        assert.equal(row.compiledSourceRevision, sourceSha);
        assert.equal(row.fixtureRootPhysical, physical);
        assert.equal(row.normalOptimizedSourceOperation, true);
        assert.equal(row.profilingFeature, false);
        assert.ok(Number.isSafeInteger(row.elapsedNs) && row.elapsedNs > 0);
        assert.deepEqual(seal(), fixtureSeal);
        assert.equal(hash(fs.readFileSync(executable)), binary.sha256);
        rows.push(row);
      }
    }
const median = (values: number[]) =>
  values.toSorted((a, b) => a - b)[Math.floor(values.length / 2)];
const results = groups.flatMap((group) =>
  sizes.map((imports) => {
    const selected = rows.filter((row) => row.group === group && row.imports === imports);
    const legacy = selected.filter((row) => row.role === "legacy"),
      candidate = selected.filter((row) => row.role === "candidate");
    assert.equal(legacy.length, 5);
    assert.equal(candidate.length, 5);
    const work = (row: Window) => [row.files, row.bytes, row.edges, row.parses, row.scans];
    for (const row of selected) assert.deepEqual(work(row), work(legacy[0]));
    const old = legacy.map((row) => row.elapsedNs),
      next = candidate.map((row) => row.elapsedNs);
    return {
      group,
      imports,
      legacyNs: old,
      candidateNs: next,
      legacyMedianNs: median(old),
      candidateMedianNs: median(next),
      medianRatio: median(next) / median(old),
      work: work(legacy[0]),
    };
  }),
);
assert.equal(git("rev-parse", "HEAD"), sourceSha);
assert.equal(git("status", "--porcelain", "--untracked-files=all"), "");
write("summary.json", {
  schema: "vize.cli-probe-history.normal-source.summary",
  version: 1,
  sourceSha,
  sourceTree,
  binary,
  fixtureSha256: hash(JSON.stringify(fixtureSeal)),
  results,
  sourceCleanAfter: true,
  normalOptimizedSourceOperation: true,
  historicalMachineBinaryComparison: false,
  wholeCliSpeedupQualified: false,
  coldQualified: false,
  memoryQualified: false,
  hardwareQualified: false,
  scope:
    "one normal OPT3 libtest image with production MiMalloc, no profiling/counter overlay; legacy source-equivalent consttrue versus candidate constfalse; actual untracked scan/interner and retained alias caches; setup/hash/report outside every timer",
});
console.log(JSON.stringify({ sourceSha, binarySha256: binary.sha256, results }));
