const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");
const { featureSettingKeys } = require("./extension-host-fixtures.cjs");

function assertPackagedExtension(extension) {
  const extensionsPath = getRequiredPath(
    "VIZE_TEST_PACKAGED_EXTENSIONS_DIR",
    "packaged extension directory",
  );
  const sourceExtensionPath = getRequiredPath(
    "VIZE_TEST_SOURCE_EXTENSION_PATH",
    "source extension directory",
  );
  const installedPath = fs.realpathSync(extension.extensionPath);
  const relativeToInstallRoot = path.relative(extensionsPath, installedPath);

  assert.ok(
    relativeToInstallRoot &&
      !relativeToInstallRoot.startsWith(`..${path.sep}`) &&
      !path.isAbsolute(relativeToInstallRoot),
    `extension must load from the isolated install directory: ${installedPath}`,
  );
  assert.notEqual(installedPath, sourceExtensionPath, "extension must not load from repo source");
  assert.equal(extension.packageJSON.main, "./dist/extension.cjs");
  assert.ok(
    fs.existsSync(path.resolve(installedPath, extension.packageJSON.main)),
    "installed extension must contain its packaged entrypoint",
  );
}

function getRequiredPath(environmentName, label) {
  const value = process.env[environmentName];
  assert.ok(value, `${environmentName} must be set`);
  assert.ok(fs.existsSync(value), `missing ${label}: ${value}`);
  return fs.realpathSync(value);
}

function getRealServer() {
  const serverPath = process.env.VIZE_TEST_SERVER_PATH;
  assert.ok(serverPath, "VIZE_TEST_SERVER_PATH must be set");
  assert.ok(fs.existsSync(serverPath), `missing real server: ${serverPath}`);
  return serverPath;
}

async function prepareConfiguredRealServer(serverPath, overrides = {}) {
  await updateVizeConfiguration("enable", false);
  await updateVizeConfiguration("trace.server", undefined);
  for (const key of featureSettingKeys) {
    await updateVizeConfiguration(key, undefined);
  }
  for (const [key, value] of Object.entries(overrides)) {
    await updateVizeConfiguration(key, value);
  }
  await updateVizeConfiguration("serverPath", serverPath);
  // This short settle only lets the extension coalesce the configuration
  // writes above into a single client restart; it is deliberately not a
  // readiness wait. Callers must not fire one-shot provider requests straight
  // after this: every suite first blocks on `waitForDiagnostics` for a document
  // it opens after the restart, which is the deterministic gate that proves the
  // new session is serving.
  await sleep(300);
}

async function updateVizeConfiguration(key, value) {
  await vscode.workspace
    .getConfiguration("vize")
    .update(key, value, vscode.ConfigurationTarget.Workspace);
}

async function assertStaysDiagnosticFree(uri, label) {
  const settleUntil = Date.now() + 2_000;

  while (Date.now() < settleUntil) {
    const diagnostics = vscode.languages.getDiagnostics(uri);
    assert.deepEqual(diagnostics, [], `${label} must stay diagnostic-free`);
    await sleep(100);
  }
}

function positionAfter(document, needle, prefix) {
  const offset = document.getText().indexOf(needle);
  assert.notEqual(offset, -1, `fixture must contain ${JSON.stringify(needle)}`);
  return document.positionAt(offset + prefix.length);
}

function getWorkspaceFolderPath() {
  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  assert.ok(workspaceFolder, "expected a workspace folder");
  return workspaceFolder.uri.fsPath;
}

async function openWorkspaceDocument(...segments) {
  return vscode.workspace.openTextDocument(
    vscode.Uri.file(path.join(getWorkspaceFolderPath(), ...segments)),
  );
}

async function waitForDiagnostics(uri, predicate, label, timeoutMs) {
  const timeoutAt = Date.now() + timeoutMs;
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

async function waitFor(produce, predicate, label, timeoutMs) {
  const timeoutAt = Date.now() + timeoutMs;
  let value;

  while (Date.now() < timeoutAt) {
    value = await produce();
    if (predicate(value)) {
      return value;
    }

    await sleep(100);
  }

  assert.fail(`${label} did not happen. Last value: ${JSON.stringify(value)}`);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// The `describe*` helpers below enumerate every field of the corresponding
// `vscode` API type, so a `deepEqual` against their output is a complete
// assertion on the provider result rather than a spot check. They exist only
// because `vscode.WorkspaceEdit` and `vscode.SemanticTokens` carry private
// state that cannot be reconstructed for a direct instance comparison.

function describeRange(range) {
  return [range.start.line, range.start.character, range.end.line, range.end.character];
}

function describeDiagnostic(diagnostic) {
  const code = diagnostic.code;
  return {
    code:
      code !== null && typeof code === "object"
        ? { target: code.target.toString(), value: code.value }
        : code,
    message: diagnostic.message,
    range: describeRange(diagnostic.range),
    relatedInformation: diagnostic.relatedInformation,
    severity: diagnostic.severity,
    source: diagnostic.source,
    tags: diagnostic.tags,
  };
}

function describeTextEdit(edit) {
  return { newEol: edit.newEol, newText: edit.newText, range: describeRange(edit.range) };
}

function describeWorkspaceEdit(edit) {
  if (edit === undefined || edit === null) {
    return edit;
  }

  return {
    entries: edit
      .entries()
      .map(([uri, edits]) => [uri.toString(), edits.map((textEdit) => describeTextEdit(textEdit))]),
    size: edit.size,
  };
}

function describeCodeAction(action) {
  return {
    command: action.command,
    diagnostics: action.diagnostics,
    disabled: action.disabled,
    edit: describeWorkspaceEdit(action.edit),
    isPreferred: action.isPreferred,
    kind: action.kind?.value,
    title: action.title,
  };
}

function describeSemanticTokens(tokens) {
  if (tokens === undefined || tokens === null) {
    return tokens;
  }

  return { data: Array.from(tokens.data), resultId: tokens.resultId };
}

module.exports = {
  assertPackagedExtension,
  assertStaysDiagnosticFree,
  describeCodeAction,
  describeDiagnostic,
  describeRange,
  describeSemanticTokens,
  describeTextEdit,
  describeWorkspaceEdit,
  featureSettingKeys,
  getRealServer,
  getWorkspaceFolderPath,
  openWorkspaceDocument,
  positionAfter,
  prepareConfiguredRealServer,
  sleep,
  updateVizeConfiguration,
  waitFor,
  waitForDiagnostics,
};
