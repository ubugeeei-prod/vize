import assert from "node:assert/strict";
import { spawnSync, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const write = (file, value) => fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
const selector =
  "lsp_client::editor_lsp::native_shutdown::tests::beta_provider::exact_beta_editor_owner_keeps_whole_diagnostics_and_clean_eof_shutdown";
const root = fs.realpathSync(process.cwd());
const capture = process.env.VIZE_NATIVE_BETA_CAPTURE_DIR;
assert(capture, "explicit source capture directory is required");
fs.mkdirSync(capture, { recursive: true });
const provider = fs.realpathSync(process.env.VIZE_TEST_NATIVE_BETA_TSGO);
const providerRoot = path.dirname(path.dirname(provider));
const manifest = JSON.parse(fs.readFileSync(path.join(providerRoot, "package.json")));
assert.equal(manifest.name, "@typescript/typescript-linux-x64");
assert.equal(manifest.version, "7.1.0-beta");
const image = hash(fs.readFileSync(provider));
assert.equal(image, "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70");
const build = fs.readFileSync(path.join(capture, "build.jsonl"), "utf8");
const artifacts = build
  .trim()
  .split("\n")
  .map((line) => JSON.parse(line))
  .filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      row.target.name === "vize_canon" &&
      row.profile.test &&
      row.executable,
  );
assert.equal(artifacts.length, 1);
const executable = fs.realpathSync(artifacts[0].executable);
assert(executable.startsWith(path.join(root, "target/ci/deps/")));
const source = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
assert.equal(source, process.env.GITHUB_SHA, "original producer and executing source actor agree");
const strace = fs.realpathSync(execFileSync("which", ["strace"], { encoding: "utf8" }).trim());
const args = [
  "--kill-on-exit",
  "-ff",
  "-s",
  "65536",
  "-e",
  "trace=process",
  "-o",
  path.join(capture, "process"),
  executable,
  "--exact",
  selector,
  "--ignored",
  "--nocapture",
];
write(path.join(capture, "producer.json"), {
  scope:
    "Actual production EditorLspSession stdio/diagnostics/owned close only; no default project API or other OS source execution credit",
  source,
  githubSha: process.env.GITHUB_SHA,
  run: process.env.GITHUB_RUN_ID,
  attempt: process.env.GITHUB_RUN_ATTEMPT,
  executable,
  executableSha256: hash(fs.readFileSync(executable)),
  provider,
  providerSha256: image,
  manifest,
  strace,
  args,
  protocolCustody:
    "Source complete diagnostic DTOs and process exit custody; direct whole native wires sealed independently in #10741",
});
const result = spawnSync(strace, args, {
  cwd: root,
  env: { ...process.env, NO_COLOR: "1", RUST_TEST_THREADS: "1" },
  timeout: 45_000,
  killSignal: "SIGKILL",
});
fs.writeFileSync(path.join(capture, "test.stdout"), result.stdout ?? Buffer.alloc(0));
fs.writeFileSync(path.join(capture, "test.stderr"), result.stderr ?? Buffer.alloc(0));
write(path.join(capture, "process-result.json"), {
  status: result.status,
  signal: result.signal,
  error: result.error ? String(result.error) : null,
  stdoutSha256: hash(result.stdout ?? Buffer.alloc(0)),
  stderrSha256: hash(result.stderr ?? Buffer.alloc(0)),
});
assert.equal(result.error, undefined);
assert.equal(result.signal, null);
assert.equal(result.status, 0, result.stderr?.toString());
assert.deepEqual(result.stderr, Buffer.alloc(0), "whole source harness and SDK stderr");
const stdout = result.stdout.toString("utf8");
assert(stdout.includes(`test ${selector} ... ok`));
assert.match(stdout, /^test result: ok\. 1 passed; 0 failed; 0 ignored; /m);
const traces = fs.readdirSync(capture).filter((name) => /^process\.[0-9]+$/.test(name));
const sdk = [];
for (const name of traces) {
  const bytes = fs.readFileSync(path.join(capture, name));
  const text = bytes.toString("utf8");
  assert(!text.includes("killed by"), `all owned processes must terminate naturally: ${name}`);
  if (!text.includes(`execve(${JSON.stringify(provider)},`)) continue;
  assert(text.includes(`[${JSON.stringify(provider)}, "--lsp", "--stdio"]`));
  assert.match(text, /\+\+\+ exited with 0 \+\+\+\n$/);
  sdk.push({ file: name, bytes: bytes.length, sha256: hash(bytes), terminalCode: 0 });
}
assert.equal(
  sdk.length,
  1,
  "one real native editor owner; repeated shutdown must not spawn another",
);
assert.equal(hash(fs.readFileSync(provider)), image);
write(path.join(capture, "terminal-proof.json"), {
  source,
  sdk,
  sourceTestExecuted: 1,
  allSdkStderrBytes: 0,
  actualSourceEditorOwnerQualified: true,
  defaultProjectClientApiCredit: false,
});
process.stdout.write(stdout);
