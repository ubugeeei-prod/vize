const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");
const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
async function eventually(callback) {
  let last;
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    try {
      return await callback();
    } catch (error) {
      last = error;
      await pause(200);
    }
  }
  throw last;
}
async function run() {
  const workspace = vscode.workspace.workspaceFolders[0].uri.fsPath;
  const uri = vscode.Uri.file(path.join(workspace, "src/main.ts"));
  const document = await vscode.workspace.openTextDocument(uri);
  await vscode.window.showTextDocument(document);
  const extension = vscode.extensions.getExtension("ubugeeei.vize");
  assert(extension, "Installed VSIX must be present");
  const diagnostics = await eventually(() => {
    const values = vscode.languages.getDiagnostics(uri);
    assert(
      values.some((value) => [2322, 2741].includes(Number(value.code))),
      JSON.stringify(values),
    );
    assert(
      values.some((value) => Number(value.code) === 2307 && value.message.includes("Missing.vue")),
      JSON.stringify(values),
    );
    assert(
      !values.some((value) => Number(value.code) === 2307 && value.message.includes("App.vue")),
      JSON.stringify(values),
    );
    return values;
  });
  const text = document.getText();
  const offset = text.indexOf("props.title") + "props.".length + 1;
  const position = document.positionAt(offset);
  const hover = await eventually(async () => {
    const values = await vscode.commands.executeCommand(
      "vscode.executeHoverProvider",
      uri,
      position,
    );
    assert(values?.length > 0);
    assert(
      values.some((value) =>
        value.contents.some((content) =>
          /string/.test(typeof content === "string" ? content : content.value),
        ),
      ),
    );
    return values;
  });
  const completion = await eventually(async () => {
    const value = await vscode.commands.executeCommand(
      "vscode.executeCompletionItemProvider",
      uri,
      document.positionAt(text.indexOf("props.title") + "props.".length),
    );
    assert(
      value.items.some(
        (entry) => (typeof entry.label === "string" ? entry.label : entry.label.label) === "title",
      ),
    );
    return value;
  });
  const definition = await eventually(async () => {
    const values = await vscode.commands.executeCommand(
      "vscode.executeDefinitionProvider",
      uri,
      document.positionAt(text.indexOf("App from") + 1),
    );
    assert(
      values.some(
        (value) => (value.targetUri || value.uri).fsPath === path.join(workspace, "src/App.vue"),
      ),
    );
    return values;
  });
  fs.writeFileSync(
    process.env.VIZE_TYPESCRIPT_PROVIDER_RESULT,
    JSON.stringify(
      { extensionPath: extension.extensionPath, diagnostics, hover, completion, definition },
      null,
      2,
    ),
  );
}
module.exports = { run };
