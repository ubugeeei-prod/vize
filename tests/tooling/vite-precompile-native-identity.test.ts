import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const worker = `
import fs from 'node:fs';
import crypto from 'node:crypto';
import { createRequire } from 'node:module';
const require = createRequire(process.env.IDENTITY_FIXTURE + '/worker.cjs');
require.extensions['.node'] = (module, filename) => {
  const body = fs.readFileSync(filename, 'utf8');
  const token = crypto.createHash('sha256').update(body).digest('hex').slice(0, 32);
  module.exports = process.env.LEGACY_PROVIDER === '1' ? { body } : { body, getNativeBuildToken: () => token };
};
const { computePrecompileCacheKey: key } = await import(process.env.IDENTITY_FIXTURE + '/key.ts');
const first = key({ ssr: false, vapor: true });
const binding = require('@vizejs/native');
const reordered = key({ vapor: true, ssr: false });
const differentOptions = key({ vapor: false, ssr: false });
if (process.env.REPLACE_BODY === '1') {
  const binary = require.resolve('@vizejs/native/provider.node');
  const before = fs.statSync(binary);
  fs.writeFileSync(binary, 'new-provider');
  fs.utimesSync(binary, before.atime, before.mtime);
}
const again = key({ ssr: false, vapor: true });
console.log(JSON.stringify({ first, reordered, differentOptions, again, body: binding.body }));
`;

function fixture(withIdentity: boolean): string {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-key-native-"));
  const native = path.join(directory, "node_modules/@vizejs/native");
  fs.mkdirSync(native, { recursive: true });
  fs.writeFileSync(path.join(directory, "package.json"), '{"type":"module"}');
  fs.copyFileSync(
    path.join(root, "npm/builder/vite/src/plugin/precompile-cache-key.ts"),
    path.join(directory, "key.ts"),
  );
  fs.writeFileSync(path.join(native, "package.json"), '{"version":"0.443.0","main":"index.cjs"}');
  fs.writeFileSync(path.join(native, "provider.node"), "old-provider");
  if (withIdentity) {
    fs.copyFileSync(
      path.join(root, "npm/native/native-binding-identity.js"),
      path.join(native, "native-binding-identity.js"),
    );
  }
  fs.writeFileSync(
    path.join(native, "index.cjs"),
    withIdentity
      ? "module.exports = require('./native-binding-identity').requireWithBindingIdentity(require, './provider.node');"
      : "module.exports = require('./provider.node');",
  );
  return directory;
}

function observe(directory: string, replace = false, legacy = false) {
  const result = spawnSync(process.execPath, ["--input-type=module", "--eval", worker], {
    env: {
      ...process.env,
      IDENTITY_FIXTURE: directory,
      REPLACE_BODY: replace ? "1" : "0",
      LEGACY_PROVIDER: legacy ? "1" : "0",
    },
    encoding: "utf8",
  });
  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
  return JSON.parse(result.stdout) as {
    first: string;
    reordered: string;
    differentOptions: string;
    again: string;
    body: string;
  };
}

test("same-version cache keys follow actual image tokens across hot and fresh processes", () => {
  const directory = fixture(true);
  try {
    const before = observe(directory, true);
    assert.equal(before.body, "old-provider");
    assert.equal(before.first, before.again, "disk replacement cannot relabel old memory");
    assert.equal(before.first, before.reordered);
    assert.notEqual(before.first, before.differentOptions);
    const after = observe(directory);
    assert.equal(after.body, "new-provider");
    assert.notEqual(after.first, before.first, "same package version must not restore old output");
    assert.equal(
      observe(directory).first,
      after.first,
      "unchanged native bodies retain disk reuse",
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("older wrappers without identity support cannot reuse another process's cache", () => {
  const directory = fixture(false);
  try {
    const first = observe(directory);
    assert.equal(first.first, first.again);
    assert.notEqual(first.first, observe(directory).first);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("legacy native images behind the new wrapper cannot authorize cross-process persistence", () => {
  const directory = fixture(true);
  try {
    const first = observe(directory, false, true);
    assert.equal(first.first, first.again);
    assert.notEqual(first.first, observe(directory, false, true).first);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
