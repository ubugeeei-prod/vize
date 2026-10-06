import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

import { LspWire, decodeFrames, frameMessage } from "../differential/lsp-wire.ts";
import { createVueInlayContract } from "./create-vue-inlay-contract.ts";
import { repoRoot } from "./realworld-patch.ts";
import { symlinkVueTypes } from "./realworld-typecheck.ts";

const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const json = (value: unknown) => `${JSON.stringify(value, null, 2)}\n`;

/** Independent stock process; never obtain expected types from Vize or hover text. */
export async function createVueStockInlay(
  source: string,
  originalConfig: { compilerOptions: Record<string, unknown>; include: string[] },
  executable: string,
) {
  const binary = fs.realpathSync(executable);
  const identity = { path: binary, sha256: hash(fs.readFileSync(binary)) };
  const version = spawnSync(binary, ["--version"], { encoding: "utf8" });
  assert.equal(version.error, undefined);
  assert.equal(version.signal, null);
  assert.equal(version.status, 0);
  assert.equal(version.stdout, "Version 7.0.2\n");
  assert.equal(version.stderr, "");
  const root = path.join(repoRoot, "target/vize-tests/create-vue-stock-inlay");
  fs.mkdirSync(root, { recursive: true });
  const workspace = fs.realpathSync(fs.mkdtempSync(path.join(root, "workspace-")));
  const script = source.slice(source.indexOf(">") + 1, source.indexOf("</script>"));
  const frozenScript = fs.readFileSync(
    path.join(repoRoot, "tests/_fixtures/differential/lsp/create-vue-native-inlay/script.ts.txt"),
    "utf8",
  );
  assert.equal(script, frozenScript, "the exact original authored script is the native input");
  const originalConfigBytes = json(originalConfig);
  // Vue files are not native TS roots. Only the explicit bare-script membership
  // changes; every original compiler option stays exact in this stock control.
  const nativeConfig = { ...originalConfig, include: ["Oracle.ts"] };
  const configBytes = json(nativeConfig);
  symlinkVueTypes(workspace);
  fs.writeFileSync(path.join(workspace, "tsconfig.json"), configBytes);
  const file = path.join(workspace, "Oracle.ts");
  fs.writeFileSync(file, script);
  const uri = pathToFileURL(file).href;
  const contract = createVueInlayContract(workspace);
  const output = path.join(repoRoot, "target/differential/lsp-sessions");
  fs.mkdirSync(output, { recursive: true });
  const capture = fs.mkdtempSync(path.join(output, "stock-create-vue-inlay-"));
  fs.writeFileSync(path.join(capture, "original-App.vue"), source);
  fs.writeFileSync(path.join(capture, "original-tsconfig.json"), originalConfigBytes);
  fs.writeFileSync(path.join(capture, "Oracle.ts"), script);
  fs.writeFileSync(path.join(capture, "native-tsconfig.json"), configBytes);
  fs.writeFileSync(path.join(capture, "reactivity.d.ts"), contract.declarationBytes);
  fs.writeFileSync(path.join(capture, "expected-whole-hints.json"), json(contract.expected));
  const wire = new LspWire(binary, ["--lsp", "--stdio"], workspace);
  let readersJoined = false;
  wire.child.once("close", () => {
    readersJoined = true;
  });
  let hints: unknown = null;
  let diagnostics: unknown = null;
  let failure: unknown;
  try {
    const initialized = await wire.request(
      "initialize",
      {
        processId: null,
        rootUri: pathToFileURL(workspace).href,
        capabilities: {
          textDocument: {
            diagnostic: {
              dynamicRegistration: false,
              relatedDocumentSupport: true,
              relatedInformation: true,
            },
          },
        },
        initializationOptions: {
          userPreferences: {
            tsserver: { automaticTypeAcquisition: { enabled: false } },
            inlayHints: { variableTypes: { enabled: true } },
          },
        },
      },
      20_000,
    );
    assert.ok(initialized.result && typeof initialized.result === "object");
    wire.notify("initialized", {});
    const registration = await wire.waitFor(
      (message) => message.method === "client/registerCapability",
      20_000,
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
    // The existing generic client's declared ID type is numeric. The standard
    // native registration has the literal string ID above; retain its exact
    // framed acknowledgement in the same client's attempted-write capture.
    const acknowledgement = frameMessage({ jsonrpc: "2.0", id: "ts1", result: null });
    wire.clientChunks.push(acknowledgement);
    assert.ok(Buffer.concat(wire.clientChunks).length <= 16 * 1024 * 1024);
    wire.child.stdin.write(acknowledgement);
    wire.notify("textDocument/didOpen", {
      textDocument: {
        uri,
        languageId: "typescript",
        version: 1,
        text: script,
      },
    });
    const diagnosticResponse = await wire.request(
      "textDocument/diagnostic",
      { textDocument: { uri } },
      20_000,
    );
    diagnostics = diagnosticResponse;
    fs.writeFileSync(path.join(capture, "whole-diagnostic-response.json"), json(diagnostics));
    assert.deepEqual(diagnosticResponse, {
      jsonrpc: "2.0",
      id: wire.nextId,
      result: {
        kind: "full",
        items: [
          {
            range: { start: { line: 4, character: 6 }, end: { line: 4, character: 13 } },
            code: 6133,
            severity: 4,
            message: "'doubled' is declared but its value is never read.",
            source: "ts",
          },
        ],
      },
    });
    const response = await wire.request(
      "textDocument/inlayHint",
      {
        textDocument: { uri },
        range: { start: { line: 0, character: 0 }, end: { line: 5, character: 0 } },
      },
      20_000,
    );
    hints = response;
    fs.writeFileSync(path.join(capture, "whole-inlay-response.json"), json(hints));
    assert.deepEqual(response, { jsonrpc: "2.0", id: wire.nextId, result: contract.expected });
    const shutdown = await wire.request("shutdown", undefined, 10_000);
    assert.deepEqual(shutdown, { jsonrpc: "2.0", id: wire.nextId, result: null });
    wire.child.stdin.end(); // Native7's accepted acknowledged EOF route; no context-cancelling exit notification.
    await wire.stop(false);
    assert.equal(wire.exitStatus, 0);
    assert.equal(wire.signal, null);
    assert.equal(wire.processError, null);
    assert.equal(Buffer.concat(wire.stderrChunks).length, 0);
    assert.deepEqual(
      wire.messages.filter((message) => message.method === "textDocument/publishDiagnostics"),
      [
        {
          jsonrpc: "2.0",
          method: "textDocument/publishDiagnostics",
          params: {
            uri: pathToFileURL(path.join(workspace, "tsconfig.json")).href,
            diagnostics: [],
          },
        },
      ],
    );
    assert.equal(hash(fs.readFileSync(binary)), identity.sha256);
    assert.deepEqual(createVueInlayContract(workspace).expected, contract.expected);
    return contract.expected;
  } catch (error) {
    failure = error;
    throw error;
  } finally {
    try {
      await wire.stop();
    } finally {
      const observation = wire.observation();
      fs.writeFileSync(
        path.join(capture, "stock-native.json"),
        json({
          schema: "vize.create-vue.stock-native.inlay",
          source: process.env.GITHUB_SHA ?? null,
          binary: identity,
          providers: contract.providers,
          version: {
            status: version.status,
            signal: version.signal,
            stdout: version.stdout,
            stderr: version.stderr,
          },
          workspace,
          uri,
          originalConfig,
          nativeConfig,
          membershipDelta: "only include=[Oracle.ts]; compilerOptions identical",
          diagnostics,
          hints,
          observation,
          stdoutReaderJoined: readersJoined,
          stderrReaderJoined: readersJoined,
          failure: failure instanceof Error ? failure.message : (failure ?? null),
          nativeProgramCount: null,
          nativeGroups: null,
          performanceCredit: false,
        }),
      );
      fs.writeFileSync(path.join(capture, "client.bin"), Buffer.concat(wire.clientChunks));
      fs.writeFileSync(path.join(capture, "server.bin"), Buffer.concat(wire.serverChunks));
      fs.writeFileSync(path.join(capture, "stderr.bin"), Buffer.concat(wire.stderrChunks));
      try {
        decodeFrames(Buffer.concat(wire.clientChunks));
        decodeFrames(Buffer.concat(wire.serverChunks));
      } finally {
        fs.rmSync(workspace, { recursive: true, force: true });
      }
    }
  }
}
