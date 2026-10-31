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
type CancellationAttempt = { id: number; method: string; response: Promise<unknown> };

function observeResponses(session: LspSession): Map<number, JsonRpcMessage> {
  const responses = new Map<number, JsonRpcMessage>();
  session.responseObservers.push((message) => {
    assert.equal(typeof message.id, "number");
    assert.ok(!responses.has(message.id as number), "request must receive only one response");
    responses.set(message.id as number, message);
  });
  return responses;
}

function assertCancellationOrEmpty(
  attempt: CancellationAttempt,
  result: PromiseSettledResult<unknown>,
  responses: Map<number, JsonRpcMessage>,
): "cancelled" | "completed" {
  if (result.status === "fulfilled") {
    assert.equal(
      result.value,
      null,
      `${attempt.method} request ${attempt.id} has no matching symbol`,
    );
    assert.deepEqual(responses.get(attempt.id), {
      jsonrpc: "2.0",
      id: attempt.id,
      result: null,
    });
    return "completed";
  }
  assert.ok(result.reason instanceof LspRequestError, String(result.reason));
  assert.equal(result.reason.id, attempt.id);
  assert.equal(result.reason.method, attempt.method);
  assert.equal(result.reason.code, -32800);
  assert.deepEqual(responses.get(attempt.id), {
    jsonrpc: "2.0",
    id: attempt.id,
    error: { code: -32800, message: "Canceled" },
  });
  return "cancelled";
}

function codesOf(params: PublishDiagnosticsParams): string[] {
  return params.diagnostics
    .map((diagnostic) => `${diagnostic.source ?? "?"}:${diagnostic.message}`)
    .sort();
}

test("vize lsp handles a cancellation attempt and remains usable", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-request-cancellation");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const filePath = path.join(workspaceDir, "Cancellable.vue");
  const uri = pathToFileURL(filePath).href;
  fs.writeFileSync(filePath, CLEAN_SOURCE, "utf8");

  const session = new LspSession();
  const responses = observeResponses(session);
  let primaryError: unknown;
  try {
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: false });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: CLEAN_SOURCE },
    });
    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    let id = -1;
    const response = session.request(
      "workspace/symbol",
      { query: "cancellableSymbol-missing" },
      30000,
      (requestId) => {
        id = requestId;
        return [{ jsonrpc: "2.0", method: "$/cancelRequest", params: { id } }];
      },
    );
    const [result] = await Promise.allSettled([response]);
    const outcome = assertCancellationOrEmpty(
      { id, method: "workspace/symbol", response },
      result,
      responses,
    );
    console.log("lsp-cancellation-outcomes", {
      cancelled: Number(outcome === "cancelled"),
      completed: Number(outcome === "completed"),
    });

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
  const responses = observeResponses(session);
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
    const cancelled: CancellationAttempt[] = [];
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

    // Each unmatched query and cancellation share one stdio write, but the
    // server may complete with null before decoding the cancel frame. Strict
    // in-flight cancellation is witnessed by the production-service Rust test.
    // Edits still cover all three queue orderings without timing sleeps.
    for (const round of [1, 2, 3]) {
      session.notify("textDocument/didChange", changeMessage(BROKEN_SOURCE).params);
      cancelWorkspaceSymbol(`cancellableSymbol-before-${round}`);
      cancelWorkspaceSymbol(`cancellableSymbol-batched-${round}`, [changeMessage(CLEAN_SOURCE)]);
      cancelWorkspaceSymbol(`cancellableSymbol-after-${round}`);
    }

    const settled = await Promise.allSettled(cancelled.map(({ response }) => response));
    const counts = { cancelled: 0, completed: 0 };
    settled.forEach((result, index) => {
      counts[assertCancellationOrEmpty(cancelled[index], result, responses)] += 1;
    });
    assert.equal(counts.cancelled + counts.completed, 9);
    console.log("lsp-cancellation-outcomes", counts);

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
