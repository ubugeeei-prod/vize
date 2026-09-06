import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const require = createRequire(import.meta.url);

type ExtensionHostFixtures = {
  granularEditorCapabilitySettings: Array<[string, string]>;
};

type HostProviderSmokeContract = {
  command: string;
  method: string;
  setting: string;
};

const hostProviderSmokeContracts: Record<string, HostProviderSmokeContract> = {
  codeActions: {
    command: "vscode.executeCodeActionProvider",
    method: "textDocument/codeAction",
    setting: "codeActions.enable",
  },
  codeLens: {
    command: "vscode.executeCodeLensProvider",
    method: "textDocument/codeLens",
    setting: "codeLens.enable",
  },
  completion: {
    command: "vscode.executeCompletionItemProvider",
    method: "textDocument/completion",
    setting: "completion.enable",
  },
  definition: {
    command: "vscode.executeDefinitionProvider",
    method: "textDocument/definition",
    setting: "definition.enable",
  },
  documentLinks: {
    command: "vscode.executeLinkProvider",
    method: "textDocument/documentLink",
    setting: "documentLinks.enable",
  },
  documentSymbols: {
    command: "vscode.executeDocumentSymbolProvider",
    method: "textDocument/documentSymbol",
    setting: "documentSymbols.enable",
  },
  foldingRanges: {
    command: "vscode.executeFoldingRangeProvider",
    method: "textDocument/foldingRange",
    setting: "foldingRanges.enable",
  },
  formatting: {
    command: "vscode.executeFormatDocumentProvider",
    method: "textDocument/formatting",
    setting: "formatting.enable",
  },
  hover: {
    command: "vscode.executeHoverProvider",
    method: "textDocument/hover",
    setting: "hover.enable",
  },
  inlayHints: {
    command: "vscode.executeInlayHintProvider",
    method: "textDocument/inlayHint",
    setting: "inlayHints.enable",
  },
  references: {
    command: "vscode.executeReferenceProvider",
    method: "textDocument/references",
    setting: "references.enable",
  },
  rename: {
    command: "vscode.executeDocumentRenameProvider",
    method: "textDocument/rename",
    setting: "rename.enable",
  },
  semanticTokens: {
    command: "vscode.provideDocumentSemanticTokens",
    method: "textDocument/semanticTokens/full",
    setting: "semanticTokens.enable",
  },
  signatureHelp: {
    command: "vscode.executeSignatureHelpProvider",
    method: "textDocument/signatureHelp",
    setting: "signatureHelp.enable",
  },
  workspaceSymbols: {
    command: "vscode.executeWorkspaceSymbolProvider",
    method: "workspace/symbol",
    setting: "workspaceSymbols.enable",
  },
};

const intentionallyNonCommandProviderOptions = new Set(["fileRename"]);

test("VS Code host provider smoke covers every command-addressable editor switch", () => {
  const fixtures = require(
    path.join(root, "editors/vscode/test/suite/extension-host-fixtures.cjs"),
  ) as ExtensionHostFixtures;
  const configured = new Map(
    fixtures.granularEditorCapabilitySettings.map(([setting, option]) => [option, setting]),
  );
  const uncovered = [...configured.keys()].filter(
    (option) =>
      !Object.hasOwn(hostProviderSmokeContracts, option) &&
      !intentionallyNonCommandProviderOptions.has(option),
  );

  assert.deepEqual(uncovered, []);
  assert.ok(configured.has("fileRename"), "file rename remains covered by real-server LSP suites");

  const smoke = readRepoFile("editors/vscode/test/suite/editor-capability-smoke.cjs");
  const fakeServer = readRepoFile("editors/vscode/test/fixtures/fake-vize-server.cjs");
  for (const [option, contract] of Object.entries(hostProviderSmokeContracts)) {
    assert.equal(configured.get(option), contract.setting, `${option} fixture setting drifted`);
    assert.ok(
      smoke.includes(contract.command),
      `${option} host smoke must execute VS Code command`,
    );
    assert.ok(smoke.includes(contract.method), `${option} host smoke must request LSP method`);
    assert.ok(fakeServer.includes(contract.method), `${option} fake server must answer LSP method`);
  }
});

function readRepoFile(relativePath: string): string {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}
