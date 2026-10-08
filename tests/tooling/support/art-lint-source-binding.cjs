// Record the actual source addon used by the public Oxlint plugin process.
const assert = require("node:assert/strict");
const crypto = require("node:crypto");
const fs = require("node:fs");
const Module = require("node:module");
const path = require("node:path");

const binary = fs.realpathSync(process.env.VIZE_ART_SOURCE_BINDING);
const sha256 = crypto.createHash("sha256").update(fs.readFileSync(binary)).digest("hex");
assert.equal(sha256, process.env.VIZE_ART_SOURCE_BINDING_SHA256);
const append = (event) =>
  fs.appendFileSync(
    process.env.VIZE_ART_NATIVE_CALLS,
    JSON.stringify({ pid: process.pid, binary, sha256, ...event }) + "\n",
  );
// The generic NAPI override must never affect VP/Oxlint's own native addons.
const index = fs.realpathSync(path.join(process.env.VIZE_ART_NATIVE_DIRECTORY, "index.js"));
const requireModule = Module._load;
Module._load = function (request, parent, isMain) {
  if (parent?.filename !== index || !/^\.\/native-binding(?:\.js)?$/u.test(request))
    return requireModule.call(this, request, parent, isMain);
  const previous = process.env.NAPI_RS_NATIVE_LIBRARY_PATH;
  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = binary;
  try {
    return requireModule.call(this, request, parent, isMain);
  } finally {
    if (previous === undefined) delete process.env.NAPI_RS_NATIVE_LIBRARY_PATH;
    else process.env.NAPI_RS_NATIVE_LIBRARY_PATH = previous;
  }
};
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
