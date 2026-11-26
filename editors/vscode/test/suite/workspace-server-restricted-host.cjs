const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");

// The server info command reports the server the extension selected, if any.
async function selectedServer() {
  await vscode.commands.executeCommand("vize.test.waitForConfigurationIdle");
  try {
    return await vscode.commands.executeCommand("vize.test.getServerInfo");
  } catch {
    return null;
  }
}

exports.run = async function run() {
  assert.equal(vscode.workspace.isTrusted, false);
  const extension = vscode.extensions.getExtension("ubugeeei.vize");
  assert.ok(extension);
  const installedExtensionPath = fs.realpathSync(extension.extensionPath);
  const bundle = fs.readFileSync(path.join(installedExtensionPath, "dist", "extension.cjs"));
  assert.equal(
    createHash("sha256").update(bundle).digest("hex"),
    process.env.VIZE_TEST_EXTENSION_BUNDLE_SHA256,
  );

  await extension.activate();
  const document = await vscode.workspace.openTextDocument(
    vscode.Uri.joinPath(vscode.workspace.workspaceFolders[0].uri, "App.vue"),
  );
  await vscode.window.showTextDocument(document);
  // The language server waits for trust: neither the user's configured server
  // nor anything in the workspace runs, and a manual restart changes nothing.
  assert.equal(await selectedServer(), null);
  await vscode.commands.executeCommand("vize.restartServer");
  const server = await selectedServer();
  assert.equal(server, null);
  assert.equal(fs.existsSync(process.env.VIZE_TEST_SAFE_SERVER_LOG), false);
  assert.equal(fs.existsSync(process.env.VIZE_TEST_FORBIDDEN_SERVER_MARKER), false);
  fs.writeFileSync(
    process.env.VIZE_TEST_RESTRICTED_RESULT,
    JSON.stringify({ installedExtensionPath, server }, null, 2) + "\n",
  );
};
