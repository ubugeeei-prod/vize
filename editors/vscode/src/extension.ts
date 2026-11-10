import * as fs from "fs";
import {
  ExtensionContext,
  OutputChannel,
  StatusBarAlignment,
  commands,
  window,
  workspace,
  type StatusBarItem,
} from "vscode";
import { LanguageClient, Trace } from "vscode-languageclient/node.js";
import { createServerOptions } from "./server-options.js";
import {
  WORKSPACE_LSP_CONFIG_FILES,
  getInitializationOptions,
  shouldStartFromConfiguration,
  type LspInitializationOptions,
} from "./extension-core.js";
import { registerHostTestCommands } from "./host-test-commands.js";
import {
  trackConfigurationSyncForHostTest,
  waitForConfigurationIdle,
} from "./configuration-sync-test-state.js";
import { registerTypeScriptContentMapperDiscovery } from "./content-mapper-discovery.js";
import { createClientOptions, disposeClientFileEvents } from "./client-options.js";
import { WorkspaceLspConfigDiscovery } from "./workspace-config.js";
import { createExtensionSetup } from "./extension-setup.js";
import { createServerDiscovery, getExtensionVersion } from "./server-discovery.js";
import type {
  ExtensionRuntimeState,
  InspectedServerCandidate,
  VizeStatus,
} from "./server-state-types.js";

let client: LanguageClient | undefined;
let outputChannel: OutputChannel;
let statusBarItem: StatusBarItem | undefined;
let selectedServerCandidate: InspectedServerCandidate | undefined;
let activeInitializationOptions: LspInitializationOptions = {};
let activeClientInputs: string | undefined;
let currentStatus: VizeStatus = "disabled";
let currentStatusDetail = "";
let configurationSyncTimer: ReturnType<typeof setTimeout> | undefined;
let configurationSyncGeneration = 0;
let suppressConfigurationSync = false;
let startingClientGeneration: number | undefined;
const workspaceLspConfig = new WorkspaceLspConfigDiscovery(WORKSPACE_LSP_CONFIG_FILES, traceConfig);

const runtime: ExtensionRuntimeState = {
  get outputChannel() {
    return outputChannel;
  },
  get statusBarItem() {
    return statusBarItem;
  },
  get selectedServerCandidate() {
    return selectedServerCandidate;
  },
  set selectedServerCandidate(value) {
    selectedServerCandidate = value;
  },
  get activeInitializationOptions() {
    return activeInitializationOptions;
  },
  get currentStatus() {
    return currentStatus;
  },
  set currentStatus(value) {
    currentStatus = value;
  },
  get currentStatusDetail() {
    return currentStatusDetail;
  },
  set currentStatusDetail(value) {
    currentStatusDetail = value;
  },
  get configurationSyncGeneration() {
    return configurationSyncGeneration;
  },
  get suppressConfigurationSync() {
    return suppressConfigurationSync;
  },
  set suppressConfigurationSync(value) {
    suppressConfigurationSync = value;
  },
};
let {
  maybeOfferInitialSetup,
  maybeOfferCapabilitySetup,
  applyRecommendedConfiguration,
  applyLintOnlyConfiguration,
  disableVize,
  selectServerExecutable,
  showStatus,
  showServerNotFoundMessage,
  updateStatusBar,
  warnAboutVersionMismatch,
} = createExtensionSetup(runtime, workspaceLspConfig, (context, reason) =>
  syncClientToConfiguration(context, reason),
);
let { findServerPath } = createServerDiscovery(
  runtime,
  (status, detail) => updateStatusBar(status, detail),
  (candidate, version) => warnAboutVersionMismatch(candidate, version),
);

function traceConfig(message: string): void {
  if (process.env.VIZE_TEST_ENABLE_HOST_COMMANDS === "1") {
    try {
      outputChannel.appendLine(`[config ${new Date().toISOString()}] ${message}`);
    } catch {
      // Diagnostic logging must not replace an existing lifecycle result.
    }
  }
}

