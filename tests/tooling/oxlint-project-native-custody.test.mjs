// Filesystem/process controls only: the loader below never executes an addon.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const hook = fileURLToPath(
  new URL("../../npm/oxlint/scripts/project-native-custody.cjs", import.meta.url),
);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const program = String.raw`
const assert = require('node:assert/strict');
const fs = require('node:fs');
const Module = require('node:module');
const path = require('node:path');
const [hook, directory, mode] = process.argv.slice(1);
const binary = path.join(directory, 'source.node');
const read = fs.readFileSync;
const readChunk = fs.readSync;
const close = fs.closeSync;
const open = fs.openSync;
const nativeDescriptors = new Set();
const original = read(binary);
const stats = fs.statSync(binary);
let reads = 0, bytes = 0, largest = 0, closed = 0, loads = 0;
fs.readFileSync = function(file, ...args) {
  if (mode !== 'mutated')
    assert.notEqual(String(file), binary, 'a whole native-image allocation is forbidden');
  return Reflect.apply(read, this, [file, ...args]);
};
fs.openSync = function(file, flags, ...args) {
  const descriptor = Reflect.apply(open, this, [file, flags, ...args]);
  if (String(file) === binary && flags === 'r') nativeDescriptors.add(descriptor);
  return descriptor;
};
fs.readSync = function(fd, buffer, offset, length, position) {
  if (!nativeDescriptors.has(fd))
    return Reflect.apply(readChunk, this, [fd, buffer, offset, length, position]);
  reads++;
  largest = Math.max(largest, length);
  if (mode === 'read-error') throw Error('authored physical read failure');
  const result = Reflect.apply(readChunk, this, [fd, buffer, offset, length, position]);
  bytes += result;
  if (mode === 'during-read' && reads === 1) {
    const changed = Buffer.from(original);
    changed[changed.length - 1] ^= 1;
    fs.writeFileSync(binary, changed);
    fs.utimesSync(binary, stats.atime, stats.mtime);
  }
  if (mode === 'symlink-read' && reads === 1) {
    fs.renameSync(binary, binary + '.previous');
    fs.symlinkSync(binary + '.previous', binary);
  }
  if (mode === 'replace-path' && reads === 1) {
    fs.renameSync(binary, binary + '.previous');
    fs.writeFileSync(binary, original);
    fs.utimesSync(binary, stats.atime, stats.mtime);
  }
  return result;
};
fs.closeSync = function(fd) {
  if (nativeDescriptors.delete(fd)) closed++;
  return Reflect.apply(close, this, [fd]);
};
Module._extensions['.node'] = function(module, filename) {
  loads++;
  module.exports = { lintPatinaSfc(source, options) { return {source, options}; } };
};
process.env.VIZE_OXLINT_NATIVE_CUSTODY = path.join(directory, 'custody.json');
delete process.env.NAPI_RS_NATIVE_LIBRARY_PATH;
delete process.env.NAPI_RS_FORCE_WASI;
require(hook);
if (mode === 'mutated') { reads = 0; bytes = 0; largest = 0; closed = 0; }
else assert.equal(reads, 0, 'registration makes no native-load qualification claim');
if (mode === 'pure') {
  assert.equal(loads, 0);
} else if (mode === 'unrelated') {
  require(path.join(directory, 'unrelated.node'));
  assert.equal(loads, 1);
  assert.equal(reads, 0);
} else {
  if (mode === 'mutated') {
    const changed = Buffer.from(original);
    changed[changed.length - 1] ^= 1;
    fs.writeFileSync(binary, changed);
    fs.utimesSync(binary, stats.atime, stats.mtime);
    assert.equal(fs.statSync(binary).size, stats.size);
    assert.equal(fs.statSync(binary).mtimeMs, stats.mtimeMs);
  }
  if (mode === 'missing') fs.unlinkSync(binary);
  let error;
  try {
    const native = require(mode === 'alias' ? path.join(directory, 'vize-vitrine.alias.node') : binary);
    const file = path.join(directory, 'original.vue');
    const source = read(file, 'utf8');
    assert.deepEqual(native.lintPatinaSfc(source, { filename: file }), {
      source, options: { filename: file },
    });
  } catch (reason) { error = reason; }
  if (mode === 'valid' || mode === 'alias') {
    if (error) throw error;
    assert.equal(loads, 1);
    assert.equal(bytes, original.length);
    assert.ok(reads >= 3, 'whole body must cross multiple bounded reads and EOF');
    assert.ok(largest <= 64 * 1024);
    assert.equal(closed, 1);
  } else {
    assert.ok(error, 'failed whole-image authentication must refuse');
    assert.equal(loads, 0, 'authentication must precede the actual loader');
    assert.equal(closed, mode === 'missing' ? 0 : 1);
  }
}
process.stdout.write(JSON.stringify({reads, bytes, largest, closed, loads}) + '\n');
`;

