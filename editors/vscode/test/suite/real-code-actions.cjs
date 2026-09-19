const assert = require("node:assert/strict");
const vscode = require("vscode");
const { openWorkspaceDocument, waitForDiagnostics } = require("./real-server-support.cjs");

exports.runRealCodeActionSmoke = async function runRealCodeActionSmoke() {
  const document = await openWorkspaceDocument("src", "QuickFix.vue");
  await vscode.window.showTextDocument(document);
  const original = document.getText();
  const diagnostics = await waitForDiagnostics(
    document.uri,
    (items) => items.some((item) => item.code === 2420),
    "missing interface implementation",
    120_000,
  );
  const diagnostic = diagnostics.find((item) => item.code === 2420);
  const actions = await vscode.commands.executeCommand(
    "vscode.executeCodeActionProvider",
    document.uri,
    diagnostic.range,
    vscode.CodeActionKind.QuickFix.value,
  );
  const action = actions.find((item) => item.title === "Implement interface 'Greeting'");
  assert.ok(action?.edit, JSON.stringify(actions));
  const entries = action.edit.entries();
  assert.equal(entries.length, 1);
  assert.equal(entries[0][0].toString(), document.uri.toString());
  assert.equal(entries[0][1].length, 1);
  const insertion = document.positionAt(original.indexOf("{}") + 1);
  assert.deepEqual(entries[0][1][0].range, new vscode.Range(insertion, insertion));
  assert.equal(await vscode.workspace.applyEdit(action.edit), true);
  await waitForDiagnostics(
    document.uri,
    (items) => items.every((item) => item.severity !== vscode.DiagnosticSeverity.Error),
    "applied interface implementation",
    120_000,
  );
  assert.match(document.getText(), /greet\(name: string\): string/);
  assert.doesNotMatch(document.getText(), /__Vize|__vize/);
  // The fixture stays reusable; restoration is observed by the live checker.
  const restore = new vscode.WorkspaceEdit();
  restore.replace(
    document.uri,
    new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)),
    original,
  );
  assert.equal(await vscode.workspace.applyEdit(restore), true);
  await waitForDiagnostics(
    document.uri,
    (items) => items.some((item) => item.code === 2420),
    "restored interface diagnostic",
    120_000,
  );
};
