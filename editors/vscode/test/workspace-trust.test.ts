import assert from "node:assert/strict";
import fs from "node:fs";
import { stripTypeScriptTypes } from "node:module";
import path from "node:path";
import test from "node:test";
import vm from "node:vm";

import {
  LINT_ONLY_CONFIGURATION_UPDATES,
  WORKSPACE_LSP_CONFIG_FILES,
  describeCapabilities,
  getInitializationOptions,
  hasAnyEnabledCapability,
  hasExplicitConfigurationValue,
  parseVizeVersion,
  serverMatchesExtension,
  shouldStartFromConfiguration,
} from "../src/extension-core.ts";
import { isJsxDocument } from "../src/jsx-routing.ts";
import { trustedServerPath, workspaceDevServerPaths } from "../src/server-trust.ts";
import { discoverWorkspaceServers, workspaceServerProbePaths } from "../src/workspace-server.ts";

// The extension's real activation, sync, discovery and status sources, run
// against a scripted VS Code surface. Only the editor API, the language client
// and process execution are replaced.
const body = [
  "extension-setup.ts",
  "server-discovery.ts",
  "configuration-sync-test-state.ts",
  "content-mapper-discovery.ts",
  "extension.ts",
]
  .map((file) =>
    stripTypeScriptTypes(fs.readFileSync(new URL(`../src/${file}`, import.meta.url), "utf8"))
      .replace(/^import[\s\S]*?;\n/gm, "")
      .replace(/^export /gm, ""),
  )
  .join("\n");

/** The extension modules that run unchanged. */
const realModules = {
  LINT_ONLY_CONFIGURATION_UPDATES,
  WORKSPACE_LSP_CONFIG_FILES,
  describeCapabilities,
  getInitializationOptions,
  hasAnyEnabledCapability,
  hasExplicitConfigurationValue,
  parseVizeVersion,
  serverMatchesExtension,
  shouldStartFromConfiguration,
  isJsxDocument,
  trustedServerPath,
  workspaceDevServerPaths,
  discoverWorkspaceServers,
  workspaceServerProbePaths,
};

const bundledServer = "/extension/dist/vize";
const subscription = { dispose() {} };

/** Everything the extension does that a user or the machine could observe. */
interface Observed {
  executed: string[];
  started: string[];
  logs: string[];
  timers: Array<() => void>;
  commands: Map<string, () => Promise<void>>;
  statusBar: { text: string; tooltip: string; command: string; show(): void };
  trustGranted: () => void;
}

/** The VS Code API for one project folder with a Vite config. */
function editorApi(trusted: boolean, observed: Observed) {
  const workspace = {
    isTrusted: trusted,
    workspaceFolders: [{ uri: { fsPath: "/work/project", scheme: "file" } }],
    textDocuments: [],
    getConfiguration: () => ({
      get: (key: string, fallback: unknown) => (key === "enable" ? true : fallback),
      inspect: () => undefined,
    }),
    onDidChangeConfiguration: () => subscription,
    onDidOpenTextDocument: () => subscription,
    onDidGrantWorkspaceTrust(listener: () => void) {
      observed.trustGranted = listener;
      return subscription;
    },
  };
  class WorkspaceLspConfigDiscovery {
    hasConfig = true;
    hasRootConfig = true;
    async refresh() {}
    async findNearConfig() {
      return false;
    }
    describeCapabilities(options: Parameters<typeof describeCapabilities>[0]) {
      return describeCapabilities(options, true);
    }
    watch() {
      return [];
    }
  }
  return {
    workspace,
    WorkspaceLspConfigDiscovery,
    window: {
      activeTextEditor: undefined,
      createOutputChannel: () => ({
        appendLine(line: string) {
          if (!line.startsWith("[config ")) observed.logs.push(line);
        },
        show() {},
        dispose() {},
      }),
      createStatusBarItem: () => observed.statusBar,
      showInformationMessage: async () => undefined,
      showWarningMessage: async () => undefined,
      showErrorMessage: async () => undefined,
    },
    commands: {
      registerCommand(name: string, handler: () => Promise<void>) {
        observed.commands.set(name, handler);
        return subscription;
      },
      executeCommand: async () => undefined,
    },
    extensions: { getExtension: () => undefined },
    StatusBarAlignment: { Right: 2 },
  };
}

