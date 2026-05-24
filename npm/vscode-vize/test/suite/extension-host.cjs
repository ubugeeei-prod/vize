const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");

const extensionId = "ubugeeei.vize";
const recommendedInitializationOptions = {
  editor: true,
  ecosystem: true,
  lint: true,
  typecheck: true,
};
const featureSettingKeys = [
  "lint.enable",
  "diagnostics.enable",
  "typecheck.enable",
  "editor.enable",
  "ecosystem.enable",
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
];
const granularEditorCapabilitySettings = [
  ["completion.enable", "completion"],
  ["hover.enable", "hover"],
  ["definition.enable", "definition"],
  ["references.enable", "references"],
  ["documentSymbols.enable", "documentSymbols"],
  ["workspaceSymbols.enable", "workspaceSymbols"],
  ["codeActions.enable", "codeActions"],
  ["rename.enable", "rename"],
  ["codeLens.enable", "codeLens"],
  ["formatting.enable", "formatting"],
  ["semanticTokens.enable", "semanticTokens"],
  ["documentLinks.enable", "documentLinks"],
  ["foldingRanges.enable", "foldingRanges"],
  ["inlayHints.enable", "inlayHints"],
  ["fileRename.enable", "fileRename"],
];
const commandIds = [
  "vize.disable",
  "vize.enableLintOnlyProfile",
  "vize.enableRecommendedProfile",
  "vize.findReferences",
  "vize.restartServer",
  "vize.selectServerPath",
  "vize.showOutput",
  "vize.showStatus",
];

