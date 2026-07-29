const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");

const extensionId = "ubugeeei.vize";
const featureSettingKeys = [
  "lint.enable",
  "diagnostics.enable",
  "typecheck.enable",
  "editor.enable",
  "ecosystem.enable",
  "optionsApi.enable",
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
];
// Derived from a raw LSP probe of `vize lsp` against this fixture: the prop
// mismatch in `<Child :count="label" />` publishes exactly one TS 2322 error
// anchored on the authored `count` attribute name.
const expectedMismatchDiagnostic = {
  code: 2322,
  message: "Type 'string' is not assignable to type 'number'.",
  range: new vscode.Range(9, 12, 9, 17),
  severity: vscode.DiagnosticSeverity.Error,
  source: "vize/types",
};
const mismatchRepairRange = new vscode.Range(9, 19, 9, 24);
const requiredTemplateCompletionLabels = ["Child", "amount", "label"];
const forbiddenTemplateCompletionLabels = [
  "Fake Vize Completion",
  "v-bind",
  "v-else",
  "v-else-if",
  "v-for",
  "v-html",
  "v-if",
  "v-model",
  "v-on",
  "v-once",
  "v-pre",
  "v-show",
  "v-slot",
  "v-text",
];

exports.run = async function run() {
  const serverPath = getRealServer();
  const extension = vscode.extensions.getExtension(extensionId);
  assert.ok(extension, `missing extension: ${extensionId}`);
  await extension.activate();
  assert.equal(extension.isActive, true);

  await prepareConfiguredRealServer(serverPath);
  await vscode.commands.executeCommand("vize.enableRecommendedProfile");

  const cleanDocument = await openWorkspaceDocument("src", "Clean.vue");
  await vscode.window.showTextDocument(cleanDocument);
  const mismatchDocument = await openWorkspaceDocument("src", "App.vue");
  await vscode.window.showTextDocument(mismatchDocument);

  await runRealDiagnosticSmoke(mismatchDocument, cleanDocument);
  await runRealCompletionSmoke(mismatchDocument);
  await runRealHoverSmoke(mismatchDocument);
  await runRealDidChangeRepairSmoke(mismatchDocument, extension);

  await vscode.commands.executeCommand("vize.disable");
  assert.equal(vscode.workspace.getConfiguration("vize").get("enable"), false);
};

async function runRealDiagnosticSmoke(mismatchDocument, cleanDocument) {
  const diagnostics = await waitForDiagnostics(
    mismatchDocument.uri,
    (nextDiagnostics) => nextDiagnostics.length > 0,
    "real server prop type mismatch diagnostic",
    120_000,
  );

  assert.equal(
    diagnostics.length,
    1,
    `expected exactly the prop mismatch diagnostic, got: ${JSON.stringify(diagnostics)}`,
  );
  const diagnostic = diagnostics[0];
  assert.equal(diagnostic.source, expectedMismatchDiagnostic.source);
  assert.equal(diagnostic.code, expectedMismatchDiagnostic.code);
  assert.equal(diagnostic.message, expectedMismatchDiagnostic.message);
  assert.equal(diagnostic.severity, expectedMismatchDiagnostic.severity);
  assert.deepEqual(diagnostic.range, expectedMismatchDiagnostic.range);

  await assertStaysDiagnosticFree(cleanDocument.uri, "clean SFC");
}

async function runRealCompletionSmoke(mismatchDocument) {
  const position = positionAfter(mismatchDocument, "{{ label }}", "{{ label");
  const completions = await vscode.commands.executeCommand(
    "vscode.executeCompletionItemProvider",
    mismatchDocument.uri,
    position,
  );
  const labels = (completions?.items ?? []).map((item) =>
    typeof item.label === "string" ? item.label : item.label.label,
  );

  for (const required of requiredTemplateCompletionLabels) {
    assert.ok(
      labels.includes(required),
      `template completion must include script binding ${JSON.stringify(required)}: ${JSON.stringify(labels)}`,
    );
  }
  for (const forbidden of forbiddenTemplateCompletionLabels) {
    assert.equal(
      labels.includes(forbidden),
      false,
      `template expression completion must not surface ${JSON.stringify(forbidden)}: ${JSON.stringify(labels)}`,
    );
  }
}

