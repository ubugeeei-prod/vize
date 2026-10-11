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
    const filename = path.join(directory, "probe.cjs");
    fs.writeFileSync(
      filename,
      `const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { createRequire } = require('node:module');
const requireWithBindingIdentity = (...args) => require(${JSON.stringify(helper)}).requireWithBindingIdentity(...args);
const getBindingIdentity = (...args) => require(${JSON.stringify(helper)}).getBindingIdentity(...args);
const localRequire = createRequire(__filename);
const binary = path.join(__dirname, 'provider.node');
const oldBody = 'old-provider';
const newBody = 'new-provider';
fs.writeFileSync(binary, oldBody);
// Only this isolated worker replaces dlopen, exercising Node's real require cache.
require.extensions['.node'] = (module, filename) => {
  module.exports = Object.freeze({ body: fs.readFileSync(filename, 'utf8') });
};
${script}
`,
    );
    const result = spawnSync(process.execPath, [filename], { encoding: "utf8" });
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

test("identity belongs to loaded memory and is streamed only on its first load", () => {
  probe(`
const originalRead = fs.readSync;
let reads = 0;
fs.readSync = (...args) => { reads++; return originalRead(...args); };
const binding = requireWithBindingIdentity(localRequire, binary);
const identity = getBindingIdentity(binding);
assert.equal(binding.body, oldBody);
assert.equal(identity.kind, 'native');
assert.equal(identity.sha256, crypto.createHash('sha256').update(oldBody).digest('hex'));
assert.equal(identity.bytes, Buffer.byteLength(oldBody));
assert.equal(Object.isFrozen(identity), true);
const firstReads = reads;
assert.ok(firstReads > 0);
const stat = fs.statSync(binary);
fs.writeFileSync(binary, newBody);
fs.utimesSync(binary, stat.atime, stat.mtime);
assert.equal(requireWithBindingIdentity(localRequire, binary), binding);
assert.equal(getBindingIdentity(binding), identity);
assert.equal(binding.body, oldBody);
assert.equal(reads, firstReads, 'cached memory must never be rehashed from current disk');
const fresh = require('node:child_process').spawnSync(process.execPath, ['-e', \`
  require.extensions['.node'] = (m, f) => m.exports = { body: require('node:fs').readFileSync(f, 'utf8') };
  const h = require(${JSON.stringify(helper)});
  console.log(JSON.stringify(h.getBindingIdentity(h.requireWithBindingIdentity(require, \${JSON.stringify(binary)}))));
\`], { encoding: 'utf8' });
assert.equal(fresh.status, 0, fresh.stderr);
const successor = JSON.parse(fresh.stdout);
assert.equal(successor.sha256, crypto.createHash('sha256').update(newBody).digest('hex'));
assert.notEqual(successor.sha256, identity.sha256);
`);
});

test("previously loaded and non-native providers fail toward a process identity", () => {
  probe(`
const previous = localRequire(binary);
fs.writeFileSync(binary, newBody);
assert.equal(requireWithBindingIdentity(localRequire, binary), previous);
const identity = getBindingIdentity(previous);
assert.equal(identity.kind, 'process');
assert.equal(getBindingIdentity(previous), identity);
assert.equal(previous.body, oldBody);
const custom = path.join(__dirname, 'custom.cjs');
fs.writeFileSync(custom, 'module.exports = Object.freeze({ provider: "custom" });');
const customBinding = requireWithBindingIdentity(localRequire, custom);
const customIdentity = getBindingIdentity(customBinding);
assert.equal(customIdentity.kind, 'process');
assert.notEqual(customIdentity.nonce, identity.nonce);
assert.equal(Object.isFrozen(customIdentity), true);
`);
});

test("a file replaced during successful loading cannot claim its earlier body", () => {
  probe(`
require.extensions['.node'] = (module, filename) => {
  const replacement = filename + '.replacement';
  const stat = fs.statSync(filename);
  fs.writeFileSync(replacement, newBody);
  fs.utimesSync(replacement, stat.atime, stat.mtime);
  fs.renameSync(replacement, filename);
  module.exports = { body: fs.readFileSync(filename, 'utf8') };
};
const binding = requireWithBindingIdentity(localRequire, binary);
assert.equal(binding.body, newBody);
assert.equal(getBindingIdentity(binding).kind, 'process');
`);
});
