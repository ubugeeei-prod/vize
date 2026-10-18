const crypto = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");
const { getWorkspaceFolderPath } = require("./real-server-support.cjs");

const packets = [];

function beginHover(document, position, expected) {
  const source = document.getText();
  return {
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
  writePackets();
}

function beginCompletion(document, position, expected, itemResolveCount) {
  const request = beginHover(document, position, expected);
  request.request.method = "vscode.executeCompletionItemProvider";
  request.request.itemResolveCount = itemResolveCount;
  return request;
}

function retainCompletion(request, completions) {
  // Preserve the complete real VSCode API result before checking any item.
  // This API resolves up to itemResolveCount entries; it is not a raw LSP tap.
  packets.push({
    ...request,
    resultType: completions === null ? "null" : typeof completions,
    actual: completions,
    publicDocumentation: completions?.items.map((item) => ({
      label: item.label,
      raw: item.documentation,
      constructor: item.documentation?.constructor?.name,
      // VSCode's MarkdownString stores its fields privately; JSON.stringify
      // alone produces {}. Read the public getters without rewriting them.
      language: item.documentation?.language,
      value: item.documentation?.value,
      isTrusted: item.documentation?.isTrusted,
      supportThemeIcons: item.documentation?.supportThemeIcons,
      supportHtml: item.documentation?.supportHtml,
      baseUri: item.documentation?.baseUri?.toString(),
    })),
  });
  writePackets();
}

function writePackets() {
  const output = path.join(getWorkspaceFolderPath(), "node_modules/.vize/rich-hover.json");
  fs.mkdirSync(path.dirname(output), { recursive: true });
  const bytes = `${JSON.stringify({ schemaVersion: 1, packets }, null, 2)}\n`;
  fs.writeFileSync(output, bytes);
  if (process.env.VIZE_RICH_CUSTODY_OUTPUT) {
    fs.writeFileSync(process.env.VIZE_RICH_CUSTODY_OUTPUT, bytes);
  }
}

module.exports = { beginHover, retainHover, beginCompletion, retainCompletion };
