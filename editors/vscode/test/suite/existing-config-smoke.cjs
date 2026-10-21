const assert = require("node:assert/strict");
const path = require("node:path");
const fs = require("node:fs");
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
    "jsconfig.json",
    "packages/app/tsconfig.app.json",
    "packages/app/vite.config.ts",
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
  const nodeModulesUri = vscode.Uri.file(path.join(root, "node_modules"));
  const dependencyDirectory = vscode.Uri.file(path.dirname(dependencyUri.fsPath));
  const unrelatedDirectory = vscode.Uri.file(
    path.join(root, "node_modules/existing-config-unrelated-dependency"),
  );
  const unrelatedFile = vscode.Uri.file(path.join(unrelatedDirectory.fsPath, "retained.txt"));
  const unrelatedBytes = Buffer.from("pre-existing dependency contents\n");
  let createdNodeModules = false;
  try {
    await vscode.workspace.fs.stat(nodeModulesUri);
  } catch (error) {
    if (error.code !== "FileNotFound") throw error;
    createdNodeModules = true;
  }
  await assert.rejects(vscode.workspace.fs.stat(dependencyDirectory), {
    code: "FileNotFound",
  });
  await assert.rejects(vscode.workspace.fs.stat(unrelatedDirectory), {
    code: "FileNotFound",
  });
  await vscode.workspace.fs.createDirectory(unrelatedDirectory);
  await vscode.workspace.fs.writeFile(unrelatedFile, unrelatedBytes);
  await vscode.workspace.fs.createDirectory(vscode.Uri.file(path.dirname(dependencyUri.fsPath)));
  await vscode.workspace.fs.writeFile(dependencyUri, Buffer.from("{}\n"));
  await updateVizeConfiguration("enable", true);
  let entries = await waitForReadyServer(logPath, "dependency configs are ignored");
  assertInitializationOptions(entries, recommendedInitializationOptions);

  const manualPid = lspPids(entries).at(-1);
  assert.ok(Number.isInteger(manualPid), "manual profile has a real LSP process PID");
  const projectUri = vscode.Uri.file(path.join(root, "tsconfig.json"));
  await vscode.workspace.fs.writeFile(projectUri, Buffer.from("{}\n"));
  entries = await waitForLogEntries(
    logPath,
    (next) => initializeMessages(next).length >= 2,
    "project config creation refreshes initialization",
  );
  assertInitializationOptions(entries, {});
  const projectPid = lspPids(entries).at(-1);
  assert.ok(Number.isInteger(projectPid));
  assert.notEqual(projectPid, manualPid, "project creation replaces the actual LSP process");
  await vscode.workspace.fs.delete(projectUri);
  entries = await waitForLogEntries(
    logPath,
    (next) => initializeMessages(next).length >= 3,
    "project config removal restores the manual profile",
  );
  assertInitializationOptions(entries, recommendedInitializationOptions);
  assert.notEqual(
    lspPids(entries).at(-1),
    projectPid,
    "project removal replaces the actual LSP process",
  );
  await disableVizeAndWaitForShutdown(logPath);
  await vscode.workspace.fs.delete(dependencyDirectory, { recursive: true });
  assert.deepEqual(Buffer.from(await vscode.workspace.fs.readFile(unrelatedFile)), unrelatedBytes);
  await vscode.workspace.fs.delete(unrelatedDirectory, { recursive: true });
  if (createdNodeModules) await vscode.workspace.fs.delete(nodeModulesUri);
  await runExistingConfigurationContentChangeSmoke();
};

async function runExistingConfigurationContentChangeSmoke() {
  const { logPath, serverPath } = getFakeServer();
  const root = getWorkspaceFolder().uri.fsPath;
  // Nested config discovery and create/delete routing are covered above. Keep
  // content-change delivery on root configs: VS Code can drop a nested event
  // when its recursive watcher is repeatedly replaced in one extension host.
  for (const filename of ["vite.config.ts", "vite.config.mjs"]) {
    await prepareConfiguredFakeServer({ logPath, serverPath });
    const uri = vscode.Uri.file(path.join(root, filename));
    await vscode.workspace.fs.createDirectory(vscode.Uri.file(path.dirname(uri.fsPath)));
    await vscode.workspace.fs.writeFile(uri, Buffer.from("{}\n"));
    await updateVizeConfiguration("typecheck.enable", false);
    await updateVizeConfiguration("enable", undefined);
    const options = { typecheck: false };
    let entries = await waitForReadyServer(logPath, `${filename} content-change startup`);
    assertInitializationOptions(entries, options);
    const pid = lspPids(entries).at(-1);
    assert.ok(Number.isInteger(pid), "content-change server has a real LSP process PID");
    entries = await waitForLogEntries(
      logPath,
      (next) =>
        next.some(
          (entry) =>
            entry.event === "configuration-watch-ack" &&
            entry.pid === pid &&
            entry.response.result === null &&
            entry.response.error === undefined,
        ),
      `${filename} real client ACKs dynamic configuration watches`,
    );
    const previousPids = lspPids(entries);
    const previousInitializations = initializeMessages(entries).length;

    const packetPrefix = path.join(path.dirname(logPath), "existing-config-events", filename);
    fs.mkdirSync(path.dirname(packetPrefix), { recursive: true });
    const retain = (stage) => {
      fs.copyFileSync(logPath, `${packetPrefix}.${stage}.jsonl`);
      fs.copyFileSync(uri.fsPath, `${packetPrefix}.${stage}.source.txt`);
    };
    retain("before");
    const changeStart = readLogEntries(logPath).length;
    try {
      await vscode.workspace.fs.writeFile(
        uri,
        Buffer.from('{"compilerOptions":{"strict":true}}\n'),
      );
      entries = await waitForLogEntries(
        logPath,
        (next) =>
          next
            .slice(changeStart)
            .some(
              (entry) =>
                entry.event === "message" &&
                entry.method === "workspace/didChangeWatchedFiles" &&
                entry.params?.changes?.some(
                  (change) => change.uri === uri.toString() && change.type === 2,
                ),
            ),
        `${filename} existing content edit reaches the same server through an OS watcher`,
      );
      const watchPackets = entries
        .slice(changeStart)
        .filter(
          (entry) =>
            entry.event === "message" && entry.method === "workspace/didChangeWatchedFiles",
        );
      const changes = watchPackets.flatMap((entry) => entry.params.changes);
      assert.ok(changes.length > 0, "only a real client notification can satisfy the edit oracle");
      for (const change of changes) assert.deepEqual(change, { uri: uri.toString(), type: 2 });
      assert.equal(initializeMessages(entries).length, previousInitializations);
      assert.deepEqual(
        lspPids(entries),
        previousPids,
        "same-options edit retains the entire LSP PID history",
      );
      assert.deepEqual(
        initializeMessages(entries).map((message) => message.params.initializationOptions),
        Array.from({ length: previousInitializations }, () => options),
      );
    } finally {
      retain("after");
    }
    await disableVizeAndWaitForShutdown(logPath);
    await vscode.workspace.fs.delete(uri);
  }
}

function lspPids(entries) {
  return entries
    .filter((entry) => entry.event === "process-start" && entry.args?.[0] === "lsp")
    .map((entry) => entry.pid);
}
