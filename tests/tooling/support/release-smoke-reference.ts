// Separate the driver's protocol from rust-script/Cargo's compiler diagnostics.
// The Actions carrier retains those raw diagnostics and authenticates the exact
// source closure and compiled image before the parity tests call it directly.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { shellQuote } from "../../../tools/support/release/smoke_release_install/process.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const script = "tools/commands/release/npm/smoke-release-install.rs";
const closure = [
  script,
  "tools/support/common.rs",
  "tools/commands/release/npm/npm_publish.rs",
  "tools/commands/release/npm/npm_smoke_init.rs",
];
function sha256(file: string) {
  return createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}
function object(value: unknown): Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}
export function rustSmokeInvocation(args: readonly string[]) {
  const executable = process.env.VIZE_SMOKE_RUST_REFERENCE_EXECUTABLE;
  const receiptFile = process.env.VIZE_SMOKE_RUST_REFERENCE_RECEIPT;
  if (executable === undefined && receiptFile === undefined)
    return { command: "rust-script", args: [path.join(root, script), ...args] };
  assert.ok(executable && receiptFile, "both compiled-reference custody fields required");
  const receipt = object(JSON.parse(fs.readFileSync(receiptFile, "utf8")));
  assert.equal(receipt.executable, executable);
  assert.ok(fs.lstatSync(executable).isFile() && !fs.lstatSync(executable).isSymbolicLink());
  assert.equal(receipt.sha256, sha256(executable));
  const sources = object(receipt.sources);
  assert.deepEqual(Object.keys(sources).sort(), [...closure].sort());
  for (const relative of closure)
    assert.equal(sources[relative], sha256(path.join(root, relative)), relative);
  return { command: executable, args: [...args] };
}
function primeReference() {
  const temporary = process.env.RUNNER_TEMP,
    githubEnv = process.env.GITHUB_ENV;
  assert.ok(
    temporary && githubEnv,
    "compiled-reference qualification requires the Actions carrier",
  );
  const wrapper = path.join(temporary, "capture-smoke-reference.cjs"),
    receipt = path.join(temporary, "smoke-reference.json");
  fs.writeFileSync(
    wrapper,
    `
const fs = require('node:fs'), crypto = require('node:crypto');
const executable = fs.realpathSync.native(process.argv[2]);
const record = {executable, sha256:crypto.createHash('sha256').update(fs.readFileSync(executable)).digest('hex'),sources:JSON.parse(process.env.VIZE_SMOKE_REFERENCE_SOURCES), environment:Object.fromEntries(['RUST_SCRIPT_PATH','RUST_SCRIPT_BASE_PATH','RUST_SCRIPT_SAFE_NAME','RUST_SCRIPT_PKG_NAME'].map(key=>[key,process.env[key]]))};
fs.writeFileSync(process.env.VIZE_SMOKE_REFERENCE_CAPTURE, JSON.stringify(record,null,2)+'\\n');
const result = require('node:child_process').spawnSync(executable, process.argv.slice(3), {stdio:'inherit'});
if (result.error) throw result.error;
process.exitCode=result.status ?? 1;
`,
  );
  const sources = Object.fromEntries(
    closure.map((relative) => [relative, sha256(path.join(root, relative))]),
  );
  const result = spawnSync(
    "rust-script",
    [
      "--force",
      "--wrapper",
      [process.execPath, wrapper].map(shellQuote).join(" "),
      path.join(root, script),
      "--prime-reference",
    ],
    {
      cwd: root,
      encoding: "utf8",
      maxBuffer: Infinity,
      env: {
        ...process.env,
        VIZE_SMOKE_REFERENCE_SOURCES: JSON.stringify(sources),
        VIZE_SMOKE_REFERENCE_CAPTURE: receipt,
      },
    },
  );
  fs.writeFileSync(path.join(temporary, "smoke-reference-compile.stdout"), result.stdout);
  fs.writeFileSync(path.join(temporary, "smoke-reference-compile.stderr"), result.stderr);
  process.stdout.write(result.stdout);
  process.stderr.write(result.stderr);
  assert.ifError(result.error);
  assert.equal(result.status, 1);
  assert.ok(result.stderr.endsWith("Unknown argument: --prime-reference\n"));
  const record = object(JSON.parse(fs.readFileSync(receipt, "utf8")));
  assert.ok(typeof record.executable === "string");
  process.env.VIZE_SMOKE_RUST_REFERENCE_EXECUTABLE = record.executable;
  process.env.VIZE_SMOKE_RUST_REFERENCE_RECEIPT = receipt;
  const environment = object(record.environment);
  for (const key of [
    "RUST_SCRIPT_PATH",
    "RUST_SCRIPT_BASE_PATH",
    "RUST_SCRIPT_SAFE_NAME",
    "RUST_SCRIPT_PKG_NAME",
  ]) {
    const value = environment[key];
    assert.ok(typeof value === "string" && !/[\r\n]/.test(value));
    process.env[key] = value;
    fs.appendFileSync(githubEnv, `${key}=${value}\n`);
  }
  rustSmokeInvocation([]);
  fs.appendFileSync(
    githubEnv,
    `VIZE_SMOKE_RUST_REFERENCE_EXECUTABLE=${record.executable}\nVIZE_SMOKE_RUST_REFERENCE_RECEIPT=${receipt}\n`,
  );
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  primeReference();
