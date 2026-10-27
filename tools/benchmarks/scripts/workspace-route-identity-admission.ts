/** Actual source functional admission; test/build durations are not performance evidence. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";

const { values } = parseArgs({
  options: { "source-root": { type: "string" }, out: { type: "string" } },
});
assert.ok(values["source-root"] && values.out);
const sourceRoot = fs.realpathSync(values["source-root"]),
  out = path.resolve(values.out);
assert.ok(!fs.existsSync(out), "never overwrite an earlier admission");
fs.mkdirSync(out, { recursive: true });
const target = path.join(out, "cargo-target");
assert.ok(!fs.existsSync(target));
const hash = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const git = (...args: string[]) => {
  const r = spawnSync("git", args, { cwd: sourceRoot, encoding: "utf8" });
  assert.equal(r.status, 0, r.stderr);
  return r.stdout.trim();
};
const sourceSha = git("rev-parse", "HEAD"),
  sourceTree = git("rev-parse", "HEAD^{tree}");
assert.equal(git("status", "--porcelain", "--untracked-files=all"), "");
const env = {
  ...process.env,
  CARGO_TARGET_DIR: target,
  CARGO_BUILD_JOBS: "4",
  CARGO_INCREMENTAL: "0",
};
const tool = (name: string, args: string[]) => {
  const r = spawnSync(name, args, { cwd: sourceRoot, env, encoding: "utf8", timeout: 30_000 });
  assert.equal(r.status, 0, r.stderr);
  return r.stdout.trim();
};
for (const config of [".cargo/config", ".cargo/config.toml"]) {
  if (fs.existsSync(path.join(sourceRoot, config)))
    git("ls-files", "--error-unmatch", "--", config);
}
const compiler = { cargo: tool("cargo", ["-Vv"]), rustc: tool("rustc", ["-Vv"]) };
const args = [
  "test",
  "--locked",
  "--profile",
  "ci",
  "-p",
  "vize_canon",
  "--lib",
  "--no-run",
  "--message-format=json-render-diagnostics",
  "--jobs",
  "4",
];
const built = spawnSync("cargo", args, {
  cwd: sourceRoot,
  env,
  encoding: "utf8",
  timeout: 1_200_000,
  maxBuffer: 128 * 1024 * 1024,
  killSignal: "SIGKILL",
});
fs.writeFileSync(path.join(out, "cargo.stdout.jsonl"), built.stdout ?? "");
fs.writeFileSync(path.join(out, "cargo.stderr.txt"), built.stderr ?? "");
assert.equal(built.error, undefined);
assert.equal(built.signal, null);
assert.equal(built.status, 0, "inspect retained actual Cargo output");
const artifacts = (built.stdout ?? "").split("\n").flatMap((line) => {
  try {
    const row = JSON.parse(line);
    return row.reason === "compiler-artifact" &&
      row.target?.name === "vize_canon" &&
      row.profile?.test &&
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
assert.ok(artifact.features.includes("native"));
const executable = fs.realpathSync(artifact.executable),
  tail = path.relative(target, executable);
assert.ok(
  tail !== "" && tail !== ".." && !tail.startsWith(".." + path.sep) && !path.isAbsolute(tail),
);
const binarySha256 = hash(executable);
const tests = [
  [
    "new importer visibility",
    "batch::virtual_project::project::package_routes::identity::tests::identity_is_bounded_to_the_importer_without_crossing_package_visibility",
  ],
  [
    "new late-add/unlink/mode history",
    "batch::virtual_project::project::package_routes::identity::tests::negative_late_add_unlink_and_mode_replacement_keep_the_importer_index_complete",
  ],
  [
    "new exact ambiguous first choice",
    "batch::virtual_project::project::package_routes::identity::tests::ambiguous_mode_roots_use_the_original_exact_first_choice",
  ],
  [
    "new replacement/index completeness",
    "batch::virtual_project::project::package_routes::identity::tests::replacing_duplicate_and_reduced_bindings_never_keeps_a_removed_owner",
  ],
  [
    "existing private package scope",
    "batch::virtual_project::tests::workspace_package_routes::private_typescript_imports_keep_the_authored_package_scope",
  ],
  [
    "existing workspace dependency shadow",
    "batch::virtual_project::tests::workspace_package_dependency_shadows::workspace_build_shadow_mirrors_source_dependency_tree",
  ],
  [
    "existing actual editor rebuild/overlays",
    "corsa_bridge::vue_dependencies_alias::context::build::import_facts_tests::syntax_fact_reuse_preserves_editor_fixpoint_and_fresh_revisions",
  ],
  [
    "existing editor finalized input stamps",
    "corsa_bridge::vue_dependencies_alias::context::build::stamp_import_facts_tests::immediate_stamp_reuses_every_finalized_parse_and_keeps_exact_input_stamps",
  ],
];
const rows = [];
for (const [index, [purpose, name]] of tests.entries()) {
  const argv = ["--exact", name, "--nocapture", "--test-threads", "1"];
  const result = spawnSync(executable, argv, {
    cwd: sourceRoot,
    env,
    encoding: "utf8",
    timeout: 120_000,
    maxBuffer: 64 * 1024 * 1024,
  });
  const stem = String(index + 1).padStart(2, "0");
  fs.writeFileSync(path.join(out, stem + ".stdout.txt"), result.stdout ?? "");
  fs.writeFileSync(path.join(out, stem + ".stderr.txt"), result.stderr ?? "");
  const qualified =
    result.error === undefined &&
    result.signal === null &&
    result.status === 0 &&
    /test result: ok\. 1 passed; 0 failed; 0 ignored;/u.test(result.stdout ?? "");
  const row = {
    purpose,
    name,
    argv,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    qualified,
  };
  rows.push(row);
  console.log(JSON.stringify(row));
}
assert.equal(git("rev-parse", "HEAD"), sourceSha);
assert.equal(git("status", "--porcelain", "--untracked-files=all"), "");
assert.equal(hash(executable), binarySha256);
const receipt = {
  schema: "vize.workspace-identity.admission",
  version: 1,
  sourceSha,
  sourceTree,
  scope:
    "source functional laws; no editor transport, native typecheck accuracy or speed qualification",
  sourceRoot,
  target,
  targetStartedAbsent: true,
  actualCargoExecutable: executable,
  binarySha256,
  binaryBytes: fs.statSync(executable).size,
  artifact,
  compiler,
  command: { executable: "cargo", argv: args, cwd: sourceRoot },
  compilerEnvironment: Object.fromEntries(
    Object.entries(env).filter(
      ([key]) =>
        [
          "RUSTFLAGS",
          "CARGO_ENCODED_RUSTFLAGS",
          "RUSTC",
          "RUSTC_WRAPPER",
          "RUSTC_WORKSPACE_WRAPPER",
          "CARGO_BUILD_TARGET",
          "CARGO_INCREMENTAL",
          "CARGO_BUILD_JOBS",
        ].includes(key) || key.startsWith("CARGO_PROFILE_CI_"),
    ),
  ),
  manifests: Object.fromEntries(
    ["Cargo.toml", "Cargo.lock", ".cargo/config", ".cargo/config.toml"]
      .filter((p) => fs.existsSync(path.join(sourceRoot, p)))
      .map((p) => [p, hash(path.join(sourceRoot, p))]),
  ),
  tests: rows,
  passed: rows.length === 8 && rows.every((row) => row.qualified),
};
fs.writeFileSync(path.join(out, "acceptance.json"), JSON.stringify(receipt, null, 2) + "\n");
assert.equal(receipt.passed, true, "all eight exact tests must each pass once with zero ignored");
