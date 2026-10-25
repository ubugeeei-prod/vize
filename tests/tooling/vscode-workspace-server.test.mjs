import assert from "node:assert/strict";
import { execFile, spawn } from "node:child_process";
import fs from "node:fs";
import { stripTypeScriptTypes } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import { discoverWorkspaceServers } from "../../editors/vscode/src/workspace-server.ts";
import { serverInvocation } from "../../editors/vscode/src/server-command.ts";
import {
  parseVizeVersion,
  serverMatchesExtension,
} from "../../editors/vscode/src/extension-core.ts";
import {
  trustedServerPath,
  workspaceDevServerPaths,
} from "../../editors/vscode/src/server-trust.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const exeName = process.platform === "win32" ? "vize.exe" : "vize";
function load(file, dependencies, name) {
  const source = fs.readFileSync(path.join(root, "editors/vscode/src", file), "utf8");
  const body = stripTypeScriptTypes(source)
    .replace(/^import[\s\S]*?;\n/gm, "")
    .replace(/^export /gm, "");
  return new vm.Script(`${body}\n${name};`).runInNewContext({
    Buffer,
    process,
    setTimeout,
    clearTimeout,
    ...dependencies,
  });
}
const inspectWorkspaceServerProtocol = load(
  "workspace-server-protocol.ts",
  { spawn, serverInvocation },
  "inspectWorkspaceServerProtocol",
);

function fixture(t) {
  const directory = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize workspace 日本語 ")),
  );
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const folder = path.join(directory, "project");
  fs.mkdirSync(folder);
  fs.writeFileSync(path.join(folder, "package.json"), "{}");
  const log = path.join(directory, "events.jsonl");
  fs.writeFileSync(log, "");
  const program = fs.readFileSync(
    path.join(root, "editors/vscode/test/fixtures/workspace-protocol-server.cjs"),
    "utf8",
  );
  function server(destination, version = "9.8.7", mode = "valid") {
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(
      destination,
      `#!/usr/bin/env node\nconst options = ${JSON.stringify({ log, version, mode })};\n${program}`,
      { mode: 0o755 },
    );
    return destination;
  }
  function install(base = folder, version = "9.8.7", mode = "valid") {
    const packageRoot = path.join(base, "node_modules", "vize");
    const executable = server(path.join(packageRoot, "bin", "vize.cjs"), version, mode);
    fs.writeFileSync(
      path.join(packageRoot, "package.json"),
      JSON.stringify({
        name: "vize",
        version,
        exports: { ".": "./hidden.cjs" },
        bin: { vize: "bin/vize.cjs" },
      }),
    );
    return executable;
  }
  return {
    directory,
    folder,
    log,
    server,
    install,
    events: () => fs.readFileSync(log, "utf8").trim().split("\n").filter(Boolean).map(JSON.parse),
  };
}

function discovery(
  f,
  folders = [f.folder],
  trusted = true,
  settings = {},
  protocol = inspectWorkspaceServerProtocol,
) {
  const lines = [];
  const workspace = {
    workspaceFolders: folders.map((folder) => ({ uri: { fsPath: folder } })),
    isTrusted: trusted,
  };
  const state = {
    configurationSyncGeneration: 1,
    outputChannel: { appendLine: (line) => lines.push(line) },
  };
  const createServerDiscovery = load(
    "server-discovery.ts",
    {
      fs,
      path,
      process: { ...process, env: {} },
      execFile,
      promisify,
      workspace,
      parseVizeVersion,
      serverMatchesExtension,
      trustedServerPath,
      workspaceDevServerPaths,
      discoverWorkspaceServers,
      inspectWorkspaceServerProtocol: protocol,
      serverInvocation,
    },
    "createServerDiscovery",
  );
  const resolver = createServerDiscovery(
    state,
    () => {},
    () => {},
  );
  const config = {
    get: (key, fallback) => settings[key] ?? fallback,
    inspect: () => (settings.serverPath ? { globalValue: settings.serverPath } : undefined),
  };
  return {
    state,
    lines,
    resolve: (workspaceFailure) =>
      resolver.findServerPath(
        {
          extensionPath: path.join(f.directory, "extension"),
          extension: { packageJSON: { version: "1.2.3" } },
        },
        config,
        1,
        workspaceFailure,
      ),
  };
}

