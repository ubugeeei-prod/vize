import {
  ConfigurationTarget,
  Uri,
  commands,
  env,
  window,
  workspace,
  type ExtensionContext,
} from "vscode";
import {
  LINT_ONLY_CONFIGURATION_UPDATES,
  describeCapabilities,
  getInitializationOptions,
  hasAnyEnabledCapability,
  hasExplicitConfigurationValue,
  serverMatchesExtension,
  shouldStartFromConfiguration,
  type LspInitializationOptions,
} from "./extension-core.js";
import { isJsxDocument } from "./jsx-routing.js";
import type { WorkspaceLspConfigDiscovery } from "./workspace-config.js";
import type {
  ExtensionRuntimeState,
  InspectedServerCandidate,
  VizeStatus,
  VizeStatusAction,
  VizeStatusQuickPickItem,
} from "./server-state-types.js";

/** Setup prompts, explicit profiles and status UI share the live extension state. */
export function createExtensionSetup(
  state: ExtensionRuntimeState,
  workspaceLspConfig: WorkspaceLspConfigDiscovery,
  syncClientToConfiguration: (context: ExtensionContext, reason: string) => Promise<void>,
) {
  const RECOMMENDED_SETUP_ACTION = "Enable Recommended";
  const LINT_ONLY_SETUP_ACTION = "Enable Lint Only";
  const OPEN_SETTINGS_ACTION = "Open Settings";
  const DISMISS_ACTION = "Dismiss";
  const OPEN_SETUP_DOCS_ACTION = "Open Setup Docs";
  const SELECT_SERVER_ACTION = "Select Binary";
  const SHOW_OUTPUT_ACTION = "Show Output";
  const INITIAL_SETUP_PROMPT_DISMISSED_KEY = "vize.initialSetupPrompt.dismissed";
  const CAPABILITY_PROMPT_DISMISSED_KEY = "vize.capabilityPrompt.dismissed";

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

    state.suppressConfigurationSync = true;
    try {
      for (const [key, value] of updates) {
        await config.update(key, value, target);
      }
    } finally {
      state.suppressConfigurationSync = false;
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
        description: state.selectedServerCandidate
          ? `${state.selectedServerCandidate.source}: ${state.selectedServerCandidate.path}`
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

  async function runStatusAction(
    context: ExtensionContext,
    action: VizeStatusAction,
  ): Promise<void> {
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
      state.outputChannel.show();
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
      state.outputChannel.show();
    }
  }

  function updateStatusBar(status: VizeStatus, detail: string): void {
    state.currentStatus = status;
    state.currentStatusDetail = detail;

    if (!state.statusBarItem) {
      return;
    }

    const statusText: Record<VizeStatus, string> = {
      disabled: "$(circle-slash) Vize",
      failed: "$(error) Vize",
      "missing-server": "$(warning) Vize",
      ready: "$(check) Vize",
      starting: "$(sync~spin) Vize",
    };

    state.statusBarItem.text = statusText[status];
    state.statusBarItem.tooltip = createStatusTooltip();
    state.statusBarItem.show();
  }

  function createStatusTooltip(): string {
    const lines = [`Vize: ${formatStatus(state.currentStatus)}`];

    if (state.currentStatusDetail) {
      lines.push(state.currentStatusDetail);
    }

    if (Object.keys(state.activeInitializationOptions).length > 0) {
      lines.push(`Features: ${describeCapabilities(state.activeInitializationOptions)}`);
    }

    if (state.selectedServerCandidate) {
      const version = state.selectedServerCandidate.version
        ? ` ${state.selectedServerCandidate.version}`
        : " unknown version";
      lines.push(
        `Server: ${state.selectedServerCandidate.source}${version}`,
        state.selectedServerCandidate.path,
      );
    }

    lines.push("Click to manage Vize.");
    return lines.join("\n");
  }

  function createStatusSummary(
    config: ReturnType<typeof workspace.getConfiguration>,
    initializationOptions: LspInitializationOptions,
  ): string {
    const enabled = shouldStartFromConfiguration(config, workspaceLspConfig.hasConfig);
    const server = state.selectedServerCandidate
      ? `${state.selectedServerCandidate.source} ${state.selectedServerCandidate.version ?? "unknown"}`
      : "server not resolved";

    return `Vize is ${formatStatus(state.currentStatus)} (${enabled ? "enabled" : "disabled"}). Features: ${workspaceLspConfig.describeCapabilities(initializationOptions)}. Server: ${server}.`;
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

  async function warnAboutVersionMismatch(
    candidate: InspectedServerCandidate,
    expectedVersion: string | undefined,
  ): Promise<void> {
    if (
      !expectedVersion ||
      !candidate.version ||
      serverMatchesExtension(candidate.version, expectedVersion)
    ) {
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
      state.outputChannel.show();
    }
  }

  return {
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
  };
}
