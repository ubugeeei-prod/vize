const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { inspect } = require("node:util");
const vscode = require("vscode");
const { assertPackagedExtension, describeRange, waitFor } = require("./real-server-support.cjs");

exports.run = async function run() {
  const { fixtureSource, hoverCases, vscodeVersion } =
    await import("../css-hover-visual-contract.mjs");
  const { currentHoverPackets, currentDocumentColors, currentColorPresentations } =
    await import("../css-hover-current-packets.mjs");
  const destination = process.env.VIZE_CSS_VISUAL_OUTPUT;
  const packets = [];
  let document;
  const retain = (name, value) =>
    fs.writeFileSync(path.join(destination, name), `${JSON.stringify(value, null, 2)}\n`);
  const request = async (method, ...args) => {
    const start = performance.now();
    const actual = await vscode.commands.executeCommand(method, ...args);
    const packet = { method, args, actual, elapsedMs: performance.now() - start };
    packets.push(packet);
    fs.writeFileSync(
      path.join(destination, "api-inspect.txt"),
      inspect(packets, { depth: null, getters: true, maxArrayLength: null }),
    );
    retain("api-packets.json", packets);
    return actual;
  };
  const phase = async (id, position) => {
    const editor = await vscode.window.showTextDocument(document);
    editor.selection = new vscode.Selection(position, position);
    editor.revealRange(new vscode.Range(position, position), vscode.TextEditorRevealType.InCenter);
    // Ordinary keyboard hover uses ColorData.provider for concrete colors;
    // never invoke the standalone picker backed by VS Code's default provider.
    await vscode.commands.executeCommand("editor.action.showHover");
    retain("phase.json", {
      id,
      document: document.uri.toString(),
      position,
      version: document.version,
    });
    await waitFor(
      () => fs.existsSync(path.join(destination, `${id}.ack`)),
      Boolean,
      `${id} DOM capture`,
      60_000,
    );
  };

  try {
    assert.equal(vscode.version, vscodeVersion);
    const extension = vscode.extensions.getExtension("ubugeeei.vize");
    assert.ok(extension);
    assertPackagedExtension(extension);
    await extension.activate();
    document = await vscode.workspace.openTextDocument(
      path.join(vscode.workspace.workspaceFolders[0].uri.fsPath, "Colors.vue"),
    );
    assert.equal(document.getText(), fixtureSource);
    await vscode.window.showTextDocument(document);
    const server = await waitFor(
      () => vscode.commands.executeCommand("vize.test.getServerInfo").catch(() => undefined),
      (info) => info?.status === "ready",
      "configured real server ready",
      120_000,
    );
    assert.equal(fs.realpathSync(server.path), fs.realpathSync(process.env.VIZE_TEST_SERVER_PATH));
    assert.equal(server.source, "configured");
    retain("host-identity.json", {
      vscodeVersion: vscode.version,
      server,
      extensionPath: extension.extensionPath,
      extensionVersion: extension.packageJSON.version,
      extensions: vscode.extensions.all.map((item) => ({
        id: item.id,
        extensionPath: item.extensionPath,
        isActive: item.isActive,
      })),
      effectiveSettings: {
        defaultColorDecorators: vscode.workspace
          .getConfiguration("editor", document)
          .get("defaultColorDecorators"),
        colorDecorators: vscode.workspace
          .getConfiguration("editor", document)
          .get("colorDecorators"),
      },
    });
    assert.equal(
      vscode.workspace.getConfiguration("editor", document).get("defaultColorDecorators"),
      "never",
    );
    await waitFor(
      () => fs.existsSync(path.join(destination, "renderer-ready")),
      Boolean,
      "offline renderer capture ready",
      60_000,
    );
    const firstPosition = document.positionAt(fixtureSource.indexOf("display") + 3);
    await waitFor(
      async () => {
        const hover = await vscode.commands.executeCommand(
          "vscode.executeHoverProvider",
          document.uri,
          firstPosition,
        );
        return hover?.some((item) =>
          item.contents.some((content) => content.value?.includes("CSS property")),
        );
      },
      Boolean,
      "real CSS hover provider registered",
      120_000,
    );

    for (const expected of hoverCases) {
      const position = document.positionAt(
        fixtureSource.indexOf(expected.needle) + expected.offset,
      );
      const hovers = await request("vscode.executeHoverProvider", document.uri, position);
      const expanded = hovers?.map((hover) => ({
        range: hover.range ? describeRange(hover.range) : null,
        contents: hover.contents.map((content) => ({
          constructor: content.constructor.name,
          language: content.language,
          value: content.value,
          isTrusted: content.isTrusted,
          supportThemeIcons: content.supportThemeIcons,
          supportHtml: content.supportHtml,
          baseUri: content.baseUri?.toString(),
        })),
      }));
      const raw = await request("vize.test.executeLspRequest", {
        method: "textDocument/hover",
        params: { textDocument: { uri: document.uri.toString() }, position },
      });
      retain(`${expected.id}-hover.json`, { expected, position, hovers: expanded, raw });
      if (expected.baselineNull) {
        // Immutable baseline expectation. Product acceptance gets its own contract.
        assert.equal(raw, null);
        assert.deepEqual(hovers, []);
      } else {
        const expectedPacket = currentHoverPackets[expected.id];
        assert.deepEqual(raw, expectedPacket, "complete current native hover packet");
        assert.deepEqual(
          expanded.map((hover) => ({
            range: hover.range,
            contents: hover.contents.map(({ constructor: _constructor, ...fields }) => fields),
          })),
          [
            {
              range: [
                expectedPacket.range.start.line,
                expectedPacket.range.start.character,
                expectedPacket.range.end.line,
                expectedPacket.range.end.character,
              ],
              contents: [
                {
                  language: undefined,
                  value: expectedPacket.contents.value,
                  isTrusted: false,
                  supportThemeIcons: false,
                  supportHtml: false,
                  baseUri: undefined,
                },
              ],
            },
          ],
          "complete public MarkdownString getters and authored range",
        );
        assert.ok(
          expanded.some((hover) =>
            hover.contents.some(
              (content) =>
                content.value.includes(expected.markdown) && content.value.includes("```css"),
            ),
          ),
        );
      }
    }

    const colors = await request("vscode.executeDocumentColorProvider", document.uri);
    const publicColors = colors.map((entry) => ({
      range: describeRange(entry.range),
      color: {
        red: entry.color.red,
        green: entry.color.green,
        blue: entry.color.blue,
        alpha: entry.color.alpha,
      },
      source: document.getText(entry.range),
    }));
    const rawColors = await request("vize.test.executeLspRequest", {
      method: "textDocument/documentColor",
      params: { textDocument: { uri: document.uri.toString() } },
    });
    retain("colors.json", { publicColors, rawColors });
    assert.deepEqual(publicColors, currentDocumentColors, "complete unchanged public colors");
    assert.deepEqual(
      rawColors,
      currentDocumentColors.map((entry) => ({
        color: entry.color,
        range: {
          start: { line: entry.range[0], character: entry.range[1] },
          end: { line: entry.range[2], character: entry.range[3] },
        },
      })),
      "complete unchanged native colors",
    );
    assert.deepEqual(
      publicColors.map((entry) => entry.source),
      ["red", "#336699", "rgba(51, 102, 153, 0.5)"],
    );
    assert.equal(publicColors[2].color.alpha, 0.5);
    const hex = colors.find((entry) => document.getText(entry.range) === "#336699");
    assert.ok(hex, "real provider must supply the hex color");
    for (const [channel, expected] of Object.entries({
      red: 0.2,
      green: 0.4,
      blue: 0.6,
      alpha: 1,
    })) {
      assert.ok(Math.abs(hex.color[channel] - expected) < 0.00001, channel);
    }
    const presentations = await request("vscode.executeColorPresentationProvider", hex.color, {
      uri: document.uri,
      range: hex.range,
    });
    assert.deepEqual(
      JSON.parse(JSON.stringify(presentations)),
      currentColorPresentations,
      "complete unchanged public color presentations",
    );
    for (const expected of hoverCases.filter((entry) => !entry.baselineNull)) {
      await phase(
        expected.id,
        document.positionAt(fixtureSource.indexOf(expected.needle) + expected.offset),
      );
    }
    await phase("picker", hex.range.start);
    assert.equal(document.getText(), fixtureSource);
    retain("complete.json", { unchangedDocument: true, captured: ["property", "named", "picker"] });
  } catch (error) {
    retain("host-error.json", { message: error.message, stack: error.stack });
    throw error;
  }
};
