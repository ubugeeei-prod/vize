/** Untimed fresh builds for opt-in custody admission; never a release performance receipt. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
const { values } = parseArgs({
  options: { source: { type: "string" }, base: { type: "string" }, out: { type: "string" } },
});
assert.ok(values.source && values.base && values.out);
const source = fs.realpathSync(values.source),
  out = path.resolve(values.out);
assert.match(values.base, /^[a-f0-9]{40}$/u);
assert.ok(!fs.existsSync(out));
fs.mkdirSync(out, { recursive: true });
const hash = (p: string) => createHash("sha256").update(fs.readFileSync(p)).digest("hex");
function execute(program: string, args: string[], env = process.env) {
  const r = spawnSync(program, args, {
    cwd: source,
    env,
    encoding: "utf8",
    maxBuffer: 128 * 1024 * 1024,
    timeout: 1_200_000,
  });
  assert.equal(r.error, undefined);
  assert.equal(r.signal, null);
  return r;
}
function git(...args: string[]) {
  const r = execute("git", args);
  assert.equal(r.status, 0, r.stderr);
  return r.stdout.trim();
}
const head = git("rev-parse", "HEAD");
assert.equal(git("merge-base", "--is-ancestor", values.base, head), "");
const versions = {
  cargo: execute("cargo", ["-Vv"]).stdout,
  rustc: execute("rustc", ["-Vv"]).stdout,
};
const roles = [
  { role: "normal-base", revision: values.base, features: [] as string[] },
  { role: "normal-head", revision: head, features: [] as string[] },
  { role: "observer", revision: head, features: ["cache-custody"] },
];
const builds = [];
let baseCli = "",
  baseDefaults = "";
for (const role of roles) {
  git("checkout", "--detach", role.revision);
  assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
  const cli = fs.readFileSync(
    path.join(source, "crates/vize_canon/src/batch/executor/cli.rs"),
    "utf8",
  );
  const defaults = ["crates/vize/Cargo.toml", "crates/vize_canon/Cargo.toml"]
    .map((p) => fs.readFileSync(path.join(source, p), "utf8").match(/^default = .*$/mu)?.[0])
    .join("\n");
  if (role.role === "normal-base") {
    baseCli = cli;
    baseDefaults = defaults;
  } else {
    const erased = cli
      .replace('#[cfg(feature = "cache-custody")]\nmod cache_custody_receipt;\n', "")
      .replace(
        '        #[cfg(feature = "cache-custody")]\n        let cache_receipt = cache_custody_receipt::requested(&command, project, config_path);\n',
        "",
      )
      .replace(
        '        #[cfg(feature = "cache-custody")]\n        cache_custody_receipt::completed(cache_receipt, &output);\n',
        "",
      );
    assert.equal(erased, baseCli, "feature-erased normal executor changed");
    assert.equal(defaults, baseDefaults, "default feature lists changed");
  }
  const target = path.join(out, "target-" + role.role);
  assert.equal(fs.existsSync(target), false);
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    CARGO_TARGET_DIR: target,
    CARGO_INCREMENTAL: "0",
    CARGO_BUILD_JOBS: "8",
    RUSTFLAGS:
      "--remap-path-prefix=" +
      target +
      "=/vize-custody-target --remap-path-prefix=" +
      source +
      "=/vize-custody-source -Cstrip=symbols",
  };
  delete env.CARGO_ENCODED_RUSTFLAGS;
  delete env.VIZE_CACHE_OBSERVER_SOURCE_SHA;
  if (role.role === "observer") env.VIZE_CACHE_OBSERVER_SOURCE_SHA = head;
  const args = [
    "build",
    "--locked",
    "--profile",
    "ci",
    "-p",
    "vize",
    "--message-format=json-render-diagnostics",
    ...(role.features.length ? ["--features", role.features.join(",")] : []),
  ];
  const start = process.hrtime.bigint(),
    r = execute("cargo", args, env);
  fs.writeFileSync(path.join(out, role.role + ".cargo.jsonl"), r.stdout);
  fs.writeFileSync(path.join(out, role.role + ".cargo.stderr.txt"), r.stderr);
  assert.equal(r.status, 0, "inspect retained Cargo stderr for " + role.role);
  const rows = r.stdout
    .split("\n")
    .filter(Boolean)
    .map(
      (line) =>
        JSON.parse(line) as {
          reason?: string;
          target?: { name: string; kind: string[] };
          executable?: string;
          fresh?: boolean;
          features?: string[];
        },
    );
  const artifacts = rows.filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      row.target?.name === "vize" &&
      row.target.kind.includes("bin") &&
      row.executable,
  );
  assert.equal(artifacts.length, 1);
  const artifact = artifacts[0]!;
  assert.equal(artifact.fresh, false);
  assert.equal(artifact.features?.includes("cache-custody"), role.role === "observer");
  assert.equal(artifact.features?.includes("profiling"), false);
  const binary = fs.realpathSync(artifact.executable!);
  assert.ok(binary.startsWith(fs.realpathSync(target) + path.sep));
  const version = execute(binary, ["--version"]);
  assert.equal(version.status, 0, version.stderr);
  if (role.role === "observer") {
    const tests = execute(
      "cargo",
      [
        "test",
        "--locked",
        "--profile",
        "ci",
        "-p",
        "vize_canon",
        "--features",
        "cache-custody",
        "--lib",
        "cache_custody_receipt",
        "--",
        "--nocapture",
      ],
      env,
    );
    fs.writeFileSync(path.join(out, "observer-tests.stdout.txt"), tests.stdout);
    fs.writeFileSync(path.join(out, "observer-tests.stderr.txt"), tests.stderr);
    assert.equal(tests.status, 0, "observer source-unit controls failed");
    assert.match(tests.stdout, /6 passed; 0 failed/u);
  }
  assert.equal(git("rev-parse", "HEAD"), role.revision);
  assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
  builds.push({
    ...role,
    sourceTree: git("rev-parse", "HEAD^{tree}"),
    sourceParents: git("show", "-s", "--format=%P", "HEAD").split(" "),
    targetStartedAbsent: true,
    target,
    actualCargoArtifact: artifact,
    binary,
    binarySha256: hash(binary),
    binaryBytes: fs.statSync(binary).size,
    cliVersion: version.stdout.trim(),
    compilerSourceSeal: env.VIZE_CACHE_OBSERVER_SOURCE_SHA ?? null,
    cargo: {
      args,
      env: {
        CARGO_TARGET_DIR: target,
        RUSTFLAGS: env.RUSTFLAGS,
        CARGO_INCREMENTAL: env.CARGO_INCREMENTAL,
      },
      ...versions,
    },
    manifests: Object.fromEntries(
      ["Cargo.toml", "Cargo.lock", "crates/vize/Cargo.toml", "crates/vize_canon/Cargo.toml"].map(
        (p) => [p, hash(path.join(source, p))],
      ),
    ),
    buildWallNs: String(process.hrtime.bigint() - start),
  });
}
git("checkout", "--detach", head);
assert.equal(new Set(builds.map((b) => b.cliVersion)).size, 1);
const report = {
  schema: "vize.cache-custody-build-admission.v1",
  sourceHead: head,
  base: values.base,
  builds,
  featureErasedNormalExecutorByteIdentical: true,
  oldTestReceiptPreserved: true,
  defaultFeaturesUnchanged: true,
  normalBinaryByteIdentical: builds[0]!.binarySha256 === builds[1]!.binarySha256,
  normalBinaryEqualityQualification:
    "recorded separately; remapped path metadata and linker BuildID may differ",
  profile: "ci, unoptimized correctness admission; not shipping or performance evidence",
  timingEligible: false,
};
fs.writeFileSync(path.join(out, "builds.json"), JSON.stringify(report, null, 2) + "\n");
console.log(
  JSON.stringify({
    sourceHead: head,
    builds: builds.map((b) => ({ role: b.role, sha256: b.binarySha256, bytes: b.binaryBytes })),
    normalBinaryByteIdentical: report.normalBinaryByteIdentical,
  }),
);
