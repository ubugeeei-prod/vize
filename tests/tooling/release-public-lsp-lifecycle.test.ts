import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import test from "node:test";
import { assertDecodedLspLifecycle } from "../../tools/support/release/public_acceptance/lsp_packets.ts";
import { diagnosticContract } from "../../tools/support/release/public_acceptance/lsp_case.ts";

const fixture = JSON.parse(
  fs.readFileSync(
    new URL("../_fixtures/release/public-lsp-boundary-eof.json", import.meta.url),
    "utf8",
  ),
);
const original = (): ReturnType<typeof JSON.parse>[] =>
  fixture.protocol
    .trim()
    .split("\n")
    .map((row: string) => JSON.parse(row));
type Ledger = ReturnType<typeof original>;
const originalUris = Object.fromEntries(
  original().flatMap((row) => {
    const document = row.message.params?.textDocument;
    return row.direction === "request" && row.message.method === "textDocument/didOpen"
      ? [[new URL(document.uri).pathname.split("/").at(-1)!, document.uri]]
      : [];
  }),
);
const originalAuthority = diagnosticContract(originalUris).allowed;
const request = (rows: Ledger, method: string) =>
  rows.findIndex((row) => row.direction === "request" && row.message.method === method);
const shutdownResponse = (rows: Ledger) => {
  const id = rows[request(rows, "shutdown")].message.id;
  return rows.findIndex((row) => row.direction === "response" && row.message.id === id);
};

test("actual public 0.458 decoded ledger ends at exit without rewriting rows", () => {
  assert.equal(
    createHash("sha256").update(fixture.protocol).digest("hex"),
    fixture.provenance.protocolSha256,
  );
  assert.equal(fixture.provenance.version, "0.458.0");
  const rows = original(),
    before = structuredClone(rows);
  assertDecodedLspLifecycle(rows, fixture.terminal, originalAuthority);
  assert.deepEqual(rows, before);
  assert.equal(rows.length, 94);
  assert.deepEqual(rows.at(-1), {
    direction: "request",
    message: { jsonrpc: "2.0", method: "exit" },
  });
  assert.equal(fixture.all32.length, 32);
  assert.equal(new Set(fixture.all32.map((row: { trace: string }) => row.trace)).size, 32);
  assert.equal(
    fixture.all32.filter(
      (row: { namespace: string }) => row.namespace === "named-model-transactions",
    ).length,
    12,
  );
  assert.equal(
    fixture.all32.filter(
      (row: { namespace: string }) => row.namespace === "bound-event-casing-transactions",
    ).length,
    20,
  );
  assert.equal(fixture.provenance.source.head, "6951e6fd56c14dd8e466579cb96652932ec216c6");
});

const historical = JSON.parse(
  fs.readFileSync(
    new URL("../_fixtures/release/public-lsp-clean-eof.json", import.meta.url),
    "utf8",
  ),
);
const legacyEof = () => ({
  direction: "stdoutError",
  message: { kind: "UnexpectedEof", message: "LSP stdout reached EOF" },
});
test("historical 0.457 error ledger remains evidence, not an accepted successor shape", () => {
  assert.equal(
    createHash("sha256").update(historical.protocol).digest("hex"),
    historical.provenance.protocolSha256,
  );
  assert.equal(historical.provenance.version, "0.457.0");
  assert.throws(() =>
    assertDecodedLspLifecycle(
      historical.protocol.trim().split("\n").map(JSON.parse),
      historical.terminal,
      originalAuthority,
    ),
  );
});

