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
    { label: "typed ref hover surfaces", run: () => stepTypedRefHoverSurfaces() },
    { label: "component contract hover surfaces", run: () => stepComponentContractHoverSurfaces() },
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

/** Step 1b: ref/computed hover text remains backend-typed in the packaged host. */
async function stepTypedRefHoverSurfaces() {
  const diskPath = path.join(getWorkspaceFolderPath(), "src", "RefSurface.vue");
  const brokenSource = expected.refSurfaceSource.replace(
    "</script>",
    "const broken: string = 1;\n</script>",
  );
  fs.writeFileSync(diskPath, brokenSource, "utf8");
  const uri = vscode.Uri.file(diskPath);
  const document = await vscode.workspace.openTextDocument(uri);
  await vscode.window.showTextDocument(document);

  await waitForDiagnostics(
    uri,
    (next) => next.some((diagnostic) => diagnostic.source === "vize/types"),
    "ref surface initial type diagnostic",
    180_000,
  );
  const edit = new vscode.WorkspaceEdit();
  edit.replace(
    uri,
    new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)),
    expected.refSurfaceSource,
  );
  const applied = await vscode.workspace.applyEdit(edit);
  assert.equal(applied, true, "expected ref surface repair edit to apply");

  const diagnostics = await waitForDiagnostics(
    uri,
    (next) => next.length === 0,
    "ref surface diagnostics",
    180_000,
  );
  assert.deepEqual(diagnostics, []);

  const hovers = {
    scriptCount: await hoverAt(uri, 3, 8),
    scriptDoubled: await hoverAt(uri, 4, 8),
    scriptButton: await hoverAt(uri, 5, 8),
    templateCount: await hoverAt(uri, 9, 28),
    templateDoubled: await hoverAt(uri, 9, 40),
    templateButton: await hoverAt(uri, 9, 54),
  };
  assert.deepEqual(hovers, expected.refSurfaceHovers);
  for (const value of Object.values(hovers).flatMap((hover) =>
    hover.flatMap((item) => item.contents),
  )) {
    assert.doesNotMatch(value, /Ref<unknown>|ComputedRef<unknown>|MaybeRef<unknown>/);
  }
}

/** Step 1c: imported component hover text stays marker-free in the packaged host. */
async function stepComponentContractHoverSurfaces() {
  fs.writeFileSync(
    path.join(getWorkspaceFolderPath(), "src", "ContractChild.vue"),
    expected.componentContractChildSource,
    "utf8",
  );
  const diskPath = path.join(getWorkspaceFolderPath(), "src", "ContractHost.vue");
  fs.writeFileSync(diskPath, expected.componentContractHostSource, "utf8");
  const uri = vscode.Uri.file(diskPath);
  const document = await vscode.workspace.openTextDocument(uri);
  await vscode.window.showTextDocument(document);

  const hovers = await waitFor(
    async () => ({
      importBinding: await hoverAt(uri, 1, 8),
      scriptUsage: await hoverAt(uri, 3, 1),
    }),
    (next) => deepEqual(next, expected.componentContractHovers),
    "component contract hovers",
    180_000,
  );
  assert.deepEqual(hovers, expected.componentContractHovers);
  for (const value of Object.values(hovers).flatMap((hover) =>
    hover.flatMap((item) => item.contents),
  )) {
    assert.doesNotMatch(value, /__vizeComponentMarker|__vizeRawProps|__VizeComponentConstructor/);
  }
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

function deepEqual(actual, expectedValue) {
  try {
    assert.deepEqual(actual, expectedValue);
    return true;
  } catch {
    return false;
  }
}

async function hoverAt(uri, line, character) {
  const hovers = await vscode.commands.executeCommand(
    "vscode.executeHoverProvider",
    uri,
    new vscode.Position(line, character),
  );
  return hovers.map(describeHover);
}

function describeHover(hover) {
  return {
    contents: hover.contents.map((content) => {
      if (typeof content === "string") return content;
      if (typeof content?.value === "string") return content.value;
      return String(content);
    }),
    range: hover.range === undefined ? undefined : describeRange(hover.range),
  };
}

function describeRange(range) {
  return [range.start.line, range.start.character, range.end.line, range.end.character];
}
