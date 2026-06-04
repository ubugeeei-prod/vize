import * as fs from "fs";
import * as https from "https";
import * as path from "path";
import * as zlib from "zlib";
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
  type QuickPickItem,
  type StatusBarItem,
} from "vscode";
import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  Trace,
  TransportKind,
} from "vscode-languageclient/node";

const execFileAsync = promisify(execFile);
let client: LanguageClient | undefined;
let outputChannel: OutputChannel;
let statusBarItem: StatusBarItem | undefined;
let selectedServerCandidate: InspectedServerCandidate | undefined;
let activeInitializationOptions: LspInitializationOptions = {};
let currentStatus: VizeStatus = "disabled";
let currentStatusDetail = "";
let configurationSyncTimer: ReturnType<typeof setTimeout> | undefined;
let suppressConfigurationSync = false;

type LspInitializationOptions = Partial<Record<string, boolean>>;
type ServerCandidateSource =
  | "configured"
  | "bundled"
  | "development"
  | "release"
  | "cargo"
  | "path";
type ServerCandidate = {
  path: string;
  source: ServerCandidateSource;
};
type InspectedServerCandidate = ServerCandidate & {
  version?: string;
  versionError?: string;
};
type ReleaseServerTarget = {
  archiveName: string;
  targetName: string;
};
type VizeStatus = "disabled" | "starting" | "ready" | "missing-server" | "failed";
type VizeStatusAction =
  | "recommended"
  | "lintOnly"
  | "selectServer"
  | "restart"
  | "settings"
  | "output"
  | "disable";
type VizeStatusQuickPickItem = QuickPickItem & {
  action: VizeStatusAction;
};
const SUPPORTED_LANGUAGE_IDS = ["vue", "art-vue", "html"] as const;
const SUPPORTED_URI_SCHEMES = ["file", "untitled"] as const;
const RECOMMENDED_SETUP_ACTION = "Enable Recommended";
const LINT_ONLY_SETUP_ACTION = "Enable Lint Only";
const OPEN_SETTINGS_ACTION = "Open Settings";
const DISMISS_ACTION = "Dismiss";
const OPEN_SETUP_DOCS_ACTION = "Open Setup Docs";
const SELECT_SERVER_ACTION = "Select Binary";
const SHOW_OUTPUT_ACTION = "Show Output";
const INITIAL_SETUP_PROMPT_DISMISSED_KEY = "vize.initialSetupPrompt.dismissed";
const CAPABILITY_PROMPT_DISMISSED_KEY = "vize.capabilityPrompt.dismissed";
const RELEASE_DOWNLOAD_TIMEOUT_MS = 30_000;
const RELEASE_DOWNLOAD_MAX_REDIRECTS = 5;
const FEATURE_SETTING_KEYS = [
  "lint.enable",
  "diagnostics.enable",
  "typecheck.enable",
  "editor.enable",
  "ecosystem.enable",
  "legacyVue2.enable",
  "completion.enable",
  "hover.enable",
  "definition.enable",
  "references.enable",
  "documentSymbols.enable",
  "workspaceSymbols.enable",
  "codeActions.enable",
  "rename.enable",
  "codeLens.enable",
  "formatting.enable",
  "semanticTokens.enable",
  "documentLinks.enable",
  "foldingRanges.enable",
  "inlayHints.enable",
  "fileRename.enable",
] as const;
const CAPABILITY_LABELS: Record<string, string> = {
  lint: "lint",
  typecheck: "type check",
  editor: "editor bundle",
  legacyVue2: "Vue 2.7 / Nuxt 2",
  completion: "completion",
  hover: "hover",
  definition: "definition",
  references: "references",
  documentSymbols: "document symbols",
  workspaceSymbols: "workspace symbols",
  codeActions: "code actions",
  rename: "rename",
  codeLens: "code lens",
  formatting: "formatting",
  semanticTokens: "semantic tokens",
  documentLinks: "document links",
  foldingRanges: "folding",
  inlayHints: "inlay hints",
  fileRename: "file rename",
};

