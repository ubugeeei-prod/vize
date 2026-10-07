// Complete native arguments/results, with exact source bytes stored once by
// digest rather than repeated in every one of the unhinted 51-rule calls.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const Module = require("node:module");
const { createHash } = require("node:crypto");
const custody = JSON.parse(fs.readFileSync(process.env.VIZE_N8N_NATIVE_CUSTODY, "utf8"));
assert.equal(custody.schema, "vize.oxlint.n8n-source-native");
assert.equal(custody.version, 1);
assert.equal(process.env.GITHUB_ACTIONS, "true");
assert.equal(custody.source.head, process.env.GITHUB_SHA);
for (const key of [
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "VIZE_OXLINT_NATIVE_CUSTODY",
])
  assert.ok(!process.env[key], `ambient ${key} is not source authority`);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const binary = fs.realpathSync(custody.binary.path);
assert.equal(hash(fs.readFileSync(binary)), custody.binary.sha256);
fs.mkdirSync(custody.sources, { recursive: true });
const event = (row) =>
  fs.appendFileSync(
    custody.calls,
    JSON.stringify({
      pid: process.pid,
      phase: process.env.VIZE_N8N_REPLAY_PHASE,
      source: custody.source,
      binary,
      binarySha256: custody.binary.sha256,
      ...row,
    }) + "\n",
  );
const load = Module._extensions[".node"];
Module._extensions[".node"] = (module, filename) => {
  const actual = fs.realpathSync(filename);
  if (actual !== binary && !/^vize[-_]vitrine.*\.node$/u.test(path.basename(actual)))
    return load(module, filename);
  assert.equal(hash(fs.readFileSync(binary)), custody.binary.sha256);
  load(module, binary);
  event({ kind: "load", requested: actual });
  const descriptors = Object.getOwnPropertyDescriptors(module.exports);
  assert.equal(typeof descriptors.lintPatinaSfc?.value, "function");
  const lint = descriptors.lintPatinaSfc.value;
  descriptors.lintPatinaSfc.value = (...args) => {
    assert.equal(typeof args[0], "string");
    assert.equal(typeof args[1]?.filename, "string");
    const bytes = Buffer.from(args[0], "utf8");
    assert.ok(
      fs.readFileSync(args[1].filename).equals(bytes),
      "native must lint complete original bytes",
    );
    const sourceSha256 = hash(bytes);
    const sourcePath = path.join(custody.sources, sourceSha256 + ".vue");
    try {
      fs.writeFileSync(sourcePath, bytes, { flag: "wx" });
    } catch (error) {
      if (error.code !== "EEXIST") throw error;
      assert.ok(fs.readFileSync(sourcePath).equals(bytes), "source digest collision or mutation");
    }
    const capturedArgs = [{ sourceSha256, sourcePath }, ...args.slice(1)];
    try {
      const result = Reflect.apply(lint, module.exports, args);
      event({ kind: "call", args: capturedArgs, result, outcome: "return" });
      return result;
    } catch (error) {
      event({ kind: "call", args: capturedArgs, outcome: "throw", error: String(error) });
      throw error;
    }
  };
  module.exports = Object.defineProperties({}, descriptors);
};
