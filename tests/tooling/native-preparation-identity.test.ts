import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { test } from "node:test";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const helper = "npm/native/scripts/test-preparation.mjs";
const observation = pathToFileURL(join(root, "tests/differential/formatter-vite-observation.mjs"));

function inspectPreparedFixture(
  body: string,
  entrySource = 'module.exports = require("./vize-vitrine.fixture.node");\n',
) {
  const fixture = mkdtempSync(join(tmpdir(), "vize-preparation-identity-"));
  try {
    const native = join(fixture, "npm/native");
    mkdirSync(join(native, "scripts"), { recursive: true });
    mkdirSync(join(fixture, "node_modules/@vizejs"), { recursive: true });
    copyFileSync(join(root, helper), join(fixture, helper));
    writeFileSync(join(fixture, "package.json"), '{"type":"module"}\n');
    writeFileSync(
      join(fixture, ".gitignore"),
      "node_modules/\nnpm/native/.artifacts/\nnpm/native/*.node\n",
    );
    writeFileSync(join(fixture, "tracked-source"), "original source\n");
    writeFileSync(join(native, "package.json"), '{"type":"commonjs","main":"index.js"}\n');
    writeFileSync(join(native, "index.js"), entrySource);
    writeFileSync(join(native, "vize-vitrine.fixture.node"), "fixture addon");
    symlinkSync(native, join(fixture, "node_modules/@vizejs/native"), "dir");
    writeFileSync(
      join(fixture, "inspect.ts"),
      `import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire, syncBuiltinESMExports } from 'node:module';
import { mock } from 'node:test';
import { nativePreparationIsActive, readActiveNativePreparation } from './${helper}';
import { currentAddon } from ${JSON.stringify(observation.href)};
const root = process.cwd();
const native = root + '/npm/native';
const binding = native + '/vize-vitrine.fixture.node';
const file = native + '/.artifacts/native/js-test-preparation.json';
const original = JSON.parse(fs.readFileSync(file, 'utf8'));
const expected = { generated: { name: original.binding }, frozen: { sha256: original.sha256 }, source: original };
const require = createRequire(root + '/package.json');
// Synthetic negative controls: no native execution or formatter credit.
const exports = { runCli() {}, normalizeVizeConfig() {} };
let duringLoad = () => {};
mock.method(process, 'dlopen', (module, filename) => {
  assert.equal(filename, binding);
  module.exports = exports;
  duringLoad();
});
const active = () => readActiveNativePreparation(native);
let custody;
const reject = () => {
  assert.equal(active(), null);
  assert.equal(nativePreparationIsActive(native), false);
  assert.throws(() => custody.observe());
};
try {
  ${body}
} finally {
  mock.restoreAll();
  syncBuiltinESMExports();
}
`,
    );
    const git = (...args: string[]) => execFileSync("git", args, { cwd: fixture, stdio: "ignore" });
    git("init", "-q");
    git("add", ".");
    git(
      "-c",
      "user.name=CI Fixture",
      "-c",
      "user.email=ci@example.invalid",
      "commit",
      "-qm",
      "fixture",
    );
    const result = spawnSync(process.execPath, [helper, process.execPath, "inspect.ts"], {
      cwd: fixture,
      encoding: "utf8",
      timeout: 30_000,
    });
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
}

void test("prepared addon observations freshly verify one hash and initial loads verify it again", () => {
  inspectPreparedFixture(`
    let reads = 0;
    const read = fs.readFileSync;
    mock.method(fs, 'readFileSync', (file, ...args) => {
      if (file === binding) reads++;
      return read(file, ...args);
    });
    syncBuiltinESMExports();
    const initial = currentAddon(root, expected, require);
    assert.equal(reads, 2, 'retain the independent initial post-load hash');
    reads = 0;
    const next = initial.observe();
    assert.equal(reads, 1, 'reuse only the hash freshly verified in this call');
    assert.deepEqual(next, { local: initial.local, entry: initial.entry, preparation: initial.preparation });
    assert.notEqual(next.preparation, initial.preparation);
    next.preparation.sha256 = 'caller mutation';
    assert.deepEqual(active(), original, 'returned identities are not a mutable cache');
    expected.frozen.sha256 = 'caller mutation';
    expected.source = { ...original, head: 'caller mutation' };
    assert.deepEqual(initial.observe().preparation, original, 'the expected identity is private');
    const cached = require.cache[binding];
    delete require.cache[binding];
    assert.throws(() => initial.observe(), /prepared data property changed/);
    require.cache[binding] = { ...cached };
    assert.throws(() => initial.observe(), /prepared module changed/);
    let getterReads = 0;
    const mutateAddon = () => {
      getterReads++;
      fs.writeFileSync(binding, 'changed addon');
      return exports;
    };
    require.cache[binding] = { loaded: true, get exports() { return mutateAddon(); } };
    assert.throws(() => initial.observe(), /prepared module changed/);
    require.cache[binding] = cached;
    const exportsDescriptor = Object.getOwnPropertyDescriptor(cached, 'exports');
    Object.defineProperty(cached, 'exports', { configurable: true, get: mutateAddon });
    assert.throws(() => initial.observe(), /prepared data property changed: exports/);
    Object.defineProperty(cached, 'exports', exportsDescriptor);
    const apiDescriptor = Object.getOwnPropertyDescriptor(exports, 'runCli');
    Object.defineProperty(exports, 'runCli', { configurable: true, get: mutateAddon });
    assert.throws(() => initial.observe(), /prepared data property changed: runCli/);
    Object.defineProperty(exports, 'runCli', apiDescriptor);
    assert.equal(getterReads, 0, 'observations must not execute replacement getters');
    assert.equal(fs.readFileSync(binding, 'utf8'), 'fixture addon');
    assert.deepEqual(initial.observe().preparation, original);
  `);
});

void test("changed addon bytes and source edits cannot reuse a previously verified identity", () => {
  inspectPreparedFixture(`
    custody = currentAddon(root, expected, require);
    const stat = fs.statSync(binding);
    fs.writeFileSync(binding, 'changed addon');
    fs.utimesSync(binding, stat.atime, stat.mtime);
    reject();
    fs.writeFileSync(binding, 'fixture addon');
    assert.deepEqual(active(), original);
    fs.appendFileSync('tracked-source', 'changed');
    reject();
    fs.writeFileSync('tracked-source', 'original source\\n');
    assert.deepEqual(active(), original);
  `);
});

void test("stale owners, forged receipts and unexpected binding names are refused", () => {
  inspectPreparedFixture(`
    custody = currentAddon(root, expected, require);
    for (const change of [
      { ownerPid: 2147483647 }, { ownerPid: process.pid }, { ownerArgument: 'stale owner' },
      { head: 'different' }, { tree: 'different' }, { workingDiff: 'different' },
      { sha256: 'different' }, { binding: 'vize-vitrine.other.node' }, { nativeDir: root },
    ]) {
      fs.writeFileSync(file, JSON.stringify({ ...original, ...change }));
      reject();
    }
    fs.writeFileSync(file, JSON.stringify(original));
    assert.deepEqual(active(), original);
    for (const change of [
      { generated: { name: 'vize-vitrine.other.node' } },
      { frozen: { sha256: 'different' } },
      { source: { ...original, head: 'different' } },
    ]) assert.throws(() => currentAddon(root, { ...expected, ...change }, require));
    fs.writeFileSync(native + '/vize-vitrine.other.node', 'fixture addon');
    reject();
  `);
});

void test("initial loading still refuses addon or preparation changes made during the load", () => {
  inspectPreparedFixture(`
    duringLoad = () => fs.writeFileSync(binding, 'changed addon');
    assert.throws(() => currentAddon(root, expected, require));
    fs.writeFileSync(binding, 'fixture addon');
    delete require.cache[binding];
    delete require.cache[native + '/index.js'];
    duringLoad = () => fs.writeFileSync(file, JSON.stringify({ ...original, id: 'changed during load' }));
    assert.throws(() => currentAddon(root, expected, require), /receipt changed/);
  `);
});

void test("public entry reloads cannot change addon bytes after the initial hash", () => {
  inspectPreparedFixture(
    `
      let loadCalls = 0;
      duringLoad = () => {
        loadCalls++;
        if (loadCalls === 2) fs.writeFileSync(binding, 'changed addon');
      };
      assert.throws(() => currentAddon(root, expected, require), {
        code: 'ERR_ASSERTION',
        actual: require('node:crypto').createHash('sha256').update('changed addon').digest('hex'),
        expected: original.sha256,
      });
      assert.equal(loadCalls, 2, 'mutation occurred inside the second actual provider load');
      assert.equal(fs.readFileSync(binding, 'utf8'), 'changed addon');
    `,
    'delete require.cache[require.resolve("./vize-vitrine.fixture.node")];\n' +
      'module.exports = require("./vize-vitrine.fixture.node");\n',
  );
});
