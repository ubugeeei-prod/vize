const assert = require("node:assert/strict");
const vscode = require("vscode");
const { openWorkspaceDocument, waitForDiagnostics } = require("./real-server-support.cjs");

function assertReadableDocumentation(value) {
  assert.match(value, /```typescript\n/);
  assert.match(value, /formatTotal\(amount: number, currency\?:/);
  assert.match(value, /Format the total shown on an invoice/);
  assert.match(value, /\*\*JPY\*\*/);
  assert.match(value, /Amount before currency formatting/);
  assert.match(value, /A localized invoice total/);
  assert.match(value, /```(?:ts|typescript|tsx)\ninvoice\.formatTotal\(1200, 'JPY'\)/);
}

exports.runRichAuthoring = async function runRichAuthoring() {
  const document = await openWorkspaceDocument("src", "RichAuthoring.vue");
  const editor = await vscode.window.showTextDocument(document);
  await waitForDiagnostics(
    document.uri,
    (items) => items.length === 0,
    "rich authoring initial document",
  );
  const original = document.getText();
  const authoredCall = "invoice.formatTotal(1200)";
  const callStart = original.lastIndexOf(authoredCall);
  assert.ok(callStart >= 0);
  const hoverPosition = document.positionAt(callStart + "invoice.for".length);
  const hovers = await vscode.commands.executeCommand(
    "vscode.executeHoverProvider",
    document.uri,
    hoverPosition,
  );
  const documentedHover = hovers?.find((hover) =>
    hover.contents.some((content) => content.value?.includes("Format the total")),
  );
  assert.ok(documentedHover, "the packaged provider must expose authored JSDoc");
  assertReadableDocumentation(
    documentedHover.contents.map((content) => content.value).join("\n\n"),
  );
  assert.deepEqual(
    documentedHover.range,
    new vscode.Range(document.positionAt(callStart + 8), document.positionAt(callStart + 19)),
  );
  const definitions = await vscode.commands.executeCommand(
    "vscode.executeDefinitionProvider",
    document.uri,
    hoverPosition,
  );
  const target = definitions?.[0];
  assert.ok(target, "the documented method must navigate to its authored declaration");
  const declaration = original.indexOf("formatTotal(amount");
  assert.equal((target.uri ?? target.targetUri).toString(), document.uri.toString());
  assert.deepEqual(
    target.range ?? target.targetSelectionRange,
    new vscode.Range(document.positionAt(declaration), document.positionAt(declaration + 11)),
  );

  const incomplete = "invoice.for";
  assert.equal(
    await editor.edit((edit) =>
      edit.replace(
        new vscode.Range(
          document.positionAt(callStart),
          document.positionAt(callStart + authoredCall.length),
        ),
        incomplete,
      ),
    ),
    true,
  );
  await waitForDiagnostics(
    document.uri,
    (items) => items.some((item) => Number(item.code) === 2339),
    "incomplete member diagnostic",
  );
  const completions = await vscode.commands.executeCommand(
    "vscode.executeCompletionItemProvider",
    document.uri,
    document.positionAt(callStart + incomplete.length),
    undefined,
    1,
  );
  const method = completions?.items.find(
    (item) => (typeof item.label === "string" ? item.label : item.label.label) === "formatTotal",
  );
  assert.ok(method, "the candidate must remain useful while its name is incomplete");
  assert.equal(method.kind, vscode.CompletionItemKind.Method);
  assert.ok(
    method.documentation instanceof vscode.MarkdownString,
    "VS Code must receive renderable Markdown rather than literal markup",
  );
  assertReadableDocumentation(method.documentation.value);
  assert.equal(method.documentation.isTrusted ?? false, false);

  assert.equal(
    await editor.edit((edit) =>
      edit.replace(
        new vscode.Range(
          document.positionAt(callStart),
          document.positionAt(callStart + incomplete.length),
        ),
        authoredCall,
      ),
    ),
    true,
  );
  await waitForDiagnostics(document.uri, (items) => items.length === 0, "rich authoring repair");
};
