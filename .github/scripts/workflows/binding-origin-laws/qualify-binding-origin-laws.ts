import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, requiredEnv, writeJson, fileSha256, fileIdentity } from "../shared/files.ts";

import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  realpathSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { isAbsolute, relative } from "node:path";
import type { CargoArtifact } from "../shared/cargo-artifacts.ts";

const root = realpathSync(requiredEnv("GITHUB_WORKSPACE")),
  out = join(requiredEnv("RUNNER_TEMP"), "binding-origin-admission"),
  target = join(out, "cargo-target");
mkdirSync(out);
const env: NodeJS.ProcessEnv = {
  ...process.env,
  CARGO_TARGET_DIR: target,
  CARGO_BUILD_JOBS: "4",
  CARGO_INCREMENTAL: "0",
};
function run(command: string[], seconds = 30, cwd = root, variables = env) {
  const result = spawnSync(command[0]!, command.slice(1), {
    cwd,
    env: variables,
    encoding: "utf8",
    timeout: seconds * 1000,
    maxBuffer: 256 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  return result;
}
function git(...args: string[]) {
  const result = run(["git", ...args]);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}
const source = git("rev-parse", "HEAD"),
  tree = git("rev-parse", "HEAD^{tree}"),
  foundation = "8978c2d35133162cfb2bda80171beeed9959353c";
git("merge-base", "--is-ancestor", foundation, source);
assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
const compiler = Object.fromEntries(
  ["cargo", "rustc"].map((name) => {
    const r = run([name, "-Vv"]);
    assert.equal(r.status, 0, r.stderr);
    return [name, r.stdout];
  }),
);
function capture(stem: string, command: string[], seconds: number, cwd = root, variables = env) {
  const r = run(command, seconds, cwd, variables);
  writeFileSync(join(out, stem + ".stdout.txt"), r.stdout);
  writeFileSync(join(out, stem + ".stderr.txt"), r.stderr);
  return r;
}
const observer = capture(
  "observer-unit",
  ["node", "--test", "--test-reporter=tap", "tests/tooling/native-api-wire-origin.test.ts"],
  60,
);
assert.equal(observer.status, 0, observer.stdout + observer.stderr);
for (const marker of ["# tests 38", "# pass 38", "# fail 0", "# skipped 0"])
  assert.ok(observer.stdout.includes(marker), observer.stdout);
const guard = run(["setpriv", "--version"]);
assert.equal(guard.status, 0);
writeFileSync(join(out, "setpriv-version.txt"), guard.stdout + guard.stderr);
const argv = [
  "cargo",
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
const built = run(argv, 1200);
writeFileSync(join(out, "cargo.stdout.jsonl"), built.stdout);
writeFileSync(join(out, "cargo.stderr.txt"), built.stderr);
const messages = built.stdout.split("\n").flatMap((line) => {
  try {
    return [
      JSON.parse(line) as CargoArtifact & { message?: { level?: string; rendered?: string } },
    ];
  } catch {
    return [];
  }
});
if (built.status !== 0) {
  console.error(built.stderr);
  for (const row of messages)
    if (row.message?.level === "error") console.error(row.message.rendered ?? row.message);
}
assert.equal(built.status, 0, "inspect retained actual compiler output");
const artifacts = messages.filter(
  (row) =>
    row.reason === "compiler-artifact" &&
    row.target?.name === "vize_canon" &&
    row.profile?.test &&
    row.executable,
);
assert.equal(artifacts.length, 1);
const artifact = artifacts[0]!;
assert.equal(artifact.fresh, false);
assert.ok(artifact.features.includes("native"));
const binary = realpathSync(artifact.executable),
  rel = relative(target, binary);
assert.ok(!isAbsolute(rel) && rel !== ".." && !rel.startsWith("../"));
const image = fileIdentity(binary);
const tests = [
  "binding_origins::tests::source_callable_shape_never_authorizes_vue_identity",
  "binding_origins::tests::actual_type_queries_and_type_only_aliases_retain_provider_identity",
  "binding_origins::tests::module_spelling_and_stale_provider_revisions_cannot_authorize_identity",
  "binding_origins::tests::source_and_project_owners_are_explicit_snapshot_inputs",
  "binding_origins::tests::alias_cycles_do_not_require_a_recursion_limit_or_fabricate_an_origin",
  "binding_origins::tests::cyclic_export_evidence_can_reach_one_provider_without_guessing_missing_edges",
  "binding_origins::tests_graph::twenty_thousand_aliases_expand_without_recursion_or_repeated_resolution",
  "binding_origins::tests_graph::distinct_global_declarations_keep_ambiguous_ownership",
  "binding_origins::tests_graph::stale_or_foreign_resolver_images_never_authorize_a_provider",
  "binding_origins::tests_graph::type_only_runtime_values_do_not_become_callable_providers",
  "binding_origins::tests_graph::requested_roots_do_not_resolve_unrelated_global_values",
  "binding_origins::tests_graph::import_memoization_preserves_distinct_effective_usage",
  "binding_origins::tests_graph::import_memoization_preserves_distinct_source_owners",
  "binding_origins::tests_ambient::annotated_runtime_custom_functions_cannot_borrow_provider_identity",
  "binding_origins::tests_ambient::uninitialized_runtime_annotations_are_not_ambient_provider_declarations",
  "binding_origins::tests_ambient::actual_ambient_declarations_and_immutable_value_aliases_keep_provider_identity",
  "binding_origins::tests_ambient::merged_overload_declarations_preserve_custom_symbol_identity",
  "binding_origins::tests_requested_globals::requested_global_names_do_not_lower_ten_thousand_unrelated_values",
  "binding_origins::tests_requested_globals::existing_root_scope_maps_preserve_opaque_value_declaration_forms",
  "binding_origins::tests_requested_globals::existing_global_augmentation_scope_retains_unsupported_values",
  "binding_origins::tests_requested_globals::ambient_runtime_writes_cannot_borrow_provider_identity",
  "binding_origins::tests_requested_globals::requested_global_roots_retain_actual_alias_dependencies",
  "binding_origins::tests_requested_globals::missing_original_semantic_scopes_refuse_the_whole_source",
];
const listing = capture("registered-tests", [binary, "--list"], 30);
assert.equal(listing.status, 0);
const registered = new Set(
  listing.stdout
    .split(/\r?\n/u)
    .filter((line) => line.endsWith(": test"))
    .map((line) => line.slice(0, -6)),
);
for (const name of tests) assert.ok(registered.has(name), name);
function qualified(result: ReturnType<typeof run>) {
  return (
    result.status === 0 && /test result: ok[.] 1 passed; 0 failed; 0 ignored;/u.test(result.stdout)
  );
}
const rows = tests.map((name, index) => {
  const command = [binary, "--exact", name, "--nocapture", "--test-threads", "1"],
    r = capture(String(index + 1).padStart(2, "0"), command, 120);
  const row = { name, command, status: r.status, qualified: qualified(r) };
  console.log(JSON.stringify(row));
  return row;
});
const nativeName =
  "binding_origins::native_api_probe::standalone_native_api_retains_one_project_symbol_owner";
assert.ok(registered.has(nativeName));
function copyPresent(sourceDir: string, destination: string, files: string[]) {
  for (const name of files) {
    const from = join(sourceDir, name);
    if (statSync(from, { throwIfNoEntry: false })?.isFile())
      copyFileSync(from, join(destination, name));
  }
}
function nativeRole(version: string) {
  const provider = join(out, "providers", version);
  mkdirSync(provider, { recursive: true });
  writeJson(join(provider, "package.json"), {
    name: "vize-origin-api-provider",
    version: "1.0.0",
    private: true,
    dependencies: {
      typescript: version,
      vue: "3.5.42",
      "vue-authentic": "npm:vue@3.5.42",
      "vue-other": "npm:vue@3.5.44",
    },
  });
  const install = capture(
    version + ".npm",
    [
      "npm",
      "install",
      "--ignore-scripts",
      "--no-audit",
      "--no-fund",
      "--registry=https://registry.npmjs.org",
    ],
    300,
    provider,
  );
  assert.equal(install.status, 0, install.stdout + install.stderr);
  const lock = join(provider, "package-lock.json");
  copyFileSync(lock, join(out, version + ".package-lock.json.txt"));
  const packageDir = join(provider, "node_modules"),
    backendDir = join(packageDir, "@typescript/typescript-linux-x64"),
    backend = realpathSync(join(backendDir, "lib/tsc"));
  const manifest = (dir: string) =>
    readJson<{ name: string; version: string }>(join(dir, "package.json"));
  const metadata = manifest(backendDir);
  assert.equal(metadata.name, "@typescript/typescript-linux-x64");
  assert.equal(metadata.version, version);
  assert.equal(manifest(join(packageDir, "typescript")).version, version);
  assert.equal(manifest(join(packageDir, "vue")).version, "3.5.42");
  for (const [alias, expected] of [
    ["vue-authentic", "3.5.42"],
    ["vue-other", "3.5.44"],
  ] as const) {
    const pkg = manifest(join(packageDir, alias));
    assert.equal(pkg.name, "vue");
    assert.equal(pkg.version, expected);
  }
  const dir = join(out, "native-" + version),
    command = [binary, "--exact", nativeName, "--ignored", "--nocapture", "--test-threads", "1"];
  const result = capture(version + ".native", command, 180, root, {
    ...env,
    VIZE_ORIGIN_API_PROVIDER: provider,
    VIZE_ORIGIN_API_BACKEND: backend,
    VIZE_ORIGIN_API_CAPTURE: dir,
    VIZE_ORIGIN_API_VERSION: version,
  });
  let passed = qualified(result),
    wire: unknown = null,
    wireError: string | null = null,
    mutationControls: { controls: unknown[]; passed: boolean } | null = null;
  if (
    ["client-to-native.wire.bin", "native-to-client.wire.bin"].every((name) =>
      statSync(join(dir, name), { throwIfNoEntry: false })?.isFile(),
    )
  ) {
    try {
      function observe(name: string, script: string) {
        const obs = capture(
          version + "." + name,
          [
            "node",
            join(root, "crates/vize_canon/src/binding_origins/native_api_probe", script),
            dir,
            version,
          ],
          60,
        );
        assert.equal(obs.status, 0, obs.stdout + obs.stderr);
        return obs.stdout;
      }
      wire = JSON.parse(observe("wire", "verify-wire.ts")) as unknown;
      const controls = JSON.parse(observe("mutation-controls", "verify-mutation-controls.ts")) as {
        controls: unknown[];
        passed: boolean;
      };
      mutationControls = controls;
      assert.equal(controls.controls.length, 3);
      assert.equal(controls.passed, true);
    } catch (error) {
      wireError = String(error);
      passed = false;
    }
  }
  const carrierPath = join(dir, "carrier-receipt.json"),
    carrier = existsSync(carrierPath) ? readJson(carrierPath) : null;
  const row = {
    role: version,
    test: nativeName,
    command,
    status: result.status,
    qualified: passed,
    wire,
    mutationControls,
    wireError,
    backendSha256: fileSha256(backend),
    backendBytes: statSync(backend).size,
    package: metadata,
    lockSha256: fileSha256(lock),
  };
  console.log(JSON.stringify(row));
  if (!passed) console.error(result.stdout, result.stderr);
  if (carrier !== null) console.log("NATIVE_ORIGIN_API_RECEIPT " + JSON.stringify(carrier));
  if (wire !== null) console.log("NATIVE_ORIGIN_WIRE_RECEIPT " + JSON.stringify(wire));
  const lean = join(out, "native-metadata", version);
  mkdirSync(lean, { recursive: true });
  copyPresent(dir, lean, [
    "backend-version.json",
    "actual-launch.json",
    "carrier-receipt.json",
    "raw-symbol-candidates.json",
    "full-wire-symbol-packets.json",
    "full-wire-receipt.json",
    "semantic-mutation-wire.json",
    "semantic-mutation-controls.json",
    "native.stderr.txt",
  ]);
  for (const phase of [
    "valid",
    "partial",
    "repaired",
    "initializer-vue",
    "initializer-custom",
    "initializer-repaired",
  ]) {
    const target = join(lean, phase);
    mkdirSync(target);
    copyPresent(join(dir, phase), target, [
      "baseline-normalized.json",
      "cases-selected.json",
      "cases-normalized.json",
      "source-file-names.json",
      "source-input-manifest.json",
      "getSyntacticDiagnostics.json",
      "getSemanticDiagnostics.json",
      "phase-receipt.json",
      "mutation-receipt.json",
      "original-partial.ts",
    ]);
  }
  copyPresent(join(dir, "project"), lean, [
    "probe.ts",
    "tsconfig.json",
    "custom.ts",
    "custom-vue.ts",
    "globals.d.ts",
    "merged.d.ts",
    "opaque-globals.ts",
    "partial.ts",
  ]);
  for (const suffix of [
    "native.stdout.txt",
    "native.stderr.txt",
    "package-lock.json.txt",
    "npm.stdout.txt",
    "npm.stderr.txt",
  ]) {
    const from = join(out, version + "." + suffix);
    if (statSync(from, { throwIfNoEntry: false })?.isFile()) copyFileSync(from, join(lean, suffix));
  }
  return row;
}
const nativeCarrierRoles = ["7.0.2", "7.1.0-beta"].map(nativeRole);
assert.equal(git("rev-parse", "HEAD"), source);
assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
assert.equal(fileSha256(binary), image.sha256);
const manifests = Object.fromEntries(
  ["Cargo.toml", "Cargo.lock", ".cargo/config", ".cargo/config.toml"]
    .filter((file) => existsSync(join(root, file)))
    .map((file) => [file, fileSha256(join(root, file))]),
);
const origin = "crates/vize_canon/src/binding_origins";
const originSource = Object.fromEntries(
  readdirSync(join(root, origin), { recursive: true, encoding: "utf8" })
    .sort()
    .map((file) => join(origin, file))
    .filter((file) => /\.(?:rs|ts|mts)$/u.test(file) && statSync(join(root, file)).isFile())
    .map((file) => [file, fileSha256(join(root, file))]),
);
const compilerEnvironment = Object.fromEntries(
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
);
const receipt = {
  schema: "vize.binding-origin.source-laws.v1",
  sourceSha: source,
  sourceTree: tree,
  foundationSource: foundation,
  requestedRef: process.env.GITHUB_REF ?? null,
  eventSourceSha: process.env.GITHUB_SHA ?? null,
  scope:
    "Unwired origin foundation plus test-only same-session native symbol/closure laws; not product CLI, mapper, LSP or performance qualification",
  targetStartedAbsent: true,
  target,
  cargoCommand: argv,
  actualCargoArtifact: artifact,
  actualTestImage: image,
  compiler,
  compilerEnvironment,
  manifests,
  originSource,
  tests: rows,
  nativeCarrierRoles,
  observerUnitTests: 38,
  passed:
    rows.length === 23 &&
    rows.every((row) => row.qualified) &&
    nativeCarrierRoles.length === 2 &&
    nativeCarrierRoles.every((row) => row.qualified),
};
writeJson(join(out, "acceptance.json"), receipt);
copyFileSync(
  join(out, "acceptance.json"),
  join(out, "native-metadata/actual-source-acceptance.json"),
);
console.log(
  "SOURCE_IMAGE_RECEIPT " +
    JSON.stringify({
      sourceSha: source,
      sourceTree: tree,
      binarySha256: image.sha256,
      binaryBytes: image.bytes,
      nativeRoles: nativeCarrierRoles,
    }),
);
assert.ok(receipt.passed, "all 23 source laws and both native carrier roles must pass");
