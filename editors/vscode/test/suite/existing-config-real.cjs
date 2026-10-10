const assert = require("node:assert/strict");
const fs = require("node:fs");
const vscode = require("vscode");
const {
  assertPackagedExtension,
  openWorkspaceDocument,
  positionAfter,
  waitForDiagnostics,
} = require("./real-server-support.cjs");
const { waitFor } = require("./wait-for.cjs");

exports.run = async function run() {
  const extension = vscode.extensions.getExtension("ubugeeei.vize");
  assert.ok(extension);
  assertPackagedExtension(extension);
  await extension.activate();
  const root = vscode.workspace.workspaceFolders[0].uri.fsPath;
  assert.equal(
    fs.readdirSync(root).some((name) => name.startsWith("vize.config.")),
    false,
  );
  const config = vscode.workspace.getConfiguration("vize");
  for (const key of [
    "enable",
    "lint.enable",
    "typecheck.enable",
    "editor.enable",
    "ecosystem.enable",
    "formatting.enable",
  ]) {
    assert.equal(config.inspect(key).workspaceValue, undefined, `${key} must use project defaults`);
  }

  const document = await openWorkspaceDocument("src", "App.vue");
  await vscode.window.showTextDocument(document);
  const diagnostics = await waitForDiagnostics(
    document.uri,
    (values) => values.some((value) => value.code === 2322),
    "automatic project startup publishes the real prop type mismatch",
    120_000,
  );
  assert.ok(diagnostics.some((value) => value.source === "vize/types"));
  const hover = () =>
    vscode.commands.executeCommand(
      "vscode.executeHoverProvider",
      document.uri,
      positionAfter(document, "{{ amount }}", "{{ am"),
    );
  if (process.env.VIZE_TEST_CONFIG_MODE === "vite") {
    assert.equal(
      (await hover())?.length ?? 0,
      0,
      "Vite hover disable must survive automatic startup",
    );
  } else {
    const hovers = await waitFor(
      hover,
      (values) => values?.length > 0,
      "default editor hover",
      30_000,
    );
    assert.ok(hovers[0].contents.length > 0);
  }
  const formattingDocument = await openWorkspaceDocument("src", "Formatting.vue");
  const edits = await waitFor(
    () =>
      vscode.commands.executeCommand(
        "vscode.executeFormatDocumentProvider",
        formattingDocument.uri,
        { insertSpaces: true, tabSize: 2 },
      ),
    (values) => values?.length > 0,
    "default project formatting",
    30_000,
  );
  assert.ok(edits[0].newText.includes("<template>"));
  await vscode.commands.executeCommand("vize.disable");
  assert.equal(vscode.workspace.getConfiguration("vize").get("enable"), false);
};