const invalid: Record<string, (rows: Ledger) => void> = {
  "the legacy EOF error after exit": (rows) => {
    rows.push(legacyEof());
  },
  "a duplicated reader error": (rows) => {
    rows.push(legacyEof(), legacyEof());
  },
  "an early reader EOF despite later successful shutdown": (rows) => {
    rows.splice(3, 0, legacyEof());
  },
  "a truncated response body": (rows) => {
    rows.push({
      direction: "stdoutError",
      message: { kind: "UnexpectedEof", message: "failed to fill whole buffer" },
    });
  },
  "a truncated header": (rows) => {
    rows.push({
      direction: "stdoutError",
      message: { kind: "UnexpectedEof", message: "incomplete LSP header line" },
    });
  },
  "a partial following frame": (rows) => {
    rows.push({
      direction: "stdoutError",
      message: { kind: "UnexpectedEof", message: "LSP stdout closed inside frame headers" },
    });
  },
  "malformed complete JSON": (rows) => {
    rows.push({
      direction: "stdoutError",
      message: { kind: "InvalidData", message: "expected value at line 1 column 1" },
    });
  },
  "a missing Content-Length": (rows) => {
    rows.push({
      direction: "stdoutError",
      message: { kind: "InvalidData", message: "missing Content-Length header" },
    });
  },
  "a stderr reader error": (rows) => {
    rows.push({ ...legacyEof(), direction: "stderrError" });
  },
  "a broken pipe reader error": (rows) => {
    rows.push({ direction: "stdoutError", message: { kind: "BrokenPipe", message: "closed" } });
  },
  "a reader error with extra fields": (rows) => {
    rows.push({ ...legacyEof(), extra: true });
  },
  "a response after exit": (rows) => {
    rows.push(structuredClone(rows[1]));
  },
  "exit before shutdown acknowledgement": (rows) => {
    const last = rows.length - 1;
    [rows[last], rows[last - 1]] = [rows[last - 1], rows[last]];
  },
  "a missing initialize request": (rows) => {
    rows.splice(request(rows, "initialize"), 1);
  },
  "a duplicate initialize request": (rows) => {
    rows.splice(1, 0, structuredClone(rows[0]));
  },
  "a missing initialize response": (rows) => {
    rows.splice(1, 1);
  },
  "a duplicate initialize response": (rows) => {
    rows.splice(2, 0, structuredClone(rows[1]));
  },
  "a failed initialize response": (rows) => {
    delete rows[1].message.result;
    rows[1].message.error = { code: -32603, message: "failed" };
  },
  "a missing initialized notification": (rows) => {
    rows.splice(request(rows, "initialized"), 1);
  },
  "a duplicate initialized notification": (rows) => {
    rows.splice(3, 0, structuredClone(rows[request(rows, "initialized")]));
  },
  "initialize acknowledgement after initialized": (rows) => {
    [rows[1], rows[2]] = [rows[2], rows[1]];
  },
  "a missing shutdown request": (rows) => {
    rows.splice(request(rows, "shutdown"), 1);
  },
  "a duplicate shutdown request": (rows) => {
    rows.splice(request(rows, "shutdown"), 0, structuredClone(rows[request(rows, "shutdown")]));
  },
  "a missing shutdown response": (rows) => {
    rows.splice(shutdownResponse(rows), 1);
  },
  "a duplicate shutdown response": (rows) => {
    rows.splice(shutdownResponse(rows), 0, structuredClone(rows[shutdownResponse(rows)]));
  },
  "a shutdown error response": (rows) => {
    const message = rows[shutdownResponse(rows)].message;
    delete message.result;
    message.error = { code: -32603, message: "failed" };
  },
  "a non-null shutdown result": (rows) => {
    rows[shutdownResponse(rows)].message.result = {};
  },
  "a foreign shutdown response ID": (rows) => {
    rows[shutdownResponse(rows)].message.id = 9999;
  },
  "shutdown acknowledgement before shutdown": (rows) => {
    const response = rows.splice(shutdownResponse(rows), 1)[0];
    rows.splice(request(rows, "shutdown"), 0, response);
  },
  "a reused lifecycle request ID": (rows) => {
    rows[request(rows, "shutdown")].message.id = 1;
  },
  "an unsafe lifecycle request ID": (rows) => {
    rows[request(rows, "shutdown")].message.id = Number.MAX_SAFE_INTEGER + 1;
  },
  "an extra request after shutdown": (rows) => {
    rows.splice(rows.length - 2, 0, {
      direction: "request",
      message: { jsonrpc: "2.0", id: 9999, method: "hover" },
    });
  },
  "a missing exit notification": (rows) => {
    rows.splice(request(rows, "exit"), 1);
  },
  "a duplicate exit notification": (rows) => {
    rows.splice(request(rows, "exit"), 0, structuredClone(rows[request(rows, "exit")]));
  },
  "exit used as a request instead of notification": (rows) => {
    rows[request(rows, "exit")].message.id = 9999;
  },
};
for (const [label, mutate] of Object.entries(invalid))
  test(`decoded lifecycle refuses ${label}`, () => {
    const rows = original();
    mutate(rows);
    assert.throws(() => assertDecodedLspLifecycle(rows, fixture.terminal, originalAuthority));
  });

for (const [key, value] of [
  ["success", false],
  ["exitCode", 1],
  ["exitCode", null],
  ["stdoutReaderJoined", false],
  ["stdoutReaderJoined", null],
  ["stderrReaderJoined", false],
  ["stderrReaderJoined", null],
] as const)
  test(`boundary EOF cannot qualify terminal ${key}=${value}`, () => {
    assert.throws(() =>
      assertDecodedLspLifecycle(
        original(),
        { ...fixture.terminal, [key]: value },
        originalAuthority,
      ),
    );
  });

