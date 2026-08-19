const assert = require("node:assert/strict");
const path = require("node:path");
const vscode = require("vscode");
const { runAutoInsertSmoke } = require("./auto-insert-smoke.cjs");
const { runEditorCapabilityProviderSmoke } = require("./editor-capability-smoke.cjs");
const { runSyntaxHighlightContributionSmoke } = require("./extension-host-grammar-smoke.cjs");
const {
  commandIds,
  explicitlyDisabledInitializationOptions,
  extensionId,
  featureSettingKeys,
  granularEditorCapabilitySettings,
  lintOnlyInitializationOptions,
  recommendedInitializationOptions,
} = require("./extension-host-fixtures.cjs");
const {
  assertDiagnostic,
  assertInitializationOptions,
  assertLocation,
  assertTextDocumentRequest,
  disableVizeAndWaitForShutdown,
  getFakeServer,
  getWorkspaceFolder,
  initializeMessages,
  methodMessages,
  openWorkspaceDocument,
  prepareConfiguredFakeServer,
  readLogEntries,
  runProviderCommand,
  updateVizeConfiguration,
  updateVizeConfigurationEntries,
  waitForDiagnostics,
  waitForLogEntries,
  waitForMethodWithUri,
  waitForReadyServer,
  waitForWatchedFileChange,
} = require("./extension-host-support.cjs");

exports.run = async function run() {
  await runDisabledContributionSmoke();
  await runSyntaxHighlightContributionSmoke();
  await runFakeServerLifecycleSmoke();
  await runConfigurationEdgeCaseSmoke();
  await runAutoInsertSmoke();
  await runDiagnosticSmoke();
  await runEditorCapabilityProviderSmoke({
    assertLocation,
    assertTextDocumentRequest,
    disableVizeAndWaitForShutdown,
    getFakeServer,
    methodMessages,
    openWorkspaceDocument,
    prepareConfiguredFakeServer,
    readLogEntries,
    runProviderCommand,
    waitForLogEntries,
    waitForReadyServer,
  });
  await runDocumentSelectorAndWatcherSmoke();
};

async function runDisabledContributionSmoke() {
  const extension = vscode.extensions.getExtension(extensionId);
  assert.ok(extension, `missing extension: ${extensionId}`);
  assert.equal(extension.packageJSON.name, "vize");
  assert.equal(extension.packageJSON.publisher, "ubugeeei");

  await extension.activate();
  assert.equal(extension.isActive, true);

  const allCommands = await vscode.commands.getCommands(true);
  for (const commandId of commandIds) {
    assert.ok(allCommands.includes(commandId), `missing command: ${commandId}`);
  }
  const config = vscode.workspace.getConfiguration("vize");
  assert.equal(config.get("enable"), false);
  assert.equal(config.get("serverPath"), "");

  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  assert.ok(workspaceFolder, "expected a workspace folder");

  const vueDocument = await vscode.workspace.openTextDocument(
    vscode.Uri.file(path.join(workspaceFolder.uri.fsPath, "src", "App.vue")),
  );
  assert.equal(vueDocument.languageId, "vue");

  const artVueDocument = await vscode.workspace.openTextDocument(
    vscode.Uri.file(path.join(workspaceFolder.uri.fsPath, "src", "Variant.art.vue")),
  );
  assert.equal(artVueDocument.languageId, "art-vue");

  await vscode.window.showTextDocument(vueDocument);
  await vscode.commands.executeCommand("vize.showOutput");
  await vscode.commands.executeCommand("vize.disable");

  assert.equal(vscode.workspace.getConfiguration("vize").get("enable"), false);
}