await test("workspace npm bin wins over extension version after real stdio initialize and shutdown", async (t) => {
  const f = fixture(t);
  const local = f.install();
  f.server(path.join(f.directory, "extension", "dist", exeName), "1.2.3");
  const current = discovery(f);
  assert.equal(await current.resolve(), local);
  assert.equal(current.state.selectedServerCandidate.source, "workspace");
  assert.equal(current.state.selectedServerCandidate.version, "9.8.7");
  const events = f.events();
  assert.ok(events.some((event) => event.event === "shutdown"));
  const parameters = events.find((event) => event.event === "initialize-params").params;
  assert.equal(parameters.rootUri, null);
  assert.equal(parameters.workspaceFolders, null);
  assert.deepEqual(parameters.initializationOptions, {
    lint: false,
    typecheck: false,
    editor: false,
    ecosystem: false,
  });
  const pid = events.find((event) => event.args[0] === "lsp").pid;
  assert.throws(() => process.kill(pid, 0));
});

await test("configured overrides, disabled preference and untrusted folders never execute workspace bins", async (t) => {
  const f = fixture(t);
  f.install();
  const bundled = f.server(path.join(f.directory, "extension", "dist", exeName), "1.2.3");
  const explicit = f.server(path.join(f.directory, "explicit.cjs"), "2.3.4");
  assert.equal(await discovery(f, [f.folder], true, { serverPath: explicit }).resolve(), explicit);
  assert.equal(
    await discovery(f, [f.folder], true, { "server.preferWorkspace": false }).resolve(),
    bundled,
  );
  assert.equal(await discovery(f, [f.folder], false).resolve(), bundled);
  assert.equal(
    f.events().some((event) => event.args[0] === "lsp"),
    false,
  );
});

await test("incompatible initialize falls back with the selected path, version and reason", async (t) => {
  const f = fixture(t);
  const local = f.install(f.folder, "0.1.0", "incompatible");
  const bundled = f.server(path.join(f.directory, "extension", "dist", exeName), "1.2.3");
  const current = discovery(f);
  assert.equal(await current.resolve(), bundled);
  assert.equal(current.state.selectedServerCandidate.version, "1.2.3");
  assert.match(
    current.state.selectedServerCandidate.workspaceFallbackReason,
    /0\.1\.0.*incompatible/,
  );
  assert.ok(current.state.selectedServerCandidate.workspaceFallbackReason.includes(local));
});

await test("disable during a failing workspace probe prevents fallback execution", async (t) => {
  const f = fixture(t);
  f.install();
  f.server(path.join(f.directory, "extension", "dist", exeName), "1.2.3");
  let release;
  let entered;
  const started = new Promise((resolve) => {
    entered = resolve;
  });
  const probe = new Promise((_, reject) => {
    release = () => reject(new Error("incompatible"));
  });
  const current = discovery(f, [f.folder], true, {}, () => {
    entered();
    return probe;
  });
  const pending = current.resolve();
  await started;
  ++current.state.configurationSyncGeneration;
  release();
  assert.equal(await pending, undefined);
  assert.equal(current.state.selectedServerCandidate, undefined);
  assert.equal(f.events().length, 1, "only the workspace version inspection may execute");
});

await test("nearest package, hidden exports, pnpm symlinks and Windows cmd wrappers resolve the package bin", (t) => {
  const f = fixture(t);
  const installed = f.install();
  const storage = path.join(f.directory, ".pnpm", "vize@9.8.7", "node_modules", "vize");
  fs.mkdirSync(path.dirname(storage), { recursive: true });
  fs.renameSync(path.dirname(path.dirname(installed)), storage);
  fs.symlinkSync(storage, path.join(f.folder, "node_modules", "vize"), "junction");
  const nested = path.join(f.folder, "packages", "app", "src");
  fs.mkdirSync(nested, { recursive: true });
  fs.writeFileSync(path.join(f.folder, "packages", "app", "package.json"), "{}");
  fs.mkdirSync(path.join(f.folder, "node_modules", ".bin"));
  fs.writeFileSync(
    path.join(f.folder, "node_modules", ".bin", "vize.cmd"),
    "@echo MUST NOT EXECUTE",
  );
  const expected = path.join(storage, "bin", "vize.cjs");
  const aliases = [nested, f.folder];
  if (process.platform === "win32") {
    aliases.push(
      f.folder.replace(
        /^([a-z]):/i,
        (_, drive) =>
          `${drive === drive.toUpperCase() ? drive.toLowerCase() : drive.toUpperCase()}:`,
      ),
    );
  }
  const discovered = discoverWorkspaceServers(aliases, true, "win32");
  assert.equal(
    discovered.reason,
    undefined,
    "drive-letter casing must not create conflicting engines",
  );
  assert.equal(discovered.candidates.length, 1);
  assert.equal(path.relative(discovered.candidates[0].path, expected), "");
  assert.deepEqual(discoverWorkspaceServers([nested, f.folder], true, "win32").candidates, [
    { path: expected, source: "workspace" },
  ]);
  const invocation = serverInvocation(expected, { PATH: "/example" });
  assert.equal(invocation.command, process.execPath);
  assert.deepEqual(invocation.args, [expected]);
  assert.equal(invocation.env.ELECTRON_RUN_AS_NODE, "1");
});

