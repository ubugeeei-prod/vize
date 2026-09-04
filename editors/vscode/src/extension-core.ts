export type LspInitializationOptions = Partial<Record<string, boolean>>;

export type ConfigurationInspection<T = unknown> = {
  globalValue?: T;
  workspaceValue?: T;
  workspaceFolderValue?: T;
};

export type VizeConfigurationLike = {
  get<T>(key: string, defaultValue: T): T;
  inspect<T>(key: string): ConfigurationInspection<T> | undefined;
};

export type InitializationOptionBehavior = {
  hasWorkspaceLspConfig?: boolean;
  log?: (message: string) => void;
  logDefaultProfile?: boolean;
};

export type TestCompletionRequest = {
  character: number;
  line: number;
  uri: string;
};

export type HostTestLanguageClient = {
  sendRequest(method: string, params: unknown): Promise<unknown>;
};

export type HostTestServerInfo = {
  extensionVersion?: string;
  path: string;
  source: string;
  status: string;
  version?: string;
  versionError?: string;
};

export type HostTestCommand = {
  command: string;
  handler: (request?: unknown) => Promise<unknown>;
};

export const HOST_TEST_COMMAND_ENVIRONMENT_FLAG = "VIZE_TEST_ENABLE_HOST_COMMANDS";
export const HOST_TEST_COMPLETION_COMMAND = "vize.test.executeCompletion";
export const HOST_TEST_SERVER_INFO_COMMAND = "vize.test.getServerInfo";

export const SUPPORTED_LANGUAGE_IDS = ["vue", "art-vue", "html"] as const;
export const SUPPORTED_URI_SCHEMES = ["file", "untitled"] as const;
export const FEATURE_SETTING_KEYS = [
  "lint.enable",
  "diagnostics.enable",
  "typecheck.enable",
  "editor.enable",
  "ecosystem.enable",
  "optionsApi.enable",
  "legacyVue2.enable",
  "completion.enable",
  "signatureHelp.enable",
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
  "autoInsert.enable",
] as const;

export const LINT_ONLY_CONFIGURATION_UPDATES: Array<[string, boolean]> = [
  ["enable", true],
  ["lint.enable", true],
  ["diagnostics.enable", false],
  ["typecheck.enable", false],
  ["editor.enable", false],
  ["ecosystem.enable", false],
  ["optionsApi.enable", false],
  ["legacyVue2.enable", false],
  ["completion.enable", false],
  ["signatureHelp.enable", false],
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
  ["autoInsert.enable", false],
];

export const CAPABILITY_LABELS: Record<string, string> = {
  lint: "lint",
  typecheck: "type check",
  editor: "editor bundle",
  optionsApi: "Vue 3 Options API",
  legacyVue2: "Vue 2.7 / Nuxt 2",
  completion: "completion",
  signatureHelp: "signature help",
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
  autoInsert: "automatic insertion",
};

export function createDocumentSelector(): Array<{ language: string; scheme: string }> {
  return SUPPORTED_URI_SCHEMES.flatMap((scheme) =>
    SUPPORTED_LANGUAGE_IDS.map((language) => ({
      scheme,
      language,
    })),
  );
}

/**
 * Hidden host-smoke hooks keep the packaged extension test on the Vize
 * LanguageClient without waiting on unrelated VS Code completion providers.
 * They only exist when the host smoke sets the environment flag, so the
 * shipped extension never contributes them.
 */
export function createHostTestCommands(behavior: {
  environment: Partial<Record<string, string>>;
  getClient: () => HostTestLanguageClient | undefined;
  getServerInfo?: () => HostTestServerInfo | undefined;
}): HostTestCommand[] {
  if (behavior.environment[HOST_TEST_COMMAND_ENVIRONMENT_FLAG] !== "1") {
    return [];
  }

  return [
    {
      command: HOST_TEST_COMPLETION_COMMAND,
      handler: async (request) => {
        const activeClient = behavior.getClient();
        if (!activeClient) {
          throw new Error("Vize test completion command requires an active language client.");
        }
        assertTestCompletionRequest(request);

        return activeClient.sendRequest("textDocument/completion", {
          textDocument: { uri: request.uri },
          position: { line: request.line, character: request.character },
        });
      },
    },
    {
      command: HOST_TEST_SERVER_INFO_COMMAND,
      handler: async () => {
        const serverInfo = behavior.getServerInfo?.();
        if (!serverInfo) {
          throw new Error("Vize test server info command requires selected server evidence.");
        }
        return serverInfo;
      },
    },
  ];
}

