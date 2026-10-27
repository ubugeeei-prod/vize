import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import {
  arg,
  readJson,
  sha256,
  type PackageManifest,
  type ContributionResult,
  type RuntimeResult,
  type ConfigResult,
  type RoleResult,
} from "./utils.ts";

interface ProcessRecord {
  script: string;
  status: number | null;
  signal: NodeJS.Signals | null;
  error: string | null;
  stdout: string;
  stderr: string;
}
interface PublicProviderReceipt {
  source: string;
  version: string;
  provider: { files: Array<{ path: string; bytes: number; sha256: string }> };
}
const workspace = fs.realpathSync(arg(0));
const version = arg(1);
const out = path.resolve(arg(2));
const candidate = arg(3);
const capsule = path.dirname(fileURLToPath(import.meta.url));
assert.match(version, /^0\.\d+\.0$/);
assert.match(candidate, /^[a-f0-9]{40}$/);
fs.mkdirSync(out, { recursive: true });
const custody = readJson<{ frozenInputs: Array<{ path: string; sha256: string }> }>(
  path.join(capsule, "source-custody.json.txt"),
);
const logicalPrefix = "tools/support/release/public_release_contracts/";
for (const input of custody.frozenInputs) {
  assert.ok(input.path.startsWith(logicalPrefix));
  assert.equal(
    sha256(fs.readFileSync(path.join(capsule, input.path.slice(logicalPrefix.length)))),
    input.sha256,
    input.path,
  );
}
assert.equal(
  sha256(fs.readFileSync(path.join(capsule, "diagnostic-reference-0.600.json.txt"))),
  "c0bc78179f9d0fa1b06b608862f7dd6298f272c6cbc769b7fef304f98d4fc41b",
);

const identities = ["vize", "@vizejs/native", "@vizejs/wasm", "vue"].map((name) => {
  const root = fs.realpathSync(path.join(workspace, "node_modules", name));
  assert.ok(root.startsWith(workspace + path.sep));
  const manifest = readJson<PackageManifest>(path.join(root, "package.json"));
  assert.equal(manifest.version, name === "vue" ? "3.6.0-rc.13" : version);
  return { name, version: manifest.version, root };
});
const tag = spawnSync(
  "git",
  ["ls-remote", "https://github.com/ubugeeei-prod/vize.git", "refs/tags/v" + version],
  { encoding: "utf8", timeout: 30000 },
);
assert.equal(tag.status, 0, tag.stderr);
const tagFields = tag.stdout.trim().split(/\s+/);
assert.equal(tagFields[0], candidate);
assert.equal(tagFields[1], "refs/tags/v" + version);
fs.writeFileSync(
  path.join(out, "tag-identity.json"),
  JSON.stringify(
    { version, candidate, status: tag.status, stdout: tag.stdout, stderr: tag.stderr },
    null,
    2,
  ),
);
const calls: Array<[string, string[]]> = [
  [
    "public-contribution-contracts.ts",
    [
      workspace,
      path.join(capsule, "fixtures"),
      path.join(out, "public-contribution-contracts.json"),
    ],
  ],
  [
    "runtime-contracts.ts",
    [workspace, path.join(out, "public-compiler-contracts-admitted.json"), version, "native"],
  ],
  [
    "runtime-contracts.ts",
    [workspace, path.join(out, "public-wasm-compiler-contracts.json"), version, "wasm"],
  ],
  ["public-config-recovery.ts", [workspace, path.join(out, "public-config-recovery.json")]],
  [
    "public-dynamic-role.ts",
    [workspace, version, path.join(out, "public-dynamic-role-10926.json")],
  ],
];
const processPackets: ProcessRecord[] = [];
for (const [script, args] of calls) {
  const result = spawnSync(process.execPath, [path.join(capsule, script), ...args], {
    cwd: workspace,
    encoding: "utf8",
    timeout: 180000,
    maxBuffer: 8 * 1024 * 1024,
  });
  const packet: ProcessRecord = {
    script,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdout: result.stdout,
    stderr: result.stderr,
  };
  processPackets.push(packet);
  fs.writeFileSync(path.join(out, "process-packets.json"), JSON.stringify(processPackets, null, 2));
  assert.equal(result.status, 0, script + "\n" + result.stdout + "\n" + result.stderr);
  assert.equal(result.signal, null);
}
const diagnostics = readJson<ContributionResult>(
  path.join(out, "public-contribution-contracts.json"),
);
const reference = readJson<ContributionResult>(
  path.join(capsule, "diagnostic-reference-0.600.json.txt"),
);
assert.equal(diagnostics.native, version);
assert.deepEqual(diagnostics.rows, reference.rows);
const native = readJson<RuntimeResult>(path.join(out, "public-compiler-contracts-admitted.json"));
const wasm = readJson<RuntimeResult>(path.join(out, "public-wasm-compiler-contracts.json"));
const config = readJson<ConfigResult>(path.join(out, "public-config-recovery.json"));
const role = readJson<RoleResult>(path.join(out, "public-dynamic-role-10926.json"));
assert.equal(native.rows.length, 4);
assert.ok(native.rows.every((row) => row.passed));
assert.equal(wasm.rows.length, 4);
assert.ok(wasm.rows.every((row) => row.passed));
assert.equal(config.rows.length, 6);
assert.ok(config.rows.every((row) => row.equal));
assert.equal(role.positive.warningCount, 0);
assert.equal(role.negative.warningCount, 1);
const require = createRequire(path.join(workspace, "package.json"));
require("@vizejs/native");
const binaries = Object.keys(require.cache)
  .filter((name) => name.endsWith(".node"))
  .map((name) => {
    const file = fs.realpathSync(name);
    assert.ok(file.startsWith(workspace + path.sep));
    const bytes = fs.readFileSync(file);
    return { file, bytes: bytes.length, sha256: sha256(bytes) };
  });
