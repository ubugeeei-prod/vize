const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");
const { featureSettingKeys } = require("./extension-host-fixtures.cjs");

/**
 * The fake-server plumbing every extension-host suite shares.
 *
 * The suites talk to a stub language server that appends one JSON line per LSP
 * message to `VIZE_TEST_SERVER_LOG`, so "wait until the extension did X" is
 * always "poll that log until the predicate holds". Those waits, the workspace
 * configuration resets that keep one suite from inheriting the previous one's
 * settings, and the assertion helpers over log entries live here so the suites
 * read as the scenario they exercise rather than as their scaffolding.
 *
 * `editor-capability-smoke.cjs` already takes this surface as an argument
 * rather than importing it, and that stays true: the exports below are what
 * `extension-host.cjs` hands it.
 */

async function prepareConfiguredFakeServer({ logPath, serverPath }) {
  fs.writeFileSync(logPath, "");
  await resetVizeConfiguration();
  await updateVizeConfiguration("serverPath", serverPath);
  await sleep(300);
  assert.equal(initializeMessages(readLogEntries(logPath)).length, 0);
}

async function resetVizeConfiguration() {
  await updateVizeConfiguration("enable", false);
  await updateVizeConfiguration("serverPath", undefined);
  await updateVizeConfiguration("trace.server", undefined);
  await updateVizeConfigurationEntries(featureSettingKeys.map((key) => [key, undefined]));
  await updateVizeConfiguration("enable", false);
  await sleep(300);
}

async function updateVizeConfigurationEntries(entries) {
  for (const [key, value] of entries) {
    await updateVizeConfiguration(key, value);
  }
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

  assert.equal(vscode.workspace.getConfiguration("vize").get("enable"), false);
}

async function waitForReadyServer(logPath, label) {
  return waitForLogEntries(
    logPath,
    (entries) =>
      initializeMessages(entries).length >= 1 && methodMessages(entries, "initialized").length >= 1,
    label,
  );
}

async function runProviderCommand(logPath, spec) {
  const requestCount = methodMessages(readLogEntries(logPath), spec.method).length;
  const result = await executeFirstAvailableCommand(spec.commandIds, spec.args);
  const entries = await waitForLogEntries(
    logPath,
    (entries) => methodMessages(entries, spec.method).length > requestCount,
    `${spec.label} request`,
  );
  spec.validate(result, methodMessages(entries, spec.method).at(-1));
}

async function executeFirstAvailableCommand(commandIds, args) {
  let missingCommandError;

  for (const commandId of commandIds) {
    try {
      return await vscode.commands.executeCommand(commandId, ...args);
    } catch (error) {
      if (!String(error).includes("command") || !String(error).includes("not found")) {
        throw error;
      }

      missingCommandError = error;
    }
  }

  assert.fail(
    `missing VS Code provider command: ${commandIds.join(" or ")}. Last error: ${String(
      missingCommandError,
    )}`,
  );
}

async function waitForMethodWithUri(logPath, method, uri) {
  await waitForLogEntries(
    logPath,
    (entries) =>
      methodMessages(entries, method).some((entry) => entry.params?.textDocument?.uri === uri),
    `${method} for ${uri}`,
  );
}

async function waitForWatchedFileChange(logPath, uri, type, label) {
  await waitForLogEntries(
    logPath,
    (entries) =>
      methodMessages(entries, "workspace/didChangeWatchedFiles").some((entry) =>
        entry.params?.changes?.some((change) => change.uri === uri && change.type === type),
      ),
    label,
  );
}

async function openWorkspaceDocument(...segments) {
  const workspaceFolder = getWorkspaceFolder();
  return vscode.workspace.openTextDocument(
    vscode.Uri.file(path.join(workspaceFolder.uri.fsPath, ...segments)),
  );
}

function getWorkspaceFolder() {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  assert.ok(workspaceFolder, "expected a workspace folder");
  return workspaceFolder;
}

function getFakeServer() {
  const serverPath = process.env.VIZE_TEST_SERVER_PATH;
  const logPath = process.env.VIZE_TEST_SERVER_LOG;
  assert.ok(serverPath, "VIZE_TEST_SERVER_PATH must be set");
  assert.ok(logPath, "VIZE_TEST_SERVER_LOG must be set");
  assert.ok(fs.existsSync(serverPath), `missing fake server: ${serverPath}`);
  return { logPath, serverPath };
}

function assertInitializationOptions(entries, expected) {
  assert.deepEqual(lastInitialize(entries).params.initializationOptions ?? {}, expected);
}

function assertTextDocumentRequest(entry, uri, position) {
  assert.equal(entry.params.textDocument.uri, uri.toString());
  assert.deepEqual(entry.params.position, {
    character: position.character,
    line: position.line,
  });
}

function assertLocation(location, uri, range) {
  assert.equal(location.uri.toString(), uri.toString());
  assert.deepEqual(location.range, range);
}

function assertDiagnostic(diagnostics, expected) {
  const diagnostic = diagnostics.find(
    (nextDiagnostic) =>
      nextDiagnostic.source === expected.source && nextDiagnostic.code === expected.code,
  );
  assert.ok(diagnostic, `missing diagnostic ${expected.source} ${expected.code}`);
  assert.equal(diagnostic.message, expected.message);
  assert.equal(diagnostic.severity, expected.severity);
  assert.deepEqual(diagnostic.range, expected.range);
}

async function waitForDiagnostics(uri, predicate, label) {
  const timeoutAt = Date.now() + 20_000;
  let diagnostics = [];

  while (Date.now() < timeoutAt) {
    diagnostics = vscode.languages.getDiagnostics(uri);
    if (predicate(diagnostics)) {
      return diagnostics;
    }

    await sleep(100);
  }

  assert.fail(`${label} did not happen. Last diagnostics: ${JSON.stringify(diagnostics)}`);
}

const initializeMessages = (entries) => entries.filter((entry) => entry.method === "initialize");

const methodMessages = (entries, method) => entries.filter((entry) => entry.method === method);

function lastInitialize(entries) {
  const message = initializeMessages(entries).at(-1);
  assert.ok(message, "expected at least one initialize message");
  return message;
}

async function waitForLogEntries(logPath, predicate, label) {
  const timeoutAt = Date.now() + 20_000;
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

module.exports = {
  assertDiagnostic,
  assertInitializationOptions,
  assertLocation,
  assertTextDocumentRequest,
  disableVizeAndWaitForShutdown,
  getFakeServer,
  getWorkspaceFolder,
  initializeMessages,
  methodMessages,
  openWorkspaceDocument,
  prepareConfiguredFakeServer,
  readLogEntries,
  runProviderCommand,
  sleep,
  updateVizeConfiguration,
  updateVizeConfigurationEntries,
  waitForDiagnostics,
  waitForLogEntries,
  waitForMethodWithUri,
  waitForReadyServer,
  waitForWatchedFileChange,
};