async function runFakeServerLifecycleSmoke() {
  const { logPath, serverPath } = getFakeServer();

  await prepareConfiguredFakeServer({ logPath, serverPath });

  await vscode.commands.executeCommand("vize.enableRecommendedProfile");

  let entries = await waitForLogEntries(
    logPath,
    (nextEntries) => initializeMessages(nextEntries).length >= 1,
    "recommended profile initialization",
  );
  assertInitializationOptions(entries, recommendedInitializationOptions);

  await vscode.commands.executeCommand("vize.enableLintOnlyProfile");
  entries = await waitForLogEntries(
    logPath,
    (nextEntries) => initializeMessages(nextEntries).length >= 2,
    "lint-only profile initialization",
  );
  assertInitializationOptions(entries, lintOnlyInitializationOptions);

  await vscode.commands.executeCommand("vize.restartServer");
  entries = await waitForLogEntries(
    logPath,
    (nextEntries) => initializeMessages(nextEntries).length >= 3,
    "manual restart initialization",
  );
  assertInitializationOptions(entries, lintOnlyInitializationOptions);

  await vscode.commands.executeCommand("vize.disable");
  entries = await waitForLogEntries(
    logPath,
    (nextEntries) => nextEntries.filter((entry) => entry.method === "exit").length >= 3,
    "language server shutdown",
  );

  assert.equal(vscode.workspace.getConfiguration("vize").get("enable"), false);
  assert.ok(
    entries.some((entry) => entry.event === "version"),
    "expected configured server version inspection",
  );
}

async function runConfigurationEdgeCaseSmoke() {
  const { logPath, serverPath } = getFakeServer();

  await prepareConfiguredFakeServer({ logPath, serverPath: `  ${serverPath}  ` });
  await updateVizeConfiguration("enable", true);
  let entries = await waitForReadyServer(logPath, "manual enable default profile");
  assertInitializationOptions(entries, recommendedInitializationOptions);
  assert.ok(
    entries.some((entry) => entry.event === "version"),
    "expected trimmed server path",
  );
  await disableVizeAndWaitForShutdown(logPath);

  await prepareConfiguredFakeServer({ logPath, serverPath });
  await updateVizeConfigurationEntries(featureSettingKeys.map((key) => [key, false]));
  await updateVizeConfiguration("enable", true);
  entries = await waitForReadyServer(logPath, "explicitly empty capability profile");
  assertInitializationOptions(entries, explicitlyDisabledInitializationOptions);
  await disableVizeAndWaitForShutdown(logPath);

  await prepareConfiguredFakeServer({ logPath, serverPath });
  await updateVizeConfiguration("diagnostics.enable", true);
  await updateVizeConfiguration("enable", true);
  entries = await waitForReadyServer(logPath, "deprecated diagnostics alias profile");
  assertInitializationOptions(entries, { lint: true });
  await disableVizeAndWaitForShutdown(logPath);

  await prepareConfiguredFakeServer({ logPath, serverPath });
  await updateVizeConfigurationEntries(
    granularEditorCapabilitySettings.map(([setting]) => [setting, true]),
  );
  await updateVizeConfiguration("enable", true);
  entries = await waitForReadyServer(logPath, "granular editor capability profile");
  assertInitializationOptions(
    entries,
    Object.fromEntries(granularEditorCapabilitySettings.map(([, option]) => [option, true])),
  );
  await disableVizeAndWaitForShutdown(logPath);
}

