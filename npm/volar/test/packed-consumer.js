import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { preparePublishManifest } from "../../../tools/support/release/smoke_release_install/prepare.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const output = path.join(process.env.VIZE_VOLAR_EVIDENCE_DIR, "packed-consumer");
const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-volar-packed-"));
const hash = (file) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const childEnvironment = { ...process.env, VIZE_VOLAR_EVIDENCE_DIR: output };
delete childEnvironment.NAPI_RS_NATIVE_LIBRARY_PATH;
delete childEnvironment.NODE_PATH;
fs.mkdirSync(output, { recursive: true });
const packs = path.join(temporary, "packs");
fs.mkdirSync(packs);
const commands = [];
function run(command, args, cwd, env = childEnvironment) {
  const result = spawnSync(command, args, {
    cwd,
    env,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  });
  commands.push({
    command,
    args,
    cwd,
    status: result.status,
    signal: result.signal,
    stdout: result.stdout,
    stderr: result.stderr,
  });
  fs.writeFileSync(path.join(output, "commands.json"), JSON.stringify(commands, null, 2) + "\n");
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.equal(result.signal, null);
  return result.stdout;
}
function stage(relative) {
  const source = path.join(root, relative);
  const original = fs.readFileSync(path.join(source, "package.json"));
  let manifest;
  try {
    preparePublishManifest(source);
    manifest = JSON.parse(fs.readFileSync(path.join(source, "package.json"), "utf8"));
  } finally {
    fs.writeFileSync(path.join(source, "package.json"), original);
  }
  const target = path.join(temporary, path.basename(relative));
  fs.mkdirSync(target);
  const files = new Set([...manifest.files, "README.md", "LICENSE", "LICENSE.md"]);
  for (const file of files) {
    const filename = path.join(source, file);
    if (fs.existsSync(filename)) fs.cpSync(filename, path.join(target, file), { recursive: true });
  }
  fs.writeFileSync(path.join(target, "package.json"), JSON.stringify(manifest, null, 2) + "\n");
  return target;
}
function pack(directory) {
  const [result] = JSON.parse(
    run("npm", ["pack", "--json", "--ignore-scripts", "--pack-destination", packs], directory),
  );
  const file = path.join(packs, result.filename);
  return { ...result, file, sha256: hash(file) };
}

assert.equal(process.platform, "linux");
assert.equal(process.arch, "x64");
const native = stage("npm/native");
const volar = stage("npm/volar");
const nativeManifest = JSON.parse(fs.readFileSync(path.join(native, "package.json"), "utf8"));
const platform = path.join(temporary, "platform");
fs.mkdirSync(platform);
const imageName = "vize-vitrine.linux-x64-gnu.node";
fs.copyFileSync(process.env.NAPI_RS_NATIVE_LIBRARY_PATH, path.join(platform, imageName));
fs.writeFileSync(
  path.join(platform, "package.json"),
  JSON.stringify(
    {
      name: "@vizejs/native-linux-x64-gnu",
      version: nativeManifest.version,
      main: imageName,
      files: [imageName],
      os: ["linux"],
      cpu: ["x64"],
      libc: ["glibc"],
      license: "MIT",
    },
    null,
    2,
  ) + "\n",
);
const packages = [pack(platform), pack(native), pack(volar)];
const consumer = path.join(temporary, "consumer");
fs.mkdirSync(consumer);
fs.writeFileSync(
  path.join(consumer, "package.json"),
  JSON.stringify(
    {
      private: true,
      type: "module",
      dependencies: {
        ...Object.fromEntries(packages.map((entry) => [entry.name, `file:${entry.file}`])),
        typescript: "6.0.3",
        vue: "3.5.42",
      },
    },
    null,
    2,
  ) + "\n",
);
run("npm", ["install", "--ignore-scripts", "--no-audit", "--no-fund"], consumer);
fs.writeFileSync(path.join(output, "packages.json"), JSON.stringify(packages, null, 2) + "\n");
const probe = `
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import * as plugin from '@vizejs/volar-plugin';
const require = createRequire(import.meta.url);
const native = require('@vizejs/native');
const identity = require('@vizejs/native/native-binding-identity.js').getBindingIdentity(native);
assert.equal(identity.kind, 'native');
assert.equal(identity.token, process.env.VIZE_VOLAR_EXPECTED_NATIVE_TOKEN);
assert.equal(native.getNativeBuildToken(), identity.token);
const binary = require.resolve('@vizejs/native-linux-x64-gnu');
assert.equal(require.cache[binary]?.exports, native);
const hash = file => createHash('sha256').update(fs.readFileSync(file)).digest('hex');
assert.equal(hash(binary), process.env.VIZE_VOLAR_PACKED_IMAGE_SHA);
assert.equal(hash(require.resolve('@vizejs/native/language-projection.js')), process.env.VIZE_VOLAR_PACKED_HELPER_SHA);
assert.deepEqual(Object.keys(plugin).sort(), ['VizeVirtualCode','createVizeLanguagePlugin','createVizeLanguageService']);
fs.writeFileSync(process.env.VIZE_VOLAR_PACKED_RECEIPT, JSON.stringify({identity, binary, binarySha256:hash(binary),
  nativeEntry: require.resolve('@vizejs/native'), pluginEntry: require.resolve('@vizejs/volar-plugin'),
  helperSha256:hash(require.resolve('@vizejs/native/language-projection.js')), exports:Object.keys(plugin).sort()}, null, 2)+'\\n');
`;
fs.writeFileSync(path.join(consumer, "probe.js"), probe);
childEnvironment.VIZE_VOLAR_PACKED_IMAGE_SHA = hash(process.env.NAPI_RS_NATIVE_LIBRARY_PATH);
childEnvironment.VIZE_VOLAR_PACKED_HELPER_SHA = hash(
  path.join(root, "npm/native/language-projection.js"),
);
childEnvironment.VIZE_VOLAR_PACKED_RECEIPT = path.join(output, "loaded-image.json");
run(process.execPath, ["probe.js"], consumer);
const tests = path.join(consumer, "test");
fs.mkdirSync(tests);
for (const file of ["service.test.js", "service-harness.js"]) {
  const sourceTest = fs.readFileSync(path.join(root, "npm/volar/test", file), "utf8");
  assert(sourceTest.includes('from "../src/index.js"'));
  fs.writeFileSync(
    path.join(tests, file),
    sourceTest.replace('from "../src/index.js"', 'from "@vizejs/volar-plugin"'),
  );
}
for (const file of [
  "expected-packets",
  "expected-packets.provenance.json",
  "public-instance-reference.js",
])
  fs.cpSync(path.join(root, "npm/volar/test", file), path.join(tests, file), {
    recursive: true,
  });
run(process.execPath, ["--test", "test/service.test.js"], consumer);
fs.copyFileSync(
  path.join(consumer, "package-lock.json"),
  path.join(output, "consumer-package-lock.json"),
);
const retainedPackages = path.join(output, "packages");
fs.mkdirSync(retainedPackages);
for (const entry of packages)
  fs.copyFileSync(entry.file, path.join(retainedPackages, entry.filename));
assert.equal(
  fs.readFileSync(path.join(output, "packets.json"), "utf8"),
  fs.readFileSync(path.join(process.env.VIZE_VOLAR_EVIDENCE_DIR, "packets.json"), "utf8"),
);