/**
 * Registers the gated host commands through the caller's command registry so
 * the wiring stays observable without a VS Code host.
 */
export function bindHostTestCommands<TRegistration>(behavior: {
  environment: Partial<Record<string, string>>;
  getClient: () => HostTestLanguageClient | undefined;
  getServerInfo?: () => HostTestServerInfo | undefined;
  register: (command: string, handler: (request?: unknown) => Promise<unknown>) => TRegistration;
}): TRegistration[] {
  return createHostTestCommands(behavior).map(({ command, handler }) =>
    behavior.register(command, handler),
  );
}

function assertTestCompletionRequest(request: unknown): asserts request is TestCompletionRequest {
  const candidate = request as Partial<TestCompletionRequest> | null;
  const line = candidate?.line;
  const character = candidate?.character;
  if (
    candidate == null ||
    typeof candidate.uri !== "string" ||
    typeof line !== "number" ||
    typeof character !== "number" ||
    !Number.isInteger(line) ||
    !Number.isInteger(character) ||
    line < 0 ||
    character < 0
  ) {
    throw new TypeError("Invalid Vize test completion request.");
  }
}

export function parseVizeVersion(output: string): string | undefined {
  const match = output.match(/\bvize\s+([0-9]+\.[0-9]+\.[0-9]+(?:[-+][^\s]+)?)/);
  return match?.[1];
}

export function hasExplicitConfigurationValue(config: VizeConfigurationLike, key: string): boolean {
  const inspected = config.inspect(key);

  return (
    inspected?.globalValue !== undefined ||
    inspected?.workspaceValue !== undefined ||
    inspected?.workspaceFolderValue !== undefined
  );
}

export function hasAnyEnabledCapability(config: VizeConfigurationLike): boolean {
  return FEATURE_SETTING_KEYS.some((key) => config.get<boolean>(key, false));
}

export function hasAnyExplicitCapabilityValue(config: VizeConfigurationLike): boolean {
  return FEATURE_SETTING_KEYS.some((key) => hasExplicitConfigurationValue(config, key));
}

export function shouldStartFromConfiguration(
  config: VizeConfigurationLike,
  hasWorkspaceLspConfig = false,
): boolean {
  if (config.get<boolean>("enable", false)) {
    return true;
  }

  if (hasExplicitConfigurationValue(config, "enable")) {
    return false;
  }

  return hasWorkspaceLspConfig;
}

export function getInitializationOptions(
  config: VizeConfigurationLike,
  behavior: InitializationOptionBehavior = {},
): LspInitializationOptions {
  const options: LspInitializationOptions = {};

  setFeatureOption(options, config, "lint.enable", "lint", true);
  setDiagnosticsAliasOption(options, config);
  setFeatureOption(options, config, "typecheck.enable", "typecheck", true);
  setFeatureOption(options, config, "editor.enable", "editor", true);
  setFeatureOption(options, config, "ecosystem.enable", "ecosystem", true);
  setFeatureOption(options, config, "optionsApi.enable", "optionsApi", false);
  setFeatureOption(options, config, "legacyVue2.enable", "legacyVue2", false);
  setFeatureOption(options, config, "completion.enable", "completion", true);
  setFeatureOption(options, config, "signatureHelp.enable", "signatureHelp", true);
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
  setFeatureOption(options, config, "autoInsert.enable", "autoInsert", false);

  if (
    Object.keys(options).length === 0 &&
    config.get<boolean>("enable", false) &&
    !hasAnyExplicitCapabilityValue(config) &&
    !behavior.hasWorkspaceLspConfig
  ) {
    if (behavior.logDefaultProfile !== false) {
      behavior.log?.(
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

export function describeCapabilities(options: LspInitializationOptions): string {
  const capabilities = Object.entries(options)
    .filter(([, enabled]) => enabled === true)
    .map(([name]) => CAPABILITY_LABELS[name] ?? name);

  return capabilities.length ? capabilities.join(", ") : "none";
}

function setDiagnosticsAliasOption(
  options: LspInitializationOptions,
  config: VizeConfigurationLike,
): void {
  if (hasExplicitConfigurationValue(config, "lint.enable")) {
    return;
  }

  const enabled = config.get<boolean>("diagnostics.enable", false);
  if (enabled === true) {
    options.lint = true;
    return;
  }
  if (hasExplicitConfigurationValue(config, "diagnostics.enable")) {
    options.lint = false;
  }
}

function setFeatureOption(
  options: LspInitializationOptions,
  config: VizeConfigurationLike,
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