assert.equal(binaries.length, 1);
const binary = binaries[0];
assert.ok(binary);
const target =
  process.platform === "linux"
    ? "linux-" + process.arch
    : process.platform === "darwin"
      ? "darwin-" + process.arch
      : "win32-" + process.arch;
const receiptUrl =
  "https://github.com/ubugeeei-prod/vize/releases/download/v" +
  version +
  "/vize-" +
  target +
  ".vsix.receipt.json";
const response = await fetch(receiptUrl);
assert.equal(response.status, 200);
const receipt = (await response.json()) as PublicProviderReceipt;
assert.equal(receipt.source, candidate);
assert.equal(receipt.version, version);
const providers = receipt.provider.files.filter((file) => file.path.endsWith(".node"));
assert.equal(providers.length, 1);
const provider = providers[0];
assert.ok(provider);
assert.equal(provider.sha256, binary.sha256);
assert.equal(provider.bytes, binary.bytes);
const cli = spawnSync(path.join(workspace, "node_modules/.bin/vize"), ["--version"], {
  encoding: "utf8",
});
assert.equal(cli.status, 0);
assert.equal(cli.stdout.trim(), "vize " + version);
const lock = fs.readFileSync(path.join(workspace, "package-lock.json"));
fs.copyFileSync(path.join(workspace, "package-lock.json"), path.join(out, "package-lock.json"));
for (const name of ["source-manifest.original.json.txt", "source-custody.json.txt"]) {
  fs.copyFileSync(path.join(capsule, name), path.join(out, name));
}
const result = {
  version,
  candidate,
  workspace,
  node: process.version,
  identities,
  binaries,
  harness: { runId: process.env.GITHUB_RUN_ID ?? null, commit: process.env.GITHUB_SHA ?? null },
  publicReceipt: { url: receiptUrl, source: receipt.source, provider: providers },
  lockSha256: sha256(lock),
  cli: { status: cli.status, stdout: cli.stdout, stderr: cli.stderr },
  counts: {
    completeDiagnosticPackets: 6,
    nativeMountedControls: 4,
    wasmMountedControls: 4,
    configRecoveryPackets: 6,
    dynamicRolePositiveAndNegative: 2,
  },
  passed: true,
};
fs.writeFileSync(
  path.join(out, "public-consumer-acceptance.json"),
  JSON.stringify(result, null, 2),
);
console.log(JSON.stringify(result));
