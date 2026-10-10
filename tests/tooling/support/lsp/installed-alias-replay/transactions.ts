import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { applyWorkspaceEdit, utf16Position } from "./apply.ts";
import type { FrozenAliasCase } from "./cases.ts";
import { InstalledAliasSession, type Packet } from "./protocol.ts";

type ObjectValue = Record<string, unknown>;
export function rpcResult(packet: Packet): unknown {
  assert.equal(packet.jsonrpc, "2.0");
  assert.ok(Number.isSafeInteger(packet.id));
  assert.deepEqual(Object.keys(packet).toSorted(), ["id", "jsonrpc", "result"]);
  return packet.result;
}
function diagnostics(packet: Packet, uri: string, version: number): unknown {
  assert.equal(packet.jsonrpc, "2.0");
  assert.equal(packet.method, "textDocument/publishDiagnostics");
  const params = packet.params as ObjectValue;
  assert.equal(params.uri, uri);
  assert.equal(params.version, version);
  assert.ok(Array.isArray(params.diagnostics));
  return params.diagnostics;
}
function query(source: string, needle: string, uri: string) {
  const offset = source.indexOf(needle);
  assert.ok(
    offset >= 0 && source.indexOf(needle, offset + 1) < 0,
    "one exact authored query origin",
  );
  return { textDocument: { uri }, position: utf16Position(source, offset) };
}

/** Whole actual edits are applied before independent goldens are installed. */
export async function observeFrozenCase(
  session: InstalledAliasSession,
  testCase: FrozenAliasCase,
  expected: Readonly<ObjectValue>,
  projectRoot: string,
  progress: (actual: ObjectValue) => void,
): Promise<ObjectValue> {
  const ePath = path.join(projectRoot, "E.vue");
  const eUri = pathToFileURL(ePath).href;
  const guard = expected.nativeGuard as { source: string; repairedSource: string };
  const guardPath = path.join(projectRoot, "NativeGuard.vue");
  const guardUri = pathToFileURL(guardPath).href;
  const nativeGuard: ObjectValue = { source: guard.source, repairedSource: guard.repairedSource };
  const actual: ObjectValue = { nativeGuard };
  const capture = () => {
    progress(actual);
    fs.writeFileSync(
      path.join(session.directory, "phase-observations.json"),
      JSON.stringify(actual, null, 2),
    );
  };
  capture();
  fs.writeFileSync(guardPath, guard.source);
  const invalidDiagnostics = diagnostics(await session.open(guardUri, guard.source), guardUri, 1);
  nativeGuard.invalidDiagnostics = invalidDiagnostics;
  capture();
  fs.writeFileSync(guardPath, guard.repairedSource);
  const repairedDiagnostics = diagnostics(
    await session.change(guardUri, guard.repairedSource, 2),
    guardUri,
    2,
  );
  nativeGuard.repairedDiagnostics = repairedDiagnostics;
  capture();
  const initialDiagnostics = diagnostics(await session.open(eUri, testCase.input), eUri, 1);
  if (testCase.kind === "definitions") {
    const definitions: ObjectValue[] = [];
    actual.initialDiagnostics = initialDiagnostics;
    actual.definitions = definitions;
    capture();
    for (const item of testCase.definitionQueries) {
      const definition = rpcResult(
        await session.request("textDocument/definition", query(testCase.input, item.query, eUri)),
      );
      definitions.push({ name: item.name, query: item.query, definition });
      capture();
    }
    return actual;
  }
  const transaction: ObjectValue = {};
  actual.transaction = transaction;
  capture();
  const references = rpcResult(
    await session.request("textDocument/references", {
      ...query(testCase.input, testCase.query, eUri),
      context: { includeDeclaration: true },
    }),
  );
  transaction.references = references;
  capture();
  const rename = rpcResult(
    await session.request("textDocument/rename", {
      ...query(testCase.input, testCase.query, eUri),
      newName: testCase.newName,
    }),
  );
  transaction.rename = rename;
  capture();
  let text = testCase.input;
  let applicationError: string | undefined;
  try {
    text = applyWorkspaceEdit(rename, [{ uri: eUri, text, version: 1 }])[0].text;
  } catch (error) {
    applicationError = error instanceof Error ? error.message : String(error);
  }
  fs.writeFileSync(ePath, text);
  const version2 = diagnostics(await session.change(eUri, text, 2), eUri, 2);
  const actualFile = {
    file: "E.vue",
    text,
    disk: fs.readFileSync(ePath, "utf8"),
    diagnostics: version2,
    ...(applicationError === undefined ? {} : { applicationError }),
  };
  transaction.files = [actualFile];
  capture();
  // Complete actual response/application/V2 publication remains in the session and
  // output record before the independent source is written; no expected edit is applied.
  fs.writeFileSync(
    path.join(session.directory, "actual-before-independent-repair.json"),
    JSON.stringify({ references, rename, initialDiagnostics, file: actualFile }, null, 2),
  );
  fs.writeFileSync(ePath, testCase.golden);
  const version3 = diagnostics(await session.change(eUri, testCase.golden, 3), eUri, 3);
  const independentRepair = [
    {
      file: "E.vue",
      text: testCase.golden,
      disk: fs.readFileSync(ePath, "utf8"),
      version: 3,
      diagnostics: version3,
    },
  ];
  transaction.independentRepair = independentRepair;
  capture();
  assert.deepEqual(
    initialDiagnostics,
    [],
    "original stock initial diagnostics remain a whole empty array",
  );
  return actual;
}
