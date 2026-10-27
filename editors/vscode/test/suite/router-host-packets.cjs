const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { createHash } = require("node:crypto");
const { createRequire } = require("node:module");
const vscode = require("vscode");
const { describeDiagnostic, describeRange } = require("./real-server-support.cjs");
const { waitFor } = require("./wait-for.cjs");
const { captureCompilerDocument } = require("./router-host-compiler-capture.cjs");
const {
  diagnostics,
  plainArt,
  pos,
  publicDiagnostics,
  range,
} = require("./router-host-diagnostics.cjs");

const output = process.env.VIZE_TEST_ROUTER_OUTPUT;
function retain(name, value) {
  fs.writeFileSync(path.join(output, `${name}.json`), `${JSON.stringify(value, null, 2)}\n`);
}
function records(file = process.env.VIZE_TEST_CONFIG_PROTOCOL_PATH) {
  if (!fs.existsSync(file)) return [];
  const source = fs.readFileSync(file, "utf8");
  return source
    .slice(0, source.lastIndexOf("\n") + 1)
    .split("\n")
    .filter(Boolean)
    .map(JSON.parse);
}
const frames = () => records().filter((record) => record.kind === "frame");
async function publication(document, after = 0) {
  return waitFor(
    () =>
      frames()
        .slice(after)
        .find(
          ({ direction, packet }) =>
            direction === "server" &&
            packet.method === "textDocument/publishDiagnostics" &&
            packet.params.uri === document.uri.toString() &&
            packet.params.version === document.version,
        ),
    Boolean,
    `native diagnostic publication ${document.uri}`,
    60_000,
  );
}

function documentRevision(uri) {
  let sequence = 0;
  let revision;
  for (const { direction, packet } of frames()) {
    if (direction !== "client") continue;
    if (packet.method === "textDocument/didOpen") {
      sequence++;
      if (packet.params.textDocument.uri === uri) revision = sequence;
    } else if (packet.method === "textDocument/didChange") {
      sequence += packet.params.contentChanges.length;
      if (packet.params.textDocument.uri === uri) revision = sequence;
    }
  }
  assert.ok(Number.isSafeInteger(revision));
  return revision;
}

const markdown = (value) =>
  value == null
    ? null
    : {
        value: value.value,
        isTrusted: value.isTrusted ?? null,
        supportThemeIcons: value.supportThemeIcons ?? null,
        supportHtml: value.supportHtml ?? null,
        baseUri: value.baseUri?.toString() ?? null,
      };
const publicLocation = (location) => ({
  uri: location.uri.toString(),
  range: describeRange(location.range),
});
function expectedMarkdown(value) {
  const result = new vscode.MarkdownString(value);
  result.isTrusted = false;
  result.supportHtml = false;
  return result;
}
function publicCompletion(item) {
  return Object.fromEntries(
    [
      "additionalTextEdits",
      "command",
      "commitCharacters",
      "detail",
      "filterText",
      "insertText",
      "keepWhitespace",
      "kind",
      "label",
      "preselect",
      "sortText",
      "tags",
      "textEdit",
    ]
      .map((key) => [key, item[key] ?? null])
      .concat([
        [
          "documentation",
          typeof item.documentation === "string"
            ? item.documentation
            : markdown(item.documentation),
        ],
        [
          "range",
          item.range
            ? item.range.inserting
              ? {
                  inserting: describeRange(item.range.inserting),
                  replacing: describeRange(item.range.replacing),
                }
              : describeRange(item.range)
            : null,
        ],
      ]),
  );
}