export async function activate(context: ExtensionContext): Promise<void> {
  outputChannel = window.createOutputChannel("Vize");
  outputChannel.appendLine("Vize extension activating...");
  context.subscriptions.push(outputChannel);
  registerTypeScriptContentMapperDiscovery(context, outputChannel);

  statusBarItem = window.createStatusBarItem(StatusBarAlignment.Right, 95);
  statusBarItem.command = "vize.showStatus";
  context.subscriptions.push(statusBarItem);
  updateStatusBar("starting", "Initializing Vize");

  context.subscriptions.push(
    workspace.onDidChangeConfiguration(async (event) => {
      if (!event.affectsConfiguration("vize")) {
        traceConfig("configuration callback rejected unrelated");
        return;
      }
      traceConfig("configuration callback accepted vize");

      if (suppressConfigurationSync) {
        traceConfig("configuration callback rejected applying-profile");
        outputChannel.appendLine("Vize configuration changed while applying a profile.");
        return;
      }

      outputChannel.appendLine("Vize configuration changed. Refreshing language server...");
      scheduleClientSync(context, "configuration changed");
    }),
  );

  context.subscriptions.push(
    ...workspaceLspConfig.watch((reason) => scheduleClientSync(context, reason)),
    workspace.onDidGrantWorkspaceTrust(() =>
      scheduleClientSync(context, "workspace trust granted"),
    ),
  );

  context.subscriptions.push(
    commands.registerCommand("vize.enableRecommendedProfile", async () => {
      await applyRecommendedConfiguration();
      await syncClientToConfiguration(context, "recommended profile applied");
    }),
    commands.registerCommand("vize.enableLintOnlyProfile", async () => {
      await applyLintOnlyConfiguration();
      await syncClientToConfiguration(context, "lint-only profile applied");
    }),
    commands.registerCommand("vize.selectServerPath", async () => {
      await selectServerExecutable(context);
    }),
    commands.registerCommand("vize.showStatus", async () => {
      await showStatus(context);
    }),
    commands.registerCommand("vize.disable", async () => {
      await disableVize(context);
    }),
    commands.registerCommand("vize.restartServer", async () => {
      outputChannel.appendLine("Restarting language server...");
      await syncClientToConfiguration(context, "manual restart");
    }),
    commands.registerCommand("vize.showOutput", () => {
      outputChannel.show();
    }),
    commands.registerCommand("vize.findReferences", async () => {
      const editor = window.activeTextEditor;
      if (editor) {
        await commands.executeCommand("editor.action.referenceSearch.trigger");
      }
    }),
    ...registerHostTestCommands(
      () => client,
      () => waitForConfigurationIdle(() => configurationSyncTimer !== undefined),
      process.env,
      () =>
        selectedServerCandidate && {
          ...selectedServerCandidate,
          extensionVersion: getExtensionVersion(context),
          path: fs.realpathSync(selectedServerCandidate.path),
          status: currentStatus,
        },
    ),
  );
  await syncClientToConfiguration(context, "initial activation");
}
function scheduleClientSync(context: ExtensionContext, reason: string): void {
  traceConfig(`schedule reason=${reason} generation=${configurationSyncGeneration}`);
  if (
    (client || startingClientGeneration !== undefined) &&
    reason === "project configuration changed"
  ) {
    // Preserve the running transport or its legitimate initialization attempt.
    // The server owns project reloads; pending settings synchronization must survive.
    void syncClientToConfiguration(context, reason);
    return;
  }
  if (configurationSyncTimer) {
    traceConfig(`timer cancel generation=${configurationSyncGeneration}`);
    clearTimeout(configurationSyncTimer);
  }

  configurationSyncTimer = setTimeout(() => {
    traceConfig(`timer fire reason=${reason} generation=${configurationSyncGeneration}`);
    configurationSyncTimer = undefined;
    void syncClientToConfiguration(context, reason);
  }, 150);
}

function syncClientToConfiguration(context: ExtensionContext, reason: string): Promise<void> {
  return trackConfigurationSyncForHostTest(syncClient(context, reason));
}

