const assert = require("node:assert/strict");
const fs = require("node:fs");
const Module = require("node:module");
const path = require("node:path");
const [hook, directory, mode] = process.argv.slice(2);
const binary = path.join(directory, "source.node");
const read = fs.readFileSync;
const readChunk = fs.readSync;
const close = fs.closeSync;
const open = fs.openSync;
const nativeDescriptors = new Set();
const original = read(binary);
const stats = fs.statSync(binary);
let reads = 0,
  bytes = 0,
  largest = 0,
  closed = 0,
  loads = 0;
fs.readFileSync = function (file, ...args) {
  if (mode !== "mutated")
    assert.notEqual(String(file), binary, "a whole native-image allocation is forbidden");
  return Reflect.apply(read, this, [file, ...args]);
};
fs.openSync = function (file, flags, ...args) {
  const descriptor = Reflect.apply(open, this, [file, flags, ...args]);
  if (String(file) === binary && flags === "r") nativeDescriptors.add(descriptor);
  return descriptor;
};
fs.readSync = function (fd, buffer, offset, length, position) {
  if (!nativeDescriptors.has(fd))
    return Reflect.apply(readChunk, this, [fd, buffer, offset, length, position]);
  reads++;
  largest = Math.max(largest, length);
  if (mode === "read-error") throw Error("authored physical read failure");
  const result = Reflect.apply(readChunk, this, [fd, buffer, offset, length, position]);
  bytes += result;
  if (mode === "during-read" && reads === 1) {
    const changed = Buffer.from(original);
    changed[changed.length - 1] ^= 1;
    fs.writeFileSync(binary, changed);
    fs.utimesSync(binary, stats.atime, stats.mtime);
  }
  if (mode === "symlink-read" && reads === 1) {
    fs.renameSync(binary, binary + ".previous");
    fs.symlinkSync(binary + ".previous", binary);
  }
  if (mode === "replace-path" && reads === 1) {
    fs.renameSync(binary, binary + ".previous");
    fs.writeFileSync(binary, original);
    fs.utimesSync(binary, stats.atime, stats.mtime);
  }
  return result;
};
fs.closeSync = function (fd) {
  if (nativeDescriptors.delete(fd)) closed++;
  return Reflect.apply(close, this, [fd]);
};
Module._extensions[".node"] = function (module) {
  loads++;
  module.exports = {
    lintPatinaSfc(source, options) {
      return { source, options };
    },
  };
};
process.env.VIZE_OXLINT_NATIVE_CUSTODY = path.join(directory, "custody.json");
delete process.env.NAPI_RS_NATIVE_LIBRARY_PATH;
delete process.env.NAPI_RS_FORCE_WASI;
require(hook);
if (mode === "mutated") {
  reads = 0;
  bytes = 0;
  largest = 0;
  closed = 0;
} else assert.equal(reads, 0, "registration makes no native-load qualification claim");
if (mode === "pure") {
  assert.equal(loads, 0);
} else if (mode === "unrelated") {
  require(path.join(directory, "unrelated.node"));
  assert.equal(loads, 1);
  assert.equal(reads, 0);
} else {
  if (mode === "mutated") {
    const changed = Buffer.from(original);
    changed[changed.length - 1] ^= 1;
    fs.writeFileSync(binary, changed);
    fs.utimesSync(binary, stats.atime, stats.mtime);
    assert.equal(fs.statSync(binary).size, stats.size);
    assert.equal(fs.statSync(binary).mtimeMs, stats.mtimeMs);
  }
  if (mode === "missing") fs.unlinkSync(binary);
  let error;
  try {
    const native = require(
      mode === "alias" ? path.join(directory, "vize-vitrine.alias.node") : binary,
    );
    const file = path.join(directory, "original.vue");
    const source = read(file, "utf8");
    assert.deepEqual(native.lintPatinaSfc(source, { filename: file }), {
      source,
      options: { filename: file },
    });
  } catch (reason) {
    error = reason;
  }
  if (mode === "valid" || mode === "alias") {
    if (error) throw error;
    assert.equal(loads, 1);
    assert.equal(bytes, original.length);
    assert.ok(reads >= 3, "whole body must cross multiple bounded reads and EOF");
    assert.ok(largest <= 64 * 1024);
    assert.equal(closed, 1);
  } else {
    assert.ok(error, "failed whole-image authentication must refuse");
    assert.equal(loads, 0, "authentication must precede the actual loader");
    assert.equal(closed, mode === "missing" ? 0 : 1);
  }
}
process.stdout.write(JSON.stringify({ reads, bytes, largest, closed, loads }) + "\n");
