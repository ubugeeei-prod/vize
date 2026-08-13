const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");

const expected = require("./real-scenario-expected.cjs");
const {
  describeCodeAction,
  describeDiagnostic,
  describeSemanticTokens,
  describeTextEdit,
  describeWorkspaceEdit,
  getWorkspaceFolderPath,
  openWorkspaceDocument,
  waitFor,
  waitForDiagnostics,
} = require("./real-server-support.cjs");

const scenarioSegments = ["src", "Scenario.vue"];

/**
 * The #3224 parity scorecard scenario, end to end against the real `vize`
 * binary: type bug -> diagnostic at the authored span -> quick fix ->
 * format-on-save -> semantic tokens -> rename.
 *
 * Every step asserts the complete provider result, so a capability that
 * regresses to "returns nothing" fails here instead of silently passing.
 */
exports.runRealServerScenario = async function runRealServerScenario() {
  // The suite opted `vize.formatting.enable` in before the client started; the
  // formatting step below is what proves the setting reached the server.
  assert.equal(vscode.workspace.getConfiguration("vize").get("formatting.enable"), true);
  await enableFormatOnSave();

  const document = await openWorkspaceDocument(...scenarioSegments);
  const editor = await vscode.window.showTextDocument(document);
  assert.equal(document.getText(), expected.authoredSource, "scenario fixture as authored");

  // Announced one by one: the scenario is the longest phase of the host run, and
  // the host is aborted on a wall-clock budget without a suite failure, so the
  // last step logged here is the only evidence of where a stall happened.
  const steps = [
    { label: "diagnostic at authored span", run: () => stepDiagnosticAtAuthoredSpan(document) },
    { label: "quick fix", run: () => stepQuickFix(document, editor) },
    { label: "format on save", run: () => stepFormatOnSave(document) },
    { label: "semantic tokens", run: () => stepSemanticTokens(document) },
    { label: "rename", run: () => stepRename(document) },
  ];
  for (const { label, run } of steps) {
    const startedAt = Date.now();
    console.log(`[vize-host-real] scenario ${label}`);
    await run();
    console.log(`[vize-host-real] scenario ${label} finished after ${Date.now() - startedAt}ms`);
  }
};

async function enableFormatOnSave() {
  const editorConfiguration = vscode.workspace.getConfiguration("editor", {
    languageId: "vue",
    uri: vscode.Uri.file(path.join(getWorkspaceFolderPath(), ...scenarioSegments)),
  });
  await editorConfiguration.update(
    "formatOnSave",
    true,
    vscode.ConfigurationTarget.Workspace,
    true,
  );
  await editorConfiguration.update(
    "defaultFormatter",
    "ubugeeei.vize",
    vscode.ConfigurationTarget.Workspace,
    true,
  );
}

/** Step 1: the authored type bug and the fixable lint warning are published. */
async function stepDiagnosticAtAuthoredSpan(document) {
  const diagnostics = await waitForDiagnostics(
    document.uri,
    (next) => next.length >= expected.diagnostics.length,
    "real server scenario diagnostics",
    180_000,
  );

  assert.deepEqual(sortDiagnostics(diagnostics).map(describeDiagnostic), expected.diagnostics);
}

/** Step 2: the quick fix the server offers on the lint warning's own span. */
async function stepQuickFix(document, editor) {
  const actions = await vscode.commands.executeCommand(
    "vscode.executeCodeActionProvider",
    document.uri,
    expected.quickFixRange,
  );

  assert.deepEqual(actions.map(describeCodeAction), expected.codeActions(document.uri.toString()));

  const applied = await vscode.workspace.applyEdit(actions[0].edit);
  assert.equal(applied, true, "expected the quick fix edit to apply");
  assert.equal(document.getText(), expected.quickFixedSource, "document after the quick fix");
  assert.equal(editor.document.uri.toString(), document.uri.toString());
}

/** Step 3: format-on-save rewrites the buffer and the file on disk. */
async function stepFormatOnSave(document) {
  const edits = await vscode.commands.executeCommand(
    "vscode.executeFormatDocumentProvider",
    document.uri,
    { insertSpaces: true, tabSize: 2 },
  );
  assert.deepEqual(edits.map(describeTextEdit), expected.formattingEdits);

  // `vscode.executeFormatDocumentProvider` only computes the edits, it never
  // applies them, so the document is still unformatted here. `save()` is what
  // exercises format-on-save: it runs the `editor.formatOnSave` save
  // participant configured in `enableFormatOnSave`, which applies the very
  // edits asserted above before the file hits disk.
  const saved = await document.save();
  assert.equal(saved, true, "expected the scenario document to save");
  assert.equal(document.getText(), expected.formattedSource, "document after format-on-save");
  assert.equal(document.isDirty, false, "format-on-save must leave the document saved");

  const diskPath = path.join(getWorkspaceFolderPath(), ...scenarioSegments);
  assert.equal(fs.readFileSync(diskPath, "utf8"), expected.formattedSource, "file after save");
}

/** Step 4: semantic tokens for the formatted document. */
async function stepSemanticTokens(document) {
  const tokens = await waitFor(
    () => vscode.commands.executeCommand("vscode.provideDocumentSemanticTokens", document.uri),
    (value) => value !== undefined && value !== null,
    "real server semantic tokens",
    60_000,
  );

  assert.deepEqual(describeSemanticTokens(tokens), expected.semanticTokens);
}

/** Step 5: rename the script binding the template consumes. */
async function stepRename(document) {
  const edit = await vscode.commands.executeCommand(
    "vscode.executeDocumentRenameProvider",
    document.uri,
    expected.renamePosition,
    expected.renameNewName,
  );

  assert.deepEqual(describeWorkspaceEdit(edit), expected.renameEdit(document.uri.toString()));

  const applied = await vscode.workspace.applyEdit(edit);
  assert.equal(applied, true, "expected the rename edit to apply");
  assert.equal(document.getText(), expected.renamedSource, "document after the rename");
}

function sortDiagnostics(diagnostics) {
  return [...diagnostics].sort((left, right) => {
    if (left.range.start.line !== right.range.start.line) {
      return left.range.start.line - right.range.start.line;
    }
    return left.range.start.character - right.range.start.character;
  });
}
