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
import { resolveCorsaServerEnv } from "./corsa-runtime.js";
import { createServerOptions } from "./server-options.js";
import {
  WORKSPACE_LSP_CONFIG_FILES,
  getInitializationOptions,
  shouldStartFromConfiguration,
  type LspInitializationOptions,
} from "./extension-core.js";
import { registerHostTestCommands } from "./host-test-commands.js";
import { registerTypeScriptContentMapperDiscovery } from "./content-mapper-discovery.js";
import { createClientOptions } from "./client-options.js";
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

async function syncClientToConfiguration(context: ExtensionContext, reason: string): Promise<void> {
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
  await workspaceLspConfig.refresh(generation);
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
): Promise<void> {
  traceConfig(`start enter generation=${generation}`);
  const initializationOptions = getInitializationOptions(config, {
    hasWorkspaceLspConfig: workspaceLspConfig.hasConfig,
    log: (message) => outputChannel.appendLine(message),
  });
  activeInitializationOptions = initializationOptions;
  const capabilityDescription = workspaceLspConfig.describeCapabilities(initializationOptions);
  updateStatusBar("starting", `Starting with ${capabilityDescription}`);
  if (Object.keys(initializationOptions).length === 0) {
    outputChannel.appendLine(`Starting Vize with ${capabilityDescription}.`);
    if (!workspaceLspConfig.hasConfig) {
      void maybeOfferCapabilitySetup(context, config);
    }
  }
  const serverPath = await findServerPath(context, config, generation);
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
  const serverEnv = await resolveCorsaServerEnv({
    extensionPath: context.extensionPath,
    globalStoragePath: context.globalStorageUri.fsPath,
    output: outputChannel,
  });
  if (generation !== configurationSyncGeneration) {
    traceConfig(`start refused generation=${generation} stage=corsa`);
    return;
  }
  const serverOptions = createServerOptions(serverPath, serverEnv);
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
    client = nextClient;
    outputChannel.appendLine("Vize language server started successfully");
    updateStatusBar("ready", `Ready with ${capabilityDescription}`);
  } catch (error) {
    traceConfig(`client start error generation=${generation}`);
    outputChannel.appendLine(`Failed to start language server: ${String(error)}`);
    if (generation !== configurationSyncGeneration) return;
    updateStatusBar("failed", "Failed to start language server");
    window.showErrorMessage(`Vize: Failed to start language server: ${String(error)}`);
  }
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
}
