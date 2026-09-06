const assert = require("node:assert/strict");
const path = require("node:path");
const vscode = require("vscode");

const extensionId = "ubugeeei.vize";

async function runEditorCapabilityProviderSmoke({
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
}) {
  await runTypeScriptVueImportSmoke(openWorkspaceDocument);

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
    validate(result, request) {
      const items = Array.isArray(result) ? result : result?.items;
      const item = items?.find((nextItem) => nextItem.label === "Fake Vize Completion");
      assert.ok(item, "expected fake completion item");
      assert.equal(item.detail, "Fake Vize autocomplete property");
      assert.equal(item.insertText, "fakeCompletion");
      assert.equal(item.kind, vscode.CompletionItemKind.Property);
      assert.equal(item.sortText, "0001");
      assert.equal(item.documentation.value, "Completion supplied by fake-vize.");
      assertTextDocumentRequest(request, document.uri, position);
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
    args: [document.uri, position, "("],
    commandIds: ["vscode.executeSignatureHelpProvider"],
    label: "signature help",
    method: "textDocument/signatureHelp",
    validate(result, request) {
      assert.equal(result?.activeSignature, 0);
      assert.equal(result?.activeParameter, 0);
      assert.equal(result?.signatures?.[0]?.label, "fakeAction(value: string): void");
      assert.deepEqual(result?.signatures?.[0]?.parameters?.[0]?.label, "value");
      assertTextDocumentRequest(request, document.uri, position);
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri, position],
    commandIds: ["vscode.executeDefinitionProvider"],
    label: "definition",
    method: "textDocument/definition",
    validate(result, request) {
      assert.equal(result?.length, 1);
      assertLocation(result[0], document.uri, new vscode.Range(1, 6, 1, 13));
      assertTextDocumentRequest(request, document.uri, position);
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri, position],
    commandIds: ["vscode.executeReferenceProvider"],
    label: "references",
    method: "textDocument/references",
    validate(result, request) {
      assert.equal(result?.length, 2);
      assertLocation(result[0], document.uri, new vscode.Range(1, 6, 1, 13));
      assertLocation(result[1], document.uri, new vscode.Range(5, 12, 5, 19));
      assertTextDocumentRequest(request, document.uri, position);
      assert.equal(request.params.context.includeDeclaration, true);
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri, position],
    commandIds: ["vscode.executeDocumentHighlights"],
    label: "document highlights",
    method: "textDocument/documentHighlight",
    validate(result, request) {
      assert.equal(result?.length, 2);
      assert.deepEqual(result[0].range, new vscode.Range(1, 6, 1, 13));
      assert.deepEqual(result[1].range, new vscode.Range(5, 12, 5, 19));
      assert.equal(result[0].kind, vscode.DocumentHighlightKind.Write);
      assert.equal(result[1].kind, vscode.DocumentHighlightKind.Read);
      assertTextDocumentRequest(request, document.uri, position);
    },
  });

  await runProviderCommand(logPath, {
    args: [document.uri],
    commandIds: ["vscode.executeDocumentSymbolProvider"],
    label: "document symbols",
    method: "textDocument/documentSymbol",
    validate(result) {
      assertIncludesWhere(
        result,
        (symbol) => symbol.name === "FakeComponent",
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
      assertIncludesWhere(
        result,
        (symbol) => symbol.name === "FakeWorkspaceSymbol",
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
      assertIncludesWhere(
        result,
        (action) => action.title === "Fake Vize Quick Fix",
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
      assertIncludesWhere(
        result,
        (lens) => lens.command?.title === "Fake Vize Lens",
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
      assertIncludesWhere(result, (hint) => hint.label === ": string", "expected inlay hint");
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
      assert.deepEqual(Array.from(result?.data ?? []), [1, 6, 7, 0, 1, 4, 2, 4, 1, 0]);
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
  vscode.window.activeTextEditor.selection = new vscode.Selection(position, position);
  await vscode.commands.executeCommand("vize.findReferences");
  const entries = await waitForLogEntries(
    logPath,
    (entries) => methodMessages(entries, "textDocument/references").length > referenceCount,
    "find references command delegation",
  );
  assertTextDocumentRequest(
    methodMessages(entries, "textDocument/references").at(-1),
    document.uri,
    position,
  );

  await disableVizeAndWaitForShutdown(logPath);
}

async function runTypeScriptVueImportSmoke(openWorkspaceDocument) {
  const extension = vscode.extensions.getExtension(extensionId);
  assert.ok(extension, `missing extension: ${extensionId}`);
  assert.deepEqual(extension.packageJSON.contributes?.typescriptServerPlugins, [
    {
      enableForWorkspaceTypeScriptVersions: true,
      name: "@vizejs/typescript-vue-plugin",
    },
  ]);

  const document = await openWorkspaceDocument("src", "main.ts");
  assert.equal(document.languageId, "typescript");
  await vscode.window.showTextDocument(document);

  const source = document.getText();
  const specifierOffset = source.indexOf("./App.vue");
  assert.notEqual(specifierOffset, -1, "main.ts must import App.vue");

  const definitions = await vscode.commands.executeCommand(
    "vscode.executeDefinitionProvider",
    document.uri,
    document.positionAt(specifierOffset + 3),
  );
  const appVueUri = vscode.Uri.file(path.join(getWorkspaceFolder().uri.fsPath, "src", "App.vue"));

  assert.ok(
    Array.isArray(definitions) &&
      definitions.some(
        (definition) => definitionUri(definition)?.toString() === appVueUri.toString(),
      ),
    `expected TypeScript Vue plugin to resolve ./App.vue, got ${JSON.stringify(definitions)}`,
  );

  const diagnostics = vscode.languages.getDiagnostics(document.uri);
  assert.equal(
    diagnostics.some((diagnostic) =>
      /Cannot find module ['"]\.\/App\.vue['"]/.test(diagnostic.message),
    ),
    false,
    `main.ts should not report TS2307 for App.vue. Diagnostics: ${JSON.stringify(diagnostics)}`,
  );
}

function assertIncludesWhere(items, predicate, message) {
  assert.ok(items?.some(predicate), message);
}

function definitionUri(definition) {
  return definition?.uri ?? definition?.targetUri;
}

function getWorkspaceFolder() {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  assert.ok(workspaceFolder, "expected a workspace folder");
  return workspaceFolder;
}

module.exports = { runEditorCapabilityProviderSmoke };
