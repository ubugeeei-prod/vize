import fs from "node:fs";
import { stripTypeScriptTypes } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

// Runs the extension's real sync, discovery and setup sources against a
// scripted VS Code surface, so lifecycle tests observe the actual ordering.
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const body = ["extension-setup.ts", "server-discovery.ts", "extension.ts"]
  .map((file) => {
    const source = fs.readFileSync(path.join(root, "editors/vscode/src", file), "utf8");
    return stripTypeScriptTypes(source, { mode: "transform" })
      .replace(/^import[\s\S]*?;\n/gm, "")
      .replace(/^export /gm, "");
  })
  .join("\n");

export interface Deferred {
  promise: Promise<void>;
  resolve: () => void;
}

export function deferred(): Deferred {
  let resolve: () => void = () => {};
  const promise = new Promise<void>((complete) => {
    resolve = complete;
  });
  return { promise, resolve };
}

/** One client start: resolved when it may finish, with an optional entry signal. */
export interface StartGate {
  promise: Promise<void>;
  entered?: Deferred;
}

export interface Startup extends Partial<StartGate> {
  gates?: StartGate[];
}

export interface SyncPolicy {
  hasConfig?: boolean;
  hasRootConfig?: boolean;
  nearConfig?: boolean;
  /** serverInfo.version of each started client; also scripts a cached release server. */
  reported?: string[];
  actualOptions?: boolean;
  configuration?: unknown;
}

export type Operation = [string, ...unknown[]];

export interface SyncControl {
  scheduleClientSync(context: object, reason: string): void;
  syncClientToConfiguration(context: object, reason: string): Promise<void>;
  deactivate(): Promise<void>;
  setClient(value: unknown): void;
  setHasConfig(value: boolean): void;
  getClient(): unknown;
  getCandidate(): Record<string, unknown> | undefined;
  state(): { generation: number; hasClient: boolean; starting: number | undefined };
}

export interface Sync {
  control: SyncControl;
  operations: Operation[];
  logs: string[];
  timers: Map<number, () => void>;
  started: Deferred;
  retained: Deferred;
  initializations: unknown[];
}

interface ClientOptions {
  initializationOptions: unknown;
}

export function createSync(
  refresh: () => Promise<unknown>,
  activeClient?: unknown,
  refuseLogging = false,
  startup?: Startup,
  policy: SyncPolicy = {},
): Sync {
  const operations: Operation[] = [];
  const logs: string[] = [];
  const timers = new Map<number, () => void>();
  const started = deferred();
  const retained = deferred();
  const initializations: unknown[] = [];
  const reported = policy.reported;
  let nextTimer = 0;
  const output = {
    appendLine(message: string) {
      if (refuseLogging && message.startsWith("[config ")) throw new Error("logger disposed");
      logs.push(message);
      if (message.includes("retaining live language server")) retained.resolve();
    },
  };
  class Discovery {
    hasConfig = policy.hasConfig ?? true;
    hasRootConfig = policy.hasRootConfig ?? false;
    refresh = refresh;
    findNearConfig = async () => policy.nearConfig ?? false;
    describeCapabilities() {
      return "test capabilities";
    }
  }
  class LanguageClient {
    initializeResult?: { serverInfo: { version: string | undefined } };
    constructor(_id: string, _name: string, _server: unknown, options: ClientOptions) {
      initializations.push(structuredClone(options.initializationOptions));
      if (reported) this.initializeResult = { serverInfo: { version: reported.shift() } };
    }
    async start() {
      operations.push(["real-start"]);
      started.resolve();
      const gate = startup?.gates?.shift() ?? startup;
      gate?.entered?.resolve();
      await gate?.promise;
    }
    async stop() {
      operations.push(["real-stop"]);
    }
    setTrace() {}
  }
  const context: vm.Context = vm.createContext({
    process: { env: { VIZE_TEST_ENABLE_HOST_COMMANDS: "1" } },
    Date,
    promisify: () => () => {},
    execFile: () => {},
    WorkspaceLspConfigDiscovery: Discovery,
    WORKSPACE_LSP_CONFIG_FILES: [],
    workspace: {
      getConfiguration: (name: string) => {
        operations.push(["configuration", name]);
        return (
          policy.configuration ?? {
            enabled: true,
            get: (_key: string, fallback: unknown) => fallback,
          }
        );
      },
    },
    shouldStartFromConfiguration: (configuration: { enabled: boolean }) => configuration.enabled,
    setTimeout(callback: () => void, milliseconds: number) {
      const id = ++nextTimer;
      operations.push(["schedule", id, milliseconds]);
      timers.set(id, callback);
      return id;
    },
    clearTimeout(id: number) {
      operations.push(["cancel", id]);
      timers.delete(id);
    },
    output,
    activeClient,
    policyConfiguration: policy.configuration,
    policyRelease: Boolean(reported),
    startup,
    getInitializationOptions: () => ({ lint: true }),
    resolveCorsaServerEnv: async () => ({ PATH: "/usr/bin" }),
    createServerOptions: (serverPath: string) => ({ serverPath }),
    createClientOptions: (initializationOptions: unknown) => ({ initializationOptions }),
    disposeClientFileEvents: () => operations.push(["dispose-file-events"]),
    Trace: { Off: "off" },
    LanguageClient,
    operations,
    started,
  });
  if (policy.actualOptions) {
    const core = stripTypeScriptTypes(
      fs.readFileSync(path.join(root, "editors/vscode/src/extension-core.ts"), "utf8"),
      { mode: "transform" },
    ).replace(/^export /gm, "");
    context.getInitializationOptions = new vm.Script(
      `${core}\ngetInitializationOptions`,
    ).runInNewContext();
  }
  const control: SyncControl = new vm.Script(`${body}\n
    outputChannel = output;
    client = activeClient;
    if (client) activeInitializationOptions = getInitializationOptions(
      policyConfiguration, { hasWorkspaceLspConfig: workspaceLspConfig.hasConfig });
    updateStatusBar = (...args) => operations.push(["status", ...args]);
    if (startup && policyRelease) {
      findServerPath = async (_context, _config, _generation, _failure, probe) => {
        operations.push(["find", probe]);
        selectedServerCandidate = { path: "/test/vize", source: "release", version: "1.2.3",
          ...(probe ? {} : { versionUnconfirmed: true }) };
        return "/test/vize";
      };
    } else if (startup) {
      findServerPath = async () => "/test/vize";
    } else {
      startClient = async (_context, _config, generation) => {
        operations.push(["start", generation]); started.resolve();
      };
    }
    ({ scheduleClientSync, syncClientToConfiguration: (extensionContext, reason) =>
       syncClientToConfiguration({
         extensionPath: "/test/extension",
         globalStorageUri: { fsPath: "/test/global-storage" },
         ...extensionContext,
       }, reason), deactivate,
       setClient: (value) => { client = value; activeInitializationOptions = getInitializationOptions(
         policyConfiguration, { hasWorkspaceLspConfig: workspaceLspConfig.hasConfig }); },
       setHasConfig: (value) => { workspaceLspConfig.hasConfig = value; },
       getClient: () => client,
       getCandidate: () => selectedServerCandidate && { ...selectedServerCandidate },
       state: () => ({ generation: configurationSyncGeneration, hasClient: Boolean(client), starting: startingClientGeneration }) });
  `).runInContext(context);
  return { control, operations, logs, timers, started, retained, initializations };
}
