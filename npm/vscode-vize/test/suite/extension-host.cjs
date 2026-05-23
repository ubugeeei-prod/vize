const assert = require("node:assert/strict");
const path = require("node:path");
const vscode = require("vscode");

const extensionId = "ubugeeei.vize";
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
};