async function runRealHoverSmoke(mismatchDocument) {
  const position = positionAfter(mismatchDocument, "{{ label }}", "{{ la");
  const hovers = await vscode.commands.executeCommand(
    "vscode.executeHoverProvider",
    mismatchDocument.uri,
    position,
  );
  assert.ok(hovers?.length >= 1, "expected a hover for the template binding");

  const markdown = hovers
    .flatMap((hover) => hover.contents)
    .map((content) => (typeof content === "string" ? content : content.value))
    .join("\n");
  // This profile enables typechecking, so the hover type text comes from the
  // live backend (#3321): the real literal type of the const, not the
  // script-binding heuristic that used to answer here.
  assert.match(markdown, /\*\*TypeScript quick info\*\*/);
  assert.ok(
    markdown.includes('const label: "hello from vize"'),
    `hover must report the backend type of the binding: ${JSON.stringify(markdown)}`,
  );
  assert.ok(
    markdown.includes("Resolved through Vize virtual TypeScript"),
    `hover must identify the type backend as its source: ${JSON.stringify(markdown)}`,
  );
  assert.ok(
    !markdown.includes("Template binding from script"),
    `hover must not fall back to the script-binding heuristic: ${JSON.stringify(markdown)}`,
  );
}

async function runRealDidChangeRepairSmoke(mismatchDocument, extension) {
  assert.equal(mismatchDocument.getText(mismatchRepairRange), "label");

  const editor = await vscode.window.showTextDocument(mismatchDocument);
  const applied = await editor.edit((editBuilder) => {
    editBuilder.replace(mismatchRepairRange, "amount");
  });
  assert.equal(applied, true, "expected the repair edit to apply");

  const diagnostics = await waitForDiagnostics(
    mismatchDocument.uri,
    (nextDiagnostics) => nextDiagnostics.length === 0,
    "didChange repair clearing the type diagnostic",
    60_000,
  );
  assert.deepEqual(diagnostics, []);

  // The repair must flow through textDocument/didChange alone: the buffer is
  // still dirty (never saved) and the same extension host instance is active.
  assert.equal(mismatchDocument.isDirty, true);
  assert.equal(extension.isActive, true);
}

async function prepareConfiguredRealServer(serverPath) {
  await updateVizeConfiguration("enable", false);
  await updateVizeConfiguration("trace.server", undefined);
  for (const key of featureSettingKeys) {
    await updateVizeConfiguration(key, undefined);
  }
  await updateVizeConfiguration("serverPath", serverPath);
  await sleep(300);
}

async function updateVizeConfiguration(key, value) {
  await vscode.workspace
    .getConfiguration("vize")
    .update(key, value, vscode.ConfigurationTarget.Workspace);
}

async function assertStaysDiagnosticFree(uri, label) {
  const settleUntil = Date.now() + 2_000;

  while (Date.now() < settleUntil) {
    const diagnostics = vscode.languages.getDiagnostics(uri);
    assert.deepEqual(diagnostics, [], `${label} must stay diagnostic-free`);
    await sleep(100);
  }
}

function positionAfter(document, needle, prefix) {
  const offset = document.getText().indexOf(needle);
  assert.notEqual(offset, -1, `fixture must contain ${JSON.stringify(needle)}`);
  return document.positionAt(offset + prefix.length);
}

async function openWorkspaceDocument(...segments) {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  assert.ok(workspaceFolder, "expected a workspace folder");
  return vscode.workspace.openTextDocument(
    vscode.Uri.file(path.join(workspaceFolder.uri.fsPath, ...segments)),
  );
}

function getRealServer() {
  const serverPath = process.env.VIZE_TEST_SERVER_PATH;
  assert.ok(serverPath, "VIZE_TEST_SERVER_PATH must be set");
  assert.ok(fs.existsSync(serverPath), `missing real server: ${serverPath}`);
  return serverPath;
}

async function waitForDiagnostics(uri, predicate, label, timeoutMs) {
  const timeoutAt = Date.now() + timeoutMs;
  let diagnostics = [];

  while (Date.now() < timeoutAt) {
    diagnostics = vscode.languages.getDiagnostics(uri);
    if (predicate(diagnostics)) {
      return diagnostics;
    }

    await sleep(100);
  }

  assert.fail(`${label} did not happen. Last diagnostics: ${JSON.stringify(diagnostics)}`);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
