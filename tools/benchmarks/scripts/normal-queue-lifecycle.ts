import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const root = required("GITHUB_WORKSPACE");
const temp = required("RUNNER_TEMP");
const evidence = path.join(root, "queue-normal-evidence");
fs.mkdirSync(evidence, { recursive: true });
const files = JSON.parse(
  fs.readFileSync(path.join(root, "tools/benchmarks/fixtures/queue-normal-observer.json"), "utf8"),
) as Record<string, string>;
const sources = {
  baseline: "8978c2d35133162cfb2bda80171beeed9959353c",
  candidate: "067c59d32822b982ed88712db4b21251d760995b",
} as const;
type Role = keyof typeof sources;
type BuildArtifact = {
  reason: string;
  target?: { name: string };
  profile?: Record<string, unknown>;
  executable?: string;
  fresh?: boolean;
  features?: string[];
};
type Executable = {
  path: string;
  sha256: string;
  bytes: number;
  source: string;
  tree: string;
  parents: string[];
  manifestSha256: string;
  lockSha256: string;
  packageManifestSha256: string;
  overlaySha256: string;
  overlay: { path: string; sha256: string; bytes: number }[];
  profile: Record<string, unknown>;
  features: string[];
  fresh: false;
};
const executables = {} as Record<Role, Executable>;
function required(name: string): string {
  const value = process.env[name];
  assert.ok(value, name);
  return value;
}
function digest(bytes: Buffer | string): string {
  return createHash("sha256").update(bytes).digest("hex");
}
function output(command: string, args: string[], cwd = root): string {
  const result = spawnSync(command, args, { cwd, encoding: "utf8" });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}
function run(
  argv: string[],
  cwd = root,
  env = process.env,
  stdout?: number,
  stderr?: number,
): void {
  const command = argv[0];
  assert.ok(command);
  const result = spawnSync(command, argv.slice(1), {
    cwd,
    env,
    stdio: ["ignore", stdout ?? "inherit", stderr ?? "inherit"],
  });
  assert.ifError(result.error);
  assert.equal(result.status, 0, argv.join(" "));
}
for (const role of Object.keys(sources) as Role[]) {
  const source = sources[role],
    work = path.join(temp, "queue-normal-" + role + "-source"),
    target = path.join(temp, "queue-normal-" + role + "-target");
  assert.equal(fs.existsSync(work), false);
  assert.equal(fs.existsSync(target), false);
  run(["git", "worktree", "add", "--detach", work, source]);
  assert.equal(output("git", ["rev-parse", "HEAD"], work), source);
  const tree = output("git", ["rev-parse", "HEAD^{tree}"], work);
  const parents = output("git", ["show", "-s", "--format=%P", "HEAD"], work)
    .split(" ")
    .filter(Boolean);
  for (const [relative, text] of Object.entries(files)) {
    if (relative.endsWith("metadata-baseline.rs") || relative.endsWith("metadata-candidate.rs"))
      continue;
    const destination = path.join(work, relative);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, text);
  }
  const metadata =
    files[
      "crates/vize_maestro/src/server/initial_diagnostics/normal_carrier/metadata-" + role + ".rs"
    ];
  assert.ok(metadata);
  fs.writeFileSync(
    path.join(
      work,
      "crates/vize_maestro/src/server/initial_diagnostics/normal_carrier/metadata.rs",
    ),
    metadata,
  );
  const module = path.join(work, "crates/vize_maestro/src/server/initial_diagnostics.rs");
  fs.appendFileSync(module, "\n#[cfg(test)]\nmod normal_carrier;\n");
  const overlayPaths = [
    ...Object.keys(files).filter(
      (p) => !p.includes("metadata-baseline.rs") && !p.includes("metadata-candidate.rs"),
    ),
    "crates/vize_maestro/src/server/initial_diagnostics/normal_carrier/metadata.rs",
    "crates/vize_maestro/src/server/initial_diagnostics.rs",
  ];
  if (role === "candidate") {
    const relative = "crates/vize_maestro/src/server/initial_diagnostics/queue.rs",
      file = path.join(work, relative),
      old = fs.readFileSync(file, "utf8");
    const needle = '    #[cfg(feature = "micro-profiling")]\n    pub(super) fn storage';
    assert.ok(old.includes(needle));
    fs.writeFileSync(file, old.replace(needle, "    pub(super) fn storage"));
    overlayPaths.push(relative);
  }
  const overlay = overlayPaths.sort().map((relative) => {
    const bytes = fs.readFileSync(path.join(work, relative));
    return { path: relative, sha256: digest(bytes), bytes: bytes.length };
  });
  fs.writeFileSync(
    path.join(evidence, role + "-observer.diff"),
    output("git", ["diff", "--", "crates/vize_maestro"], work),
  );
  const buildJson = path.join(evidence, role + "-build.jsonl"),
    buildError = path.join(evidence, role + "-build.stderr");
  const out = fs.openSync(buildJson, "w"),
    err = fs.openSync(buildError, "w");
  try {
    run(
      [
        "cargo",
        "test",
        "--locked",
        "--profile",
        "ci-opt",
        "-p",
        "vize_maestro",
        "--lib",
        "--no-run",
        "--message-format=json",
      ],
      work,
      { ...process.env, CARGO_TARGET_DIR: target, CARGO_BUILD_JOBS: "4", CARGO_INCREMENTAL: "0" },
      out,
      err,
    );
  } finally {
    fs.closeSync(out);
    fs.closeSync(err);
  }
  const rows = fs
    .readFileSync(buildJson, "utf8")
    .split("\n")
    .filter((line) => line.startsWith("{"))
    .map((line) => JSON.parse(line) as BuildArtifact);
  const found = rows.filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      row.target?.name === "vize_maestro" &&
      row.profile?.test &&
      row.executable,
  );
  assert.equal(found.length, 1);
  const artifact = found[0];
  assert.ok(artifact?.executable && artifact.profile && artifact.features);
  assert.equal(artifact.fresh, false, "separate clean targets must actually rebuild this source");
  assert.equal(artifact.profile.test, true);
  assert.notEqual(artifact.profile.opt_level, "0");
  assert.equal(typeof artifact.profile.debug_assertions, "boolean");
  assert.notEqual(artifact.profile.debuginfo, undefined);
  assert.equal(artifact.features.includes("micro-profiling"), false);
  const binary = artifact.executable;
  assert.ok(fs.statSync(binary).isFile());
  executables[role] = {
    path: binary,
    sha256: digest(fs.readFileSync(binary)),
    bytes: fs.statSync(binary).size,
    source,
    tree,
    parents,
    manifestSha256: digest(fs.readFileSync(path.join(work, "Cargo.toml"))),
    lockSha256: digest(fs.readFileSync(path.join(work, "Cargo.lock"))),
    packageManifestSha256: digest(
      fs.readFileSync(path.join(work, "crates/vize_maestro/Cargo.toml")),
    ),
    overlaySha256: digest(JSON.stringify(overlay)),
    overlay,
    profile: artifact.profile,
    features: artifact.features,
    fresh: false,
  };
}
assert.notEqual(executables.baseline.sha256, executables.candidate.sha256);
assert.deepEqual(executables.baseline.profile, executables.candidate.profile);
assert.deepEqual(executables.baseline.features, executables.candidate.features);
assert.equal(executables.baseline.manifestSha256, executables.candidate.manifestSha256);
assert.equal(executables.baseline.lockSha256, executables.candidate.lockSha256);
assert.equal(
  executables.baseline.packageManifestSha256,
  executables.candidate.packageManifestSha256,
);
fs.writeFileSync(path.join(evidence, "binaries.json"), JSON.stringify(executables, null, 2));
const allowed = fs
  .readFileSync("/proc/self/status", "utf8")
  .match(/^Cpus_allowed_list:\\s*([0-9,-]+)$/m)?.[1];