async function runDiagnosticSmoke() {
  const { logPath, serverPath } = getFakeServer();

  await prepareConfiguredFakeServer({ logPath, serverPath });
  await vscode.commands.executeCommand("vize.enableRecommendedProfile");
  await waitForReadyServer(logPath, "diagnostic recommended profile setup");

  const document = await openWorkspaceDocument("src", "App.vue");
  await vscode.window.showTextDocument(document);
  let diagnostics = await waitForDiagnostics(
    document.uri,
    (nextDiagnostics) => nextDiagnostics.length === 2,
    "recommended profile type and lint diagnostics",
  );
  assertDiagnostic(diagnostics, {
    code: "fake-type-mismatch",
    message: "Fake Vize type error: string is not assignable to number.",
    range: new vscode.Range(1, 6, 1, 13),
    severity: vscode.DiagnosticSeverity.Error,
    source: "vize:typecheck",
  });
  assertDiagnostic(diagnostics, {
    code: "fake-lint-rule",
    message: "Fake Vize lint error: template expression should be simplified.",
    range: new vscode.Range(5, 12, 5, 19),
    severity: vscode.DiagnosticSeverity.Warning,
    source: "vize:lint",
  });

  await vscode.commands.executeCommand("vize.enableLintOnlyProfile");
  await waitForLogEntries(
    logPath,
    (entries) =>
      initializeMessages(entries).length >= 2 && methodMessages(entries, "initialized").length >= 2,
    "diagnostic lint-only profile setup",
  );

  const lintOnlyUri = vscode.Uri.file(
    path.join(getWorkspaceFolder().uri.fsPath, "src", `LintOnly-${Date.now()}.vue`),
  );
  await vscode.workspace.fs.writeFile(lintOnlyUri, Buffer.from(document.getText(), "utf-8"));
  const lintOnlyDocument = await vscode.workspace.openTextDocument(lintOnlyUri);
  await vscode.window.showTextDocument(lintOnlyDocument);
  diagnostics = await waitForDiagnostics(
    lintOnlyUri,
    (nextDiagnostics) => nextDiagnostics.length === 1,
    "lint-only profile diagnostics",
  );
  assertDiagnostic(diagnostics, {
    code: "fake-lint-rule",
    message: "Fake Vize lint error: template expression should be simplified.",
    range: new vscode.Range(5, 12, 5, 19),
    severity: vscode.DiagnosticSeverity.Warning,
    source: "vize:lint",
  });
  assert.equal(
    diagnostics.some((diagnostic) => diagnostic.source === "vize:typecheck"),
    false,
    "lint-only profile should not publish typecheck diagnostics",
  );

  await vscode.workspace.fs.delete(lintOnlyUri);
  await disableVizeAndWaitForShutdown(logPath);
}

async function runDocumentSelectorAndWatcherSmoke() {
  const { logPath, serverPath } = getFakeServer();

  await prepareConfiguredFakeServer({ logPath, serverPath });
  await vscode.commands.executeCommand("vize.enableRecommendedProfile");
  await waitForReadyServer(logPath, "document selector setup");

  const artVueDocument = await openWorkspaceDocument("src", "Variant.art.vue");
  await vscode.window.showTextDocument(artVueDocument);
  await runProviderCommand(logPath, {
    args: [artVueDocument.uri, new vscode.Position(1, 6)],
    commandIds: ["vscode.executeHoverProvider"],
    label: "art-vue hover",
    method: "textDocument/hover",
    validate(result) {
      assert.ok(result?.length >= 1, "expected art-vue hover");
    },
  });
  await waitForMethodWithUri(logPath, "textDocument/didOpen", artVueDocument.uri.toString());

  const untitledDocument = await vscode.workspace.openTextDocument({
    content:
      '<script setup lang="ts">\nconst value = 1\n</script>\n<template>{{ value }}</template>\n',
    language: "vue",
  });
  await vscode.window.showTextDocument(untitledDocument);
  await runProviderCommand(logPath, {
    args: [untitledDocument.uri, new vscode.Position(1, 6)],
    commandIds: ["vscode.executeHoverProvider"],
    label: "untitled vue hover",
    method: "textDocument/hover",
    validate(result) {
      assert.ok(result?.length >= 1, "expected untitled hover");
    },
  });
  await waitForMethodWithUri(logPath, "textDocument/didOpen", untitledDocument.uri.toString());

  const workspaceFolder = getWorkspaceFolder();
  const watchedUri = vscode.Uri.file(
    path.join(workspaceFolder.uri.fsPath, "src", `Watched-${Date.now()}.vue`),
  );

  await vscode.workspace.fs.writeFile(
    watchedUri,
    Buffer.from('<script setup lang="ts">\nconst watched = true\n</script>\n', "utf-8"),
  );
  await waitForWatchedFileChange(logPath, watchedUri.toString(), 1, "watched Vue file create");

  await vscode.workspace.fs.delete(watchedUri);
  await waitForWatchedFileChange(logPath, watchedUri.toString(), 3, "watched Vue file delete");

  await disableVizeAndWaitForShutdown(logPath);
}