async function capture(document, label) {
  const source = document.getText();
  const offset = source.indexOf("route.params.page") + "route.params.".length;
  assert.ok(offset >= "route.params.".length);
  const position = pos(document, offset + 1);
  const uri = document.uri.toString();
  const result = {
    label,
    uri,
    source,
    version: document.version,
    native: {},
    public: {},
    timings: [],
  };
  const save = () => retain(label, result);
  const invoke = async (command, ...args) => {
    const started = performance.now();
    try {
      return await vscode.commands.executeCommand(command, ...args);
    } finally {
      result.timings.push({
        command,
        method: args[0]?.method ?? null,
        elapsedMs: performance.now() - started,
      });
      save();
    }
  };
  const request = async (name, method, params) => {
    result.native[name] = await invoke("vize.test.executeLspRequest", {
      method,
      params,
    });
    save();
  };
  try {
    await request("hover", "textDocument/hover", { textDocument: { uri }, position });
    await request("completion", "textDocument/completion", {
      textDocument: { uri },
      position: pos(document, offset),
    });
    const candidate = result.native.completion?.[0]?.data?.vizeCompletion?.item?.data?.fileName;
    if (candidate && fs.existsSync(candidate)) {
      const bytes = fs.readFileSync(candidate);
      result.generated = {
        file: candidate,
        bytes: bytes.length,
        sha256: createHash("sha256").update(bytes).digest("hex"),
        source: bytes.toString("utf8"),
      };
      save();
    } else if (candidate) {
      result.generated = captureCompilerDocument(
        records(path.join(output, "compiler-protocol.jsonl")),
        result.native.completion,
      );
      save();
    }
    result.native.resolved = [];
    for (const item of result.native.completion ?? []) {
      result.native.resolved.push(
        await invoke("vize.test.executeLspRequest", {
          method: "completionItem/resolve",
          params: item,
        }),
      );
      save();
    }
    await request("definition", "textDocument/definition", { textDocument: { uri }, position });
    await request("references", "textDocument/references", {
      textDocument: { uri },
      position,
      context: { includeDeclaration: true },
    });
    result.public.hover = (await invoke("vscode.executeHoverProvider", document.uri, position)).map(
      (hover) => ({
        contents: hover.contents.map(markdown),
        range: hover.range ? describeRange(hover.range) : null,
      }),
    );
    save();
    const completions = await invoke(
      "vscode.executeCompletionItemProvider",
      document.uri,
      pos(document, offset),
      undefined,
      5,
    );
    result.public.completion = {
      isIncomplete: completions.isIncomplete,
      items: completions.items.map(publicCompletion),
    };
    save();
    result.public.definition = (
      await invoke("vscode.executeDefinitionProvider", document.uri, position)
    ).map(publicLocation);
    save();
    result.public.references = (
      await invoke("vscode.executeReferenceProvider", document.uri, position)
    ).map(publicLocation);
    result.public.diagnostics = vscode.languages
      .getDiagnostics(document.uri)
      .map(describeDiagnostic);
    result.documentRevision = documentRevision(uri);
    return result;
  } finally {
    save();
  }
}

function syntaxLocations(project, document, generated) {
  const requireProject = createRequire(path.join(project.root, "package.json"));
  const ts = requireProject("typescript");
  assert.equal(ts.version, "5.9.3");
  const declaration = path.join(project.root, "typed-router.d.ts");
  const tree = ts.createSourceFile(
    declaration,
    fs.readFileSync(declaration, "utf8"),
    ts.ScriptTarget.Latest,
    true,
  );
  const routeName = document.getText().match(/useRoute\('([^']+)'\)/u)[1];
  const properties = [];
  function visit(node) {
    if (ts.isPropertySignature(node) && node.name?.text === routeName) {
      const member = node.type.typeArguments[3].members.find((item) => item.name.text === "page");
      properties.push(member.name);
    }
    ts.forEachChild(node, visit);
  }
  visit(tree);
  assert.equal(properties.length, 1);
  const node = properties[0];
  const point = (offset) => {
    const location = tree.getLineAndCharacterOfPosition(offset);
    return { line: location.line, character: location.character };
  };
  const definition = {
    uri: vscode.Uri.file(declaration).toString(),
    range: { start: point(node.getStart(tree)), end: point(node.end) },
  };
  const reference = {
    uri: definition.uri,
    range: { start: point(node.getStart(tree) + 1), end: point(node.end - 1) },
  };
  let privatePosition;
  if (generated) {
    const nativeTree = ts.createSourceFile(
      generated.file,
      generated.source,
      ts.ScriptTarget.Latest,
      true,
    );
    const candidates = [];
    function find(value) {
      if (
        ts.isVariableDeclaration(value) &&
        value.name.getText(nativeTree) === "page" &&
        value.initializer?.getText(nativeTree) === "route.params.page"
      )
        candidates.push(value.initializer.name.getStart(nativeTree));
      ts.forEachChild(value, find);
    }
    find(nativeTree);
    assert.equal(candidates.length, 1);
    privatePosition = Buffer.byteLength(generated.source.slice(0, candidates[0]));
    assert.equal(nativeTree.referencedFiles.length, document.languageId === "art-vue" ? 0 : 1);
    if (nativeTree.referencedFiles.length)
      assert.deepEqual(
        fs.readFileSync(
          ts.resolveTripleslashReference(nativeTree.referencedFiles[0].fileName, generated.file),
        ),
        fs.readFileSync(declaration),
      );
  }
  return { definition, reference, privatePosition };
}

