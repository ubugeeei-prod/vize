import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { hoverToText, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { JsonRpcMessage, PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspRequestError, LspSession } from "./support/lsp/session.ts";

const CLEAN_SOURCE = `<script setup lang="ts">
const cancellableSymbol = 1
</script>
<template><p>{{ cancellableSymbol }}</p></template>
`;

const BROKEN_SOURCE = `<template>
  <p>{{ cancellableSymbol }}</p>
`;

const BROKEN_CODES = ["vize/sfc:Malformed <template> block: the closing tag is missing."];

type PublishRecord = { version: number | null; codes: string[] };

function codesOf(params: PublishDiagnosticsParams): string[] {
  return params.diagnostics
    .map((diagnostic) => `${diagnostic.source ?? "?"}:${diagnostic.message}`)
    .sort();
}

test("vize lsp cancels an in-flight request and remains usable", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-request-cancellation");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const filePath = path.join(workspaceDir, "Cancellable.vue");
  const uri = pathToFileURL(filePath).href;
  fs.writeFileSync(filePath, CLEAN_SOURCE, "utf8");

  const session = new LspSession();
  let primaryError: unknown;
  try {
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: false });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: CLEAN_SOURCE },
    });
    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    const cancelled = session.request(
      "workspace/symbol",
      { query: "cancellableSymbol" },
      30000,
      (id) => [{ jsonrpc: "2.0", method: "$/cancelRequest", params: { id } }],
    );

    await assert.rejects(
      cancelled,
      (error) =>
        error instanceof LspRequestError &&
        error.method === "workspace/symbol" &&
        error.code === -32800,
    );

    const retry = (await session.request("workspace/symbol", {
      query: "cancellableSymbol",
    })) as Array<{ name: string }> | null;
    assert.ok(
      retry?.some((symbol) => symbol.name === "cancellableSymbol"),
      JSON.stringify(retry),
    );
  } catch (error) {
    primaryError = error;
    throw error;
  } finally {
    await session.shutdown().catch((error: unknown) => {
      if (primaryError == null) throw error;
    });
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

test("vize lsp preserves cancellation and diagnostics under revision churn", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-request-cancellation-churn");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const filePath = path.join(workspaceDir, "CancellationChurn.vue");
  const uri = pathToFileURL(filePath).href;
  fs.writeFileSync(filePath, CLEAN_SOURCE, "utf8");

  const session = new LspSession();
  let primaryError: unknown;
  try {
    await session.initialize(workspaceDir, {
      editor: true,
      hover: true,
      lint: false,
      typecheck: false,
    });

    const publishes: PublishRecord[] = [];
    session.notificationObservers.push((method, params) => {
      if (method !== "textDocument/publishDiagnostics") return;
      const publish = params as PublishDiagnosticsParams;
      if (!isDiagnosticsForUri(publish, uri)) return;
      publishes.push({ version: publish.version ?? null, codes: codesOf(publish) });
    });

    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: CLEAN_SOURCE },
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === 1,
    );

    const expectedCodes = new Map<number, string[]>([[1, []]]);
    const cancelled: Array<{ id: number; method: string; response: Promise<unknown> }> = [];
    let version = 1;

    const changeMessage = (text: string): JsonRpcMessage => {
      version += 1;
      expectedCodes.set(version, text === BROKEN_SOURCE ? BROKEN_CODES : []);
      return {
        jsonrpc: "2.0",
        method: "textDocument/didChange",
        params: {
          textDocument: { uri, version },
          contentChanges: [{ text }],
        },
      };
    };

    const cancelWorkspaceSymbol = (query: string, following: JsonRpcMessage[] = []): void => {
      const method = "workspace/symbol";
      let id = -1;
      const response = session.request(method, { query }, 30_000, (requestId) => {
        id = requestId;
        return [
          { jsonrpc: "2.0", method: "$/cancelRequest", params: { id: requestId } },
          ...following,
        ];
      });
      assert.ok(id > 0, "request id must be captured before the frame is sent");
      cancelled.push({ id, method, response });
    };

    // Each request and its cancellation share one stdio write. The edit is
    // alternately queued before, inside, and after those frames so three
    // repeated rounds cover both queue orderings without timing sleeps.
    for (const round of [1, 2, 3]) {
      session.notify("textDocument/didChange", changeMessage(BROKEN_SOURCE).params);
      cancelWorkspaceSymbol(`cancellableSymbol-before-${round}`);
      cancelWorkspaceSymbol(`cancellableSymbol-batched-${round}`, [changeMessage(CLEAN_SOURCE)]);
      cancelWorkspaceSymbol(`cancellableSymbol-after-${round}`);
    }

    const settled = await Promise.allSettled(cancelled.map(({ response }) => response));
    settled.forEach((result, index) => {
      const attempt = cancelled[index];
      assert.equal(
        result.status,
        "rejected",
        `${attempt.method} request ${attempt.id} must cancel`,
      );
      if (result.status !== "rejected") return;
      assert.ok(result.reason instanceof LspRequestError, String(result.reason));
      assert.equal(result.reason.id, attempt.id);
      assert.equal(result.reason.method, attempt.method);
      assert.equal(result.reason.code, -32800);
    });

    const finalVersion = version;
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === finalVersion,
    );

    let previousVersion = 0;
    for (const publish of publishes) {
      assert.ok(
        publish.version != null,
        `unexpected unversioned publish: ${JSON.stringify(publish)}`,
      );
      assert.ok(
        publish.version > previousVersion,
        `diagnostic versions must advance monotonically: ${JSON.stringify(publishes)}`,
      );
      assert.deepEqual(
        publish.codes,
        expectedCodes.get(publish.version),
        `diagnostics must describe revision ${publish.version}`,
      );
      previousVersion = publish.version;
    }
    assert.deepEqual(publishes.at(-1), { version: finalVersion, codes: [] });

    const symbols = (await session.request("workspace/symbol", {
      query: "cancellableSymbol",
    })) as Array<{ name?: string }> | null;
    assert.ok(symbols?.some((symbol) => symbol.name === "cancellableSymbol"));

    const hover = (await session.request("textDocument/hover", {
      textDocument: { uri },
      position: offsetToPosition(
        CLEAN_SOURCE,
        CLEAN_SOURCE.lastIndexOf("cancellableSymbol }}</p>") + 3,
      ),
    })) as { contents?: unknown } | null;
    assert.match(hoverToText(hover), /cancellableSymbol/);

    session.notify("textDocument/didClose", { textDocument: { uri } });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version == null,
    );
    assert.deepEqual(publishes.at(-1), { version: null, codes: [] });
  } catch (error) {
    primaryError = error;
    throw error;
  } finally {
    await session.shutdown().catch((error: unknown) => {
      if (primaryError == null) throw error;
    });
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
