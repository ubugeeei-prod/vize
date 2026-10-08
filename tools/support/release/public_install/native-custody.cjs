"use strict";
// Observe the original Node extension loader. Preserve its this, arguments,
// exports, exception and return value; never replace native functions.
const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const Module = require("node:module");
for (const key of [
  "NODE_OPTIONS",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "VIZE_OXLINT_NATIVE_CUSTODY",
]) {
  if (process.env[key]) throw new Error("Initial source/native override must be empty: " + key);
}
const config = JSON.parse(process.env.VIZE_PUBLIC_NATIVE_CUSTODY || "null");
if (!config || config.schema !== "vize-public-native-custody-v1")
  throw new Error("Missing native custody configuration");
for (const key of ["installRoot", "nativePath", "journalPath"]) {
  if (typeof config[key] !== "string" || !path.isAbsolute(config[key]))
    throw new Error("Native custody paths must be absolute");
}
if (!/^[0-9a-f]{64}$/.test(config.nativeSha256)) throw new Error("Missing exact native digest");
const installRoot = fs.realpathSync(config.installRoot);
const nativePath = fs.realpathSync(config.nativePath);
if (
  installRoot !== config.installRoot ||
  nativePath !== config.nativePath ||
  !nativePath.startsWith(installRoot + path.sep) ||
  !fs.statSync(nativePath).isFile() ||
  path.extname(nativePath) !== ".node"
)
  throw new Error("Native custody escaped the registry installation");
if (
  fs.realpathSync(path.dirname(config.journalPath)) !== path.dirname(config.journalPath) ||
  config.journalPath.startsWith(installRoot + path.sep)
)
  throw new Error("Journal path must be canonical and outside installed payloads");
const digest = (filename) =>
  crypto.createHash("sha256").update(fs.readFileSync(filename)).digest("hex");
if (digest(nativePath) !== config.nativeSha256) throw new Error("Native custody digest differs");
const original = Module._extensions[".node"];
if (typeof original !== "function") throw new Error("Original Node native loader is unavailable");
const journal = fs.openSync(config.journalPath, "wx", 0o600);
const write = (event) =>
  fs.writeSync(
    journal,
    JSON.stringify({ schema: "vize-public-native-custody-event-v1", pid: process.pid, ...event }) +
      "\n",
  );
write({
  event: "initialized",
  installRoot,
  nativePath,
  sha256: config.nativeSha256,
  node: process.execPath,
  argv: process.argv,
});
process.on("exit", (code) => write({ event: "exit", code }));
Module._extensions[".node"] = function observeNativeLoad(module, filename) {
  const actualPath = fs.realpathSync(filename);
  const isVize = /(?:vize[-_](?:native|vitrine)|@vizejs[/\\]native)/.test(actualPath);
  const expected = actualPath === nativePath;
  const actualSha256 = digest(actualPath);
  write({
    event: "attempt",
    actualPath,
    sha256: actualSha256,
    expectedNative: expected,
    corsaPath: process.env.CORSA_PATH || "",
  });
  if ((isVize && !expected) || (expected && actualSha256 !== config.nativeSha256)) {
    write({ event: "rejected", actualPath, reason: "native registry identity differs" });
    throw new Error("Vize native loader escaped the public registry binary");
  }
  let result;
  try {
    result = Reflect.apply(original, this, arguments);
  } catch (error) {
    write({ event: "failed", actualPath, message: String((error && error.message) || error) });
    throw error;
  }
  const returnedSha256 = digest(actualPath);
  if (expected && returnedSha256 !== config.nativeSha256) {
    write({ event: "rejected", actualPath, reason: "native bytes changed during load" });
    throw new Error("Native bytes changed during loading");
  }
  write({
    event: "returned",
    actualPath,
    sha256: returnedSha256,
    expectedNative: expected,
    corsaPath: process.env.CORSA_PATH || "",
  });
  return result;
};
