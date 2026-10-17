import * as crypto from "crypto";
import * as fs from "fs";
import * as path from "path";
import { execFile } from "child_process";
import { promisify } from "util";
import {
  ConfigurationTarget,
  ExtensionContext,
  OutputChannel,
  StatusBarAlignment,
  Uri,
  commands,
  env,
  window,
  workspace,
  type StatusBarItem,
} from "vscode";
import { LanguageClient, Trace } from "vscode-languageclient/node.js";
import { createServerOptions } from "./server-options.js";
import {
  LINT_ONLY_CONFIGURATION_UPDATES,
  WORKSPACE_LSP_CONFIG_FILES,
  describeCapabilities,
  getInitializationOptions,
  hasAnyEnabledCapability,
  hasExplicitConfigurationValue,
  parseVizeVersion,
  shouldStartFromConfiguration,
  type LspInitializationOptions,
} from "./extension-core.js";
import { registerHostTestCommands } from "./host-test-commands.js";
import { registerTypeScriptContentMapperDiscovery } from "./content-mapper-discovery.js";
import { createClientOptions } from "./client-options.js";
import { isJsxDocument } from "./jsx-routing.js";
import { downloadFile } from "./release-download.js";
import { extractReleaseArchive } from "./release-archive.js";
import { WorkspaceLspConfigDiscovery } from "./workspace-config.js";
import { trustedServerPath, workspaceDevServerPaths } from "./server-trust.js";
import type {
  InspectedServerCandidate,
  ReleaseServerTarget,
  ServerCandidate,
  VizeStatus,
  VizeStatusAction,
  VizeStatusQuickPickItem,
} from "./server-state-types.js";

const execFileAsync = promisify(execFile);
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
const workspaceLspConfig = new WorkspaceLspConfigDiscovery(WORKSPACE_LSP_CONFIG_FILES, traceConfig);

function traceConfig(message: string): void {
  if (process.env.VIZE_TEST_ENABLE_HOST_COMMANDS === "1") {
    try {
      outputChannel.appendLine(`[config ${new Date().toISOString()}] ${message}`);
    } catch {
      // Diagnostic logging must not replace an existing lifecycle result.
    }
  }
}

