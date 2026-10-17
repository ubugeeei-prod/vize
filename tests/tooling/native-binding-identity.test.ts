import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const helper = path.join(root, "npm/native/native-binding-identity.js");

function probe(script: string): void {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-identity-"));
  try {
    // Exercise real filesystem paths through both subprocess boundaries. A
    // literal backslash on Unix also exposes accidental JavaScript escaping.
    const helperDirectory = path.join(directory, "helper with spaces");
    fs.mkdirSync(helperDirectory);
    const helperPath = path.join(
      helperDirectory,
      process.platform === "win32" ? "binding.cjs" : "native\\binding.cjs",
    );
    fs.copyFileSync(helper, helperPath);
    const filename = path.join(directory, "probe.cjs");
    fs.writeFileSync(
      filename,
      `const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { createRequire } = require('node:module');
const helper = process.argv[2];
const requireWithBindingIdentity = (...args) => require(helper).requireWithBindingIdentity(...args);
const getBindingIdentity = (...args) => require(helper).getBindingIdentity(...args);
const localRequire = createRequire(__filename);
const binary = path.join(__dirname, 'provider.node');
const oldBody = 'old-provider';
const newBody = 'new-provider';
fs.writeFileSync(binary, oldBody);
// Only this isolated worker replaces dlopen, exercising Node's real require cache.
require.extensions['.node'] = (module, filename) => {
  const body = fs.readFileSync(filename, 'utf8');
  const token = crypto.createHash('sha256').update(body).digest('hex').slice(0, 32);
  module.exports = Object.freeze({ body, getNativeBuildToken: () => token });
};
${script}
`,
    );
    const result = spawnSync(process.execPath, [filename, helperPath], { encoding: "utf8" });
    assert.ifError(result.error);
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

test("identity belongs to the loaded image and its getter runs only once", () => {
  probe(`
const originalHandler = require.extensions['.node'];
let getterCalls = 0;
require.extensions['.node'] = (module, filename) => {
  originalHandler(module, filename);
  const original = module.exports;
  module.exports = Object.freeze({ body: original.body, getNativeBuildToken: () => { getterCalls++; return original.getNativeBuildToken(); } });
};
process.report.getReport = () => { throw Error('loader must not request a process-wide report'); };
const binding = requireWithBindingIdentity(localRequire, binary);
const identity = getBindingIdentity(binding);
assert.equal(binding.body, oldBody);
assert.equal(identity.kind, 'native');
assert.equal(identity.token, crypto.createHash('sha256').update(oldBody).digest('hex').slice(0, 32));
assert.equal(Object.isFrozen(identity), true);
assert.equal(getterCalls, 1);
const stat = fs.statSync(binary);
fs.writeFileSync(binary, newBody);
fs.utimesSync(binary, stat.atime, stat.mtime);
assert.equal(requireWithBindingIdentity(localRequire, binary), binding);
assert.equal(getBindingIdentity(binding), identity);
assert.equal(binding.body, oldBody);
assert.equal(getterCalls, 1, 'the actual image identity must stay immutable');
const fresh = require('node:child_process').spawnSync(process.execPath, ['-e', \`
  require.extensions['.node'] = (m, f) => {
    const body = require('node:fs').readFileSync(f, 'utf8');
    const token = require('node:crypto').createHash('sha256').update(body).digest('hex').slice(0, 32);
    m.exports = { body, getNativeBuildToken: () => token };
  };
  const h = require(process.argv[1]);
  console.log(JSON.stringify(h.getBindingIdentity(h.requireWithBindingIdentity(require, process.argv[2]))));
\`, helper, binary], { encoding: 'utf8' });
assert.ifError(fresh.error);
assert.equal(fresh.status, 0, fresh.stderr);
const successor = JSON.parse(fresh.stdout);
assert.equal(successor.token, crypto.createHash('sha256').update(newBody).digest('hex').slice(0, 32));
assert.notEqual(successor.token, identity.token);
`);
});

test("uninstrumented and non-native providers fail toward a process identity", () => {
  probe(`
require.extensions['.node'] = (module, filename) => { module.exports = Object.freeze({ body: fs.readFileSync(filename, 'utf8') }); };
const previous = localRequire(binary);
fs.writeFileSync(binary, newBody);
assert.equal(requireWithBindingIdentity(localRequire, binary), previous);
const identity = getBindingIdentity(previous);
assert.equal(identity.kind, 'process');
assert.equal(getBindingIdentity(previous), identity);
assert.equal(previous.body, oldBody);
const custom = path.join(__dirname, 'custom.cjs');
fs.writeFileSync(custom, 'module.exports = Object.freeze({ getNativeBuildToken: () => "00000000000000000000000000000000" });');
const customBinding = requireWithBindingIdentity(localRequire, custom);
const customIdentity = getBindingIdentity(customBinding);
assert.equal(customIdentity.kind, 'process');
assert.notEqual(customIdentity.nonce, identity.nonce);
assert.equal(Object.isFrozen(customIdentity), true);
`);
});

test("a file replaced during successful loading retains the actual image token", () => {
  probe(`
require.extensions['.node'] = (module, filename) => {
  const replacement = filename + '.replacement';
  const stat = fs.statSync(filename);
  fs.writeFileSync(replacement, newBody);
  fs.utimesSync(replacement, stat.atime, stat.mtime);
  fs.renameSync(replacement, filename);
  module.exports = { body: newBody, getNativeBuildToken: () => '11111111111111111111111111111111' };
};
const binding = requireWithBindingIdentity(localRequire, binary);
assert.equal(binding.body, newBody);
assert.deepEqual(getBindingIdentity(binding), { kind: 'native', token: '11111111111111111111111111111111' });
`);
});

test("OS image reuse survives require-cache eviction and independent helper copies", () => {
  probe(`
const images = new Map();
require.extensions['.node'] = (module, filename) => {
  if (!images.has(filename)) images.set(filename, fs.readFileSync(filename, 'utf8'));
  const body = images.get(filename);
  const token = crypto.createHash('sha256').update(body).digest('hex').slice(0, 32);
  module.exports = Object.freeze({ body, getNativeBuildToken: () => token });
};
const before = requireWithBindingIdentity(localRequire, binary);
const first = getBindingIdentity(before);
const helperCopy = path.join(__dirname, 'independent-helper.cjs');
fs.copyFileSync(helper, helperCopy);
const other = localRequire(helperCopy);
assert.equal(other.getBindingIdentity(before), first, 'same-isolate helper copies retain actual binding identity');
fs.writeFileSync(binary, newBody);
delete require.cache[binary];
const reloaded = other.requireWithBindingIdentity(localRequire, binary);
assert.notEqual(reloaded, before);
assert.equal(reloaded.body, oldBody, 'model the independently observed same-path OS image reuse');
assert.equal(other.getBindingIdentity(reloaded).token, first.token);
assert.notEqual(other.getBindingIdentity(reloaded).token, crypto.createHash('sha256').update(newBody).digest('hex').slice(0, 32));
`);
});

test("cached deleted-path bindings preserve successful require and resolver failures refuse persistence", () => {
  probe(`
const previous = localRequire(binary);
fs.unlinkSync(binary);
let calls = 0;
const cachedRequire = (specifier) => { calls++; return localRequire(specifier); };
cachedRequire.resolve = () => { throw Error('original path unavailable'); };
assert.equal(requireWithBindingIdentity(cachedRequire, binary), previous);
assert.equal(calls, 1, 'require must execute exactly once before optional resolution');
assert.equal(getBindingIdentity(previous).kind, 'process');
const failure = Error('original load failure');
const failingRequire = () => { throw failure; };
failingRequire.resolve = () => { throw Error('resolver must not run after failed loading'); };
assert.throws(() => requireWithBindingIdentity(failingRequire, binary), (error) => error === failure);
`);
});
