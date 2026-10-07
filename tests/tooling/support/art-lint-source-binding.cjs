// Record the actual source addon used by the public Oxlint plugin process.
const assert = require("node:assert/strict");
const crypto = require("node:crypto");
const fs = require("node:fs");
const Module = require("node:module");

const binary = fs.realpathSync(process.env.VIZE_ART_SOURCE_BINDING);
const sha256 = crypto.createHash("sha256").update(fs.readFileSync(binary)).digest("hex");
assert.equal(sha256, process.env.VIZE_ART_SOURCE_BINDING_SHA256);
const append = (event) =>
  fs.appendFileSync(
    process.env.VIZE_ART_NATIVE_CALLS,
    JSON.stringify({ pid: process.pid, binary, sha256, ...event }) + "\n",
  );
const load = Module._extensions[".node"];
Module._extensions[".node"] = (module, filename) => {
  load(module, filename);
  if (fs.realpathSync(filename) !== binary) return;
  append({ kind: "load" });
  const lint = module.exports.lintPatinaSfc;
  assert.equal(typeof lint, "function");
  module.exports = {
    ...module.exports,
    lintPatinaSfc(...args) {
      const result = lint(...args);
      append({ kind: "call", args, result });
      return result;
    },
  };
};
