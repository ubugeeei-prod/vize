// Test-only physical source selection for the existing Nuxt/Oxlint integration.
const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const fs = require("node:fs");
const Module = require("node:module");
const path = require("node:path");

const custody = JSON.parse(fs.readFileSync(process.env.VIZE_NUXT_LINT_NATIVE_CUSTODY, "utf8"));
assert.equal(custody.schema, "vize.nuxt.lint-source-binding");
assert.equal(custody.version, 1);
assert.ok(!process.env.NAPI_RS_NATIVE_LIBRARY_PATH);
assert.ok(!process.env.NAPI_RS_FORCE_WASI);
const binary = fs.realpathSync(custody.binary.path);
const sha256 = createHash("sha256").update(fs.readFileSync(binary)).digest("hex");
assert.equal(sha256, custody.binary.sha256);
const record = (event) =>
  fs.appendFileSync(
    custody.calls,
    JSON.stringify({
      ...event,
      pid: process.pid,
      sourceHead: custody.source.head,
      binary,
      sha256,
    }) + "\n",
  );
const original = Module._extensions[".node"];
Module._extensions[".node"] = function (module, filename) {
  const actual = fs.realpathSync(filename);
  if (actual !== binary && !/^vize[-_]vitrine.*\.node$/.test(path.basename(actual))) {
    return original(module, filename);
  }
  original(module, binary);
  record({ kind: "load", requested: actual });
  const descriptors = Object.getOwnPropertyDescriptors(module.exports);
  for (const entrypoint of ["lintPatinaSfc", "getPatinaRules"]) {
    const fn = descriptors[entrypoint]?.value;
    assert.equal(typeof fn, "function", `missing source linter ${entrypoint}`);
    descriptors[entrypoint].value = function (...args) {
      let result;
      try {
        result = Reflect.apply(fn, this, args);
      } catch (error) {
        record({ kind: "call", entrypoint, args, outcome: "throw", error: String(error) });
        throw error;
      }
      record({ kind: "call", entrypoint, args, outcome: "return", result });
      return result;
    };
  }
  module.exports = Object.create(Object.getPrototypeOf(module.exports), descriptors);
};
