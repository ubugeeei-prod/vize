const assert = require("node:assert/strict");
const { execFileSync } = require("node:child_process");
const fs = require("node:fs");
const vscode = require("vscode");

const { runRealServerScenario } = require("./real-scenario.cjs");
const {
  assertPackagedExtension,
  assertStaysDiagnosticFree,
  describeDiagnostic,
  getRealServer,
  openWorkspaceDocument,
  positionAfter,
  prepareConfiguredRealServer,
  waitForDiagnostics,
} = require("./real-server-support.cjs");

const extensionId = "ubugeeei.vize";
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

exports.run = async function run() {
  logProgress("start");
  const serverPath = getRealServer();
  const extension = vscode.extensions.getExtension(extensionId);
  assert.ok(extension, `missing extension: ${extensionId}`);
  assertPackagedExtension(extension);
  await extension.activate();
  assert.equal(extension.isActive, true);

  // `vize.formatting.enable` defaults to false on both the extension and the
  // server, and `enableRecommendedProfile` does not turn it on. Opt in before
  // the client starts so the whole suite runs against one server session: the
  // real server is busy type checking, and a mid-suite restart races its
  // shutdown.
  await prepareConfiguredRealServer(serverPath, { "formatting.enable": true });
  await vscode.commands.executeCommand("vize.enableRecommendedProfile");

  const cleanDocument = await openWorkspaceDocument("src", "Clean.vue");
  await vscode.window.showTextDocument(cleanDocument);
  const mismatchDocument = await openWorkspaceDocument("src", "App.vue");
  await vscode.window.showTextDocument(mismatchDocument);

  logProgress("diagnostics");
  await runRealDiagnosticSmoke(mismatchDocument, cleanDocument);
  logProgress("completion");
  await runRealCompletionSmoke(mismatchDocument);
  logProgress("hover");
  await runRealHoverSmoke(mismatchDocument);
  logProgress("didChange repair");
  await runRealDidChangeRepairSmoke(mismatchDocument, extension);
  logProgress("pinned create-vue oracle");
  await runPinnedCreateVuePatchOracle(extension, serverPath);

  logProgress("scorecard scenario");
  await runRealServerScenario();

  await vscode.commands.executeCommand("vize.disable");
  assert.equal(vscode.workspace.getConfiguration("vize").get("enable"), false);
  logProgress("done");
};

function logProgress(label) {
  console.log(`[vize-host-real] ${label}`);
}

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
  const { HOST_TEST_COMPLETION_COMMAND, assertRealHostCompletionLabels } =
    await import("../real-host-completion-oracle.mjs");
  const position = positionAfter(mismatchDocument, "{{ label }}", "{{ label");
  const completions = await vscode.commands.executeCommand(HOST_TEST_COMPLETION_COMMAND, {
    uri: mismatchDocument.uri.toString(),
    line: position.line,
    character: position.character,
  });

  assertRealHostCompletionLabels(completions);
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
  //
  // The hover opens with the signature code block, like Volar and tsserver:
  // no implementation-detail preamble ahead of it (#3894).
  assert.match(markdown, /^```typescript\n/);
  assert.ok(
    markdown.includes('const label: "hello from vize"'),
    `hover must report the backend type of the binding: ${JSON.stringify(markdown)}`,
  );
  assert.ok(
    !markdown.includes("TypeScript quick info"),
    `hover must not restore the removed preamble: ${JSON.stringify(markdown)}`,
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

async function runPinnedCreateVuePatchOracle(extension, serverPath) {
  const document = await openWorkspaceDocument("template", "bare", "typescript", "src", "App.vue");
  await vscode.window.showTextDocument(document);
  await assertStaysDiagnosticFree(document.uri, "clean pinned create-vue SFC");

  const cleanSource = document.getText();
  const cleanCount = "const count: number = 1";
  const brokenCount = "const count: number = 'broken'";
  const repairedCount = "const count: number = 2";
  const editor = await vscode.window.showTextDocument(document);
  const broke = await editor.edit((editBuilder) => {
    editBuilder.replace(rangeForExactText(document, cleanCount), brokenCount);
  });
  assert.equal(broke, true, "expected the pinned create-vue break edit to apply");

  const brokenDiagnostics = await waitForDiagnostics(
    document.uri,
    (diagnostics) => diagnostics.length > 0,
    "pinned create-vue broken type diagnostic",
    60_000,
  );
  assert.deepEqual(brokenDiagnostics.map(describeDiagnostic), [
    {
      code: 2322,
      message: "Type 'string' is not assignable to type 'number'.",
      range: [1, 6, 1, 11],
      relatedInformation: undefined,
      severity: vscode.DiagnosticSeverity.Error,
      source: "vize/types",
      tags: undefined,
    },
  ]);

  const repaired = await editor.edit((editBuilder) => {
    editBuilder.replace(rangeForExactText(document, brokenCount), repairedCount);
  });
  assert.equal(repaired, true, "expected the pinned create-vue repair edit to apply");
  const repairedDiagnostics = await waitForDiagnostics(
    document.uri,
    (diagnostics) => diagnostics.length === 0,
    "pinned create-vue repair clearing the type diagnostic",
    60_000,
  );
  assert.deepEqual(repairedDiagnostics, []);
  assert.equal(document.getText(), cleanSource.replace(cleanCount, repairedCount));
  assert.equal(document.isDirty, true);
  assert.equal(extension.isActive, true);
  const serverInfo = await readSelectedServerInfo(extension, serverPath);

  const resultPath = process.env.VIZE_TEST_PINNED_CREATE_VUE_RESULT_PATH;
  assert.ok(resultPath, "missing pinned create-vue host result path");
  fs.writeFileSync(
    resultPath,
    `${JSON.stringify({
      brokenDiagnostics: brokenDiagnostics.map(describeDiagnostic),
      documentDirty: document.isDirty,
      extensionActive: extension.isActive,
      fixtureId: "create-vue",
      repairedDiagnostics: repairedDiagnostics.map(describeDiagnostic),
      schemaVersion: 1,
      serverInfo,
    })}\n`,
  );
}

async function readSelectedServerInfo(extension, serverPath) {
  const { HOST_TEST_SERVER_INFO_COMMAND, assertRealHostServerInfo, parseVizeVersion } =
    await import("../real-host-server-info-oracle.mjs");
  const actual = await vscode.commands.executeCommand(HOST_TEST_SERVER_INFO_COMMAND);
  const serverVersion = parseVizeVersion(
    execFileSync(serverPath, ["--version"], { encoding: "utf8" }),
  );
  return assertRealHostServerInfo(actual, {
    extensionVersion: extension.packageJSON.version,
    serverPath,
    serverVersion,
  });
}

function rangeForExactText(document, needle) {
  const source = document.getText();
  const start = source.indexOf(needle);
  assert.notEqual(start, -1, `fixture must contain ${JSON.stringify(needle)}`);
  assert.equal(
    source.indexOf(needle, start + needle.length),
    -1,
    `fixture anchor must be unique: ${JSON.stringify(needle)}`,
  );
  return new vscode.Range(document.positionAt(start), document.positionAt(start + needle.length));
}
