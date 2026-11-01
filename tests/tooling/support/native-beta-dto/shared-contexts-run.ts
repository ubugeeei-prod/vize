import assert from "node:assert/strict";
import { execFileSync, spawnSync, type SpawnSyncReturns } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const env = process.env;
const required = (name: string): string => {
  const value = env[name];
  assert.ok(value, `missing ${name}`);
  return value;
};
assert.equal(required("ROLE"), "after");
assert.equal(required("VIZE_NATIVE_DTO_ROLE"), "after");
assert.equal(required("VIZE_NATIVE_DTO_SOURCE_SHA"), required("HEAD_SHA"));
const version = required("SDK_VERSION");
const images: Readonly<Record<string, string>> = {
  "7.0.2": "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
  "7.1.0-beta": "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
};
const image = images[version];
assert.ok(image);
const native = required("VIZE_NATIVE_DTO_ACTUAL_RUNTIME_PATH");
const hash = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
assert.equal(hash(fs.readFileSync(native)), image);
const capture = required("VIZE_NATIVE_DTO_CAPTURE");
const output = path.join(capture, "shared-contexts");
assert.ok(!fs.existsSync(output), "one immutable observation per provider");
fs.mkdirSync(output);
const root = required("VIZE_NATIVE_DTO_SOURCE_ROOT");
assert.match(required("HEAD_SHA"), /^[0-9a-f]{40}$/);
assert.equal(execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(), required("HEAD_SHA"));
assert.equal(fs.realpathSync(root), fs.realpathSync(required("GITHUB_WORKSPACE")));
type Artifact = {
  reason: string;
  target?: { name: string; src_path?: string };
  profile?: { test: boolean };
  executable?: string;
};
const artifacts = fs.readFileSync(path.join(capture, "build.jsonl"), "utf8").trim().split("\n")
  .map((line) => JSON.parse(line) as Artifact)
  .filter((row) => row.reason === "compiler-artifact" && row.target?.name === "vize_canon"
    && row.profile?.test && row.executable);
assert.equal(artifacts.length, 1);
const artifact = artifacts[0]!;
const executable = artifact.executable!;
assert.equal(fs.realpathSync(artifact.target!.src_path!),
  fs.realpathSync(path.join(root, "crates/vize_canon/src/lib.rs")));
const relativeExecutable = path.relative(fs.realpathSync(required("CARGO_TARGET_DIR")), fs.realpathSync(executable));
assert.ok(relativeExecutable && !relativeExecutable.startsWith("..") && !path.isAbsolute(relativeExecutable));
const selector = "lsp_client::editor_lsp::configured_project::shared_transport::tests::native::shared_contexts::"
  + (version === "7.1.0-beta"
    ? "production::native_two_config_shared_owner_uses_production_editor_fallback"
    : "tests::native_two_config_shared_owner_legacy_dialect_refuses_without_closing_owner");
const files = [
  "Cargo.toml", "Cargo.lock",
  ".github/workflows/native-beta-diagnostic-dto.yml",
  "tests/tooling/support/native-beta-dto/shared-contexts-run.ts",
  "crates/vize_canon/src/lsp_client/editor_lsp/completion_resolve.rs",
  "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native.rs",
  "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native/shared_contexts.rs",
  "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native/shared_contexts/tests.rs",
  "crates/vize_canon/src/lsp_client/editor_lsp/configured_project/shared_transport/tests/native/shared_contexts/production.rs",
];
fs.writeFileSync(path.join(output, "source.json"), JSON.stringify({
  source: required("HEAD_SHA"), sdkVersion: version, native, nativeSHA256: image,
  buildArtifact: artifact, testExecutable: executable, testExecutableSHA256: hash(fs.readFileSync(executable)),
  files: files.map((file) => {
    const bytes = fs.readFileSync(path.join(root, file));
    const committed = execFileSync("git", ["show", `${required("HEAD_SHA")}:${file}`],
      { cwd: root, maxBuffer: 16 * 1024 * 1024 });
    assert.deepEqual(bytes, committed, `whole source ${file} must equal actual After HEAD`);
    return { path: file, bytes: bytes.length, sha256: hash(bytes) };
  }),
  selector, scope: "test-only production-routed LSP sharing; current derived API refused; old readonly leases/config only; no resource or entered-native-read claim",
}, null, 2) + "\n");
const args = ["-ff", "-ttt", "-s", "16777216", "-xx", "-e",
  "trace=read,write,readv,writev,process,network", "-o", path.join(output, "syscalls"),
  executable, selector, "--ignored", "--exact", "--nocapture", "--test-threads=1"];
const stdout = fs.openSync(path.join(output, "stdout.bin"), "wx");
const stderr = fs.openSync(path.join(output, "stderr.bin"), "wx");
const packets = path.join(output, "packets");
let result: SpawnSyncReturns<Buffer>;
try {
  result = spawnSync("strace", args, {
    cwd: root,
    env: { ...env, CORSA_PATH: native, VIZE_TEST_REQUIRE_TSGO: "1",
      VIZE_PRIVATE_SHARED_CONTEXT_CAPTURE: packets },
    stdio: ["ignore", stdout, stderr], timeout: 600_000,
  });
} finally {
  fs.closeSync(stdout);
  fs.closeSync(stderr);
}
fs.writeFileSync(path.join(output, "execution.json"), JSON.stringify({
  command: "strace", args, status: result.status, signal: result.signal,
  error: result.error?.message ?? null,
}, null, 2) + "\n");
assert.equal(result.error, undefined);
assert.equal(result.signal, null);
assert.equal(result.status, 0);
const raw = fs.readFileSync(path.join(output, "stdout.bin"), "utf8");
assert.match(raw, /test result: ok\. 1 passed; 0 failed; 0 ignored;/);
