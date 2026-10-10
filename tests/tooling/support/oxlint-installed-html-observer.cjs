"use strict";
// Explicit test observer after the unchanged official loader observer. Never redirect a native load.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const Module = require("node:module");
const childProcess = require("node:child_process");
const { createHash } = require("node:crypto");
const config = JSON.parse(process.env.VIZE_OXLINT_PUBLIC_CUSTODY || "null");
assert.equal(config?.schema, "vize.oxlint.public-html-observer-v1");
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const wholeError = (error) =>
  error instanceof Error
    ? {
        name: error.name,
        ...Object.fromEntries(
          Object.getOwnPropertyNames(error).map((key) => [key, wholeError(error[key])]),
        ),
      }
    : error;
const binary = fs.realpathSync(config.nativePath);
assert.equal(binary, config.nativePath);
assert.ok(binary.startsWith(fs.realpathSync(config.installRoot) + path.sep));
assert.equal(hash(fs.readFileSync(binary)), config.nativeSha256);
assert.equal(hash(fs.readFileSync(__filename)), config.observerSha256);
assert.ok(path.isAbsolute(config.eventsPath));
assert.equal(fs.realpathSync(path.dirname(config.eventsPath)), path.dirname(config.eventsPath));
const journal = fs.openSync(config.eventsPath, "wx", 0o600);
const record = (event) =>
  fs.writeSync(
    journal,
    JSON.stringify({
      ...event,
      pid: process.pid,
      source: config.source,
      binary,
      binarySha256: config.nativeSha256,
      observerSha256: config.observerSha256,
    }) + "\n",
  );
const environment = () =>
  Object.fromEntries(
    [
      "CI",
      "FORCE_COLOR",
      "NO_COLOR",
      "GITHUB_ACTIONS",
      "AI_AGENT",
      "CURSOR_AGENT",
      "CLAUDECODE",
      "CLAUDE_CODE",
      "REPL_ID",
      "GEMINI_CLI",
      "CODEX_SANDBOX",
      "CODEX_THREAD_ID",
      "COPILOT_CLI",
      "OPENCODE",
      "JUNIE_DATA",
      "JUNIE_SHIM_PATH",
      "PATH",
      "EDITOR",
      "TERM_PROGRAM",
    ].map((key) => [key, process.env[key] ?? null]),
  );
record({ kind: "observer-initialized", node: process.execPath, argv: process.argv });
process.on("exit", (code) => record({ kind: "observer-exit", code }));
const load = Module._extensions[".node"];
Module._extensions[".node"] = function (module, filename) {
  const result = Reflect.apply(load, this, arguments);
  if (fs.realpathSync(filename) !== binary) return result;
  assert.equal(hash(fs.readFileSync(binary)), config.nativeSha256);
  const descriptors = Object.getOwnPropertyDescriptors(module.exports);
  const lint = descriptors.lintOxlintHtml?.value;
  assert.equal(typeof lint, "function", "actual released provider must contain the HTML route");
  descriptors.lintOxlintHtml.value = function (...args) {
    try {
      const outcome = Reflect.apply(lint, this, args);
      record({
        kind: "html-call",
        args,
        outcome: "return",
        result: outcome,
        environment: environment(),
      });
      return outcome;
    } catch (error) {
      record({
        kind: "html-call",
        args,
        outcome: "throw",
        error: wholeError(error),
        environment: environment(),
      });
      throw error;
    }
  };
  module.exports = Object.create(Object.getPrototypeOf(module.exports), descriptors);
  return result;
};
const spawn = childProcess.spawn;
childProcess.spawn = function (executable, args, options) {
  const inputs = {};
  for (let index = 0; index < args.length; index++) {
    let file;
    if (["-c", "--config"].includes(args[index])) file = args[++index];
    else if (args[index].startsWith("--config=")) file = args[index].slice(9);
    if (file) {
      const filename = path.resolve(options.cwd, file);
      try {
        inputs[filename] = Array.from(fs.readFileSync(filename));
      } catch {
        inputs[filename] = null;
      }
    }
  }
  const child = Reflect.apply(spawn, this, arguments);
  const stdout = [],
    stderr = [];
  let error = null;
  child.stdout?.on("data", (chunk) => stdout.push(Buffer.from(chunk)));
  child.stderr?.on("data", (chunk) => stderr.push(Buffer.from(chunk)));
  child.on("error", (reason) => {
    error ??= wholeError(reason);
  });
  child.on("close", (status, signal) =>
    record({
      kind: "host-child",
      executable,
      args,
      cwd: options.cwd,
      inputs,
      environment: environment(),
      providerSha256: hash(fs.readFileSync(args[0])),
      status,
      signal,
      error,
      stdoutBytes: Array.from(Buffer.concat(stdout)),
      stderrBytes: Array.from(Buffer.concat(stderr)),
    }),
  );
  return child;
};
const spawnSync = childProcess.spawnSync;
childProcess.spawnSync = function (executable, args, options) {
  const result = Reflect.apply(spawnSync, this, arguments);
  if (args?.length === 2 && args[1] === "--version")
    record({
      kind: "host-handshake",
      executable,
      args,
      cwd: options?.cwd ?? process.cwd(),
      environment: environment(),
      status: result.status,
      signal: result.signal,
      error: result.error ? wholeError(result.error) : null,
      stdoutBytes: Array.from(Buffer.from(result.stdout ?? "")),
      stderrBytes: Array.from(Buffer.from(result.stderr ?? "")),
      providerSha256: hash(fs.readFileSync(args[0])),
    });
  return result;
};
Module.syncBuiltinESMExports();
