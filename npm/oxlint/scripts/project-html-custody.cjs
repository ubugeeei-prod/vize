// Test-only whole observations, layered over the existing authenticated addon owner.
require("./project-native-custody.cjs");
const fs = require("node:fs");
const Module = require("node:module");
const childProcess = require("node:child_process");
const { createHash } = require("node:crypto");
const custody = JSON.parse(fs.readFileSync(process.env.VIZE_OXLINT_NATIVE_CUSTODY, "utf8"));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
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
const record = (event) =>
  fs.appendFileSync(
    custody.calls,
    JSON.stringify({
      ...event,
      pid: process.pid,
      source: custody.source,
      binary: custody.binary.path,
      binarySha256: custody.binary.sha256,
    }) + "\n",
  );
const load = Module._extensions[".node"];
Module._extensions[".node"] = function (module, filename) {
  load(module, filename);
  const descriptors = Object.getOwnPropertyDescriptors(module.exports);
  const lint = descriptors.lintOxlintHtml?.value;
  if (typeof lint !== "function") return;
  descriptors.lintOxlintHtml.value = function (...args) {
    try {
      const result = Reflect.apply(lint, this, args);
      record({ kind: "html-call", args, outcome: "return", result, environment: environment() });
      return result;
    } catch (error) {
      record({
        kind: "html-call",
        args,
        outcome: "throw",
        error: String(error),
        environment: environment(),
      });
      throw error;
    }
  };
  module.exports = Object.create(Object.getPrototypeOf(module.exports), descriptors);
};
const spawn = childProcess.spawn;
childProcess.spawn = function (executable, args, options) {
  const inputs = {};
  for (let index = 0; index < args.length; index++) {
    let file;
    if (["-c", "--config"].includes(args[index])) file = args[++index];
    else if (args[index].startsWith("--config=")) file = args[index].slice(9);
    if (file) {
      const path = require("node:path").resolve(options.cwd, file);
      try {
        inputs[path] = Array.from(fs.readFileSync(path));
      } catch {
        inputs[path] = null;
      }
    }
  }
  const child = Reflect.apply(spawn, this, [executable, args, options]);
  const stdout = [],
    stderr = [];
  let error = null;
  child.stdout?.on("data", (chunk) => stdout.push(Buffer.from(chunk)));
  child.stderr?.on("data", (chunk) => stderr.push(Buffer.from(chunk)));
  child.on("error", (reason) => {
    error ??= reason.message;
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
Module.syncBuiltinESMExports();
const spawnSync = childProcess.spawnSync;
childProcess.spawnSync = function (executable, args, options) {
  const result = Reflect.apply(spawnSync, this, [executable, args, options]);
  if (args?.length === 2 && args[1] === "--version")
    record({
      kind: "host-handshake",
      executable,
      args,
      cwd: options?.cwd ?? process.cwd(),
      environment: environment(),
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdoutBytes: Array.from(Buffer.from(result.stdout ?? "")),
      stderrBytes: Array.from(Buffer.from(result.stderr ?? "")),
      providerSha256: hash(fs.readFileSync(args[0])),
    });
  return result;
};
Module.syncBuiltinESMExports();