assert.ok(allowed);
const first = allowed.split(",")[0];
assert.ok(first);
const core = Number(first.split("-")[0]);
assert.ok(Number.isSafeInteger(core) && core >= 0);
const project = path.join(temp, "normal-queue-identical-project");
const results: Record<string, unknown>[] = [];
for (let round = 0; round < 3; round++) {
  const roles: Role[] = round % 2 === 0 ? ["baseline", "candidate"] : ["candidate", "baseline"];
  for (const count of [1, 10, 100]) {
    for (const role of roles) {
      fs.rmSync(project, { recursive: true, force: true });
      const resultPath = path.join(evidence, round + "-" + count + "-" + role + ".json");
      const env = {
        ...process.env,
        VIZE_QUEUE_PROJECT: project,
        VIZE_QUEUE_COUNT: String(count),
        VIZE_QUEUE_SOURCE: sources[role],
        VIZE_QUEUE_ROLE: role,
        VIZE_QUEUE_EVIDENCE: resultPath,
      };
      const log = path.join(evidence, round + "-" + count + "-" + role + ".log"),
        stream = fs.openSync(log, "w");
      try {
        run(
          [
            "taskset",
            "-c",
            String(core),
            executables[role].path,
            "server::initial_diagnostics::normal_carrier::normal_queue_lifecycle",
            "--exact",
            "--nocapture",
            "--test-threads=1",
          ],
          root,
          env,
          stream,
          stream,
        );
      } finally {
        fs.closeSync(stream);
      }
      const text = fs.readFileSync(log, "utf8");
      assert.ok(text.includes("test result: ok. 1 passed; 0 failed; 0 ignored"), text.slice(-5000));
      const report = JSON.parse(fs.readFileSync(resultPath, "utf8")) as Record<string, unknown>;
      assert.equal(report.profiling, false);
      assert.equal(report.source, sources[role]);
      assert.equal(report.role, role);
      assert.equal(report.unique_uris, count);
      const inputDigest = report.input_sha256;
      assert.ok(typeof inputDigest === "string" && /^[a-f0-9]{64}$/u.test(inputDigest));
      const configuration = report.build_configuration as {
        test_cfg: boolean;
        micro_profiling: boolean;
        debug_assertions: boolean;
      };
      assert.equal(configuration.test_cfg, true);
      assert.equal(configuration.micro_profiling, false);
      assert.equal(configuration.debug_assertions, executables[role].profile.debug_assertions);
      const { raw_input_manifest: _raw, ...rest } = report;
      results.push({ round, count, role, ...rest });
    }
    const paired = results.filter((row) => row.round === round && row.count === count);
    assert.equal(paired.length, 2);
    assert.equal(paired[0]?.input_sha256, paired[1]?.input_sha256);
  }
}
const summary = {
  kind: "small normal component calibration; cfg(test) libtest helpers remain, not shipping-machine-code identity or whole LSP/cold500ms/size qualification",
  sources,
  cpu_affinity: [core],
  samples: results,
};
fs.writeFileSync(path.join(evidence, "summary.json"), JSON.stringify(summary, null, 2));
console.log(
  JSON.stringify({
    samples: results.length,
    all_same_input_pairs: true,
    summary: "queue-normal-evidence/summary.json",
  }),
);