export async function activate(context: ExtensionContext): Promise<void> {
  outputChannel = window.createOutputChannel("Vize");
  outputChannel.appendLine("Vize extension activating...");
  context.subscriptions.push(outputChannel);

  statusBarItem = window.createStatusBarItem(StatusBarAlignment.Right, 95);
  statusBarItem.command = "vize.showStatus";
  context.subscriptions.push(statusBarItem);
  updateStatusBar("starting", "Initializing Vize");

  context.subscriptions.push(
    workspace.onDidChangeConfiguration(async (event) => {
      if (!event.affectsConfiguration("vize")) {
        return;
      }

      if (suppressConfigurationSync) {
        outputChannel.appendLine("Vize configuration changed while applying a profile.");
        return;
      }

      outputChannel.appendLine("Vize configuration changed. Refreshing language server...");
      scheduleClientSync(context, "configuration changed");
    }),
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
  );

  await syncClientToConfiguration(context, "initial activation");
}

function scheduleClientSync(context: ExtensionContext, reason: string): void {
  if (configurationSyncTimer) {
    clearTimeout(configurationSyncTimer);
  }

  configurationSyncTimer = setTimeout(() => {
    configurationSyncTimer = undefined;
    void syncClientToConfiguration(context, reason);
  }, 150);
}

async function syncClientToConfiguration(context: ExtensionContext, reason: string): Promise<void> {
  let config = workspace.getConfiguration("vize");
  let enabled = shouldStartFromConfiguration(config);

  if (!enabled) {
    await maybeOfferInitialSetup(context, config);
    config = workspace.getConfiguration("vize");
    enabled = shouldStartFromConfiguration(config);

    if (!enabled) {
      if (client) {
        updateStatusBar("starting", "Stopping language server");
        outputChannel.appendLine(`Stopping Vize language server (${reason}; extension disabled).`);
        await stopClient();
      } else {
        outputChannel.appendLine("Vize is disabled. Set vize.enable to true to start the server.");
      }
      activeInitializationOptions = {};
      selectedServerCandidate = undefined;
      updateStatusBar("disabled", "Language server is disabled");
      return;
    }

    outputChannel.appendLine("Recommended Vize setup was applied. Starting language server...");
  } else if (!config.get<boolean>("enable", false)) {
    outputChannel.appendLine("Starting Vize language server from workspace vize.config.");
  }

  if (client) {
    updateStatusBar("starting", "Restarting language server");
    outputChannel.appendLine(`Restarting Vize language server (${reason}).`);
    await stopClient();
  }

  await startClient(context, config);
}

