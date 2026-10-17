const crypto = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");
const { getWorkspaceFolderPath } = require("./real-server-support.cjs");

const packets = [];
const completionPackets = [];
let requestOrder = 0;

function beginHover(document, position, expected) {
  const source = document.getText();
  return {
    order: ++requestOrder,
    startedAt: new Date().toISOString(),
    source,
    sourceSha256: crypto.createHash("sha256").update(source).digest("hex"),
    document: {
      uri: document.uri.toString(),
      version: document.version,
      languageId: document.languageId,
    },
    provider: {
      serverPath: process.env.VIZE_TEST_SERVER_PATH,
      packagedExtensionsPath: process.env.VIZE_TEST_PACKAGED_EXTENSIONS_DIR,
      sourceExtensionPath: process.env.VIZE_TEST_SOURCE_EXTENSION_PATH,
      githubSha: process.env.GITHUB_SHA,
    },
    request: {
      method: "vscode.executeHoverProvider",
      position: { line: position.line, character: position.character },
    },
    expected,
  };
}

function retainHover(request, hovers) {
  packets.push({
    ...request,
    resultType: hovers === null ? "null" : Array.isArray(hovers) ? "array" : typeof hovers,
    actual: hovers,
    publicContents: Array.isArray(hovers)
      ? hovers.map((hover) => ({
          range: hover?.range,
          contents: Array.isArray(hover?.contents)
            ? hover.contents.map((content) => ({
                raw: content,
                language: content?.language,
                value: content?.value,
                isTrusted: content?.isTrusted,
                supportThemeIcons: content?.supportThemeIcons,
                supportHtml: content?.supportHtml,
                baseUri: content?.baseUri?.toString(),
              }))
            : hover?.contents,
        }))
      : undefined,
  });
  const output = path.join(getWorkspaceFolderPath(), "node_modules/.vize/rich-hover.json");
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, `${JSON.stringify({ schemaVersion: 1, packets }, null, 2)}\n`);
}

function beginCompletion(document, position, expected, maxItemsToResolve) {
  const request = beginHover(document, position, expected);
  request.request.method = "vscode.executeCompletionItemProvider";
  request.request.maxItemsToResolve = maxItemsToResolve;
  return request;
}

function retainCompletion(request, completions) {
  const label = (item) => (typeof item.label === "string" ? item.label : item.label.label);
  const selectedIndex = completions?.items.findIndex(
    (item) => label(item) === request.expected.label,
  );
  const packet = {
    ...request,
    receivedAt: new Date().toISOString(),
    resultType: completions === null ? "null" : typeof completions,
    actual: completions,
    selectedIndex,
    publicItems: completions?.items.map((item, index) => ({
      index,
      ...item,
      label: item.label,
      kind: item.kind,
      detail: item.detail,
      data: item.data,
      documentation: {
        raw: item.documentation,
        type: typeof item.documentation,
        constructor: item.documentation?.constructor?.name,
        isMarkdownString: item.documentation instanceof vscode.MarkdownString,
        value: item.documentation?.value,
        isTrusted: item.documentation?.isTrusted,
        supportHtml: item.documentation?.supportHtml,
        supportThemeIcons: item.documentation?.supportThemeIcons,
        baseUri: item.documentation?.baseUri?.toString(),
      },
    })),
  };
  completionPackets.push(packet);
  const output = path.join(getWorkspaceFolderPath(), "node_modules/.vize/rich-completion.json");
  // This only records the first public result; the caller asserts the same
  // original item without issuing any diagnostic or replacement request.
  const bytes = `${JSON.stringify({ schemaVersion: 1, packets: completionPackets }, null, 2)}\n`;
  for (const destination of [output, process.env.VIZE_TEST_COMPLETION_EVIDENCE].filter(Boolean)) {
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, bytes);
  }
}

module.exports = { beginHover, retainHover, beginCompletion, retainCompletion };