/** A Linux machine whose only server is the one the extension ships. */
function machine(observed: Observed) {
  class LanguageClient {
    initializeResult = { serverInfo: { version: "1.2.3" } };
    private readonly server: string;
    constructor(_id: string, _name: string, options: { serverPath: string }) {
      this.server = options.serverPath;
    }
    async start() {
      observed.started.push(this.server);
    }
    async stop() {}
    async dispose() {}
    setTrace() {}
  }
  return {
    fs: { existsSync: () => false },
    path: path.posix,
    process: {
      platform: "linux",
      arch: "x64",
      env: { HOME: "/home/user", PATH: "", VIZE_TEST_ENABLE_HOST_COMMANDS: "1" },
    },
    Date,
    setTimeout(callback: () => void) {
      return observed.timers.push(callback);
    },
    clearTimeout() {},
    promisify: (run: unknown) => run,
    execFile: async (command: string, args: string[]) => {
      observed.executed.push([command, ...args].join(" "));
      return { stdout: "vize 1.2.3\n" };
    },
    existingPaths: async () => (file: string) => file === bundledServer,
    serverInvocation: (serverPath: string) => ({ command: serverPath, args: [], env: {} }),
    LanguageClient,
    Trace: { Off: 0, Messages: 1, Verbose: 2 },
    createServerOptions: (serverPath: string) => ({ serverPath }),
    createClientOptions: () => ({}),
    disposeClientFileEvents() {},
    registerHostTestCommands: () => [],
  };
}

const lifecycleProgram = fs.readFileSync(
  new URL("./workspace-trust-lifecycle.js", import.meta.url),
  "utf8",
);

interface Lifecycle {
  activate(context: object): Promise<void>;
  idle(): Promise<void>;
}

function openWindow(trusted: boolean) {
  const observed: Observed = {
    executed: [],
    started: [],
    logs: [],
    timers: [],
    commands: new Map(),
    statusBar: { text: "", tooltip: "", command: "", show() {} },
    trustGranted() {},
  };
  const editor = editorApi(trusted, observed);
  const context = vm.createContext({ ...realModules, ...editor, ...machine(observed) });
  new vm.Script(body).runInContext(context);
  const lifecycle = new vm.Script(lifecycleProgram).runInContext(context) as Lifecycle;
  return {
    ...observed,
    activate: () =>
      lifecycle.activate({
        subscriptions: [],
        extensionPath: "/extension",
        extension: { packageJSON: { version: "1.2.3" } },
        globalStorageUri: { fsPath: "/storage" },
        globalState: { get: () => undefined, update: async () => {} },
      }),
    async run(command: string) {
      await observed.commands.get(command)?.();
    },
    /** What VS Code does when the user trusts the workspace. */
    async grantTrust() {
      editor.workspace.isTrusted = true;
      observed.trustGranted();
      for (const fire of observed.timers.splice(0)) fire();
      await lifecycle.idle();
    },
  };
}

const waitingForTrust = {
  text: "$(shield) Vize",
  tooltip: [
    "Vize: waiting for workspace trust",
    "The language server starts once this workspace is trusted.",
    "Click to manage Vize.",
  ].join("\n"),
};
const restrictedModeLog =
  "Vize language server is not started in Restricted Mode: it evaluates the project's Vite config and runs the project's TypeScript runtime. Trust this workspace to start it.";

void test("the server starts only in a trusted workspace", async () => {
  const editor = openWindow(false);
  await editor.activate();
  assert.deepEqual(editor.executed, []);
  assert.deepEqual(editor.started, []);
  assert.deepEqual(
    { text: editor.statusBar.text, tooltip: editor.statusBar.tooltip },
    waitingForTrust,
  );
  assert.deepEqual(editor.logs, ["Vize extension activating...", restrictedModeLog]);
});

void test("restarting from the command palette does not start the server before trust", async () => {
  const editor = openWindow(false);
  await editor.activate();
  await editor.run("vize.restartServer");
  assert.deepEqual(editor.executed, []);
  assert.deepEqual(editor.started, []);
  assert.deepEqual(
    { text: editor.statusBar.text, tooltip: editor.statusBar.tooltip },
    waitingForTrust,
  );
});

void test("granting workspace trust starts the server", async () => {
  const editor = openWindow(false);
  await editor.activate();
  await editor.grantTrust();
  assert.deepEqual(editor.executed, [`${bundledServer} --version`]);
  assert.deepEqual(editor.started, [bundledServer]);
  assert.equal(editor.statusBar.text, "$(check) Vize");
});

void test("a trusted workspace starts the server on activation", async () => {
  const editor = openWindow(true);
  await editor.activate();
  assert.deepEqual(editor.executed, [`${bundledServer} --version`]);
  assert.deepEqual(editor.started, [bundledServer]);
  assert.equal(editor.statusBar.text, "$(check) Vize");
});