const delayedUris = Object.fromEntries(
  ["NativeGuard.vue", "Child.vue", "Parent.vue", "Other.vue"].map((file) => [
    file,
    `file:///controlled-project/src/${file}`,
  ]),
);
const delayedAuthority = diagnosticContract(delayedUris).allowed;
const delayedDiagnostic = {
  direction: "response",
  message: {
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: { uri: delayedUris["Parent.vue"], version: 3, diagnostics: [] },
  },
};
const delayed = (duplicate: boolean): Ledger => [
  { direction: "request", message: { jsonrpc: "2.0", id: 1, method: "initialize" } },
  { direction: "response", message: { jsonrpc: "2.0", id: 1, result: {} } },
  { direction: "request", message: { jsonrpc: "2.0", method: "initialized", params: {} } },
  ...(duplicate ? [structuredClone(delayedDiagnostic)] : []),
  { direction: "request", message: { jsonrpc: "2.0", id: 2, method: "shutdown" } },
  { direction: "response", message: { jsonrpc: "2.0", id: 2, result: null } },
  { direction: "request", message: { jsonrpc: "2.0", method: "exit" } },
  structuredClone(delayedDiagnostic),
];
for (const duplicate of [false, true])
  test(`authorized ${duplicate ? "duplicate" : "first-observed"} notification may be decoded after outbound exit`, () => {
    // Deterministic reader scheduling: queue the server's authored envelope,
    // observe the sender's exit, then release the independent inbound row.
    // No sleeps, server runtime, or cross-direction clock assumption is used.
    const rows = delayed(duplicate),
      before = structuredClone(rows);
    assertDecodedLspLifecycle(rows, fixture.terminal, delayedAuthority);
    assert.deepEqual(rows, before);
    assert.deepEqual(rows.at(-1), delayedDiagnostic);
  });
const invalidDelayed: Record<string, (rows: Ledger) => void> = {
  "an unknown diagnostic URI": (rows) => {
    rows.at(-1)!.message.params.uri = "file:///foreign-project/src/Parent.vue";
  },
  "a changed owned URI with incompatible diagnostic payload": (rows) => {
    rows.at(-1)!.message.params.uri = delayedUris["NativeGuard.vue"];
  },
  "an unauthorized diagnostic version": (rows) => {
    rows.at(-1)!.message.params.version = 4;
  },
  "changed diagnostics": (rows) => {
    rows.at(-1)!.message.params.diagnostics = [{ message: "foreign" }];
  },
  "extra diagnostic fields": (rows) => {
    rows.at(-1)!.message.params.extra = true;
  },
  "a notification of an unapproved method": (rows) => {
    rows.at(-1)!.message.method = "window/logMessage";
  },
  "a late error reply": (rows) => {
    rows.push({
      direction: "response",
      message: { jsonrpc: "2.0", id: 3, error: { code: -32603, message: "failed" } },
    });
  },
  "a late successful reply": (rows) => {
    rows.push({ direction: "response", message: { jsonrpc: "2.0", id: 3, result: null } });
  },
  "a late reply for an actual earlier request": (rows) => {
    rows.splice(request(rows, "shutdown"), 0, {
      direction: "request",
      message: { jsonrpc: "2.0", id: 3, method: "textDocument/hover", params: {} },
    });
    rows.push({ direction: "response", message: { jsonrpc: "2.0", id: 3, result: null } });
  },
  "a real reply after shutdown acknowledgement but before exit": (rows) => {
    rows.splice(request(rows, "shutdown"), 0, {
      direction: "request",
      message: { jsonrpc: "2.0", id: 3, method: "textDocument/hover", params: {} },
    });
    rows.splice(request(rows, "exit"), 0, {
      direction: "response",
      message: { jsonrpc: "2.0", id: 3, result: null },
    });
  },
  "an unknown reply before shutdown": (rows) => {
    rows.splice(request(rows, "shutdown"), 0, {
      direction: "response",
      message: { jsonrpc: "2.0", id: 99, result: null },
    });
  },
  "a missing ordinary reply": (rows) => {
    rows.splice(request(rows, "shutdown"), 0, {
      direction: "request",
      message: { jsonrpc: "2.0", id: 3, method: "textDocument/hover", params: {} },
    });
  },
  "a reused ordinary request ID": (rows) => {
    rows.splice(request(rows, "shutdown"), 0, {
      direction: "request",
      message: { jsonrpc: "2.0", id: 1, method: "textDocument/hover", params: {} },
    });
  },
  "a reply observed before its own request": (rows) => {
    rows.splice(
      request(rows, "shutdown"),
      0,
      { direction: "response", message: { jsonrpc: "2.0", id: 3, result: null } },
      {
        direction: "request",
        message: { jsonrpc: "2.0", id: 3, method: "textDocument/hover", params: {} },
      },
    );
  },
  "a missing shutdown acknowledgement": (rows) => {
    rows.splice(shutdownResponse(rows), 1);
  },
  "a client notification after exit": (rows) => {
    rows.push({
      direction: "request",
      message: { jsonrpc: "2.0", method: "textDocument/didClose", params: {} },
    });
  },
  "a client request after shutdown": (rows) => {
    rows.splice(request(rows, "exit"), 0, {
      direction: "request",
      message: { jsonrpc: "2.0", id: 3, method: "textDocument/hover", params: {} },
    });
  },
};
for (const [label, mutate] of Object.entries(invalidDelayed))
  test(`delayed decoded lifecycle refuses ${label}`, () => {
    const rows = delayed(false);
    mutate(rows);
    assert.throws(() => assertDecodedLspLifecycle(rows, fixture.terminal, delayedAuthority));
  });
