import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { LspWire } from "../../differential/lsp-wire.ts";
import type { JsonRpcMessage } from "../../differential/lsp-types.ts";

export const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
export const uri = (file: string) => pathToFileURL(file).href;
export const captureRoot =
  process.env.VIZE_EDITOR_CONFIG_CAPTURE ?? process.env.VIZE_PRIVATE_IMPORT_CAPTURE;
export function save(name: string, value: unknown): void {
  if (!captureRoot) return;
  fs.mkdirSync(captureRoot, { recursive: true });
  fs.writeFileSync(path.join(captureRoot, name + ".json"), JSON.stringify(value, null, 2) + "\n");
}
export function diagnostic(
  code: number,
  message: string,
  line: number,
  start: number,
  end: number,
  native = false,
) {
  return {
    code,
    message,
    range: { start: { line, character: start }, end: { line, character: end } },
    severity: code === 6133 ? 4 : 1,
    source: native ? "ts" : "vize/types",
  };
}
export async function initialize(wire: LspWire, directory: string, stock: boolean): Promise<void> {
  const initialized = await wire.request("initialize", {
    processId: process.pid,
    rootUri: uri(directory),
    capabilities: {},
    ...(stock
      ? {
          initializationOptions: {
            userPreferences: { tsserver: { automaticTypeAcquisition: { enabled: false } } },
          },
        }
      : {}),
  });
  assert.equal(initialized.error, undefined);
  assert.equal(typeof initialized.result, "object");
  wire.notify("initialized", {});
  if (stock) {
    const registration = await wire.waitFor(
      (message) => message.method === "client/registerCapability",
    );
    assert.deepEqual(registration, {
      jsonrpc: "2.0",
      id: "ts1",
      method: "client/registerCapability",
      params: {
        registrations: [
          {
            id: "typescript-config-watch-id",
            method: "workspace/didChangeConfiguration",
            registerOptions: { section: ["js/ts", "typescript", "javascript", "editor"] },
          },
        ],
      },
    });
    wire.send({ jsonrpc: "2.0", id: "ts1", result: null } as unknown as JsonRpcMessage);
  }
}
export function open(wire: LspWire, file: string, text: string, languageId: string): void {
  wire.notify("textDocument/didOpen", {
    textDocument: { uri: uri(file), languageId, version: 1, text },
  });
}
export function change(wire: LspWire, file: string, text: string, version: number): void {
  wire.notify("textDocument/didChange", {
    textDocument: { uri: uri(file), version },
    contentChanges: [{ text }],
  });
}
export async function publication(
  wire: LspWire,
  file: string,
  version: number,
  diagnostics: unknown[],
): Promise<void> {
  const response = await wire.waitFor(
    (message) =>
      message.method === "textDocument/publishDiagnostics" &&
      message.params?.uri === uri(file) &&
      message.params?.version === version,
  );
  assert.deepEqual(response, {
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: { uri: uri(file), version, diagnostics },
  });
}
export function observation(wire: LspWire) {
  const observed = wire.observation();
  return {
    ...observed,
    clientWireSha256: hash(Buffer.from(observed.clientWireBase64, "base64")),
    serverWireSha256: hash(Buffer.from(observed.serverWireBase64, "base64")),
  };
}

export async function stockOracle(
  directory: string,
  configName: string,
  inputs: Record<string, string>,
  bare: string,
  suffix: "js" | "ts",
  mainFile: string,
  vectors: Record<string, unknown[]>,
  name: string,
  runtime: string,
): Promise<void> {
  const file = path.join(directory, "src/Oracle." + suffix);
  fs.writeFileSync(file, bare);
  const wire = new LspWire(runtime, ["--lsp", "--stdio"], directory);
  try {
    await initialize(wire, directory, true);
    const actual: Record<string, unknown> = {};
    for (const [target, bytes] of [
      [file, bare],
      [path.join(directory, mainFile), inputs[mainFile]],
    ]) {
      open(wire, target, bytes, suffix === "js" ? "javascript" : "typescript");
      const result = await wire.request("textDocument/diagnostic", {
        textDocument: { uri: uri(target) },
      });
      actual[target] = result;
      save(name + "-native-current", {
        originalInputs: inputs,
        stockSource: bare,
        requestedConfigName: configName,
        requestedConfig: inputs[configName],
        actual,
        expected: vectors,
        wire: observation(wire),
      });
      assert.deepEqual(result, {
        jsonrpc: "2.0",
        id: wire.nextId,
        result: { kind: "full", items: vectors[target === file ? "src/App.vue" : mainFile] },
      });
    }
    const shutdown = await wire.request("shutdown");
    assert.deepEqual(shutdown, { jsonrpc: "2.0", id: wire.nextId, result: null });
    // Native 7.0.2 queues this INFO after the response. Keep its transport
    // alive until the exact completion arrives, before the existing EOF.
    const prefix = `handled method 'shutdown' (${wire.nextId}) in `;
    const completion = await wire.waitFor(
      (message) =>
        message.method === "window/logMessage" &&
        message.params?.type === 3 &&
        typeof message.params?.message === "string" &&
        message.params.message.startsWith(prefix),
    );
    const completionMessage = completion.params?.message;
    assert.ok(typeof completionMessage === "string");
    const duration = completionMessage.slice(prefix.length);
    assert.match(
      duration,
      /^(?:0s|-?(?:[1-9]\d{0,2}ns|[1-9]\d{0,2}(?:\.\d{0,2}[1-9])?µs|[1-9]\d{0,2}(?:\.\d{0,5}[1-9])?ms|(?:[1-5]\d|[1-9])(?:\.\d{0,8}[1-9])?s|(?:(?:[1-9]\d*h)?(?:[1-5]\d|[1-9])m|[1-9]\d*h0m)(?:[1-5]\d|\d)(?:\.\d{0,8}[1-9])?s))$/,
    );
    assert.deepEqual(completion, {
      jsonrpc: "2.0",
      method: "window/logMessage",
      params: { type: 3, message: prefix + duration },
    });
    wire.child.stdin.end();
    await wire.stop(false);
    assert.equal(wire.exitStatus, 0);
    assert.equal(wire.signal, null);
    assert.equal(wire.processError, null);
    assert.equal(Buffer.concat(wire.stderrChunks).length, 0);
    const publications = wire.messages.filter(
      (message) => message.method === "textDocument/publishDiagnostics",
    );
    assert.ok(publications.length > 0, "native config diagnostic family must be retained");
    for (const publication of publications)
      assert.deepEqual(publication, {
        jsonrpc: "2.0",
        method: "textDocument/publishDiagnostics",
        params: { uri: uri(path.join(directory, configName)), diagnostics: [] },
      });
  } finally {
    await wire.stop().catch(() => undefined);
    save(name + "-native-terminal", {
      wire: observation(wire),
      allDecodedMessages: wire.messages,
      joinedThroughChildClose: wire.exitStatus !== null || wire.signal !== null,
    });
    fs.rmSync(file, { force: true });
  }
}