const RECOMMENDED_SETUP_ACTION = "Enable Recommended";
const LINT_ONLY_SETUP_ACTION = "Enable Lint Only";
const OPEN_SETTINGS_ACTION = "Open Settings";
const DISMISS_ACTION = "Dismiss";
const OPEN_SETUP_DOCS_ACTION = "Open Setup Docs";
const SELECT_SERVER_ACTION = "Select Binary";
const SHOW_OUTPUT_ACTION = "Show Output";
const INITIAL_SETUP_PROMPT_DISMISSED_KEY = "vize.initialSetupPrompt.dismissed";
const CAPABILITY_PROMPT_DISMISSED_KEY = "vize.capabilityPrompt.dismissed";

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
  if (client && reason === "project configuration changed") {
    // The live server owns project reloads through its watched-file registrations.
    // Preserve both that transport and any pending initialization-settings sync.
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
  if (client && reason === "project configuration changed") {
    await workspaceLspConfig.refresh();
    traceConfig("project discovery refreshed; retaining live language server");
    return;
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

  await startClient(context, config, generation);
  traceConfig(`sync finish generation=${generation} current=${configurationSyncGeneration}`);
}

async function maybeOfferInitialSetup(
  context: ExtensionContext,
  config: ReturnType<typeof workspace.getConfiguration>,
): Promise<void> {
  if (
    hasExplicitConfigurationValue(config, "enable") ||
    workspace.textDocuments.some(isJsxDocument)
  ) {
    return;
  }

  if (context.globalState.get<boolean>(INITIAL_SETUP_PROMPT_DISMISSED_KEY)) {
    return;
  }

  const selection = await window.showInformationMessage(
    "Vize is installed but disabled. Enable the recommended diagnostics, navigation, and ecosystem profile for this workspace?",
    RECOMMENDED_SETUP_ACTION,
    LINT_ONLY_SETUP_ACTION,
    OPEN_SETTINGS_ACTION,
    DISMISS_ACTION,
  );

  if (selection === RECOMMENDED_SETUP_ACTION) {
    await applyRecommendedConfiguration();
    return;
  }

  if (selection === LINT_ONLY_SETUP_ACTION) {
    await applyLintOnlyConfiguration();
    return;
  }

  if (selection === OPEN_SETTINGS_ACTION) {
    await commands.executeCommand("workbench.action.openSettings", "vize");
    return;
  }

  if (selection === DISMISS_ACTION) {
    await context.globalState.update(INITIAL_SETUP_PROMPT_DISMISSED_KEY, true);
  }
}

async function maybeOfferCapabilitySetup(
  context: ExtensionContext,
  config: ReturnType<typeof workspace.getConfiguration>,
): Promise<void> {
  if (hasAnyEnabledCapability(config) || workspaceLspConfig.hasConfig) {
    return;
  }

  if (context.globalState.get<boolean>(CAPABILITY_PROMPT_DISMISSED_KEY)) {
    return;
  }

  const selection = await window.showInformationMessage(
    "Vize is enabled but no language features are turned on. Enable diagnostics, navigation, and ecosystem helpers for this workspace?",
    RECOMMENDED_SETUP_ACTION,
    LINT_ONLY_SETUP_ACTION,
    OPEN_SETTINGS_ACTION,
    DISMISS_ACTION,
  );

  if (selection === RECOMMENDED_SETUP_ACTION) {
    await applyRecommendedConfiguration();
    return;
  }

  if (selection === LINT_ONLY_SETUP_ACTION) {
    await applyLintOnlyConfiguration();
    return;
  }

  if (selection === OPEN_SETTINGS_ACTION) {
    await commands.executeCommand("workbench.action.openSettings", "vize");
    return;
  }

  if (selection === DISMISS_ACTION) {
    await context.globalState.update(CAPABILITY_PROMPT_DISMISSED_KEY, true);
  }
}

async function applyRecommendedConfiguration(): Promise<void> {
  await applyConfigurationUpdates([
    ["enable", true],
    ["lint.enable", true],
    ["typecheck.enable", true],
    ["editor.enable", true],
    ["ecosystem.enable", true],
  ]);
}

async function applyLintOnlyConfiguration(): Promise<void> {
  await applyConfigurationUpdates(LINT_ONLY_CONFIGURATION_UPDATES);
}

async function applyConfigurationUpdates(updates: Array<[string, unknown]>): Promise<void> {
  const config = workspace.getConfiguration("vize");
  const target = getConfigurationTarget();

  suppressConfigurationSync = true;
  try {
    for (const [key, value] of updates) {
      await config.update(key, value, target);
    }
  } finally {
    suppressConfigurationSync = false;
  }
}

async function disableVize(context: ExtensionContext): Promise<void> {
  await applyConfigurationUpdates([["enable", false]]);
  await syncClientToConfiguration(context, "disabled from command");
}

async function selectServerExecutable(context: ExtensionContext): Promise<void> {
  const selection = await window.showOpenDialog({
    canSelectFiles: true,
    canSelectFolders: false,
    canSelectMany: false,
    defaultUri: workspace.workspaceFolders?.[0]?.uri,
    openLabel: "Use as Vize Server",
    title: "Select vize language server executable",
  });
  const selectedUri = selection?.[0];
  if (!selectedUri) {
    return;
  }

  await applyConfigurationUpdates([
    ["serverPath", selectedUri.fsPath],
    ["enable", true],
  ]);
  await syncClientToConfiguration(context, "server executable selected");
}

async function showStatus(context: ExtensionContext): Promise<void> {
  const config = workspace.getConfiguration("vize");
  const initializationOptions = getInitializationOptions(config, {
    hasWorkspaceLspConfig: workspaceLspConfig.hasConfig,
    logDefaultProfile: false,
  });
  const items = createStatusItems(config);

  const selected = await window.showQuickPick(items, {
    placeHolder: createStatusSummary(config, initializationOptions),
    title: "Vize Status",
  });

  if (!selected) {
    return;
  }

  await runStatusAction(context, selected.action);
}

function createStatusItems(
  config: ReturnType<typeof workspace.getConfiguration>,
): VizeStatusQuickPickItem[] {
  const enabled = shouldStartFromConfiguration(config, workspaceLspConfig.hasConfig);
  const items: VizeStatusQuickPickItem[] = [
    {
      action: "recommended",
      description: "lint + typecheck + editor",
      detail: "Best one-click profile when evaluating Vize as a full Vue language assistant.",
      label: "$(rocket) Enable Recommended Profile",
    },
    {
      action: "lintOnly",
      description: "safe alongside Volar",
      detail:
        "Turns on Vize diagnostics while leaving navigation, completion, and formatting elsewhere.",
      label: "$(beaker) Enable Lint-Only Profile",
    },
    {
      action: "selectServer",
      description: selectedServerCandidate
        ? `${selectedServerCandidate.source}: ${selectedServerCandidate.path}`
        : "pick a vize executable",
      detail: "Use this when the auto-detected server is missing or not the version you want.",
      label: "$(folder-opened) Select Language Server Executable",
    },
    {
      action: "settings",
      description: "open settings",
      label: "$(settings-gear) Open Vize Settings",
    },
    {
      action: "output",
      description: "show logs",
      label: "$(output) Show Output Channel",
    },
  ];

  if (enabled) {
    items.splice(3, 0, {
      action: "restart",
      description: "restart now",
      label: "$(debug-restart) Restart Language Server",
    });
    items.push({
      action: "disable",
      description: "stop Vize",
      label: "$(circle-slash) Disable Language Server",
    });
  }

  return items;
}

async function runStatusAction(context: ExtensionContext, action: VizeStatusAction): Promise<void> {
  if (action === "recommended") {
    await applyRecommendedConfiguration();
    await syncClientToConfiguration(context, "recommended profile applied from status");
    return;
  }

  if (action === "lintOnly") {
    await applyLintOnlyConfiguration();
    await syncClientToConfiguration(context, "lint-only profile applied from status");
    return;
  }

  if (action === "selectServer") {
    await selectServerExecutable(context);
    return;
  }

  if (action === "restart") {
    await syncClientToConfiguration(context, "status restart");
    return;
  }

  if (action === "settings") {
    await commands.executeCommand("workbench.action.openSettings", "vize");
    return;
  }

  if (action === "output") {
    outputChannel.show();
    return;
  }

  if (action === "disable") {
    await disableVize(context);
  }
}

function getConfigurationTarget(): ConfigurationTarget {
  return workspace.workspaceFolders?.length
    ? ConfigurationTarget.Workspace
    : ConfigurationTarget.Global;
}

async function showServerNotFoundMessage(context: ExtensionContext): Promise<void> {
  const selection = await window.showErrorMessage(
    "Vize: Could not find the language server. Install a matching GitHub release binary or set vize.serverPath.",
    SELECT_SERVER_ACTION,
    OPEN_SETUP_DOCS_ACTION,
    OPEN_SETTINGS_ACTION,
    SHOW_OUTPUT_ACTION,
  );

  if (selection === SELECT_SERVER_ACTION) {
    await selectServerExecutable(context);
    return;
  }

  if (selection === OPEN_SETTINGS_ACTION) {
    await commands.executeCommand("workbench.action.openSettings", "vize.serverPath");
    return;
  }

  if (selection === OPEN_SETUP_DOCS_ACTION) {
    await env.openExternal(
      Uri.parse("https://github.com/ubugeeei-prod/vize/tree/main/editors/vscode#readme"),
    );
    return;
  }

  if (selection === SHOW_OUTPUT_ACTION) {
    outputChannel.show();
  }
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

async function findServerPath(
  context: ExtensionContext,
  config: ReturnType<typeof workspace.getConfiguration>,
  generation: number,
): Promise<string | undefined> {
  const exeName = process.platform === "win32" ? "vize.exe" : "vize";
  const expectedVersion = getExtensionVersion(context);
  selectedServerCandidate = undefined;

  const configuredPath = resolveTrustedServerPath(config);
  if (configuredPath) {
    if (fs.existsSync(configuredPath)) {
      const candidate = await inspectServerCandidate({
        path: configuredPath,
        source: "configured",
      });
      if (generation !== configurationSyncGeneration) return undefined;
      logSelectedServer(candidate, expectedVersion);
      void warnAboutVersionMismatch(candidate, expectedVersion);
      selectedServerCandidate = candidate;
      return configuredPath;
    }

    outputChannel.appendLine(`Configured server path does not exist: ${configuredPath}`);
  }

  const candidates = await inspectServerCandidates(collectServerCandidates(context, exeName));
  if (generation !== configurationSyncGeneration) return undefined;

  if (expectedVersion) {
    const matchingCandidate = candidates.find((candidate) => candidate.version === expectedVersion);
    if (matchingCandidate) {
      logSelectedServer(matchingCandidate, expectedVersion);
      selectedServerCandidate = matchingCandidate;
      return matchingCandidate.path;
    }

    const releaseCandidate = await resolveReleaseServerCandidate(
      context,
      exeName,
      expectedVersion,
      generation,
    );
    if (generation !== configurationSyncGeneration) return undefined;
    if (releaseCandidate) {
      logSelectedServer(releaseCandidate, expectedVersion);
      selectedServerCandidate = releaseCandidate;
      return releaseCandidate.path;
    }
  }

  const fallbackCandidate = candidates[0];
  if (fallbackCandidate) {
    logSelectedServer(fallbackCandidate, expectedVersion);
    void warnAboutVersionMismatch(fallbackCandidate, expectedVersion);
    selectedServerCandidate = fallbackCandidate;
    return fallbackCandidate.path;
  }

  outputChannel.appendLine("Server not found in any location");
  return undefined;
}

function updateStatusBar(status: VizeStatus, detail: string): void {
  currentStatus = status;
  currentStatusDetail = detail;

  if (!statusBarItem) {
    return;
  }

  const statusText: Record<VizeStatus, string> = {
    disabled: "$(circle-slash) Vize",
    failed: "$(error) Vize",
    "missing-server": "$(warning) Vize",
    ready: "$(check) Vize",
    starting: "$(sync~spin) Vize",
  };

  statusBarItem.text = statusText[status];
  statusBarItem.tooltip = createStatusTooltip();
  statusBarItem.show();
}

function createStatusTooltip(): string {
  const lines = [`Vize: ${formatStatus(currentStatus)}`];

  if (currentStatusDetail) {
    lines.push(currentStatusDetail);
  }

  if (Object.keys(activeInitializationOptions).length > 0) {
    lines.push(`Features: ${describeCapabilities(activeInitializationOptions)}`);
  }

  if (selectedServerCandidate) {
    const version = selectedServerCandidate.version
      ? ` ${selectedServerCandidate.version}`
      : " unknown version";
    lines.push(`Server: ${selectedServerCandidate.source}${version}`, selectedServerCandidate.path);
  }

  lines.push("Click to manage Vize.");
  return lines.join("\n");
}

function createStatusSummary(
  config: ReturnType<typeof workspace.getConfiguration>,
  initializationOptions: LspInitializationOptions,
): string {
  const enabled = shouldStartFromConfiguration(config, workspaceLspConfig.hasConfig);
  const server = selectedServerCandidate
    ? `${selectedServerCandidate.source} ${selectedServerCandidate.version ?? "unknown"}`
    : "server not resolved";

  return `Vize is ${formatStatus(currentStatus)} (${enabled ? "enabled" : "disabled"}). Features: ${workspaceLspConfig.describeCapabilities(initializationOptions)}. Server: ${server}.`;
}

function formatStatus(status: VizeStatus): string {
  const labels: Record<VizeStatus, string> = {
    disabled: "disabled",
    failed: "failed",
    "missing-server": "server missing",
    ready: "ready",
    starting: "starting",
  };
  return labels[status];
}

function collectServerCandidates(context: ExtensionContext, exeName: string): ServerCandidate[] {
  const candidates: ServerCandidate[] = [];

  const bundledPaths = [
    path.join(context.extensionPath, "dist", exeName),
    path.join(context.extensionPath, "server", exeName),
  ];
  for (const serverPath of bundledPaths) {
    candidates.push({ path: serverPath, source: "bundled" });
  }

  const devPaths = [
    path.join(context.extensionPath, "..", "..", "target", "release", exeName),
    path.join(context.extensionPath, "..", "..", "target", "debug", exeName),
    ...getWorkspaceDevPaths(exeName),
  ];
  for (const serverPath of devPaths) {
    candidates.push({ path: serverPath, source: "development" });
  }

  const homeDir = process.env.HOME || process.env.USERPROFILE || "";
  if (homeDir) {
    candidates.push({
      path: path.join(homeDir, ".cargo", "bin", exeName),
      source: "cargo",
    });
  }

  const pathEnv = process.env.PATH || "";
  const pathSeparator = process.platform === "win32" ? ";" : ":";
  const pathDirs = [...pathEnv.split(pathSeparator), ...getImplicitPathDirs()];
  for (const dir of pathDirs) {
    if (dir) {
      candidates.push({ path: path.join(dir, exeName), source: "path" });
    }
  }

  return dedupeCandidates(candidates).filter((candidate) => fs.existsSync(candidate.path));
}

function dedupeCandidates(candidates: ServerCandidate[]): ServerCandidate[] {
  const seen = new Set<string>();
  const deduped: ServerCandidate[] = [];

  for (const candidate of candidates) {
    const key = path.resolve(candidate.path);
    if (seen.has(key)) {
      continue;
    }

    seen.add(key);
    deduped.push(candidate);
  }

  return deduped;
}

async function inspectServerCandidates(
  candidates: ServerCandidate[],
): Promise<InspectedServerCandidate[]> {
  return Promise.all(candidates.map(inspectServerCandidate));
}

async function inspectServerCandidate(
  candidate: ServerCandidate,
): Promise<InspectedServerCandidate> {
  try {
    const { stdout } = await execFileAsync(candidate.path, ["--version"], {
      timeout: 3000,
    });
    return {
      ...candidate,
      version: parseVizeVersion(stdout),
    };
  } catch (error) {
    return {
      ...candidate,
      versionError: String(error),
    };
  }
}

function getExtensionVersion(context: ExtensionContext): string | undefined {
  const packageJson = context.extension.packageJSON as { version?: unknown };
  return typeof packageJson.version === "string" ? packageJson.version : undefined;
}

function logSelectedServer(
  candidate: InspectedServerCandidate,
  expectedVersion: string | undefined,
): void {
  const version = candidate.version ?? "unknown";
  const expected = expectedVersion ? `, extension ${expectedVersion}` : "";
  outputChannel.appendLine(
    `Using ${candidate.source} server: ${candidate.path} (server ${version}${expected})`,
  );

  if (candidate.versionError) {
    outputChannel.appendLine(`Could not inspect server version: ${candidate.versionError}`);
  }
}

async function warnAboutVersionMismatch(
  candidate: InspectedServerCandidate,
  expectedVersion: string | undefined,
): Promise<void> {
  if (!expectedVersion || !candidate.version || candidate.version === expectedVersion) {
    return;
  }

  const selection = await window.showWarningMessage(
    `Vize: extension version ${expectedVersion} is starting language server ${candidate.version}. Hover, completion, and navigation may not work correctly.`,
    OPEN_SETTINGS_ACTION,
    SHOW_OUTPUT_ACTION,
  );

  if (selection === OPEN_SETTINGS_ACTION) {
    await commands.executeCommand("workbench.action.openSettings", "vize.serverPath");
    return;
  }

  if (selection === SHOW_OUTPUT_ACTION) {
    outputChannel.show();
  }
}

function getWorkspaceDevPaths(exeName: string): string[] {
  const folders = (workspace.workspaceFolders ?? []).map((folder) => folder.uri.fsPath);
  return workspaceDevServerPaths(folders, exeName, workspace.isTrusted);
}

function resolveTrustedServerPath(
  config: ReturnType<typeof workspace.getConfiguration>,
): string | undefined {
  return trustedServerPath(config.inspect<string>("serverPath"), workspace.isTrusted);
}

function getImplicitPathDirs(): string[] {
  const dirs: string[] = [];
  const homeDir = process.env.HOME || process.env.USERPROFILE || "";
  if (homeDir) {
    dirs.push(path.join(homeDir, ".local", "bin"), path.join(homeDir, ".nix-profile", "bin"));
  }

  if (process.platform !== "win32") {
    dirs.push(
      "/opt/homebrew/bin",
      "/usr/local/bin",
      "/usr/bin",
      "/bin",
      "/nix/var/nix/profiles/default/bin",
      "/run/current-system/sw/bin",
    );
  }

  return dirs;
}

async function resolveReleaseServerCandidate(
  context: ExtensionContext,
  exeName: string,
  expectedVersion: string,
  generation: number,
): Promise<InspectedServerCandidate | undefined> {
  const target = getReleaseServerTarget();
  if (!target) {
    outputChannel.appendLine(
      `No Vize release binary is available for ${process.platform}/${process.arch}.`,
    );
    return undefined;
  }

  const installDir = path.join(
    context.globalStorageUri.fsPath,
    "servers",
    expectedVersion,
    target.targetName,
  );
  const serverPath = path.join(installDir, exeName);
  const cachedCandidate = await inspectCachedReleaseServer(serverPath, expectedVersion);
  if (generation !== configurationSyncGeneration) return undefined;
  if (cachedCandidate) {
    return cachedCandidate;
  }

  const archivePath = path.join(installDir, target.archiveName);
  const downloadUrl = createReleaseDownloadUrl(expectedVersion, target.archiveName);
  const checksumUrl = `${downloadUrl}.sha256`;
  const checksumPath = `${archivePath}.sha256`;

  try {
    updateStatusBar("starting", `Downloading Vize language server ${expectedVersion}`);
    outputChannel.appendLine(`Downloading Vize language server ${expectedVersion}: ${downloadUrl}`);
    fs.rmSync(installDir, { force: true, recursive: true });
    await fs.promises.mkdir(installDir, { recursive: true });
    if (generation !== configurationSyncGeneration) return undefined;
    await downloadFile(downloadUrl, archivePath);
    if (generation !== configurationSyncGeneration) return undefined;

    // Verify SHA-256 sidecars before executing downloaded servers.
    // Missing, malformed, or mismatched checksums fail closed. (#969)
    await downloadFile(checksumUrl, checksumPath);
    if (generation !== configurationSyncGeneration) return undefined;
    await verifyArchiveChecksum(archivePath, checksumPath);
    if (generation !== configurationSyncGeneration) return undefined;

    await extractReleaseArchive(archivePath, installDir, exeName);
    if (generation !== configurationSyncGeneration) return undefined;

    const candidate = await inspectServerCandidate({
      path: serverPath,
      source: "release",
    });
    if (candidate.version === expectedVersion) {
      return candidate;
    }

    outputChannel.appendLine(
      `Downloaded Vize language server version ${candidate.version ?? "unknown"} does not match extension ${expectedVersion}.`,
    );
  } catch (error) {
    outputChannel.appendLine(`Failed to download Vize language server: ${String(error)}`);
  }

  return undefined;
}

async function inspectCachedReleaseServer(
  serverPath: string,
  expectedVersion: string,
): Promise<InspectedServerCandidate | undefined> {
  if (!fs.existsSync(serverPath)) {
    return undefined;
  }

  const candidate = await inspectServerCandidate({
    path: serverPath,
    source: "release",
  });
  if (candidate.version === expectedVersion) {
    return candidate;
  }

  outputChannel.appendLine(
    `Cached Vize language server ${candidate.version ?? "unknown"} does not match extension ${expectedVersion}. Refreshing cache.`,
  );
  return undefined;
}

function getReleaseServerTarget(): ReleaseServerTarget | undefined {
  if (process.platform === "darwin") {
    if (process.arch === "arm64") {
      return releaseTarget("aarch64-apple-darwin", "tar.gz");
    }
    if (process.arch === "x64") {
      return releaseTarget("x86_64-apple-darwin", "tar.gz");
    }
  }

  if (process.platform === "linux") {
    if (process.arch === "arm64") {
      return releaseTarget("aarch64-unknown-linux-gnu", "tar.gz");
    }
    if (process.arch === "x64") {
      return releaseTarget("x86_64-unknown-linux-gnu", "tar.gz");
    }
  }

  if (process.platform === "win32") {
    if (process.arch === "arm64") {
      return releaseTarget("aarch64-pc-windows-msvc", "zip");
    }
    if (process.arch === "x64") {
      return releaseTarget("x86_64-pc-windows-msvc", "zip");
    }
  }

  return undefined;
}

function releaseTarget(targetName: string, extension: "tar.gz" | "zip"): ReleaseServerTarget {
  return {
    archiveName: `vize-${targetName}.${extension}`,
    targetName,
  };
}

function createReleaseDownloadUrl(version: string, archiveName: string): string {
  return `https://github.com/ubugeeei-prod/vize/releases/download/v${version}/${archiveName}`;
}

/// Verify that `archivePath` matches the declared SHA-256 in `checksumPath`.
/// Accepts bare hex or GNU sha256sum shape; malformed or mismatched files throw. (#969)
async function verifyArchiveChecksum(archivePath: string, checksumPath: string): Promise<void> {
  const declared = (await fs.promises.readFile(checksumPath, "utf8"))
    .trim()
    .split(/\s+/)[0]
    ?.toLowerCase();
  if (!declared || !/^[0-9a-f]{64}$/.test(declared)) {
    throw new Error(`malformed SHA-256 checksum file at ${checksumPath}`);
  }
  const hash = crypto.createHash("sha256");
  await new Promise<void>((resolve, reject) => {
    const stream = fs.createReadStream(archivePath);
    stream.on("data", (chunk: string | Buffer) => {
      hash.update(chunk);
    });
    stream.on("end", () => resolve());
    stream.on("error", reject);
  });
  const got = hash.digest("hex");
  if (got !== declared) {
    throw new Error(`SHA-256 mismatch for ${archivePath}: declared ${declared}, computed ${got}`);
  }
  outputChannel.appendLine(`Verified ${path.basename(archivePath)} (SHA-256 ${got}).`);
}
