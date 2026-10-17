import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import test from "node:test";
import { assertDecodedLspLifecycle } from "../../tools/support/release/public_acceptance/lsp_packets.ts";

const fixture = JSON.parse(
  fs.readFileSync(
    new URL("../_fixtures/release/public-lsp-clean-eof.json", import.meta.url),
    "utf8",
  ),
);
const original = (): ReturnType<typeof JSON.parse>[] =>
  fixture.protocol
    .trim()
    .split("\n")
    .map((row: string) => JSON.parse(row));
type Ledger = ReturnType<typeof original>;
const request = (rows: Ledger, method: string) =>
  rows.findIndex((row) => row.direction === "request" && row.message.method === method);
const shutdownResponse = (rows: Ledger) => {
  const id = rows[request(rows, "shutdown")].message.id;
  return rows.findIndex((row) => row.direction === "response" && row.message.id === id);
};

test("actual public decoded ledger admits only its clean shutdown EOF without rewriting rows", () => {
  assert.equal(
    createHash("sha256").update(fixture.protocol).digest("hex"),
    fixture.provenance.protocolSha256,
  );
  assert.equal(fixture.provenance.version, "0.457.0");
  const rows = original(),
    before = structuredClone(rows);
  assertDecodedLspLifecycle(rows, fixture.terminal);
  assert.deepEqual(rows, before);
  assert.equal(rows.length, 95);
  assert.equal(rows.at(-1).direction, "stdoutError");
});

const invalid: Record<string, (rows: Ledger) => void> = {
  "a missing EOF event": (rows) => {
    rows.pop();
  },
  "a duplicated EOF event": (rows) => {
    rows.push(structuredClone(rows.at(-1)));
  },
  "an early reader EOF even with a successful later shutdown": (rows) => {
    rows.splice(3, 0, structuredClone(rows.at(-1)));
  },
  "a truncated response body with the same error kind": (rows) => {
    rows.at(-1).message.message = "failed to fill whole buffer";
  },
  "a missing Content-Length error": (rows) => {
    rows.at(-1).message = { kind: "InvalidData", message: "missing Content-Length header" };
  },
  "a different reader error kind": (rows) => {
    rows.at(-1).message.kind = "BrokenPipe";
  },
  "a stderr reader error": (rows) => {
    rows.at(-1).direction = "stderrError";
  },
  "extra unverified EOF fields": (rows) => {
    rows.at(-1).message.extra = true;
  },
  "a response after EOF": (rows) => {
    rows.push(structuredClone(rows[1]));
  },
  "EOF before the exit notification": (rows) => {
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
    assert.throws(() => assertDecodedLspLifecycle(rows, fixture.terminal));
  });

for (const [key, value] of [
  ["success", false],
  ["exitCode", 1],
  ["exitCode", null],
  ["stdoutReaderJoined", false],
  ["stderrReaderJoined", false],
] as const)
  test(`clean EOF cannot qualify terminal ${key}=${value}`, () => {
    assert.throws(() =>
      assertDecodedLspLifecycle(original(), { ...fixture.terminal, [key]: value }),
    );
  });