function assertCapture(actual, project, document, enabled, invalid = false) {
  if (plainArt(document)) {
    assert.deepEqual(actual.native, {
      hover: null,
      completion: null,
      resolved: [],
      definition: null,
      references: null,
    });
    assert.deepEqual(actual.public, {
      hover: [],
      completion: { isIncomplete: false, items: [] },
      definition: [],
      references: [],
      diagnostics: publicDiagnostics(document, enabled, invalid),
    });
    return;
  }
  const source = document.getText();
  const property = source.indexOf("route.params.page") + "route.params.".length;
  const hover = {
    contents: { kind: "markdown", value: "```typescript\n(property) 'page': number\n```" },
    range: range(document, property, 4),
  };
  assert.deepEqual(actual.native.hover, hover, "complete typed script-property hover");
  const syntax = syntaxLocations(project, document, actual.generated);
  const references = [
    { uri: document.uri.toString(), range: range(document, property, 4) },
    {
      uri: document.uri.toString(),
      range: range(document, source.lastIndexOf("route.params.page") + "route.params.".length, 4),
    },
    syntax.reference,
  ];
  let completion = [
    { label: "id", kind: 10, detail: "Vue Router file route param: string", sortText: "0" },
  ];
  let resolved = completion;
  if (enabled || document.languageId === "art-vue") {
    assert.ok(actual.generated);
    const relative = path.relative(project.root, document.fileName).split(path.sep).join("/");
    const art = document.languageId === "art-vue";
    const expectedFile = art
      ? `${new URL(document.uri.toString()).pathname}.setup.ts`
      : actual.generated.file;
    if (art) assert.equal(actual.generated.file, expectedFile);
    else assert.ok(actual.generated.file.endsWith(`/${relative}.ts`));
    const virtualRoot = actual.generated.file.slice(0, -relative.length - 4);
    completion = ["at", "heading", "id", "page", "tags"].map((name) => ({
      data: {
        vizeCompletion: {
          item: {
            data: {
              fileName: expectedFile,
              name,
              position: syntax.privatePosition,
            },
            kind: 5,
            label: name,
            sortText: "11",
          },
          requestUri: vscode.Uri.file(
            art ? expectedFile : `${virtualRoot}/${relative}.ts`,
          ).toString(),
          revision: actual.documentRevision,
          uri: document.uri.toString(),
        },
      },
      kind: 5,
      label: name,
      sortText: "11",
    }));
    const types = {
      at: "Date | undefined",
      heading: "string | undefined",
      id: "number",
      page: "number",
      tags: "string[] | undefined",
    };
    resolved = completion.map((item) => ({
      ...item,
      detail: `(property) '${item.label}': ${types[item.label]}`,
      documentation: {
        kind: "markdown",
        value: `\`\`\`typescript\n(property) '${item.label}': ${types[item.label]}\n\`\`\``,
      },
    }));
  }
  assert.deepEqual(actual.native, {
    hover,
    completion,
    resolved,
    definition: syntax.definition,
    references,
  });
  const expectedPublicCompletion = resolved.map((item) => {
    const value = new vscode.CompletionItem(item.label, item.kind - 1);
    value.sortText = item.sortText;
    value.detail = item.detail;
    // Pinned host transforms: microsoft/vscode@1.107.1, CompletionsAdapter
    // defaultRanges and extHostTypeConverters.CompletionItem.to. The native
    // packet above remains unchanged; these are public command defaults.
    value.insertText = item.label;
    value.keepWhitespace = false;
    value.range = {
      inserting: new vscode.Range(document.positionAt(property), document.positionAt(property)),
      replacing: new vscode.Range(document.positionAt(property), document.positionAt(property + 4)),
    };
    if (item.documentation) value.documentation = expectedMarkdown(item.documentation.value);
    return publicCompletion(value);
  });
  const publicHover = {
    contents: [markdown(expectedMarkdown(hover.contents.value))],
    range: [
      hover.range.start.line,
      hover.range.start.character,
      hover.range.end.line,
      hover.range.end.character,
    ],
  };
  const location = (value) => ({
    uri: value.uri,
    range: [
      value.range.start.line,
      value.range.start.character,
      value.range.end.line,
      value.range.end.character,
    ],
  });
  const expectedDiagnostics = publicDiagnostics(document, enabled, invalid);
  const expectedPublic = {
    hover: [publicHover],
    completion: { isIncomplete: false, items: expectedPublicCompletion },
    definition: [location(syntax.definition)],
    references: references.map(location),
    diagnostics: expectedDiagnostics,
  };
  assert.deepEqual(actual.public, expectedPublic);
  for (const delta of [-1, 1]) {
    const changed = structuredClone(actual.public);
    changed.completion.items[0].range.replacing[3] += delta;
    assert.throws(() => assert.deepEqual(changed, expectedPublic), assert.AssertionError);
  }
  for (const [key, value] of [
    ["insertText", "foreign"],
    ["keepWhitespace", true],
  ]) {
    const changed = structuredClone(actual.public);
    changed.completion.items[0][key] = value;
    assert.throws(() => assert.deepEqual(changed, expectedPublic), assert.AssertionError);
  }
}

module.exports = {
  assertCapture,
  capture,
  diagnostics,
  frames,
  publication,
  publicCompletion,
  records,
  retain,
};
