/** Qualify the exact provider closure for the six complete conformance checks. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const self = fileURLToPath(import.meta.url);
const root = fileURLToPath(new URL("../../../../", import.meta.url));

export const providerClosureRole = "standard-go-d6-vue35";
export const providerClosureTests = [
  "content_mapper_importer_scoped_packages::standard_tsgo_content_mapper_matches_duplicate_package_identities",
  "content_mapper_tsgo_cli::standard_tsgo_checks_vue_project_and_emits_consumable_declarations",
  "content_mapper_tsgo_cli::reported_dynamic_targets::originals_and_error_repair_restore_keep_component_target_types",
  "content_mapper_tsgo_watch::standard_tsgo_watch_enters_idle_with_authored_mapper_diagnostics",
  "content_mapper_tsgo_watch::standard_tsgo_watch_revalidates_authored_vue_edits_and_repairs",
  "content_mapper_tsgo_cli::listener_anchor_11657::checks_anchored_at_wrapping_parentheses_stay_reported",
] as const;

function required(name: string): string {
  const value = process.env[name];
  assert.ok(value, `${name} is required`);
  return value;
}

function digest(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function fileIdentity(file: string): { path: string; bytes: number; sha256: string } {
  const resolved = realpathSync(file);
  const bytes = readFileSync(resolved);
  return { path: resolved, bytes: bytes.length, sha256: digest(bytes) };
}

function packageIdentity(directory: string, expected: string) {
  const manifestPath = realpathSync(path.join(directory, "package.json"));
  const bytes = readFileSync(manifestPath);
  const manifest = JSON.parse(bytes.toString("utf8")) as {
    name?: string;
    version?: string;
    gitHead?: string;
  };
  assert.equal(manifest.name, expected);
  assert.ok(manifest.version, `${expected} has no version`);
  return {
    directory: realpathSync(directory),
    manifestPath,
    manifestSha256: digest(bytes),
    name: manifest.name,
    version: manifest.version,
    gitHead: manifest.gitHead ?? null,
  };
}

function commandText(command: string, args: readonly string[]): string {
  const result = spawnSync(command, args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
  });
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.signal, null, `${command} ${args.join(" ")} received ${result.signal}`);
  assert.equal(result.status, 0, `${command} ${args.join(" ")} failed\n${result.stderr}`);
  return result.stdout.trim();
}

function errorPacket(error: unknown) {
  return {
    name: error instanceof Error ? error.name : typeof error,
    message: error instanceof Error ? error.message : String(error),
    stack: error instanceof Error ? (error.stack ?? null) : null,
  };
}

function runTest(test: string, index: number, destination: string) {
  const directory = path.join(destination, `${String(index + 1).padStart(2, "0")}-${test}`);
  mkdirSync(directory, { recursive: true });
  const args = [
    "test",
    "--locked",
    "-p",
    "vize",
    "--test",
    "integration",
    "--",
    test,
    "--exact",
    "--nocapture",
    "--test-threads=1",
  ];
  const startedAt = new Date().toISOString();
  const result = spawnSync("cargo", args, {
    cwd: root,
    env: process.env,
    maxBuffer: 64 * 1024 * 1024,
    timeout: 120_000,
  });
  const finishedAt = new Date().toISOString();
  const stdout = result.stdout ?? Buffer.alloc(0);
  const stderr = result.stderr ?? Buffer.alloc(0);
  process.stdout.write(stdout);
  process.stderr.write(stderr);
  writeFileSync(path.join(directory, "stdout.bin"), stdout);
  writeFileSync(path.join(directory, "stderr.bin"), stderr);
  const combined = Buffer.concat([stdout, stderr]).toString("utf8");
  const executed =
    result.error === undefined &&
    result.signal === null &&
    combined.includes("running 1 test") &&
    combined.includes(`test ${test} ...`) &&
    !combined.includes("skipping exact Content Mapper") &&
    !combined.includes("skipping importer package Content Mapper oracle") &&
    !combined.includes("skipping reported dynamic targets");
  const receipt = {
    test,
    command: { file: "cargo", args, cwd: root, timeoutMs: 120_000 },
    startedAt,
    finishedAt,
    status: result.status,
    signal: result.signal,
    error: result.error ? errorPacket(result.error) : null,
    executed,
    stdout: { path: "stdout.bin", bytes: stdout.length, sha256: digest(stdout) },
    stderr: { path: "stderr.bin", bytes: stderr.length, sha256: digest(stderr) },
  };
  writeFileSync(path.join(directory, "process.json"), `${JSON.stringify(receipt, null, 2)}\n`);
  return receipt;
}

function main(): void {
  const destination = path.resolve(required("VIZE_CONTENT_MAPPER_PROVIDER_CLOSURE_DIR"));
  mkdirSync(destination, { recursive: true });
  const receipt: Record<string, unknown> = {
    schema: "vize.contentMapperProviderClosure.v1",
    source: required("GITHUB_SHA"),
    runId: required("GITHUB_RUN_ID"),
    runAttempt: Number(required("GITHUB_RUN_ATTEMPT")),
    role: required("VIZE_TEST_REPORTED_DYNAMIC_TARGETS_PROVIDER_ROLE"),
    status: "PREPARING",
    providers: null,
    tests: [],
    error: null,
  };
  const save = () =>
    writeFileSync(path.join(destination, "closure.json"), `${JSON.stringify(receipt, null, 2)}\n`);
  save();
  try {
    assert.equal(receipt.role, providerClosureRole);
    const source = commandText("git", ["rev-parse", "HEAD"]);
    assert.equal(source, receipt.source);
    const upstream = commandText("git", ["-C", "typescript-content-mapper", "rev-parse", "HEAD"]);
    assert.equal(upstream, required("CONTENT_MAPPER_TYPESCRIPT_SHA"));

    const tsgo = realpathSync(required("VIZE_TEST_CONTENT_MAPPER_TSGO"));
    const javascriptTsc = realpathSync(required("VIZE_TEST_CONTENT_MAPPER_JAVASCRIPT_TSC"));
    const vue = realpathSync(required("VIZE_TEST_CONTENT_MAPPER_VUE"));
    const dynamicTargets = path.resolve(required("VIZE_TEST_REPORTED_DYNAMIC_TARGETS_CAPTURE_DIR"));
    mkdirSync(dynamicTargets, { recursive: true });
    const typescriptDirectory = path.dirname(path.dirname(javascriptTsc));
    const mapperVersion = commandText(tsgo, ["--version"]);
    assert.equal(mapperVersion, "Version 7.1.0-dev");
    const vueIdentity = packageIdentity(vue, "vue");
    assert.equal(vueIdentity.version, "3.5.35");
    assert.equal(required("VIZE_TEST_REPORTED_DYNAMIC_TARGETS_REQUIRED"), "1");
    receipt.providers = {
      mapper: {
        ...fileIdentity(tsgo),
        version: mapperVersion,
        upstream,
      },
      javascriptTsc: {
        ...fileIdentity(javascriptTsc),
        version: commandText(javascriptTsc, ["--version"]),
        package: packageIdentity(typescriptDirectory, "typescript"),
      },
      vue: vueIdentity,
      node: { path: process.execPath, version: process.version },
    };
    receipt.environment = {
      VIZE_TEST_CONTENT_MAPPER_TSGO: tsgo,
      VIZE_TEST_CONTENT_MAPPER_JAVASCRIPT_TSC: javascriptTsc,
      VIZE_TEST_CONTENT_MAPPER_VUE: vue,
      VIZE_TEST_REPORTED_DYNAMIC_TARGETS_PROVIDER_ROLE: receipt.role,
      VIZE_TEST_REPORTED_DYNAMIC_TARGETS_REQUIRED: "1",
      VIZE_TEST_REPORTED_DYNAMIC_TARGETS_CAPTURE_DIR: realpathSync(dynamicTargets),
    };
    receipt.status = "RUNNING";
    save();

    const results = providerClosureTests.map((test, index) => runTest(test, index, destination));
    receipt.tests = results;
    const incomplete = results.filter((result) => !result.executed);
    assert.deepEqual(
      incomplete.map((result) => result.test),
      [],
      "every provider-closure check must execute instead of skipping",
    );
    receipt.status = results.every((result) => result.status === 0)
      ? "COMPLETE_ALL_PASSED"
      : "COMPLETE_WITH_TEST_FAILURES";
    save();
  } catch (error) {
    receipt.status = "PROVIDER_CLOSURE_FAILED";
    receipt.error = errorPacket(error);
    save();
    throw error;
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === self) main();
