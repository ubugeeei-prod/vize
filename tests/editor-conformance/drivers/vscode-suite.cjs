// TS-45 VS Code driver, loaded by the extension host as --extensionTestsPath.
// The workspace settings (written by `vscode.ts` before launch) are what the
// recommended profile writes plus the formatting opt-in; the suite plays the
// scenario through VS Code's provider commands, so vscode-languageclient
// builds every request and every edit reaches the server through VS Code's
// own document sync.
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");

const scenario = JSON.parse(fs.readFileSync(process.env.VIZE_TS45_SCENARIO, "utf8"));
const transcript = process.env.VIZE_CONFORMANCE_TRANSCRIPT;
const timeoutMs = 180_000;

function step(id) {
  const found = scenario.steps.find((candidate) => candidate.id === id);
  if (found == null) throw new Error(`unknown scenario step ${id}`);
  return found;
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const position = ({ line, character }) => new vscode.Position(line, character);
const range = ({ start, end }) => new vscode.Range(position(start), position(end));

async function waitUntil(label, predicate) {
  const deadline = Date.now() + timeoutMs;
  while (!(await predicate())) {
    if (Date.now() > deadline) throw new Error(`timed out waiting for ${label}`);
    await sleep(100);
  }
}

/** Pacing only: the diagnostics count VS Code shows, stable for one second. */
async function settle(uri, count) {
  let since = Date.now();
  let last = null;
  await waitUntil(`${count} diagnostics`, () => {
    const current = JSON.stringify(vscode.languages.getDiagnostics(uri));
    if (current !== last) {
      last = current;
      since = Date.now();
    }
    return vscode.languages.getDiagnostics(uri).length === count && Date.now() - since >= 1000;
  });
}

async function apply(uri, edits) {
  const edit = new vscode.WorkspaceEdit();
  edit.set(uri, edits);
  if (!(await vscode.workspace.applyEdit(edit))) throw new Error("VS Code rejected the edit");
}

exports.run = async function run() {
  const settings = vscode.workspace.getConfiguration("vize");
  for (const [key, value] of [
    ["enable", true],
    ["formatting.enable", true],
    ["serverPath", process.env.VIZE_TS45_SERVER],
  ]) {
    if (settings.get(key) !== value)
      throw new Error(`workspace setting vize.${key} is not ${value}`);
  }
  await vscode.extensions.getExtension("ubugeeei.vize").activate();

  const folder = vscode.workspace.workspaceFolders[0].uri.fsPath;
  const document = await vscode.workspace.openTextDocument(path.join(folder, scenario.document));
  const editor = await vscode.window.showTextDocument(document);
  const uri = document.uri;
  await settle(uri, step("diagnostics-open").expect.length);

  await vscode.commands.executeCommand(
    "vscode.executeHoverProvider",
    uri,
    position(step("hover").match.position),
  );
  await vscode.commands.executeCommand(
    "vscode.executeCompletionItemProvider",
    uri,
    position(step("completion").match.position),
  );
  await vscode.commands.executeCommand(
    "vscode.executeDefinitionProvider",
    uri,
    position(step("definition").match.position),
  );

  const actions = await vscode.commands.executeCommand(
    "vscode.executeCodeActionProvider",
    uri,
    range(step("code-action").match.range),
  );
  const fix = actions.find((action) => action.title === step("apply-quick-fix").action.title);
  if (fix?.edit == null)
    throw new Error(`the quick fix was not offered: ${JSON.stringify(actions)}`);
  if (!(await vscode.workspace.applyEdit(fix.edit)))
    throw new Error("VS Code rejected the quick fix");
  await settle(uri, step("diagnostics-quick-fix").expect.length);

  const formatting = await vscode.commands.executeCommand(
    "vscode.executeFormatDocumentProvider",
    uri,
    {
      insertSpaces: step("formatting").match.options.insertSpaces,
      tabSize: step("formatting").match.options.tabSize,
    },
  );
  await apply(uri, formatting);

  const typed = step("edit").action;
  await editor.edit((builder) => builder.replace(range(typed.replace), typed.text));
  await settle(uri, step("diagnostics-edit").expect.length);

  const rename = step("rename").match;
  const renamed = await vscode.commands.executeCommand(
    "vscode.executeDocumentRenameProvider",
    uri,
    position(rename.position),
    rename.newName,
  );
  if (!(await vscode.workspace.applyEdit(renamed))) throw new Error("VS Code rejected the rename");
  await waitUntil("the renamed document", () => document.getText() === step("apply-rename").expect);
  await sleep(500);

  // `vize.disable` stops the language client: shutdown, exit, process exit.
  // (Earlier exit records belong to the extension's `vize --version` probes.)
  const recorded = () => fs.readFileSync(transcript, "utf8").split("\n");
  const before = recorded().length;
  await vscode.commands.executeCommand("vize.disable");
  await waitUntil("the server to exit", () =>
    recorded()
      .slice(before - 1)
      .some((line) => line.includes('"event":"exit"')),
  );
};