function run(mode, selectedHook = hook) {
  const directory = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-custody-control-")),
  );
  const binary = path.join(directory, "source.node");
  const bytes = Buffer.alloc(150_013);
  for (let index = 0; index < bytes.length; index++) bytes[index] = index % 251;
  const original = Buffer.from("<template>unchanged original</template>\n");
  const calls = path.join(directory, "calls.jsonl");
  const source = { head: "authored-process-control", tree: "no-native-qualification" };
  const custody = {
    schema: "vize.oxlint.source-native",
    version: 1,
    source,
    binary: { path: binary, sha256: hash(bytes) },
    calls,
  };
  try {
    fs.writeFileSync(binary, bytes);
    fs.utimesSync(binary, new Date(1_600_000_000_000), new Date(1_600_000_000_000));
    fs.writeFileSync(path.join(directory, "unrelated.node"), "unrelated inert addon bytes\n");
    fs.writeFileSync(path.join(directory, "vize-vitrine.alias.node"), "other requested bytes\n");
    fs.writeFileSync(path.join(directory, "original.vue"), original);
    fs.writeFileSync(path.join(directory, "custody.json"), JSON.stringify(custody));
    const environment = { ...process.env };
    delete environment.NODE_OPTIONS;
    delete environment.VIZE_OXLINT_NATIVE_CONTROL;
    delete environment.VIZE_OXLINT_NATIVE_HINT;
    const result = spawnSync(process.execPath, ["-e", program, selectedHook, directory, mode], {
      env: environment,
      timeout: 10_000,
      encoding: "utf8",
      maxBuffer: 1024 * 1024,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stderr, "");
    const packet = JSON.parse(result.stdout);
    const events = fs.existsSync(calls)
      ? fs
          .readFileSync(calls, "utf8")
          .trim()
          .split("\n")
          .map((line) => JSON.parse(line))
      : [];
    if (mode === "valid" || mode === "alias") {
      assert.deepEqual(
        events.map((event) => event.kind),
        ["load", "call"],
      );
      for (const event of events) {
        assert.deepEqual(event.source, source);
        assert.equal(event.binary, binary);
        assert.equal(event.binarySha256, custody.binary.sha256);
        assert.equal(event.pid > 0, true);
        assert.equal(event.control, null);
        assert.equal(event.hasRuleHint, false);
      }
      assert.equal(
        events[0].requested,
        mode === "alias" ? path.join(directory, "vize-vitrine.alias.node") : binary,
      );
      assert.equal(events[1].outcome, "return");
      assert.equal(events[1].originalSha256, hash(original));
      assert.equal(events[1].args[0], original.toString());
      assert.deepEqual(events[1].result, {
        source: original.toString(),
        options: { filename: path.join(directory, "original.vue") },
      });
    } else assert.deepEqual(events, [], "no authenticated Vize-native claim may escape");
    return packet;
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

void test("the complete current image is streamed before loading, with unchanged native receipts", () => {
  const packet = run("valid");
  assert.equal(packet.bytes, 150_013);
  assert.equal(packet.loads, 1);
  assert.equal(run("alias").bytes, 150_013);
});
void test("stock-only registration and unrelated addon loads make no Vize-native claim", () => {
  assert.equal(run("pure").reads, 0);
  assert.equal(run("unrelated").reads, 0);
});
void test("same-size and preserved-mtime mutation after registration refuses before load", () => {
  run("mutated");
});
void test("whole-image open and read failures refuse without a loader or receipt", () => {
  run("missing");
  run("read-error");
});
void test("mutation during hashing and replacement of the physical path both refuse", () => {
  run("during-read");
  run("replace-path");
  run("symlink-read");
});
