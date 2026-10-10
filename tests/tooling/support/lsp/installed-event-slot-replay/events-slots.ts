import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";
import { applyWorkspaceEdit, utf16Position } from "../installed-alias-replay/apply.ts";
import type { Packet } from "../installed-alias-replay/protocol-types.ts";
import { prepareEventSlotExpectation, type EventSlotCase } from "./events-slots-cases.ts";
export {
  eventSlotSourceShas,
  eventSlotTsconfig,
  eventSlotVizeConfig,
  loadEventSlotCases,
  prepareEventSlotExpectation,
  type EventSlotCase,
} from "./events-slots-cases.ts";

type ObjectValue = Record<string, unknown>;
type EventSlotSession = {
  directory: string;
  notifications: readonly Packet[];
  request(method: string, params?: unknown): Promise<Packet>;
  open(uri: string, text: string): Promise<Packet>;
  change(uri: string, text: string, version: number): Promise<Packet>;
};
export type EventSlotRecord = {
  expected: Readonly<ObjectValue>;
  actual: ObjectValue;
  initialPublications: Packet[];
  changedPublications: Packet[];
  repairedPublications: Packet[];
  referencesReply?: Packet;
  renameReply?: Packet;
  notificationStart: number;
  allNotifications: readonly Packet[];
  allowedWholePublications: readonly Packet[];
  supplementalAllowedWholePublications: readonly Packet[];
  failures: ObjectValue[];
};
function freeze<T>(value: T): T {
  if (value && typeof value === "object") {
    Object.values(value).forEach(freeze);
    Object.freeze(value);
  }
  return value;
}
const uri = (root: string, file: string) => pathToFileURL(path.join(root, file)).href;
const publication = (uriValue: string, version: number) => ({
  jsonrpc: "2.0",
  method: "textDocument/publishDiagnostics",
  params: { uri: uriValue, version, diagnostics: [] },
});
function supplementalGuardPublications(originalUri: string): Packet[] {
  const guardUri = new URL("NativeGuard.vue", originalUri).href;
  return [
    {
      jsonrpc: "2.0",
      method: "textDocument/publishDiagnostics",
      params: {
        uri: guardUri,
        version: 1,
        diagnostics: [
          {
            range: { start: { line: 1, character: 6 }, end: { line: 1, character: 17 } },
            severity: 1,
            code: 2322,
            source: "vize/types",
            message: "Type 'string' is not assignable to type 'number'.",
          },
        ],
      },
    },
    publication(guardUri, 2),
  ];
}
/** Called again after genuine shutdown so late whole publications remain audited. */
export function auditEventSlotRecord(
  record: EventSlotRecord,
  notifications: readonly Packet[],
): ObjectValue[] {
  const failures: ObjectValue[] = [];
  const check = (context: string, actual: unknown, expected: unknown) => {
    if (!isDeepStrictEqual(actual, expected)) failures.push({ context, actual, expected });
  };
  check("complete original transaction", record.actual, record.expected);
  check("references ID2 whole reply", record.referencesReply, {
    jsonrpc: "2.0",
    id: 2,
    result: record.expected.references,
  });
  check("rename ID3 whole reply", record.renameReply, {
    jsonrpc: "2.0",
    id: 3,
    result: record.expected.rename,
  });
  const fileCount = (record.expected.files as unknown[]).length;
  for (const [phase, packets, version] of [
    ["initial", record.initialPublications, 1],
    ["actual V2", record.changedPublications, 2],
    ["independent V3", record.repairedPublications, 3],
  ] as const) {
    check(
      `${phase} complete selected publications`,
      packets,
      record.allowedWholePublications.slice((version - 1) * fileCount, version * fileCount),
    );
  }
  const originalUri = (record.allowedWholePublications[0].params as ObjectValue).uri as string;
  const fixedGuardPublications = supplementalGuardPublications(originalUri);
  for (const packet of record.supplementalAllowedWholePublications)
    if (!fixedGuardPublications.some((allowed) => isDeepStrictEqual(packet, allowed)))
      failures.push({
        context: "supplemental pre-query guard contract",
        actual: packet,
        fixedGuardPublications,
      });
  const streamAllowed = [
    ...record.allowedWholePublications,
    ...record.supplementalAllowedWholePublications,
  ];
  record.allNotifications = notifications.slice(record.notificationStart);
  for (const packet of record.allNotifications)
    if (
      packet.method === "textDocument/publishDiagnostics" &&
      !streamAllowed.some((allowed) => isDeepStrictEqual(packet, allowed))
    )
      failures.push({
        context: "complete original publication stream",
        actual: packet,
        allowedWholePublications: record.allowedWholePublications,
        supplementalAllowedWholePublications: record.supplementalAllowedWholePublications,
      });
  record.failures = failures;
  return failures;
}
export async function observeEventSlotCase(
  session: EventSlotSession,
  testCase: EventSlotCase,
  projectRoot: string,
  progress: (record: EventSlotRecord) => void,
  supplementalAllowedWholePublications: readonly Packet[] = [],
): Promise<EventSlotRecord> {
  const expected = prepareEventSlotExpectation(testCase, projectRoot);
  const fixedGuardPublications = supplementalGuardPublications(
    uri(projectRoot, testCase.files[0].file),
  );
  for (const packet of supplementalAllowedWholePublications)
    assert.ok(
      fixedGuardPublications.some((allowed) => isDeepStrictEqual(packet, allowed)),
      "fixed pre-query supplemental guard envelope",
    );
  const record: EventSlotRecord = {
    expected,
    actual: {},
    initialPublications: [],
    changedPublications: [],
    repairedPublications: [],
    notificationStart: session.notifications.length,
    allNotifications: [],
    supplementalAllowedWholePublications: freeze(
      structuredClone(supplementalAllowedWholePublications),
    ),
    allowedWholePublications: freeze(
      [1, 2, 3].flatMap((version) =>
        testCase.files.map(({ file }) => publication(uri(projectRoot, file), version)),
      ),
    ),
    failures: [],
  };
  const capture = () => progress(record);
  capture();
  for (const { file, text } of testCase.files) {
    record.initialPublications.push(await session.open(uri(projectRoot, file), text));
    capture();
  }
  const source = testCase.files.find(({ file }) => file === testCase.query.file)!.text;
  const params = {
    textDocument: { uri: uri(projectRoot, testCase.query.file) },
    position: utf16Position(source, source.indexOf(testCase.query.needle)),
  };
  const request = async (
    key: "referencesReply" | "renameReply",
    method: string,
    value: unknown,
  ) => {
    try {
      record[key] = await session.request(method, value);
    } catch (error) {
      if (error && typeof error === "object" && "packet" in error)
        record[key] = error.packet as Packet;
      capture();
      throw error;
    }
    capture();
    record.actual[key === "referencesReply" ? "references" : "rename"] = record[key]!.result;
    capture();
  };
  await request("referencesReply", "textDocument/references", {
    ...params,
    context: { includeDeclaration: true },
  });
  await request("renameReply", "textDocument/rename", {
    ...params,
    newName: testCase.query.newName,
  });
  let applied = testCase.files.map(({ file, text }) => ({
    uri: uri(projectRoot, file),
    text,
    version: 1,
  }));
  let applicationError: string | undefined;
  try {
    applied = applyWorkspaceEdit(record.actual.rename, applied);
  } catch (error) {
    applicationError = error instanceof Error ? error.message : String(error);
  }
  for (const [index, { file }] of testCase.files.entries())
    fs.writeFileSync(path.join(projectRoot, file), applied[index].text);
  const results: ObjectValue[] = [];
  record.actual.files = results;
  capture();
  for (const [index, { file }] of testCase.files.entries()) {
    const text = applied[index].text;
    const packet = await session.change(uri(projectRoot, file), text, 2);
    record.changedPublications.push(packet);
    results.push({
      file,
      text,
      disk: fs.readFileSync(path.join(projectRoot, file), "utf8"),
      diagnostics: (packet.params as ObjectValue)?.diagnostics,
      ...(applicationError === undefined ? {} : { applicationError }),
    });
    capture();
  }
  fs.writeFileSync(
    path.join(session.directory, "actual-before-independent-repair.json"),
    JSON.stringify(record, null, 2),
  );
  for (const { file, golden } of testCase.files)
    fs.writeFileSync(path.join(projectRoot, file), golden);
  const repaired: ObjectValue[] = [];
  record.actual.independentRepair = repaired;
  capture();
  for (const { file, golden } of testCase.files) {
    const packet = await session.change(uri(projectRoot, file), golden, 3);
    record.repairedPublications.push(packet);
    repaired.push({
      file,
      text: golden,
      disk: fs.readFileSync(path.join(projectRoot, file), "utf8"),
      version: 3,
      diagnostics: (packet.params as ObjectValue)?.diagnostics,
    });
    capture();
  }
  auditEventSlotRecord(record, session.notifications);
  capture();
  assert.deepEqual(record.failures, [], "complete original event/slot contracts failed");
  return record;
}
