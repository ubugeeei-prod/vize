const assert = require("node:assert/strict");
const fs = require("node:fs");
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
  await runDisabledContributionSmoke();
  await runFakeServerLifecycleSmoke();
};

async function runDisabledContributionSmoke() {
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
}

async function runFakeServerLifecycleSmoke() {
  const fakeServerPath = process.env.VIZE_TEST_SERVER_PATH;
  const fakeServerLogPath = process.env.VIZE_TEST_SERVER_LOG;
  assert.ok(fakeServerPath, "VIZE_TEST_SERVER_PATH must be set");
  assert.ok(fakeServerLogPath, "VIZE_TEST_SERVER_LOG must be set");
  assert.ok(fs.existsSync(fakeServerPath), `missing fake server: ${fakeServerPath}`);

  fs.writeFileSync(fakeServerLogPath, "");

  await vscode.workspace
    .getConfiguration("vize")
    .update("serverPath", fakeServerPath, vscode.ConfigurationTarget.Workspace);
  await sleep(300);
  assert.equal(initializeMessages(readLogEntries(fakeServerLogPath)).length, 0);

  await vscode.commands.executeCommand("vize.enableRecommendedProfile");

  let entries = await waitForLogEntries(
    fakeServerLogPath,
    (nextEntries) => initializeMessages(nextEntries).length >= 1,
    "recommended profile initialization",
  );
  assert.deepEqual(lastInitialize(entries).params.initializationOptions, {
    editor: true,
    ecosystem: true,
    lint: true,
    typecheck: true,
  });

  await vscode.commands.executeCommand("vize.enableLintOnlyProfile");
  entries = await waitForLogEntries(
    fakeServerLogPath,
    (nextEntries) => initializeMessages(nextEntries).length >= 2,
    "lint-only profile initialization",
  );
  assert.deepEqual(lastInitialize(entries).params.initializationOptions, {
    lint: true,
  });

  await vscode.commands.executeCommand("vize.restartServer");
  entries = await waitForLogEntries(
    fakeServerLogPath,
    (nextEntries) => initializeMessages(nextEntries).length >= 3,
    "manual restart initialization",
  );
  assert.deepEqual(lastInitialize(entries).params.initializationOptions, {
    lint: true,
  });

  await vscode.commands.executeCommand("vize.disable");
  entries = await waitForLogEntries(
    fakeServerLogPath,
    (nextEntries) => nextEntries.filter((entry) => entry.method === "exit").length >= 3,
    "language server shutdown",
  );

  assert.equal(vscode.workspace.getConfiguration("vize").get("enable"), false);
  assert.ok(
    entries.some((entry) => entry.event === "version"),
    "expected configured server version inspection",
  );
}

function initializeMessages(entries) {
  return entries.filter((entry) => entry.method === "initialize");
}

function lastInitialize(entries) {
  return initializeMessages(entries).at(-1);
}

async function waitForLogEntries(logPath, predicate, label) {
  const timeoutAt = Date.now() + 10_000;
  let entries = [];

  while (Date.now() < timeoutAt) {
    entries = readLogEntries(logPath);
    if (predicate(entries)) {
      return entries;
    }

    await sleep(100);
  }

  assert.fail(`${label} did not happen. Last log entries: ${JSON.stringify(entries.slice(-10))}`);
}

function readLogEntries(logPath) {
  const text = fs.readFileSync(logPath, "utf-8").trim();
  if (!text) {
    return [];
  }

  return text.split("\n").map((line) => JSON.parse(line));
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
