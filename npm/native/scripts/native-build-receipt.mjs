import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const nativeDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const root = path.resolve(nativeDir, "../..");
const requireNative = createRequire(path.join(nativeDir, "package.json"));
const binding = requireNative("./index.js");
const { getBindingIdentity } = requireNative("./native-binding-identity.js");
const identity = getBindingIdentity(binding);
assert.equal(identity.kind, "native");
assert.match(identity.token, /^[0-9a-f]{32}$/);
assert.equal(binding.getNativeBuildToken(), identity.token);
assert.equal(binding.getNativeBuildToken(), identity.token);
const binaries = fs.readdirSync(nativeDir).filter((name) => name.endsWith(".node"));
assert.equal(binaries.length, 1);
const binaryPath = path.join(nativeDir, binaries[0]);
const hash = createHash("sha256");
for await (const chunk of fs.createReadStream(binaryPath)) hash.update(chunk);
const source = execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
assert.equal(source, process.env.GITHUB_SHA);
const scriptPaths = [
  "crates/vize_vitrine/build.rs",
  "crates/vize_vitrine/src/napi.rs",
  "crates/vize_vitrine/src/napi/build_token.rs",
  "npm/native/scripts/native-build-environment.mjs",
  "npm/native/scripts/build-ci.mjs",
  "npm/native/scripts/build-local.mts",
  "npm/native/scripts/native-build-receipt.mjs",
  "npm/native/native-binding.js",
  "npm/native/native-binding-identity.js",
  "npm/native/native-targets.js",
  "npm/native/index.js",
  "npm/native/index.d.ts",
  "npm/builder/vite/src/plugin/precompile-cache-key.ts",
  "tools/support/github/build-napi-package.ts",
  "tools/support/command-runtime.ts",
  "tests/tooling/native-build-token.test.ts",
  ".github/workflows/native-smoke.yml",
];
const receipt = {
  schema: "vize.native-image-build-token.v1",
  scope: "selected host source producer; no whole-matrix or publication credit",
  source,
  runId: process.env.GITHUB_RUN_ID,
  runAttempt: process.env.GITHUB_RUN_ATTEMPT,
  node: process.version,
  platform: process.platform,
  arch: process.arch,
  identity,
  binary: {
    filename: binaries[0],
    bytes: fs.statSync(binaryPath).size,
    sha256: hash.digest("hex"),
  },
  sourceFiles: Object.fromEntries(
    scriptPaths.map((file) => [
      file,
      createHash("sha256")
        .update(fs.readFileSync(path.join(root, file)))
        .digest("hex"),
    ]),
  ),
};
const directory = path.join(nativeDir, ".artifacts/native");
fs.mkdirSync(directory, { recursive: true });
fs.writeFileSync(
  path.join(directory, "build-token-receipt.json"),
  JSON.stringify(receipt, null, 2) + "\n",
);
