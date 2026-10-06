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
  const output = path.join(getWorkspaceFolderPath(), "node_modules/.vize/rich-hover.json");
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, `${JSON.stringify({ schemaVersion: 1, packets }, null, 2)}\n`);
}

module.exports = { beginHover, retainHover };