exports.run = async function run() {
  await runDisabledContributionSmoke();
  await runFakeServerLifecycleSmoke();
  await runConfigurationEdgeCaseSmoke();
  await runEditorCapabilityProviderSmoke();
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
  assertInitializationOptions(entries, { lint: true });

  await vscode.commands.executeCommand("vize.restartServer");
  entries = await waitForLogEntries(
    logPath,
    (nextEntries) => initializeMessages(nextEntries).length >= 3,
    "manual restart initialization",
  );
  assertInitializationOptions(entries, { lint: true });

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
  assert.ok(entries.some((entry) => entry.event === "version"), "expected trimmed server path");
  await disableVizeAndWaitForShutdown(logPath);

  await prepareConfiguredFakeServer({ logPath, serverPath });
  await updateVizeConfigurationEntries(
    featureSettingKeys.map((key) => [key, false]),
  );
  await updateVizeConfiguration("enable", true);
  entries = await waitForReadyServer(logPath, "explicitly empty capability profile");
  assertInitializationOptions(entries, {});
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

async function runEditorCapabilityProviderSmoke() {
  const { logPath, serverPath } = getFakeServer();

  await prepareConfiguredFakeServer({ logPath, serverPath });
  await vscode.commands.executeCommand("vize.enableRecommendedProfile");
  await waitForReadyServer(logPath, "editor capability provider setup");

  const document = await openWorkspaceDocument("src", "App.vue");
  await vscode.window.showTextDocument(document);

  const position = new vscode.Position(1, 6);
  const range = new vscode.Range(position, position.translate(0, 7));

  await runProviderCommand(logPath, {
    args: [document.uri, position],
    commandIds: ["vscode.executeCompletionItemProvider"],
    label: "completion",
    method: "textDocument/completion",
    validate(result) {
      const items = Array.isArray(result) ? result : result?.items;
      assert.ok(
        items?.some((item) => item.label === "Fake Vize Completion"),
        "expected fake completion item",
      );
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri, position],
    commandIds: ["vscode.executeHoverProvider"],
    label: "hover",
    method: "textDocument/hover",
    validate(result) {
      assert.ok(result?.length >= 1, "expected hover result");
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri, position],
    commandIds: ["vscode.executeDefinitionProvider"],
    label: "definition",
    method: "textDocument/definition",
    validate(result) {
      assert.ok(result?.length >= 1, "expected definition result");
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri, position],
    commandIds: ["vscode.executeReferenceProvider"],
    label: "references",
    method: "textDocument/references",
    validate(result) {
      assert.ok(result?.length >= 1, "expected reference result");
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri],
    commandIds: ["vscode.executeDocumentSymbolProvider"],
    label: "document symbols",
    method: "textDocument/documentSymbol",
    validate(result) {
      assert.ok(
        result?.some((symbol) => symbol.name === "FakeComponent"),
        "expected document symbol",
      );
    },
  });

  await runProviderCommand(logPath, {
    args: ["Fake"],
    commandIds: ["vscode.executeWorkspaceSymbolProvider"],
    label: "workspace symbols",
    method: "workspace/symbol",
    validate(result) {
      assert.ok(
        result?.some((symbol) => symbol.name === "FakeWorkspaceSymbol"),
        "expected workspace symbol",
      );
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri, range],
    commandIds: ["vscode.executeCodeActionProvider"],
    label: "code actions",
    method: "textDocument/codeAction",
    validate(result) {
      assert.ok(
        result?.some((action) => action.title === "Fake Vize Quick Fix"),
        "expected code action",
      );
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri],
    commandIds: ["vscode.executeCodeLensProvider"],
    label: "code lens",
    method: "textDocument/codeLens",
    validate(result) {
      assert.ok(
        result?.some((lens) => lens.command?.title === "Fake Vize Lens"),
        "expected code lens",
      );
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri, { insertSpaces: true, tabSize: 2 }],
    commandIds: ["vscode.executeFormatDocumentProvider"],
    label: "formatting",
    method: "textDocument/formatting",
    validate(result) {
      assert.ok(result?.length >= 1, "expected formatting edit");
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri],
    commandIds: ["vscode.executeLinkProvider"],
    label: "document links",
    method: "textDocument/documentLink",
    validate(result) {
      assert.ok(result?.length >= 1, "expected document link");
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri],
    commandIds: ["vscode.executeFoldingRangeProvider"],
    label: "folding ranges",
    method: "textDocument/foldingRange",
    validate(result) {
      assert.ok(result?.length >= 1, "expected folding range");
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri, new vscode.Range(0, 0, document.lineCount, 0)],
    commandIds: ["vscode.executeInlayHintProvider"],
    label: "inlay hints",
    method: "textDocument/inlayHint",
    validate(result) {
      assert.ok(result?.some((hint) => hint.label === ": string"), "expected inlay hint");
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri],
    commandIds: [
      "vscode.provideDocumentSemanticTokens",
      "vscode.executeDocumentSemanticTokensProvider",
    ],
    label: "semantic tokens",
    method: "textDocument/semanticTokens/full",
    validate(result) {
      assert.ok(result?.data?.length > 0, "expected semantic tokens");
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri, position, "renamedMessage"],
    commandIds: ["vscode.executeDocumentRenameProvider"],
    label: "rename",
    method: "textDocument/rename",
    validate(result) {
      assert.ok(result?.size >= 1, "expected rename workspace edit");
    },
  });

  const referenceCount = methodMessages(readLogEntries(logPath), "textDocument/references").length;
  await vscode.commands.executeCommand("vize.findReferences");
  await waitForLogEntries(
    logPath,
    (entries) => methodMessages(entries, "textDocument/references").length > referenceCount,
    "find references command delegation",
  );

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
    content: "<script setup lang=\"ts\">\nconst value = 1\n</script>\n<template>{{ value }}</template>\n",
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
    Buffer.from("<script setup lang=\"ts\">\nconst watched = true\n</script>\n", "utf-8"),
  );
  await waitForWatchedFileChange(logPath, watchedUri.toString(), 1, "watched Vue file create");

  await vscode.workspace.fs.delete(watchedUri);
  await waitForWatchedFileChange(logPath, watchedUri.toString(), 3, "watched Vue file delete");

  await disableVizeAndWaitForShutdown(logPath);
}

async function prepareConfiguredFakeServer({ logPath, serverPath }) {
  fs.writeFileSync(logPath, "");
  await resetVizeConfiguration();
  await updateVizeConfiguration("serverPath", serverPath);
  await sleep(300);
  assert.equal(initializeMessages(readLogEntries(logPath)).length, 0);
}

async function resetVizeConfiguration() {
  await updateVizeConfiguration("enable", false);
  await updateVizeConfiguration("serverPath", undefined);
  await updateVizeConfiguration("trace.server", undefined);
  await updateVizeConfigurationEntries(featureSettingKeys.map((key) => [key, undefined]));
  await updateVizeConfiguration("enable", false);
  await sleep(300);
}

async function updateVizeConfigurationEntries(entries) {
  for (const [key, value] of entries) {
    await updateVizeConfiguration(key, value);
  }
}

