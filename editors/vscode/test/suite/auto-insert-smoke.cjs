const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");
const { featureSettingKeys } = require("./extension-host-fixtures.cjs");

exports.runAutoInsertSmoke = async function runAutoInsertSmoke() {
  const { logPath, serverPath } = getFakeServer();

  await prepareConfiguredFakeServer({ logPath, serverPath });
  const document = await openWorkspaceDocument("src", "App.vue");
  const editor = await vscode.window.showTextDocument(document);
  const line = document.lineAt(5);
  const insertion = line.text.indexOf("</main>");
  assert.ok(insertion > 0, "expected the fixture main close tag");
  assert.ok(
    await editor.edit((edit) => edit.insert(new vscode.Position(5, insertion), "{}")),
    "expected interpolation seed edit to apply",
  );
  editor.selection = new vscode.Selection(5, insertion + 1, 5, insertion + 1);

  await updateVizeConfiguration("autoInsert.enable", true);
  await updateVizeConfiguration("enable", true);
  let entries = await waitForReadyServer(logPath, "automatic insertion profile");
  assertInitializationOptions(entries, { autoInsert: true });

  await vscode.commands.executeCommand("type", { text: "{" });

  entries = await waitForLogEntries(
    logPath,
    (nextEntries) => methodMessages(nextEntries, "volar/client/autoInsert").length >= 1,
    "automatic insertion request",
  );
  const request = methodMessages(entries, "volar/client/autoInsert").at(-1);
  assert.deepEqual(request.params.change, {
    rangeLength: 0,
    rangeOffset: document.offsetAt(new vscode.Position(5, insertion + 1)),
    text: "{}",
  });
  assert.deepEqual(request.params.selection, { character: insertion + 2, line: 5 });
  const insertedLine = await waitForDocumentText(
    document,
    (text) => text.includes("{{  }}</main>"),
    "automatic insertion snippet",
  );
  assert.ok(insertedLine.includes("{{  }}</main>"), insertedLine);

  await disableVizeAndWaitForShutdown(logPath);
};

function getFakeServer() {
  const serverPath = process.env.VIZE_TEST_SERVER_PATH;
  const logPath = process.env.VIZE_TEST_SERVER_LOG;
  assert.ok(serverPath, "VIZE_TEST_SERVER_PATH must be set");
  assert.ok(logPath, "VIZE_TEST_SERVER_LOG must be set");
  assert.ok(fs.existsSync(serverPath), `missing fake server: ${serverPath}`);
  return { logPath, serverPath };
}

async function prepareConfiguredFakeServer({ logPath, serverPath }) {
  fs.writeFileSync(logPath, "");
  const config = vscode.workspace.getConfiguration("vize");
  await config.update("enable", false, vscode.ConfigurationTarget.Workspace);
  await config.update("serverPath", undefined, vscode.ConfigurationTarget.Workspace);
  await config.update("trace.server", undefined, vscode.ConfigurationTarget.Workspace);
  for (const key of featureSettingKeys) {
    await config.update(key, undefined, vscode.ConfigurationTarget.Workspace);
  }
  await config.update("serverPath", serverPath, vscode.ConfigurationTarget.Workspace);
  await sleep(300);
  assert.equal(initializeMessages(readLogEntries(logPath)).length, 0);
}

async function updateVizeConfiguration(key, value) {
  await vscode.workspace
    .getConfiguration("vize")
    .update(key, value, vscode.ConfigurationTarget.Workspace);
}

async function disableVizeAndWaitForShutdown(logPath) {
  const exitCount = methodMessages(readLogEntries(logPath), "exit").length;
  await vscode.commands.executeCommand("vize.disable");
  await waitForLogEntries(
    logPath,
    (entries) => methodMessages(entries, "exit").length > exitCount,
    "language server shutdown",
  );
}

function waitForReadyServer(logPath, label) {
  return waitForLogEntries(
    logPath,
    (entries) =>
      initializeMessages(entries).length >= 1 && methodMessages(entries, "initialized").length >= 1,
    label,
  );
}

function openWorkspaceDocument(...segments) {
  const folder = vscode.workspace.workspaceFolders?.[0];
  assert.ok(folder, "expected a workspace folder");
  return vscode.workspace.openTextDocument(
    vscode.Uri.file(path.join(folder.uri.fsPath, ...segments)),
  );
}

function assertInitializationOptions(entries, expected) {
  assert.deepEqual(initializeMessages(entries).at(-1).params.initializationOptions ?? {}, expected);
}

const initializeMessages = (entries) => entries.filter((entry) => entry.method === "initialize");
const methodMessages = (entries, method) => entries.filter((entry) => entry.method === method);

async function waitForLogEntries(logPath, predicate, label) {
  const timeoutAt = Date.now() + 20_000;
  let entries = [];
  while (Date.now() < timeoutAt) {
    entries = readLogEntries(logPath);
    if (predicate(entries)) return entries;
    await sleep(100);
  }
  assert.fail(`${label} did not happen. Last log entries: ${JSON.stringify(entries.slice(-10))}`);
}

function readLogEntries(logPath) {
  const text = fs.readFileSync(logPath, "utf-8").trim();
  return text ? text.split("\n").map((line) => JSON.parse(line)) : [];
}

async function waitForDocumentText(document, predicate, label) {
  const timeoutAt = Date.now() + 20_000;
  let text = document.getText();
  while (Date.now() < timeoutAt) {
    text = document.getText();
    if (predicate(text)) return text;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert.fail(`${label} did not happen. Last document: ${text}`);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
