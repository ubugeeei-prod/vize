Object.freeze({
  activate,
  idle: () => waitForConfigurationIdle(() => configurationSyncTimer !== undefined),
});
