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
    30_000,
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
    30_000,
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
  await waitForDiagnostics(
    document.uri,
    (items) => items.length === 0,
    "rich authoring repair",
    30_000,
  );
  await assertComponentDocumentation(document, editor);
  await assertSlotDocumentation();
};

async function assertComponentDocumentation(document, editor) {
  const source = document.getText();
  const heading = source.indexOf('heading="April"');
  const hovers = await vscode.commands.executeCommand(
    "vscode.executeHoverProvider",
    document.uri,
    document.positionAt(heading + 2),
  );
  const hover = hovers?.find((item) =>
    item.contents.some((content) =>
      content.value?.includes("Heading displayed above the invoice total"),
    ),
  );
  assert.ok(hover, "component prop hover must expose the child's authored documentation");
  assert.match(hover.contents.map((content) => content.value).join("\n"), /```typescript\n/);
  assert.deepEqual(
    hover.range,
    new vscode.Range(document.positionAt(heading), document.positionAt(heading + 7)),
  );

  const attribute = 'tone="muted"';
  const start = source.indexOf(attribute);
  const range = new vscode.Range(
    document.positionAt(start),
    document.positionAt(start + attribute.length),
  );
  assert.equal(await editor.edit((edit) => edit.replace(range, "to")), true);
  const completions = await vscode.commands.executeCommand(
    "vscode.executeCompletionItemProvider",
    document.uri,
    document.positionAt(start + 2),
    undefined,
    1,
  );
  const tone = completions?.items.find(
    (item) => (typeof item.label === "string" ? item.label : item.label.label) === "tone",
  );
  assert.ok(tone?.documentation instanceof vscode.MarkdownString);
  assert.match(tone.documentation.value, /```typescript\n/);
  assert.match(tone.documentation.value, /\*\*Visual emphasis\*\* for overdue invoices/);
  assert.match(tone.documentation.value, /muted.*strong|strong.*muted/);
  assert.equal(
    await editor.edit((edit) =>
      edit.replace(
        new vscode.Range(document.positionAt(start), document.positionAt(start + 2)),
        attribute,
      ),
    ),
    true,
  );
  await waitForDiagnostics(
    document.uri,
    (items) => items.length === 0,
    "component prop repair",
    30_000,
  );
}

async function assertSlotDocumentation() {
  const document = await openWorkspaceDocument("src", "SlotAuthoring.vue");
  const editor = await vscode.window.showTextDocument(document);
  await waitForDiagnostics(
    document.uri,
    (items) => items.length === 0,
    "slot initial document",
    30_000,
  );
  for (const [expression, incomplete, label, description, declaration] of [
    ["names.current", "names.", "current", "**Primary** invoice slot", "current:"],
    ["invoice.total", "invoice.to", "total", "**Invoice** total", "total: number"],
    [
      "ownSlots.header",
      "ownSlots.",
      "header",
      "**Header** shown above the invoice",
      "header(props:",
    ],
    ["$slots.header", "$slots.he", "header", "**Header** shown above the invoice", "header(props:"],
  ]) {
    const source = document.getText();
    const start = source.lastIndexOf(expression);
    assert.ok(start >= 0);
    const position = document.positionAt(start + expression.indexOf(".") + 2);
    const hovers = await vscode.commands.executeCommand(
      "vscode.executeHoverProvider",
      document.uri,
      position,
    );
    const hover = hovers?.find((entry) =>
      entry.contents.some((content) => content.value?.includes(description)),
    );
    assert.ok(hover, `${expression} must expose authored Markdown`);
    assert.match(hover.contents.map((content) => content.value).join("\n"), /```typescript\n/);
    const definitions = await vscode.commands.executeCommand(
      "vscode.executeDefinitionProvider",
      document.uri,
      position,
    );
    const target = definitions?.[0];
    assert.ok(target);
    assert.equal((target.uri ?? target.targetUri).toString(), document.uri.toString());
    const declarationStart = source.indexOf(declaration);
    assert.deepEqual(
      target.range ?? target.targetSelectionRange,
      new vscode.Range(
        document.positionAt(declarationStart),
        document.positionAt(declarationStart + label.length),
      ),
    );
    assert.equal(
      await editor.edit((edit) =>
        edit.replace(
          new vscode.Range(
            document.positionAt(start),
            document.positionAt(start + expression.length),
          ),
          incomplete,
        ),
      ),
      true,
    );
    const completions = await vscode.commands.executeCommand(
      "vscode.executeCompletionItemProvider",
      document.uri,
      document.positionAt(start + incomplete.length),
      undefined,
      1,
    );
    const candidate = completions?.items.find(
      (item) => (typeof item.label === "string" ? item.label : item.label.label) === label,
    );
    assert.ok(candidate?.documentation instanceof vscode.MarkdownString);
    assert.match(candidate.documentation.value, /```typescript\n/);
    assert.ok(candidate.documentation.value.includes(description));
    assert.equal(
      await editor.edit((edit) =>
        edit.replace(
          new vscode.Range(
            document.positionAt(start),
            document.positionAt(start + incomplete.length),
          ),
          expression,
        ),
      ),
      true,
    );
    await waitForDiagnostics(
      document.uri,
      (items) => items.length === 0,
      `slot repair ${expression}`,
      30_000,
    );
  }
}
