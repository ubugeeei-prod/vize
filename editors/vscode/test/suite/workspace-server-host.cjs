const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");
const { waitFor } = require("./wait-for.cjs");

exports.run = async function run() {
  const extension = vscode.extensions.getExtension("ubugeeei.vize");
  assert.ok(extension);
  const installedExtensionPath = fs.realpathSync(extension.extensionPath);
  const bundle = fs.readFileSync(path.join(installedExtensionPath, "dist", "extension.cjs"));
  assert.equal(
    createHash("sha256").update(bundle).digest("hex"),
    process.env.VIZE_TEST_EXTENSION_BUNDLE_SHA256,
  );
  const expectedPath = fs.realpathSync(process.env.VIZE_TEST_WORKSPACE_SERVER_PATH);
  assert.equal(vscode.workspace.isTrusted, true);
  await extension.activate();
  const document = await vscode.workspace.openTextDocument(
    vscode.Uri.joinPath(vscode.workspace.workspaceFolders[0].uri, "App.vue"),
  );
  await vscode.window.showTextDocument(document);
  async function info(source) {
    return waitFor(
      async () => {
        try {
          return await vscode.commands.executeCommand("vize.test.getServerInfo");
        } catch {
          return undefined;
        }
      },
      (value) => value?.status === "ready" && value.source === source,
      `selected ${source} server`,
      60_000,
    );
  }
  const workspace = await info("workspace");
  assert.equal(path.relative(fs.realpathSync(workspace.path), expectedPath), "");
  assert.equal(workspace.version, process.env.VIZE_TEST_WORKSPACE_SERVER_VERSION);
  assert.notEqual(workspace.version, workspace.extensionVersion);
  const hover = await vscode.commands.executeCommand("vize.test.executeLspRequest", {
    method: "textDocument/hover",
    params: { textDocument: { uri: document.uri.toString() }, position: { line: 1, character: 8 } },
  });
  assert.ok(hover?.contents, `workspace server must answer real hover: ${JSON.stringify(hover)}`);
  // Explicit overrides retain priority even with workspace preference enabled.
  await vscode.workspace
    .getConfiguration("vize")
    .update("serverPath", expectedPath, vscode.ConfigurationTarget.Workspace);
  const configured = await info("configured");
  assert.equal(path.relative(fs.realpathSync(configured.path), expectedPath), "");
  fs.writeFileSync(
    process.env.VIZE_TEST_WORKSPACE_RESULT,
    JSON.stringify({ installedExtensionPath, workspace, hover, configured }, null, 2) + "\n",
  );
  await vscode.commands.executeCommand("vize.disable");
};