async function syncClient(context: ExtensionContext, reason: string): Promise<void> {
  if (
    (client || startingClientGeneration !== undefined) &&
    reason === "project configuration changed"
  ) {
    const observedGeneration = configurationSyncGeneration;
    await workspaceLspConfig.refresh();
    if (observedGeneration !== configurationSyncGeneration) return;
    const configuration = workspace.getConfiguration("vize");
    const options = getInitializationOptions(configuration, {
      hasWorkspaceLspConfig: workspaceLspConfig.hasConfig,
      log: (message) => outputChannel.appendLine(message),
    });
    const unchanged =
      Object.keys(options).length === Object.keys(activeInitializationOptions).length &&
      Object.entries(options).every(
        ([key, value]) =>
          Object.hasOwn(activeInitializationOptions, key) &&
          activeInitializationOptions[key] === value,
      );
    if (shouldStartFromConfiguration(configuration, workspaceLspConfig.hasConfig) && unchanged) {
      traceConfig("project discovery refreshed; retaining live language server");
      return;
    }
  }
  const generation = ++configurationSyncGeneration;
  traceConfig(`sync enter generation=${generation} reason=${reason}`);
  const refreshed = workspaceLspConfig.refresh(generation);
  // Observed now: a rejection while the nearby listing runs is not unhandled.
  refreshed.catch(() => {});
  if (workspaceLspConfig.hasRootConfig || (await workspaceLspConfig.findNearConfig())) {
    // A found config already decides startup and its options. The complete
    // list only feeds watchers, so the workspace-wide search must not delay the server.
    refreshed.catch((error: unknown) => {
      try {
        outputChannel.appendLine(`Project configuration discovery failed: ${String(error)}`);
      } catch {
        // A search that settles after deactivation has no channel left to report to.
      }
    });
  } else {
    await refreshed;
  }
  traceConfig(`sync refreshed generation=${generation} current=${configurationSyncGeneration}`);
  if (generation !== configurationSyncGeneration) {
    traceConfig(`sync refused generation=${generation} stage=refresh`);
    return;
  }
  let config = workspace.getConfiguration("vize");
  let enabled = shouldStartFromConfiguration(config, workspaceLspConfig.hasConfig);

  if (!enabled) {
    await maybeOfferInitialSetup(context, config);
    if (generation !== configurationSyncGeneration) {
      traceConfig(`sync refused generation=${generation} stage=setup`);
      return;
    }
    config = workspace.getConfiguration("vize");
    enabled = shouldStartFromConfiguration(config, workspaceLspConfig.hasConfig);

    if (!enabled) {
      if (client) {
        updateStatusBar("starting", "Stopping language server");
        outputChannel.appendLine(`Stopping Vize language server (${reason}; extension disabled).`);
        await stopClient();
        if (generation !== configurationSyncGeneration) {
          traceConfig(`sync refused generation=${generation} stage=disabled-stop`);
          return;
        }
      } else {
        outputChannel.appendLine("Vize is disabled. Set vize.enable to true to start the server.");
      }
      activeInitializationOptions = {};
      selectedServerCandidate = undefined;
      updateStatusBar("disabled", "Language server is disabled");
      return;
    }

    outputChannel.appendLine("Recommended Vize setup was applied. Starting language server...");
  } else if (workspaceLspConfig.hasConfig) {
    outputChannel.appendLine("Starting Vize language server with workspace configuration.");
  }

  if (client && reason === "configuration changed" && clientInputs(config) === activeClientInputs) {
    traceConfig(`sync retained generation=${generation}; the live client has these settings`);
    return;
  }

  if (client) {
    updateStatusBar("starting", "Restarting language server");
    outputChannel.appendLine(`Restarting Vize language server (${reason}).`);
    await stopClient();
    if (generation !== configurationSyncGeneration) {
      traceConfig(`sync refused generation=${generation} stage=restart-stop`);
      return;
    }
  }

  startingClientGeneration = generation;
  try {
    await startClient(context, config, generation);
  } finally {
    if (startingClientGeneration === generation) startingClientGeneration = undefined;
  }
  traceConfig(`sync finish generation=${generation} current=${configurationSyncGeneration}`);
}