async function updateVizeConfiguration(key, value) {
  await vscode.workspace
    .getConfiguration("vize")
    .update(key, value, vscode.ConfigurationTarget.Workspace);
}

async function disableVizeAndWaitForShutdown(logPath) {
  const exitCount = methodMessages(readLogEntries(logPath), "exit").length;

  await vscode.commands.executeCommand("vize.disable");
  await waitForLogEntries(
    logPath,
    (entries) => methodMessages(entries, "exit").length > exitCount,
    "language server shutdown",
  );

  assert.equal(vscode.workspace.getConfiguration("vize").get("enable"), false);
}

async function waitForReadyServer(logPath, label) {
  return waitForLogEntries(
    logPath,
    (entries) =>
      initializeMessages(entries).length >= 1 && methodMessages(entries, "initialized").length >= 1,
    label,
  );
}

async function runProviderCommand(logPath, spec) {
  const requestCount = methodMessages(readLogEntries(logPath), spec.method).length;
  const result = await executeFirstAvailableCommand(spec.commandIds, spec.args);
  spec.validate(result);

  await waitForLogEntries(
    logPath,
    (entries) => methodMessages(entries, spec.method).length > requestCount,
    `${spec.label} request`,
  );
}

async function executeFirstAvailableCommand(commandIds, args) {
  let missingCommandError;

  for (const commandId of commandIds) {
    try {
      return await vscode.commands.executeCommand(commandId, ...args);
    } catch (error) {
      if (!String(error).includes("command") || !String(error).includes("not found")) {
        throw error;
      }

      missingCommandError = error;
    }
  }

  assert.fail(
    `missing VS Code provider command: ${commandIds.join(" or ")}. Last error: ${String(
      missingCommandError,
    )}`,
  );
}

async function waitForMethodWithUri(logPath, method, uri) {
  await waitForLogEntries(
    logPath,
    (entries) =>
      methodMessages(entries, method).some((entry) => entry.params?.textDocument?.uri === uri),
    `${method} for ${uri}`,
  );
}

async function waitForWatchedFileChange(logPath, uri, type, label) {
  await waitForLogEntries(
    logPath,
    (entries) =>
      methodMessages(entries, "workspace/didChangeWatchedFiles").some((entry) =>
        entry.params?.changes?.some((change) => change.uri === uri && change.type === type),
      ),
    label,
  );
}

async function openWorkspaceDocument(...segments) {
  const workspaceFolder = getWorkspaceFolder();
  return vscode.workspace.openTextDocument(
    vscode.Uri.file(path.join(workspaceFolder.uri.fsPath, ...segments)),
  );
}

function getWorkspaceFolder() {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  assert.ok(workspaceFolder, "expected a workspace folder");
  return workspaceFolder;
}

function getFakeServer() {
  const serverPath = process.env.VIZE_TEST_SERVER_PATH;
  const logPath = process.env.VIZE_TEST_SERVER_LOG;
  assert.ok(serverPath, "VIZE_TEST_SERVER_PATH must be set");
  assert.ok(logPath, "VIZE_TEST_SERVER_LOG must be set");
  assert.ok(fs.existsSync(serverPath), `missing fake server: ${serverPath}`);
  return { logPath, serverPath };
}

function assertInitializationOptions(entries, expected) {
  assert.deepEqual(lastInitialize(entries).params.initializationOptions ?? {}, expected);
}

function initializeMessages(entries) {
  return entries.filter((entry) => entry.method === "initialize");
}

function methodMessages(entries, method) {
  return entries.filter((entry) => entry.method === method);
}

function lastInitialize(entries) {
  const message = initializeMessages(entries).at(-1);
  assert.ok(message, "expected at least one initialize message");
  return message;
}

async function waitForLogEntries(logPath, predicate, label) {
  const timeoutAt = Date.now() + 20_000;
  let entries = [];

  while (Date.now() < timeoutAt) {
    entries = readLogEntries(logPath);
    if (predicate(entries)) {
      return entries;
    }

    await sleep(100);
  }

  assert.fail(`${label} did not happen. Last log entries: ${JSON.stringify(entries.slice(-10))}`);
}

function readLogEntries(logPath) {
  const text = fs.readFileSync(logPath, "utf-8").trim();
  if (!text) {
    return [];
  }

  return text.split("\n").map((line) => JSON.parse(line));
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
