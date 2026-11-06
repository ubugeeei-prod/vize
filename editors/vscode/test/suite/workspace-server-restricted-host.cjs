const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");
const { waitFor } = require("./wait-for.cjs");

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
  const server = await waitFor(
    async () => {
      try {
        return await vscode.commands.executeCommand("vize.test.getServerInfo");
      } catch {
        return undefined;
      }
    },
    (value) => value?.status === "ready" && value.source === "configured",
    "safe configured server in Restricted Mode",
    60_000,
  );
  assert.equal(
    path.relative(
      fs.realpathSync(server.path),
      fs.realpathSync(process.env.VIZE_TEST_SAFE_SERVER_PATH),
    ),
    "",
  );
  assert.equal(server.workspaceFallbackReason, "Workspace servers skipped in Restricted Mode.");
  assert.equal(fs.existsSync(process.env.VIZE_TEST_FORBIDDEN_SERVER_MARKER), false);
  const hover = await vscode.commands.executeCommand("vize.test.executeLspRequest", {
    method: "textDocument/hover",
    params: { textDocument: { uri: document.uri.toString() }, position: { line: 1, character: 8 } },
  });
  assert.ok(hover?.contents, `safe server must answer real hover: ${JSON.stringify(hover)}`);
  fs.writeFileSync(
    process.env.VIZE_TEST_RESTRICTED_RESULT,
    JSON.stringify({ installedExtensionPath, server, hover }, null, 2) + "\n",
  );
  await vscode.commands.executeCommand("vize.disable");
};
