// Test-only hook: qualify the physical addon and retain whole native calls.
const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const fs = require("node:fs");
const Module = require("node:module");
const path = require("node:path");

const custody = JSON.parse(fs.readFileSync(process.env.VIZE_OXLINT_NATIVE_CUSTODY, "utf8"));
assert.equal(custody.schema, "vize.oxlint.source-native");
assert.equal(custody.version, 1);
assert.ok(!process.env.NAPI_RS_NATIVE_LIBRARY_PATH);
assert.ok(!process.env.NAPI_RS_FORCE_WASI);
const binary = fs.realpathSync(custody.binary.path);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
assert.equal(hash(fs.readFileSync(binary)), custody.binary.sha256);
const record = (event) =>
  fs.appendFileSync(
    custody.calls,
    JSON.stringify({
      ...event,
      pid: process.pid,
      source: custody.source,
      binary,
      binarySha256: custody.binary.sha256,
      control: process.env.VIZE_OXLINT_NATIVE_CONTROL ?? null,
      hasRuleHint: process.env.VIZE_OXLINT_NATIVE_HINT === "true",
    }) + "\n",
  );
const load = Module._extensions[".node"];
Module._extensions[".node"] = function (module, filename) {
  const actual = fs.realpathSync(filename);
  if (actual !== binary && !/^vize[-_]vitrine.*\.node$/u.test(path.basename(actual)))
    return load(module, filename);
  load(module, binary);
  record({ kind: "load", requested: actual });
  const descriptors = Object.getOwnPropertyDescriptors(module.exports);
  const lint = descriptors.lintPatinaSfc?.value;
  assert.equal(typeof lint, "function");
  descriptors.lintPatinaSfc.value = function (...args) {
    assert.equal(typeof args[0], "string");
    assert.equal(typeof args[1]?.filename, "string");
    const original = fs.readFileSync(args[1].filename);
    assert.ok(original.equals(Buffer.from(args[0], "utf8")), "native input lost original bytes");
    let result;
    try {
      result = Reflect.apply(lint, this, args);
    } catch (error) {
      record({ kind: "call", args, outcome: "throw", error: String(error) });
      throw error;
    }
    record({ kind: "call", args, outcome: "return", result, originalSha256: hash(original) });
    return result;
  };
  module.exports = Object.create(Object.getPrototypeOf(module.exports), descriptors);
};
