/** One profiling-only test image; source operation evidence, never normal latency. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { validateProbeMicroRoles, type Role } from "./cli-probe-history-micro-validate.ts";

const { values } = parseArgs({
  options: {
    out: { type: "string" },
    target: { type: "string" },
    fixture: { type: "string" },
  },
});
assert.ok(values.out && values.target && values.fixture);
const root = fs.realpathSync(process.cwd()),
  out = path.resolve(values.out),
  target = path.resolve(values.target),
  fixture = path.resolve(values.fixture);
for (const value of [out, target, fixture]) {
  assert.ok(!fs.existsSync(value), "every task-owned output must start absent");
  const tail = path.relative(root, value);
  assert.ok(tail === ".." || tail.startsWith(".." + path.sep) || path.isAbsolute(tail));
}
fs.mkdirSync(out, { recursive: true });
const hash = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
const run = (command: string, args: string[], extra: Record<string, string> = {}) =>
  spawnSync(command, args, {
    cwd: root,
    env: { ...process.env, ...extra, NO_COLOR: "1" },
    encoding: "utf8",
    timeout: 1_800_000,
    maxBuffer: 128 * 1024 * 1024,
    killSignal: "SIGKILL",
  });
const checked = (command: string, args: string[], extra: Record<string, string> = {}) => {
  const r = run(command, args, extra);
  assert.equal(r.error, undefined);
  assert.equal(r.signal, null);
  assert.equal(r.status, 0, r.stderr);
  return r.stdout.trim();
};
const git = (...args: string[]) => checked("git", args);
const sourceSha = git("rev-parse", "HEAD"),
  sourceTree = git("rev-parse", "HEAD^{tree}");
assert.equal(git("status", "--porcelain", "--untracked-files=all"), "");
for (const config of [".cargo/config", ".cargo/config.toml"])
  if (fs.existsSync(config)) git("ls-files", "--error-unmatch", "--", config);
const env = {
  CARGO_TARGET_DIR: target,
  CARGO_BUILD_JOBS: "4",
  CARGO_INCREMENTAL: "0",
  VIZE_PROBE_PROFILE_SOURCE_SHA: sourceSha,
};
const safe =
  /^(?:RUSTFLAGS|CARGO_ENCODED_RUSTFLAGS|RUSTC|RUSTC_WRAPPER|RUSTC_WORKSPACE_WRAPPER|RUSTUP_TOOLCHAIN|CARGO_INCREMENTAL|CARGO_BUILD_JOBS|CARGO_TARGET_DIR|CARGO_BUILD_RUSTFLAGS|CARGO_BUILD_RUSTC|CARGO_BUILD_RUSTC_WRAPPER|VIZE_PROBE_PROFILE_SOURCE_SHA|CARGO_TARGET_[A-Z0-9_]+_(?:RUSTFLAGS|LINKER)|CARGO_PROFILE_[A-Z0-9_]+_(?:OPT_LEVEL|DEBUG|STRIP|LTO|CODEGEN_UNITS|PANIC|DEBUG_ASSERTIONS|OVERFLOW_CHECKS|INCREMENTAL))$/;
const build = {
  sourceSha,
  sourceTree,
  sourceCleanBefore: true,
  targetStartedAbsent: true,
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
      "crates/vize/src/commands/check/imports/resolution.rs",
      "crates/vize/src/commands/check/imports/registration/reachability.rs",
      "crates/vize/src/commands/check/imports/registration/reachability/tests/profile.rs",
    ]
      .filter((file) => fs.existsSync(file))
      .map((file) => [file, hash(fs.readFileSync(file))]),
  ),
};
const args = [
  "test",
  "--locked",
  "--profile",
  "ci",
  "-p",
  "vize",
  "--lib",
  "--features",
  "profiling",
  "--no-run",
  "--message-format=json-render-diagnostics",
  "--jobs",
  "4",
];
fs.writeFileSync(path.join(out, "build-request.json"), JSON.stringify({ ...build, args }, null, 2));
const result = run("cargo", args, env);
fs.writeFileSync(path.join(out, "cargo.stdout.jsonl"), result.stdout ?? "");
fs.writeFileSync(path.join(out, "cargo.stderr.log"), result.stderr ?? "");
assert.equal(result.error, undefined);
assert.equal(result.signal, null);
assert.equal(result.status, 0);
interface Artifact {
  reason: string;
  target: { name: string; kind: string[] };
  profile: { test: boolean };
  executable?: string;
  fresh: boolean;
  features: string[];
}
const artifacts = (result.stdout ?? "").split("\n").flatMap((line) => {
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
assert.ok(artifact.executable && artifact.features.includes("profiling"));
const executable = fs.realpathSync(artifact.executable),
  actualTarget = fs.realpathSync(target),
  tail = path.relative(actualTarget, executable);
assert.ok(
  tail !== "" && tail !== ".." && !tail.startsWith(".." + path.sep) && !path.isAbsolute(tail),
);
const binary = {
  path: executable,
  sha256: hash(fs.readFileSync(executable)),
  bytes: fs.statSync(executable).size,
};
fs.writeFileSync(
  path.join(out, "build.json"),
  JSON.stringify(
    {
      ...build,
      args,
      artifact,
      binary,
      cargoStdoutSha256: hash(result.stdout),
      cargoStderrSha256: hash(result.stderr),
    },
    null,
    2,
  ),
);
const fixtureFiles: Record<string, string> = {
  "entry.ts": "export {};\n",
  "model.ts": "export const n: number = 1;\n",
  "model.js": "export const n = 'javascript';\n",
  "esm.mts": "export {};\n",
  "esm.mjs": "export {};\n",
  "common.cts": "export {};\n",
  "common.cjs": "export {};\n",
  "folder/index.ts": "export {};\n",
  "Widget.vue": "<template />\n",
  "日本😀.vue": "<template>日本😀</template>\r\n",
  "real/component.vue": "<template />\n",
  "specific.ts": "export {};\n",
  "base.ts": "export {};\n",
  "tsconfig.json": JSON.stringify({
    compilerOptions: {
      baseUrl: ".",
      paths: {
        "@/*": ["missing/*", "real/*"],
        "@/special": ["specific.ts"],
      },
    },
  }),
};
for (const [file, text] of Object.entries(fixtureFiles)) {
  fs.mkdirSync(path.dirname(path.join(fixture, file)), { recursive: true });
  fs.writeFileSync(path.join(fixture, file), text, { flag: "wx" });
}
const physicalFixture = fs.realpathSync(fixture);
const sealFixture = () => {
  const files: string[] = [];
  const visit = (directory: string) => {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const absolute = path.join(directory, entry.name);
      assert.equal(entry.isSymbolicLink(), false);
      if (entry.isDirectory()) visit(absolute);
      else {
        assert.equal(entry.isFile(), true);
        files.push(path.relative(physicalFixture, absolute));
      }
    }
  };
  visit(physicalFixture);
  files.sort();
  assert.deepEqual(files, Object.keys(fixtureFiles).sort());
  return Object.fromEntries(
    files.map((file) => [
      file,
      {
        bytes: fs.statSync(path.join(physicalFixture, file)).size,
        sha256: hash(fs.readFileSync(path.join(physicalFixture, file))),
      },
    ]),
  );
};
const fixtureSeal = sealFixture();
fs.writeFileSync(
  path.join(out, "fixture-seal.json"),
  JSON.stringify({ root: physicalFixture, files: fixtureSeal }, null, 2),
);
const prefix = "commands::check::imports::registration::reachability::tests::";
const names = [
  "untracked_probes_keep_typescript_precedence_and_omit_candidate_histories",
  "untracked_path_mode_retains_complete_alias_probe_histories",
  "untracked_graphs_keep_exact_work_cycles_and_fresh_missing_to_created_inputs",
  "tracked_graph_control_still_retains_missing_candidates_and_real_sources",
  "cross_program_alias_agreement_keeps_equal_winners_with_distinct_proof_paths_apart",
];
const carrier = prefix + "profile::profile_local_probe_histories";
const list = checked(executable, ["--list"]);
for (const name of [...names.map((name) => prefix + name), carrier])
  assert.equal(list.split("\n").filter((line) => line === name + ": test").length, 1);
const executeTest = (name: string, ignored: boolean, extra: Record<string, string> = {}) => {
  const argv = [
    "--exact",
    name,
    "--nocapture",
    "--test-threads",
    "1",
    ...(ignored ? ["--ignored"] : []),
  ];
  const r = run(executable, argv, extra),
    suffix = name.split("::").at(-1)!;
  fs.writeFileSync(
    path.join(out, suffix + "-" + (extra.VIZE_PROBE_PROFILE_ROLE ?? "control") + ".stdout.txt"),
    r.stdout ?? "",
  );
  fs.writeFileSync(
    path.join(out, suffix + "-" + (extra.VIZE_PROBE_PROFILE_ROLE ?? "control") + ".stderr.txt"),
    r.stderr ?? "",
  );
  assert.equal(r.error, undefined);
  assert.equal(r.signal, null);
  assert.equal(r.status, 0, r.stderr);
  assert.match(r.stdout, /test result: ok\. 1 passed; 0 failed; 0 ignored;/);
  assert.ok(r.stdout.split("\n").includes("test " + name + " ... ok"));
  return { name, args: argv, status: r.status, signal: r.signal, passed: 1, failed: 0, ignored: 0 };
};
const controls = names.map((name) => executeTest(prefix + name, false));
const roles: Record<string, Role> = {};
const invocations = [];
for (const role of ["legacy", "candidate"]) {
  const output = path.join(out, role + ".json");
  invocations.push(
    executeTest(carrier, true, {
      VIZE_PROBE_PROFILE_ROLE: role,
      VIZE_PROBE_FIXTURE: physicalFixture,
      VIZE_PROBE_PROFILE_OUTPUT: output,
      VIZE_PROBE_ITERATIONS: "1000",
    }),
  );
  const row = JSON.parse(fs.readFileSync(output, "utf8")) as Role;
  assert.equal(row.role, role);
  assert.equal(row.fixtureRootPhysical, physicalFixture);
  assert.equal(row.compiledSourceRevision, sourceSha);
  assert.equal(row.normalPerformanceQualified, false);
  assert.equal(row.coldQualified, false);
  roles[role] = row;
  assert.deepEqual(sealFixture(), fixtureSeal);
}
const windows = validateProbeMicroRoles(roles);
assert.deepEqual(sealFixture(), fixtureSeal);
assert.equal(git("status", "--porcelain", "--untracked-files=all"), "");
assert.equal(hash(fs.readFileSync(executable)), binary.sha256);
const receipt = {
  schema: "vize.cli-probe-history.micro.acceptance",
  version: 1,
  sourceSha,
  sourceTree,
  binary,
  fixtureRootPhysical: physicalFixture,
  fixtureSealSha256: hash(JSON.stringify(fixtureSeal)),
  controls,
  invocations,
  windows,
  qualifiedProfilingWorkComparison: true,
  counterScope:
    "selected local resolver entries and five local probe is_file sites; not all filesystem or other processes",
  allocationScope:
    "synchronous TLS requested allocations under instrumentation; not RSS or peak live",
  normalPerformanceQualified: false,
  coldQualified: false,
  shippingQualified: false,
  peakLiveBytes: null,
  processTreeRss: null,
  hardwareInstructions: null,
  legacyReference: {
    sourceRevision: "b6127a994f7e2a75eb2c89de44d54cf8a0f17fcd",
    sourceFileBlob: "b0cd5048a6553e982f83e0ebf9874e7e22b07311",
    kind: "source-body equivalence of consttrue callbacks; not the old machine binary",
  },
  caveat:
    "profiling can inhibit compiler DCE/sinking; this evidence cannot justify merge or normal speed claims",
};
const receiptBytes = JSON.stringify(receipt, null, 2);
assert.ok(Buffer.byteLength(receiptBytes) <= 75_000, "never truncate a source counter packet");
fs.writeFileSync(path.join(out, "receipt.json"), receiptBytes);
process.stdout.write(JSON.stringify(receipt) + "\n");
