import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire(import.meta.url);
const native = require("@vizejs/native");
const { getBindingIdentity } = require("@vizejs/native/native-binding-identity.js");
assert.equal(typeof native.generateVueLanguageCode, "function");
const identity = getBindingIdentity(native);
assert.equal(identity.kind, "native");
assert.equal(native.getNativeBuildToken(), identity.token);
const source = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
assert.equal(source, process.env.VIZE_SOURCE_SHA);
execFileSync("git", ["diff", "--exit-code"]);
const binary = process.env.NAPI_RS_NATIVE_LIBRARY_PATH;
assert(binary?.endsWith(".node"));
const sha256 = (file) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const packages = Object.fromEntries(
  ["@volar/language-core", "@volar/typescript", "typescript", "vue"].map((name) => {
    const file = require.resolve(`${name}/package.json`);
    return [
      name,
      {
        version: JSON.parse(fs.readFileSync(file, "utf8")).version,
        packageJsonSha256: sha256(file),
      },
    ];
  }),
);
const receipt = {
  source,
  githubWorkflowSha: process.env.GITHUB_SHA,
  run: process.env.GITHUB_RUN_ID,
  platform: process.platform,
  arch: process.arch,
  node: process.version,
  identity,
  binary: { path: binary, sha256: sha256(binary), bytes: fs.statSync(binary).size },
  packages,
  sourceFiles: Object.fromEntries(
    [
      "crates/vize_vitrine/src/napi/language_code.rs",
      "npm/native/language-projection.js",
      "npm/volar/src/language-plugin.js",
      "npm/volar/src/language-service.js",
      "npm/volar/test/service.test.js",
    ].map((file) => [file, sha256(file)]),
  ),
};
fs.mkdirSync(process.env.VIZE_VOLAR_EVIDENCE_DIR, { recursive: true });
fs.writeFileSync(
  path.join(process.env.VIZE_VOLAR_EVIDENCE_DIR, "source-receipt.json"),
  JSON.stringify(receipt, null, 2) + "\n",
);
