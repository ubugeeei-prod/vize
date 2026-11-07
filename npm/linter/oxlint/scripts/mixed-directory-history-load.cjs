// Observe historical public addons without substituting a workspace binary.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const Module = require("node:module");
const childProcess = require("node:child_process");
const { createHash } = require("node:crypto");
const root = fs.realpathSync(process.env.VIZE_OXLINT_HISTORY_INSTALL);
const calls = process.env.VIZE_OXLINT_HISTORY_CALLS;
assert.ok(calls);
const environment = (env = process.env) =>
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
      "NODE_OPTIONS",
      "VIZE_OXLINT_HISTORY_INSTALL",
      "VIZE_OXLINT_HISTORY_CALLS",
      "VIZE_PREFER_WORKSPACE_BINDING",
      "VIZE_OXLINT_NATIVE_CUSTODY",
      "NAPI_RS_NATIVE_LIBRARY_PATH",
      "NAPI_RS_FORCE_WASI",
    ].map((key) => [key, env[key] ?? null]),
  );
for (const key of [
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_OXLINT_NATIVE_CUSTODY",
])
  assert.ok(!process.env[key], `historical public execution cannot use ${key}`);
const load = Module._extensions[".node"];
const record = (event) =>
  fs.appendFileSync(
    calls,
    JSON.stringify({ pid: process.pid, environment: environment(), ...event }) + "\n",
  );
const wholeError = (value) =>
  value instanceof Error
    ? {
        name: value.name,
        ...Object.fromEntries(
          Object.getOwnPropertyNames(value).map((key) => [key, wholeError(value[key])]),
        ),
      }
    : value;
Module._extensions[".node"] = function (module, filename) {
  const file = fs.realpathSync(filename);
  assert.ok(file.startsWith(root + path.sep), "historical addon escaped its actual installation");
  const bytes = fs.readFileSync(file);
  const observation = {
    kind: "load",
    requested: filename,
    file,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    bytes: bytes.length,
    outcome: "pending",
    error: null,
  };
  try {
    load(module, filename);
    observation.outcome = "return";
  } catch (error) {
    observation.outcome = "throw";
    observation.error = wholeError(error);
    throw error;
  } finally {
    record(observation);
  }
};
const hash = (file) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const spawn = childProcess.spawn;
childProcess.spawn = function (executable, args, options) {
  const child = Reflect.apply(spawn, this, [executable, args, options]);
  const stdout = [],
    stderr = [];
  let error = null;
  child.stdout?.on("data", (bytes) => stdout.push(Buffer.from(bytes)));
  child.stderr?.on("data", (bytes) => stderr.push(Buffer.from(bytes)));
  child.on("error", (reason) => {
    error = wholeError(reason);
  });
  child.on("close", (status, signal) =>
    record({
      kind: "host-child",
      environment: environment(options?.env ?? process.env),
      executable,
      args,
      cwd: options?.cwd ?? process.cwd(),
      provider: args?.[0],
      providerSha256: hash(args[0]),
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
  const result = Reflect.apply(spawnSync, this, [executable, args, options]);
  if (args?.length === 2 && args[1] === "--version")
    record({
      kind: "host-handshake",
      environment: environment(options?.env ?? process.env),
      executable,
      args,
      cwd: options?.cwd ?? process.cwd(),
      provider: args[0],
      providerSha256: hash(args[0]),
      status: result.status,
      signal: result.signal,
      error: wholeError(result.error ?? null),
      stdoutBytes: Array.from(Buffer.from(result.stdout ?? "")),
      stderrBytes: Array.from(Buffer.from(result.stderr ?? "")),
    });
  return result;
};
Module.syncBuiltinESMExports();
