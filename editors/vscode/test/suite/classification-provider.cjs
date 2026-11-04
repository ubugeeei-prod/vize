const assert = require("node:assert/strict");
const fs = require("node:fs");
const probe = require("../../../../tests/typescript-plugin/classification-probe.cjs");
const tokenTypes = [
  "class",
  "enum",
  "interface",
  "namespace",
  "typeParameter",
  "type",
  "parameter",
  "variable",
  "enumMember",
  "property",
  "function",
  "method",
];
const tokenModifiers = ["declaration", "static", "async", "readonly", "defaultLibrary", "local"];

function decodeTokens(document, value, legend, Position) {
  assert.deepEqual(legend.tokenTypes, tokenTypes);
  assert.deepEqual(legend.tokenModifiers, tokenModifiers);
  const data = Array.from(value.data);
  assert.equal(data.length % 5, 0);
  const spans = [];
  let line = 0,
    character = 0;
  for (let at = 0; at < data.length; at += 5) {
    const [deltaLine, deltaCharacter, length, type, modifiers] = data.slice(at, at + 5);
    assert([deltaLine, deltaCharacter, length, type, modifiers].every(Number.isSafeInteger));
    assert(
      deltaLine >= 0 &&
        deltaCharacter >= 0 &&
        length > 0 &&
        type >= 0 &&
        type < tokenTypes.length &&
        modifiers >= 0 &&
        modifiers < 256,
    );
    line += deltaLine;
    character = deltaLine ? deltaCharacter : character + deltaCharacter;
    const start = document.offsetAt(new Position(line, character));
    const actual = document.positionAt(start);
    assert.equal(actual.line, line);
    assert.equal(actual.character, character);
    spans.push(start, length, ((type + 1) << 8) | modifiers);
  }
  probe.triples({ spans }, document.getText());
  return {
    spans,
    provider: {
      data,
      legend: { tokenTypes: legend.tokenTypes, tokenModifiers: legend.tokenModifiers },
    },
  };
}

async function checkInstalledClassifications(vscode, fixture) {
  const documents = new Map();
  return probe.checkClassifications(fixture, {
    transport: "vscode-built-in-range-semantic-provider",
    commands: [probe.semantic],
    open: async (file, text) => {
      let document = await vscode.workspace.openTextDocument(vscode.Uri.file(file));
      // The fixture explicitly exposes the bundled TypeScript semantic provider
      // for a mixed-content file. No replacement provider or protocol bridge.
      if (document.languageId !== "typescript")
        document = await vscode.languages.setTextDocumentLanguage(document, "typescript");
      assert.equal(document.getText(), text);
      documents.set(file, document);
      await vscode.window.showTextDocument(document);
    },
    change: async (file, before, after) => {
      const document = documents.get(file);
      assert.equal(document.getText(), before);
      const edit = new vscode.WorkspaceEdit();
      edit.replace(
        document.uri,
        new vscode.Range(document.positionAt(0), document.positionAt(before.length)),
        after,
      );
      assert.equal(await vscode.workspace.applyEdit(edit), true);
      assert.equal(document.getText(), after);
    },
    query: async (command, file, text, range) => {
      assert.equal(command, probe.semantic);
      const document = documents.get(file);
      assert.equal(document.getText(), text);
      const requestRange = new vscode.Range(
        document.positionAt(range.start),
        document.positionAt(range.start + range.length),
      );
      const legend = await vscode.commands.executeCommand(
        "vscode.provideDocumentRangeSemanticTokensLegend",
        document.uri,
        requestRange,
      );
      const value = await vscode.commands.executeCommand(
        "vscode.provideDocumentRangeSemanticTokens",
        document.uri,
        requestRange,
      );
      assert(legend && value, "the actual built-in TypeScript semantic provider must answer");
      return { success: true, body: decodeTokens(document, value, legend, vscode.Position) };
    },
  });
}

function logPackets(text) {
  const packets = [];
  const headers = /^(?:Info|Err)\s+\d+\s+\[[^\]]+\]\s+(?:request|response):\s*$/gm;
  for (const header of text.matchAll(headers)) {
    const start = text.indexOf("{", header.index + header[0].length);
    assert(start >= 0);
    let depth = 0,
      quoted = false,
      escaped = false;
    for (let at = start; at < text.length; at++) {
      const value = text[at];
      if (quoted) {
        if (escaped) escaped = false;
        else if (value === "\\") escaped = true;
        else if (value === '"') quoted = false;
        continue;
      }
      if (value === '"') quoted = true;
      else if (value === "{") depth++;
      else if (value === "}" && --depth === 0) {
        packets.push(JSON.parse(text.slice(start, at + 1)));
        break;
      }
    }
  }
  return packets;
}

function verifyClassificationLogs(logs, qualification, expectedProvider) {
  assert.match(expectedProvider.source, /^[a-f0-9]{40}$/);
  assert.match(expectedProvider.nativeBuildToken, /^[a-f0-9]{32}$/);
  assert.equal(qualification.transport, "vscode-built-in-range-semantic-provider");
  assert.deepEqual(qualification.commands, [probe.semantic]);
  assert.equal(qualification.passed, true);
  assert.deepEqual(
    qualification.observations.map((row) => row.role),
    probe.observationRoles([probe.semantic]),
    "every before/after/reference/ordinary protocol control must run",
  );
  const pairs = [];
  const providerLogs = new Map();
  for (const file of logs) {
    const pending = new Map();
    const text = fs.readFileSync(file, "utf8");
    const marker = "[vize] TypeScript native provider ";
    providerLogs.set(
      file,
      text
        .split("\n")
        .filter((line) => line.includes(marker))
        .map((line) => JSON.parse(line.slice(line.indexOf(marker) + marker.length))),
    );
    for (const packet of logPackets(text)) {
      if (packet.type === "request") pending.set(packet.seq, packet);
      else if (packet.type === "response" && pending.has(packet.request_seq)) {
        pairs.push({ file, request: pending.get(packet.request_seq), response: packet });
        pending.delete(packet.request_seq);
      }
    }
  }
  const identity = (file) => {
    const physical = fs.realpathSync(file).replaceAll("\\", "/");
    return process.platform === "win32" ? physical.toLowerCase() : physical;
  };
  const used = new Set();
  const matched = qualification.observations.map((observation) => {
    assert.equal(observation.command, probe.semantic);
    const index = pairs.findIndex(
      (pair, index) =>
        !used.has(index) &&
        pair.request.command === probe.semantic &&
        pair.response.success === true &&
        pair.response.command === probe.semantic &&
        identity(pair.request.arguments.file) === identity(observation.file) &&
        pair.request.arguments.start === observation.range.start &&
        pair.request.arguments.length === observation.range.length &&
        pair.request.arguments.format === "2020" &&
        JSON.stringify(pair.response.body.spans) ===
          JSON.stringify(observation.response.body.spans),
    );
    assert(
      index >= 0,
      `missing actual encoded semantic request/whole response for ${observation.role}`,
    );
    assert(
      providerLogs
        .get(pairs[index].file)
        .some(
          (value) =>
            value.origin === "bundled" &&
            value.source === expectedProvider.source &&
            value.nativeBuildToken === expectedProvider.nativeBuildToken,
        ),
      "the responding server must load the exact bundled native provider",
    );
    used.add(index);
    return { role: observation.role, ...pairs[index] };
  });
  return { transport: qualification.transport, syntaxExecuted: false, observations: matched };
}

module.exports = {
  checkInstalledClassifications,
  decodeTokens,
  logPackets,
  verifyClassificationLogs,
  createClassificationFixtures: probe.createClassificationFixtures,
};
