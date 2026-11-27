// Evaluated inside the extension's VM context by workspace-trust.test.ts. The
// script's completion value hands the test the activation entry point and a
// wait for pending configuration sync.
// oxlint-disable-next-line no-unused-expressions -- the VM reads the completion value
({ activate, idle: () => waitForConfigurationIdle(() => configurationSyncTimer !== undefined) });
