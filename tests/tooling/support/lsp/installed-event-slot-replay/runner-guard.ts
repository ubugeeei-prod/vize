import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";
import type { InstalledAliasSession, Packet } from "../installed-alias-replay/protocol.ts";

export const eventGuardSource =
  "<script setup lang=\"ts\">\nconst nativeGuard: number = 'wrong';\n</script>\n<template>{{ nativeGuard }}</template>\n";
export const eventGuardRepair = eventGuardSource.replace("'wrong'", "1");
export type EventGuardExpectation = {
  source: string;
  repairedSource: string;
  invalid: Packet;
  repaired: Packet;
};
export type EventGuardRecord = {
  expected: EventGuardExpectation;
  invalid?: Packet;
  repaired?: Packet;
  disk?: string;
  allPublications: Packet[];
  failures: Array<Record<string, unknown>>;
};
function freeze<T>(value: T): T {
  if (value && typeof value === "object") {
    Object.values(value).forEach(freeze);
    Object.freeze(value);
  }
  return value;
}

/** A separate authored witness; it is never added to the original18 whole transaction. */
export function prepareEventGuard(projectRoot: string): EventGuardExpectation {
  const uri = pathToFileURL(path.join(projectRoot, "NativeGuard.vue")).href;
  const publication = (version: number, diagnostics: unknown[]) => ({
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: { uri, version, diagnostics },
  });
  return freeze({
    source: eventGuardSource,
    repairedSource: eventGuardRepair,
    invalid: publication(1, [
      {
        range: { start: { line: 1, character: 6 }, end: { line: 1, character: 17 } },
        severity: 1,
        code: 2322,
        source: "vize/types",
        message: "Type 'string' is not assignable to type 'number'.",
      },
    ]),
    repaired: publication(2, []),
  });
}

/** Notifications only: initialize1/references2/rename3/shutdown4 stay unchanged. */
export async function observeEventGuard(
  session: InstalledAliasSession,
  projectRoot: string,
  expected: EventGuardExpectation,
  progress: (record: EventGuardRecord) => void,
): Promise<EventGuardRecord> {
  const file = path.join(projectRoot, "NativeGuard.vue");
  const uri = pathToFileURL(file).href;
  const record: EventGuardRecord = { expected, allPublications: [], failures: [] };
  progress(record);
  fs.writeFileSync(file, expected.source);
  record.invalid = await session.open(uri, expected.source);
  progress(record);
  assert.deepEqual(record.invalid, expected.invalid, "whole separate native invalid witness");
  fs.writeFileSync(file, expected.repairedSource);
  record.repaired = await session.change(uri, expected.repairedSource, 2);
  record.disk = fs.readFileSync(file, "utf8");
  progress(record);
  assert.deepEqual(record.repaired, expected.repaired, "whole separate native repair witness");
  assert.equal(record.disk, expected.repairedSource);
  return record;
}

/** Late and duplicate guard publications are retained and audited after actual shutdown. */
export function auditEventGuard(record: EventGuardRecord, notifications: readonly Packet[]) {
  const failures: Array<Record<string, unknown>> = [];
  const check = (context: string, actual: unknown, expected: unknown) => {
    if (!isDeepStrictEqual(actual, expected)) failures.push({ context, actual, expected });
  };
  check("whole invalid guard publication", record.invalid, record.expected.invalid);
  check("whole repaired guard publication", record.repaired, record.expected.repaired);
  check("complete repaired guard disk", record.disk, record.expected.repairedSource);
  const uri = (record.expected.invalid.params as Record<string, unknown>).uri;
  record.allPublications = notifications.filter(
    (packet) =>
      packet.method === "textDocument/publishDiagnostics" &&
      (packet.params as Record<string, unknown>)?.uri === uri,
  );
  for (const packet of record.allPublications)
    if (
      ![record.expected.invalid, record.expected.repaired].some((p) => isDeepStrictEqual(p, packet))
    )
      failures.push({ context: "complete separate guard publication stream", actual: packet });
  record.failures = failures;
  return failures;
}