async function maybeOfferInitialSetup(
  context: ExtensionContext,
  config: ReturnType<typeof workspace.getConfiguration>,
): Promise<void> {
  if (hasExplicitConfigurationValue(config, "enable")) {
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
  if (hasAnyEnabledCapability(config) || hasWorkspaceLspConfig()) {
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
  await applyConfigurationUpdates([
    ["enable", true],
    ["lint.enable", true],
    ["diagnostics.enable", false],
    ["typecheck.enable", false],
    ["editor.enable", false],
    ["ecosystem.enable", false],
    ["completion.enable", false],
    ["hover.enable", false],
    ["definition.enable", false],
    ["references.enable", false],
    ["documentSymbols.enable", false],
    ["workspaceSymbols.enable", false],
    ["codeActions.enable", false],
    ["rename.enable", false],
    ["codeLens.enable", false],
    ["formatting.enable", false],
    ["semanticTokens.enable", false],
    ["documentLinks.enable", false],
    ["foldingRanges.enable", false],
    ["inlayHints.enable", false],
    ["fileRename.enable", false],
  ]);
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
  const initializationOptions = getInitializationOptions(config, { logDefaultProfile: false });
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
  const enabled = config.get<boolean>("enable", false);
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

function hasExplicitConfigurationValue(
  config: ReturnType<typeof workspace.getConfiguration>,
  key: string,
): boolean {
  const inspected = config.inspect(key) as
    | {
        globalValue?: unknown;
        workspaceValue?: unknown;
        workspaceFolderValue?: unknown;
      }
    | undefined;

  return (
    inspected?.globalValue !== undefined ||
    inspected?.workspaceValue !== undefined ||
    inspected?.workspaceFolderValue !== undefined
  );
}

function hasAnyEnabledCapability(config: ReturnType<typeof workspace.getConfiguration>): boolean {
  return FEATURE_SETTING_KEYS.some((key) => config.get<boolean>(key, false));
}

function hasAnyExplicitCapabilityValue(
  config: ReturnType<typeof workspace.getConfiguration>,
): boolean {
  return FEATURE_SETTING_KEYS.some((key) => hasExplicitConfigurationValue(config, key));
}

function shouldStartFromConfiguration(
  config: ReturnType<typeof workspace.getConfiguration>,
): boolean {
  if (config.get<boolean>("enable", false)) {
    return true;
  }

  if (hasExplicitConfigurationValue(config, "enable")) {
    return false;
  }

  return hasWorkspaceLspConfig();
}

function hasWorkspaceLspConfig(): boolean {
  const workspaceFolders = workspace.workspaceFolders;
  if (!workspaceFolders) {
    return false;
  }

  return workspaceFolders.some((folder) =>
    ["vize.config.pkl", "vize.config.json"].some((filename) =>
      fs.existsSync(path.join(folder.uri.fsPath, filename)),
    ),
  );
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
      Uri.parse("https://github.com/ubugeeei-prod/vize/tree/main/npm/vscode-vize#readme"),
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
): Promise<void> {
  const initializationOptions = getInitializationOptions(config);
  activeInitializationOptions = initializationOptions;
  updateStatusBar("starting", `Starting with ${describeCapabilities(initializationOptions)}`);

  if (Object.keys(initializationOptions).length === 0) {
    outputChannel.appendLine(
      "Vize server is enabled with no opt-in features. Enable lint, typecheck, editor assistance, and ecosystem helpers to activate diagnostics and navigation.",
    );
    void maybeOfferCapabilitySetup(context, config);
  }

  const serverPath = await findServerPath(context, config);
  if (!serverPath) {
    updateStatusBar("missing-server", "Language server executable was not found");
    await showServerNotFoundMessage(context);
    return;
  }

  outputChannel.appendLine(`Using server: ${serverPath}`);

  const serverOptions: ServerOptions = createServerOptions(serverPath);
  const nextClient = new LanguageClient(
    "vize",
    "Vize Language Server",
    serverOptions,
    createClientOptions(initializationOptions),
  );

  applyTraceSetting(nextClient, config);

  try {
    await nextClient.start();
    client = nextClient;
    outputChannel.appendLine("Vize language server started successfully");
    updateStatusBar("ready", `Ready with ${describeCapabilities(initializationOptions)}`);
  } catch (error) {
    outputChannel.appendLine(`Failed to start language server: ${String(error)}`);
    updateStatusBar("failed", "Failed to start language server");
    window.showErrorMessage(`Vize: Failed to start language server: ${String(error)}`);
  }
}

async function stopClient(): Promise<void> {
  if (!client) {
    return;
  }

  const activeClient = client;
  client = undefined;
  await activeClient.stop();
}

function createClientOptions(
  initializationOptions: LspInitializationOptions,
): LanguageClientOptions {
  return {
    documentSelector: SUPPORTED_URI_SCHEMES.flatMap((scheme) =>
      SUPPORTED_LANGUAGE_IDS.map((language) => ({
        scheme,
        language,
      })),
    ),
    synchronize: {
      configurationSection: "vize",
      fileEvents: [
        workspace.createFileSystemWatcher("**/*.vue"),
        workspace.createFileSystemWatcher("**/*.{html,htm}"),
      ],
    },
    outputChannel,
    traceOutputChannel: outputChannel,
    initializationOptions,
  };
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

function getInitializationOptions(
  config: ReturnType<typeof workspace.getConfiguration>,
  behavior: { logDefaultProfile?: boolean } = {},
): LspInitializationOptions {
  const options: LspInitializationOptions = {};

  setFeatureOption(options, config, "lint.enable", "lint", true);
  setDiagnosticsAliasOption(options, config);
  setFeatureOption(options, config, "typecheck.enable", "typecheck", true);
  setFeatureOption(options, config, "editor.enable", "editor", true);
  setFeatureOption(options, config, "ecosystem.enable", "ecosystem", true);
  setFeatureOption(options, config, "legacyVue2.enable", "legacyVue2", false);
  setFeatureOption(options, config, "completion.enable", "completion", true);
  setFeatureOption(options, config, "hover.enable", "hover", true);
  setFeatureOption(options, config, "definition.enable", "definition", true);
  setFeatureOption(options, config, "references.enable", "references", true);
  setFeatureOption(options, config, "documentSymbols.enable", "documentSymbols", true);
  setFeatureOption(options, config, "workspaceSymbols.enable", "workspaceSymbols", true);
  setFeatureOption(options, config, "codeActions.enable", "codeActions", true);
  setFeatureOption(options, config, "rename.enable", "rename", true);
  setFeatureOption(options, config, "codeLens.enable", "codeLens", true);
  setFeatureOption(options, config, "formatting.enable", "formatting", false);
  setFeatureOption(options, config, "semanticTokens.enable", "semanticTokens", true);
  setFeatureOption(options, config, "documentLinks.enable", "documentLinks", true);
  setFeatureOption(options, config, "foldingRanges.enable", "foldingRanges", true);
  setFeatureOption(options, config, "inlayHints.enable", "inlayHints", true);
  setFeatureOption(options, config, "fileRename.enable", "fileRename", true);

  if (
    Object.keys(options).length === 0 &&
    config.get<boolean>("enable", false) &&
    !hasAnyExplicitCapabilityValue(config) &&
    !hasWorkspaceLspConfig()
  ) {
    if (behavior.logDefaultProfile !== false) {
      outputChannel.appendLine(
        "Vize is enabled with no explicit feature switches. Using the recommended diagnostics, editor, and ecosystem profile.",
      );
    }
    options.lint = true;
    options.typecheck = true;
    options.editor = true;
    options.ecosystem = true;
  }

  return options;
}

function setDiagnosticsAliasOption(
  options: LspInitializationOptions,
  config: ReturnType<typeof workspace.getConfiguration>,
): void {
  const enabled = config.get<boolean>("diagnostics.enable", false);
  if (enabled === true) {
    options.lint = true;
    return;
  }
  if (
    hasExplicitConfigurationValue(config, "diagnostics.enable") &&
    !hasExplicitConfigurationValue(config, "lint.enable")
  ) {
    options.lint = false;
  }
}

function setFeatureOption(
  options: LspInitializationOptions,
  config: ReturnType<typeof workspace.getConfiguration>,
  key: string,
  name: string,
  defaultValue: boolean,
): void {
  if (!hasExplicitConfigurationValue(config, key)) {
    return;
  }

  const enabled = config.get<boolean>(key, defaultValue);
  options[name] = enabled;
}

export async function deactivate(): Promise<void> {
  await stopClient();
}

async function findServerPath(
  context: ExtensionContext,
  config: ReturnType<typeof workspace.getConfiguration>,
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
      logSelectedServer(candidate, expectedVersion);
      void warnAboutVersionMismatch(candidate, expectedVersion);
      selectedServerCandidate = candidate;
      return configuredPath;
    }

    outputChannel.appendLine(`Configured server path does not exist: ${configuredPath}`);
  }

  const candidates = await inspectServerCandidates(collectServerCandidates(context, exeName));

  if (expectedVersion) {
    const matchingCandidate = candidates.find((candidate) => candidate.version === expectedVersion);
    if (matchingCandidate) {
      logSelectedServer(matchingCandidate, expectedVersion);
      selectedServerCandidate = matchingCandidate;
      return matchingCandidate.path;
    }

    const releaseCandidate = await resolveReleaseServerCandidate(context, exeName, expectedVersion);
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
  const enabled = config.get<boolean>("enable", false) ? "enabled" : "disabled";
  const server = selectedServerCandidate
    ? `${selectedServerCandidate.source} ${selectedServerCandidate.version ?? "unknown"}`
    : "server not resolved";

  return `Vize is ${formatStatus(currentStatus)} (${enabled}). Features: ${describeCapabilities(initializationOptions)}. Server: ${server}.`;
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

function describeCapabilities(options: LspInitializationOptions): string {
  const capabilities = Object.entries(options)
    .filter(([, enabled]) => enabled === true)
    .map(([name]) => CAPABILITY_LABELS[name] ?? name);

  return capabilities.length ? capabilities.join(", ") : "none";
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

function parseVizeVersion(output: string): string | undefined {
  const match = output.match(/\bvize\s+([0-9]+\.[0-9]+\.[0-9]+(?:[-+][^\s]+)?)/);
  return match?.[1];
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
  // Skip workspace-derived server paths in untrusted workspaces — a cloned
  // repo that ships a malicious binary at `target/{release,debug}/vize`
  // would otherwise be executed when the workspace was opened. (#969)
  if (!workspace.isTrusted) {
    return [];
  }
  const paths: string[] = [];
  const workspaceFolders = workspace.workspaceFolders;
  if (workspaceFolders) {
    for (const folder of workspaceFolders) {
      paths.push(path.join(folder.uri.fsPath, "target", "release", exeName));
      paths.push(path.join(folder.uri.fsPath, "target", "debug", exeName));
    }
  }
  return paths;
}

/// Resolve the user-/machine-scoped value of `vize.serverPath` only.
/// Workspace-scoped values (e.g. a `.vscode/settings.json` shipped in a
/// cloned repo) are intentionally ignored — that file is repo-controlled
/// and could otherwise point the extension at any executable. The
/// untrusted-workspace branch declared in `package.json::capabilities`
/// covers the first defense in depth; this is the second. (#969)
function resolveTrustedServerPath(
  config: ReturnType<typeof workspace.getConfiguration>,
): string | undefined {
  const inspected = config.inspect<string>("serverPath");
  if (!inspected) {
    return undefined;
  }
  const candidates: Array<string | undefined> = [
    inspected.globalValue,
    inspected.workspaceValue && workspace.isTrusted ? inspected.workspaceValue : undefined,
    inspected.workspaceFolderValue && workspace.isTrusted
      ? inspected.workspaceFolderValue
      : undefined,
  ];
  for (const raw of candidates) {
    const trimmed = raw?.trim();
    if (trimmed) {
      return trimmed;
    }
  }
  return undefined;
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
  if (cachedCandidate) {
    return cachedCandidate;
  }

  const archivePath = path.join(installDir, target.archiveName);
  const downloadUrl = createReleaseDownloadUrl(expectedVersion, target.archiveName);

  try {
    updateStatusBar("starting", `Downloading Vize language server ${expectedVersion}`);
    outputChannel.appendLine(`Downloading Vize language server ${expectedVersion}: ${downloadUrl}`);
    fs.rmSync(installDir, { force: true, recursive: true });
    await fs.promises.mkdir(installDir, { recursive: true });
    await downloadFile(downloadUrl, archivePath);
    await extractReleaseArchive(archivePath, installDir, exeName);

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

async function downloadFile(url: string, destination: string, redirectCount = 0): Promise<void> {
  if (redirectCount > RELEASE_DOWNLOAD_MAX_REDIRECTS) {
    throw new Error(`too many redirects while downloading ${url}`);
  }

  await new Promise<void>((resolve, reject) => {
    const request = https.get(
      url,
      {
        headers: {
          "User-Agent": "vscode-vize",
        },
      },
      (response) => {
        const statusCode = response.statusCode ?? 0;
        if (statusCode >= 300 && statusCode < 400 && response.headers.location) {
          response.resume();
          const redirectUrl = new URL(response.headers.location, url).toString();
          downloadFile(redirectUrl, destination, redirectCount + 1).then(resolve, reject);
          return;
        }

        if (statusCode !== 200) {
          response.resume();
          reject(new Error(`download failed with HTTP ${statusCode}: ${url}`));
          return;
        }

        const file = fs.createWriteStream(destination);
        file.on("error", reject);
        file.on("finish", () => {
          file.close((error) => {
            if (error) {
              reject(error);
              return;
            }
            resolve();
          });
        });
        response.pipe(file);
      },
    );

    request.setTimeout(RELEASE_DOWNLOAD_TIMEOUT_MS, () => {
      request.destroy(new Error(`download timed out after ${RELEASE_DOWNLOAD_TIMEOUT_MS}ms`));
    });
    request.on("error", reject);
  }).catch(async (error) => {
    await fs.promises.rm(destination, { force: true });
    throw error;
  });
}

async function extractReleaseArchive(
  archivePath: string,
  destinationDir: string,
  exeName: string,
): Promise<void> {
  const archive = await fs.promises.readFile(archivePath);
  if (archivePath.endsWith(".zip")) {
    await extractServerFromZip(archive, destinationDir, exeName);
    return;
  }

  const tar = zlib.gunzipSync(archive);
  await extractServerFromTar(tar, destinationDir, exeName);
}

async function extractServerFromTar(
  tar: Buffer,
  destinationDir: string,
  exeName: string,
): Promise<void> {
  for (let offset = 0; offset + 512 <= tar.byteLength; offset += 512) {
    const header = tar.subarray(offset, offset + 512);
    if (header.every((byte) => byte === 0)) {
      break;
    }

    const name = readTarString(header, 0, 100);
    const prefix = readTarString(header, 345, 155);
    const entryName = prefix ? `${prefix}/${name}` : name;
    const size = parseInt(readTarString(header, 124, 12).trim() || "0", 8);
    const typeFlag = readTarString(header, 156, 1);
    const dataOffset = offset + 512;
    const nextOffset = dataOffset + Math.ceil(size / 512) * 512;

    if ((typeFlag === "" || typeFlag === "0") && path.basename(entryName) === exeName) {
      await writeExtractedServer(
        tar.subarray(dataOffset, dataOffset + size),
        path.join(destinationDir, exeName),
      );
      return;
    }

    offset = nextOffset - 512;
  }

  throw new Error(`archive did not contain ${exeName}`);
}

async function extractServerFromZip(
  zip: Buffer,
  destinationDir: string,
  exeName: string,
): Promise<void> {
  const eocdOffset = findEndOfCentralDirectory(zip);
  const entryCount = zip.readUInt16LE(eocdOffset + 10);
  let centralOffset = zip.readUInt32LE(eocdOffset + 16);

  for (let index = 0; index < entryCount; index++) {
    if (zip.readUInt32LE(centralOffset) !== 0x02014b50) {
      throw new Error("invalid zip central directory");
    }

    const compressionMethod = zip.readUInt16LE(centralOffset + 10);
    const compressedSize = zip.readUInt32LE(centralOffset + 20);
    const uncompressedSize = zip.readUInt32LE(centralOffset + 24);
    const fileNameLength = zip.readUInt16LE(centralOffset + 28);
    const extraLength = zip.readUInt16LE(centralOffset + 30);
    const commentLength = zip.readUInt16LE(centralOffset + 32);
    const localHeaderOffset = zip.readUInt32LE(centralOffset + 42);
    const entryName = zip
      .subarray(centralOffset + 46, centralOffset + 46 + fileNameLength)
      .toString("utf-8");

    if (path.basename(entryName) === exeName) {
      const extracted = extractZipEntry(zip, {
        compressedSize,
        compressionMethod,
        localHeaderOffset,
        uncompressedSize,
      });
      await writeExtractedServer(extracted, path.join(destinationDir, exeName));
      return;
    }

    centralOffset += 46 + fileNameLength + extraLength + commentLength;
  }

  throw new Error(`archive did not contain ${exeName}`);
}

function extractZipEntry(
  zip: Buffer,
  entry: {
    compressedSize: number;
    compressionMethod: number;
    localHeaderOffset: number;
    uncompressedSize: number;
  },
): Buffer {
  if (zip.readUInt32LE(entry.localHeaderOffset) !== 0x04034b50) {
    throw new Error("invalid zip local header");
  }

  const fileNameLength = zip.readUInt16LE(entry.localHeaderOffset + 26);
  const extraLength = zip.readUInt16LE(entry.localHeaderOffset + 28);
  const dataOffset = entry.localHeaderOffset + 30 + fileNameLength + extraLength;
  const compressed = zip.subarray(dataOffset, dataOffset + entry.compressedSize);

  if (entry.compressionMethod === 0) {
    if (compressed.byteLength !== entry.uncompressedSize) {
      throw new Error("stored zip entry has an unexpected size");
    }
    return compressed;
  }

  if (entry.compressionMethod === 8) {
    const inflated = zlib.inflateRawSync(compressed);
    if (inflated.byteLength !== entry.uncompressedSize) {
      throw new Error("deflated zip entry has an unexpected size");
    }
    return inflated;
  }

  throw new Error(`unsupported zip compression method ${entry.compressionMethod}`);
}

function findEndOfCentralDirectory(zip: Buffer): number {
  const minimumOffset = Math.max(0, zip.byteLength - 65_557);
  for (let offset = zip.byteLength - 22; offset >= minimumOffset; offset--) {
    if (zip.readUInt32LE(offset) === 0x06054b50) {
      return offset;
    }
  }

  throw new Error("invalid zip archive");
}

function readTarString(buffer: Buffer, start: number, length: number): string {
  const raw = buffer.subarray(start, start + length);
  const end = raw.indexOf(0);
  return raw.subarray(0, end === -1 ? raw.byteLength : end).toString("utf-8");
}

async function writeExtractedServer(contents: Buffer, serverPath: string): Promise<void> {
  await fs.promises.writeFile(serverPath, contents);
  if (process.platform !== "win32") {
    await fs.promises.chmod(serverPath, 0o755);
  }
}

function createServerOptions(serverPath: string): ServerOptions {
  const run: Executable = {
    command: serverPath,
    args: ["lsp"],
    transport: TransportKind.stdio,
  };

  const debug: Executable = {
    command: serverPath,
    args: ["lsp", "--debug"],
    transport: TransportKind.stdio,
    options: {
      env: {
        ...process.env,
        RUST_BACKTRACE: "1",
      },
    },
  };

  return {
    run,
    debug,
  };
}