async function startClient(
  context: ExtensionContext,
  config: ReturnType<typeof workspace.getConfiguration>,
  generation: number,
  workspaceFailure?: string,
  probeRelease = false,
): Promise<void> {
  traceConfig(`start enter generation=${generation}`);
  const initializationOptions = getInitializationOptions(config, {
    hasWorkspaceLspConfig: workspaceLspConfig.hasConfig,
    log: (message) => outputChannel.appendLine(message),
  });
  activeInitializationOptions = initializationOptions;
  activeClientInputs = clientInputs(config, initializationOptions);
  const capabilityDescription = workspaceLspConfig.describeCapabilities(initializationOptions);
  updateStatusBar("starting", `Starting with ${capabilityDescription}`);
  if (Object.keys(initializationOptions).length === 0) {
    outputChannel.appendLine(`Starting Vize with ${capabilityDescription}.`);
    if (!workspaceLspConfig.hasConfig) {
      void maybeOfferCapabilitySetup(context, config);
    }
  }
  const serverPath = await findServerPath(
    context,
    config,
    generation,
    workspaceFailure,
    probeRelease,
  );
  traceConfig(`start path-return generation=${generation} current=${configurationSyncGeneration}`);
  if (generation !== configurationSyncGeneration) {
    traceConfig(`start refused generation=${generation} stage=server-path`);
    return;
  }
  if (!serverPath) {
    updateStatusBar("missing-server", "Language server executable was not found");
    await showServerNotFoundMessage(context);
    return;
  }
  outputChannel.appendLine(`Using server: ${serverPath}`);
  const serverOptions = createServerOptions(serverPath);
  let nextClient: LanguageClient | undefined;
  nextClient = new LanguageClient(
    "vize",
    "Vize Language Server",
    serverOptions,
    createClientOptions(initializationOptions, () => nextClient, config, outputChannel),
  );
  applyTraceSetting(nextClient, config);

  try {
    traceConfig(`client start enter generation=${generation}`);
    await nextClient.start();
    traceConfig(
      `client start finish generation=${generation} current=${configurationSyncGeneration}`,
    );
    if (generation !== configurationSyncGeneration) {
      traceConfig(`unpublished client stop enter generation=${generation}`);
      await nextClient.stop();
      traceConfig(`unpublished client stop finish generation=${generation}`);
      return;
    }
    const cached = selectedServerCandidate;
    if (cached?.versionUnconfirmed) {
      const reported = nextClient.initializeResult?.serverInfo?.version;
      if (reported !== cached.version) {
        outputChannel.appendLine(
          `Cached Vize language server reported ${reported ?? "no version"}, not ${cached.version}. Refreshing cache.`,
        );
        await nextClient.stop();
        if (generation !== configurationSyncGeneration) return;
        return startClient(context, config, generation, workspaceFailure, true);
      }
      const { versionUnconfirmed, ...confirmed } = cached;
      selectedServerCandidate = versionUnconfirmed ? confirmed : cached;
    }
    client = nextClient;
    outputChannel.appendLine("Vize language server started successfully");
    updateStatusBar("ready", `Ready with ${capabilityDescription}`);
  } catch (error) {
    traceConfig(`client start error generation=${generation}`);
    outputChannel.appendLine(`Failed to start language server: ${String(error)}`);
    if (generation !== configurationSyncGeneration) return;
    if (!workspaceFailure && selectedServerCandidate?.source === "workspace") {
      const reason = `Workspace server ${serverPath} (${selectedServerCandidate.version ?? "unknown version"}) failed to start: ${String(error)}`;
      outputChannel.appendLine(`${reason}. Trying the extension server.`);
      await nextClient.dispose();
      if (generation !== configurationSyncGeneration) return;
      return startClient(context, config, generation, reason);
    }
    if (!probeRelease && selectedServerCandidate?.versionUnconfirmed) {
      outputChannel.appendLine("Probing the cached Vize language server before refreshing it.");
      await nextClient.dispose();
      if (generation !== configurationSyncGeneration) return;
      return startClient(context, config, generation, workspaceFailure, true);
    }
    updateStatusBar("failed", "Failed to start language server");
    window.showErrorMessage(`Vize: Failed to start language server: ${String(error)}`);
  }
}

// Everything a started client was built from: the settings dictionary the client
// snapshots (trace, auto-insert, server selection), the serverPath scopes trust
// reads, and the initialization options, which also depend on project configs.
function clientInputs(
  config: ReturnType<typeof workspace.getConfiguration>,
  options = getInitializationOptions(config, {
    hasWorkspaceLspConfig: workspaceLspConfig.hasConfig,
  }),
): string {
  return JSON.stringify([config, config.inspect("serverPath"), options]);
}

async function stopClient(): Promise<void> {
  traceConfig(`stop enter generation=${configurationSyncGeneration} client=${Boolean(client)}`);
  if (!client) {
    return;
  }

  const activeClient = client;
  client = undefined;
  await activeClient.stop();
  traceConfig(`stop finish generation=${configurationSyncGeneration}`);
}

function applyTraceSetting(
  nextClient: LanguageClient,
  config: ReturnType<typeof workspace.getConfiguration>,
): void {
  const traceSetting = config.get<string>("trace.server", "off");
  const trace =
    traceSetting === "verbose"
      ? Trace.Verbose
      : traceSetting === "messages"
        ? Trace.Messages
        : Trace.Off;

  void nextClient.setTrace(trace);
  outputChannel.appendLine(`Vize trace level: ${traceSetting}`);
}

export async function deactivate(): Promise<void> {
  ++configurationSyncGeneration;
  traceConfig(`deactivate generation=${configurationSyncGeneration}`);
  if (configurationSyncTimer) clearTimeout(configurationSyncTimer);
  configurationSyncTimer = undefined;
  await stopClient();
  disposeClientFileEvents();
}