await test("distinct multi-root engines fall back even when versions match", async (t) => {
  const f = fixture(t);
  const firstEngine = f.install();
  const second = path.join(f.directory, "second");
  fs.mkdirSync(second);
  fs.writeFileSync(path.join(second, "package.json"), "{}");
  const secondEngine = f.install(second);
  fs.appendFileSync(secondEngine, "\n// A distinct build with the same advertised version.\n");
  assert.notEqual(fs.realpathSync(firstEngine), fs.realpathSync(secondEngine));
  assert.notDeepEqual(fs.readFileSync(firstEngine), fs.readFileSync(secondEngine));
  const bundled = f.server(path.join(f.directory, "extension", "dist", exeName), "1.2.3");
  const current = discovery(f, [f.folder, second]);
  assert.equal(await current.resolve(), bundled);
  assert.match(
    current.state.selectedServerCandidate.workspaceFallbackReason,
    /different Vize installations/,
  );
  assert.equal(
    f.events().some((event) => event.args[0] === "lsp"),
    false,
  );
});

for (const mode of ["exit", "timeout", "overflow"]) {
  await test(`protocol ${mode} is bounded and always reaps its process`, async (t) => {
    const f = fixture(t);
    const server = f.install(f.folder, "1.2.3", mode);
    await assert.rejects(inspectWorkspaceServerProtocol(server, 500), /exited|timed out|exceeds/);
    const pid = f.events().find((event) => event.event === "start").pid;
    assert.throws(() => process.kill(pid, 0));
  });
}

await test("an actual client startup failure disposes the workspace client before one fallback", async () => {
  const source = fs.readFileSync(path.join(root, "editors/vscode/src/extension.ts"), "utf8");
  const body = stripTypeScriptTypes(source);
  const start = body.slice(
    body.indexOf("async function startClient("),
    body.indexOf("async function stopClient("),
  );
  const operations = [];
  let starts = 0;
  const dependencies = {
    traceConfig() {},
    getInitializationOptions: () => ({ editor: true }),
    workspaceLspConfig: { hasConfig: true, describeCapabilities: () => "editor" },
    outputChannel: { appendLine() {} },
    updateStatusBar: (status) => operations.push(status),
    configurationSyncGeneration: 1,
    resolve: (reason) => {
      operations.push(reason ? "resolve-fallback" : "resolve-workspace");
      return { source: reason ? "bundled" : "workspace", path: "/vize", version: "1.2.3" };
    },
    resolveCorsaServerEnv: async () => ({}),
    createServerOptions: () => ({}),
    createClientOptions: () => ({}),
    applyTraceSetting() {},
    LanguageClient: class {
      async start() {
        if (++starts === 1) throw new Error("workspace initialization failed");
        operations.push("fallback-started");
      }
      async dispose() {
        operations.push("workspace-disposed");
      }
    },
  };
  await new vm.Script(`
    let client, selectedServerCandidate, activeInitializationOptions;
    async function findServerPath(context, config, generation, reason) {
      selectedServerCandidate = resolve(reason);
      return selectedServerCandidate.path;
    }
    ${start}
    startClient({ extensionPath: "/extension", globalStorageUri: { fsPath: "/storage" } }, {}, 1);
  `).runInNewContext(dependencies);
  assert.equal(starts, 2);
  assert.ok(operations.indexOf("workspace-disposed") < operations.indexOf("resolve-fallback"));
  assert.equal(operations.at(-1), "ready");
});
