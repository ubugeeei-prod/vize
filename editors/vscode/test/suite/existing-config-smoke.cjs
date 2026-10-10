const assert = require("node:assert/strict");
const path = require("node:path");
const vscode = require("vscode");
const { recommendedInitializationOptions } = require("./extension-host-fixtures.cjs");
const {
  assertInitializationOptions,
  disableVizeAndWaitForShutdown,
  getFakeServer,
  getWorkspaceFolder,
  initializeMessages,
  prepareConfiguredFakeServer,
  readLogEntries,
  updateVizeConfiguration,
  waitForLogEntries,
  waitForReadyServer,
} = require("./extension-host-support.cjs");

exports.runExistingConfigSmoke = async function runExistingConfigSmoke() {
  const { logPath, serverPath } = getFakeServer();
  const root = getWorkspaceFolder().uri.fsPath;
  for (const filename of [
    "vite.config.ts",
    "vite.config.mjs",
    "tsconfig.json",
    "packages/app/tsconfig.app.json",
    "packages/app/vite.config.ts",
    "vize.config.json",
  ]) {
    await prepareConfiguredFakeServer({ logPath, serverPath });
    const uri = vscode.Uri.file(path.join(root, filename));
    await vscode.workspace.fs.createDirectory(vscode.Uri.file(path.dirname(uri.fsPath)));
    await vscode.workspace.fs.writeFile(uri, Buffer.from("{}\n"));

    // A discovered config must never override an explicit extension disable.
    await vscode.commands.executeCommand("vize.restartServer");
    assert.equal(initializeMessages(readLogEntries(logPath)).length, 0);
    await updateVizeConfiguration("enable", undefined);
    let entries = await waitForReadyServer(logPath, `${filename} default startup`);
    assertInitializationOptions(entries, {});
    assert.equal(
      vscode.workspace.getConfiguration("vize").inspect("enable").workspaceValue,
      undefined,
    );

    // Only explicit editor switches cross the initialization boundary; native
    // workspace settings and recommended defaults keep their own precedence.
    await updateVizeConfiguration("formatting.enable", false);
    entries = await waitForLogEntries(
      logPath,
      (next) => initializeMessages(next).length >= 2,
      `${filename} explicit formatting disable`,
    );
    assertInitializationOptions(entries, { formatting: false });
    await disableVizeAndWaitForShutdown(logPath);
    await vscode.workspace.fs.delete(uri);
  }

  await prepareConfiguredFakeServer({ logPath, serverPath });
  const dependencyUri = vscode.Uri.file(path.join(root, "node_modules/dependency/tsconfig.json"));
  await vscode.workspace.fs.createDirectory(vscode.Uri.file(path.dirname(dependencyUri.fsPath)));
  await vscode.workspace.fs.writeFile(dependencyUri, Buffer.from("{}\n"));
  await updateVizeConfiguration("enable", true);
  let entries = await waitForReadyServer(logPath, "dependency configs are ignored");
  assertInitializationOptions(entries, recommendedInitializationOptions);

  const projectUri = vscode.Uri.file(path.join(root, "tsconfig.json"));
  await vscode.workspace.fs.writeFile(projectUri, Buffer.from("{}\n"));
  entries = await waitForLogEntries(
    logPath,
    (next) => initializeMessages(next).length >= 2,
    "project config creation refreshes initialization",
  );
  assertInitializationOptions(entries, {});
  await vscode.workspace.fs.delete(projectUri);
  entries = await waitForLogEntries(
    logPath,
    (next) => initializeMessages(next).length >= 3,
    "project config removal restores the manual profile",
  );
  assertInitializationOptions(entries, recommendedInitializationOptions);
  await disableVizeAndWaitForShutdown(logPath);
  await vscode.workspace.fs.delete(vscode.Uri.file(path.join(root, "node_modules")), {
    recursive: true,
  });
};
