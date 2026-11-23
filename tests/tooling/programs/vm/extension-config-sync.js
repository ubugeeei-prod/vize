outputChannel = output;
client = activeClient;
if (client)
  activeInitializationOptions = getInitializationOptions(policyConfiguration, {
    hasWorkspaceLspConfig: workspaceLspConfig.hasConfig,
  });
updateStatusBar = (...args) => operations.push(["status", ...args]);
if (startup && policyRelease) {
  findServerPath = async (_context, _config, _generation, _failure, probe) => {
    operations.push(["find", probe]);
    selectedServerCandidate = {
      path: "/test/vize",
      source: "release",
      version: "1.2.3",
      ...(probe ? {} : { versionUnconfirmed: true }),
    };
    return "/test/vize";
  };
} else if (startup) {
  findServerPath = async () => "/test/vize";
} else {
  startClient = async (_context, _config, generation) => {
    operations.push(["start", generation]);
    started.resolve();
  };
}
globalThis.extensionSyncControl = {
  scheduleClientSync,
  syncClientToConfiguration: (extensionContext, reason) =>
    syncClientToConfiguration(
      {
        extensionPath: "/test/extension",
        globalStorageUri: { fsPath: "/test/global-storage" },
        ...extensionContext,
      },
      reason,
    ),
  deactivate,
  waitForConfigurationIdle: () =>
    waitForConfigurationIdle(() => configurationSyncTimer !== undefined),
  setClient: (value) => {
    client = value;
    activeInitializationOptions = getInitializationOptions(policyConfiguration, {
      hasWorkspaceLspConfig: workspaceLspConfig.hasConfig,
    });
  },
  setHasConfig: (value) => {
    workspaceLspConfig.hasConfig = value;
  },
  getClient: () => client,
  getCandidate: () => selectedServerCandidate && { ...selectedServerCandidate },
  state: () => ({
    generation: configurationSyncGeneration,
    hasClient: Boolean(client),
    starting: startingClientGeneration,
  }),
};
