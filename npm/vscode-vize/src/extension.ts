import * as fs from "fs";
import * as path from "path";
import {
  ConfigurationTarget,
  ExtensionContext,
  OutputChannel,
  Uri,
  commands,
  env,
  window,
  workspace,
} from "vscode";
import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  Trace,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;
let outputChannel: OutputChannel;

type LspInitializationOptions = Partial<Record<string, boolean>>;
const SUPPORTED_LANGUAGE_IDS = ["vue", "art-vue"] as const;
const SUPPORTED_URI_SCHEMES = ["file", "untitled"] as const;
const RECOMMENDED_SETUP_ACTION = "Enable Recommended";
const OPEN_SETTINGS_ACTION = "Open Settings";
const DISMISS_ACTION = "Dismiss";
const OPEN_SETUP_DOCS_ACTION = "Open Setup Docs";
const SHOW_OUTPUT_ACTION = "Show Output";
const INITIAL_SETUP_PROMPT_DISMISSED_KEY = "vize.initialSetupPrompt.dismissed";
const CAPABILITY_PROMPT_DISMISSED_KEY = "vize.capabilityPrompt.dismissed";
const FEATURE_SETTING_KEYS = [
  "lint.enable",
  "diagnostics.enable",
  "typecheck.enable",
  "editor.enable",
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

export async function activate(context: ExtensionContext): Promise<void> {
  outputChannel = window.createOutputChannel("Vize");
  outputChannel.appendLine("Vize extension activating...");
  context.subscriptions.push(outputChannel);

  context.subscriptions.push(
    workspace.onDidChangeConfiguration(async (event) => {
      if (!event.affectsConfiguration("vize")) {
        return;
      }

      outputChannel.appendLine("Vize configuration changed. Refreshing language server...");
      await syncClientToConfiguration(context, "configuration changed");
    }),
  );

  context.subscriptions.push(
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

async function syncClientToConfiguration(context: ExtensionContext, reason: string): Promise<void> {
  let config = workspace.getConfiguration("vize");

  if (!config.get<boolean>("enable", false)) {
    await maybeOfferInitialSetup(context, config);
    config = workspace.getConfiguration("vize");

    if (!config.get<boolean>("enable", false)) {
      if (client) {
        outputChannel.appendLine(`Stopping Vize language server (${reason}; extension disabled).`);
        await stopClient();
      } else {
        outputChannel.appendLine("Vize is disabled. Set vize.enable to true to start the server.");
      }
      return;
    }

    outputChannel.appendLine("Recommended Vize setup was applied. Starting language server...");
  }

  if (client) {
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
    "Vize is installed but disabled. Enable the recommended diagnostics and navigation profile for this workspace?",
    RECOMMENDED_SETUP_ACTION,
    OPEN_SETTINGS_ACTION,
    DISMISS_ACTION,
  );

  if (selection === RECOMMENDED_SETUP_ACTION) {
    await applyRecommendedConfiguration();
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
    "Vize is enabled but no language features are turned on. Enable diagnostics and navigation for this workspace?",
    RECOMMENDED_SETUP_ACTION,
    OPEN_SETTINGS_ACTION,
    DISMISS_ACTION,
  );

  if (selection === RECOMMENDED_SETUP_ACTION) {
    await applyRecommendedConfiguration();
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
  const config = workspace.getConfiguration("vize");
  const target = getConfigurationTarget();

  await config.update("enable", true, target);
  await config.update("lint.enable", true, target);
  await config.update("typecheck.enable", true, target);
  await config.update("editor.enable", true, target);
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

async function showServerNotFoundMessage(): Promise<void> {
  const selection = await window.showErrorMessage(
    "Vize: Could not find the language server. Install the vize CLI with `cargo install vize` or set vize.serverPath.",
    OPEN_SETUP_DOCS_ACTION,
    OPEN_SETTINGS_ACTION,
    SHOW_OUTPUT_ACTION,
  );

  if (selection === OPEN_SETTINGS_ACTION) {
    await commands.executeCommand("workbench.action.openSettings", "vize.serverPath");
    return;
  }

  if (selection === OPEN_SETUP_DOCS_ACTION) {
    await env.openExternal(
      Uri.parse("https://github.com/ubugeeei/vize/tree/main/npm/vscode-vize#readme"),
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
  if (Object.keys(initializationOptions).length === 0) {
    outputChannel.appendLine(
      "Vize server is enabled with no opt-in features. Enable lint, typecheck, and editor assistance to activate diagnostics and navigation.",
    );
    void maybeOfferCapabilitySetup(context, config);
  }

  const serverPath = await findServerPath(context, config);
  if (!serverPath) {
    await showServerNotFoundMessage();
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
  } catch (error) {
    outputChannel.appendLine(`Failed to start language server: ${String(error)}`);
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
      fileEvents: workspace.createFileSystemWatcher("**/*.vue"),
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
): LspInitializationOptions {
  const options: LspInitializationOptions = {};

  setIfEnabled(options, "lint", config.get<boolean>("lint.enable", false));
  setIfEnabled(options, "lint", config.get<boolean>("diagnostics.enable", false));
  setIfEnabled(options, "typecheck", config.get<boolean>("typecheck.enable", false));
  setIfEnabled(options, "editor", config.get<boolean>("editor.enable", false));
  setIfEnabled(options, "completion", config.get<boolean>("completion.enable", false));
  setIfEnabled(options, "hover", config.get<boolean>("hover.enable", false));
  setIfEnabled(options, "definition", config.get<boolean>("definition.enable", false));
  setIfEnabled(options, "references", config.get<boolean>("references.enable", false));
  setIfEnabled(options, "documentSymbols", config.get<boolean>("documentSymbols.enable", false));
  setIfEnabled(options, "workspaceSymbols", config.get<boolean>("workspaceSymbols.enable", false));
  setIfEnabled(options, "codeActions", config.get<boolean>("codeActions.enable", false));
  setIfEnabled(options, "rename", config.get<boolean>("rename.enable", false));
  setIfEnabled(options, "codeLens", config.get<boolean>("codeLens.enable", false));
  setIfEnabled(options, "formatting", config.get<boolean>("formatting.enable", false));
  setIfEnabled(options, "semanticTokens", config.get<boolean>("semanticTokens.enable", false));
  setIfEnabled(options, "documentLinks", config.get<boolean>("documentLinks.enable", false));
  setIfEnabled(options, "foldingRanges", config.get<boolean>("foldingRanges.enable", false));
  setIfEnabled(options, "inlayHints", config.get<boolean>("inlayHints.enable", false));
  setIfEnabled(options, "fileRename", config.get<boolean>("fileRename.enable", false));

  return options;
}

function setIfEnabled(
  options: LspInitializationOptions,
  name: string,
  enabled: boolean | undefined,
): void {
  if (enabled === true) {
    options[name] = true;
  }
}

export async function deactivate(): Promise<void> {
  await stopClient();
}

async function findServerPath(
  context: ExtensionContext,
  config: ReturnType<typeof workspace.getConfiguration>,
): Promise<string | undefined> {
  const exeName = process.platform === "win32" ? "vize.exe" : "vize";

  const configuredPath = config.get<string>("serverPath");
  if (configuredPath && fs.existsSync(configuredPath)) {
    outputChannel.appendLine(`Found server at configured path: ${configuredPath}`);
    return configuredPath;
  }

  const homeDir = process.env.HOME || process.env.USERPROFILE || "";
  const cargoPath = path.join(homeDir, ".cargo", "bin", exeName);
  if (fs.existsSync(cargoPath)) {
    outputChannel.appendLine(`Found server at cargo path: ${cargoPath}`);
    return cargoPath;
  }

  const pathEnv = process.env.PATH || "";
  const pathSeparator = process.platform === "win32" ? ";" : ":";
  const pathDirs = pathEnv.split(pathSeparator);

  for (const dir of pathDirs) {
    const serverPath = path.join(dir, exeName);
    if (fs.existsSync(serverPath)) {
      outputChannel.appendLine(`Found server in PATH: ${serverPath}`);
      return serverPath;
    }
  }

  const bundledPaths = [
    path.join(context.extensionPath, "dist", exeName),
    path.join(context.extensionPath, "server", exeName),
  ];

  for (const serverPath of bundledPaths) {
    if (fs.existsSync(serverPath)) {
      outputChannel.appendLine(`Found bundled server: ${serverPath}`);
      return serverPath;
    }
  }

  const devPaths = [
    path.join(context.extensionPath, "..", "..", "target", "release", exeName),
    path.join(context.extensionPath, "..", "..", "target", "debug", exeName),
    ...getWorkspaceDevPaths(exeName),
  ];

  for (const serverPath of devPaths) {
    if (fs.existsSync(serverPath)) {
      outputChannel.appendLine(`Found dev server: ${serverPath}`);
      return serverPath;
    }
  }

  outputChannel.appendLine("Server not found in any location");
  return undefined;
}

function getWorkspaceDevPaths(exeName: string): string[] {
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
