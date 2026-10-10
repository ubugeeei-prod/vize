import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";
import { applyWorkspaceEdit, utf16Offset, utf16Position } from "../installed-alias-replay/apply.ts";
import type { Packet } from "../installed-alias-replay/protocol.ts";
import {
  libraryGuardSource,
  libraryGuardRepair,
  prepareLibraryExpectation,
  type LibraryCase,
  type LibraryRecord,
  type LibrarySession,
} from "./library-oracles.ts";
export * from "./library-oracles.ts";
type ObjectValue = Record<string, any>;
const names = ["src/Toggle.vue", "src/App.vue"] as const;
const digest = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
const uri = (root: string, file: string) => pathToFileURL(path.join(root, file)).href;

/** Apply returned bytes, selecting a subset only in the two deliberately partial editor controls. */
export function applyLibraryTransaction(
  oracle: LibraryCase,
  rename: unknown,
  projectRoot: string,
): string[] {
  const toggle = oracle.inputs.toggleBuffer ?? oracle.inputs.toggle;
  assert.ok(rename && typeof rename === "object" && !Array.isArray(rename));
  assert.deepEqual(Object.keys(rename), ["changes"], "original complete changes form");
  const changes = (rename as ObjectValue).changes;
  const documents = names.map((file, index) => ({
    uri: uri(projectRoot, file),
    text: index === 0 ? toggle : oracle.inputs.app,
    version: 1,
  }));
  const complete = applyWorkspaceEdit(rename, documents);
  for (const document of documents) {
    assert.ok(Array.isArray(changes[document.uri]), "original required owned edit array");
    for (const edit of changes[document.uri])
      assert.ok(
        utf16Offset(document.text, edit.range.start) < utf16Offset(document.text, edit.range.end),
        "incomplete native edit range",
      );
  }
  if (oracle.kind !== "partial") return complete.map((document) => document.text);
  let applied = rename;
  if (oracle.kind === "partial") {
    const callOnly = oracle.context.includes("CallOnly");
    applied = {
      changes: {
        [uri(projectRoot, names[0])]: [changes[uri(projectRoot, names[0])][callOnly ? 1 : 0]],
        ...(callOnly ? {} : { [uri(projectRoot, names[1])]: changes[uri(projectRoot, names[1])] }),
      },
    };
  }
  return applyWorkspaceEdit(applied, documents).map((document) => document.text);
}
/** Repeat after shutdown. Keep every packet; after a sequence, its final native ledger remains authoritative. */
export function auditLibraryRecord(
  record: LibraryRecord,
  notifications: readonly Packet[],
): ObjectValue[] {
  const failures: ObjectValue[] = [];
  const check = (context: string, actual: unknown, expected: unknown) => {
    if (!isDeepStrictEqual(actual, expected)) failures.push({ context, actual, expected });
  };
  for (const key of [
    "transaction",
    "nativeGuard",
    "initialDisk",
    "initialPublications",
    "responses",
    "publicationSequences",
    "libraryBytesUnchanged",
  ])
    check(key, record.actual[key], record.expected[key]);
  check("controller errors", record.actual.errors, []);
  record.actual.allDiagnosticPublications = notifications.filter(
    (packet) => packet.method === "textDocument/publishDiagnostics",
  );
  const groups = new Map<string, { packets: Packet[]; seen: number }>();
  for (const packets of record.expected.publicationGroups) {
    const params = packets[0].params;
    groups.set(JSON.stringify([params.uri, params.version]), { packets, seen: 0 });
  }
  for (const packet of record.actual.allDiagnosticPublications) {
    const group = groups.get(JSON.stringify([packet.params?.uri, packet.params?.version]));
    if (!group) {
      failures.push({ context: "unowned late publication", actual: packet });
      continue;
    }
    check(
      "whole ordered/late publication",
      packet,
      group.packets[Math.min(group.seen++, group.packets.length - 1)],
    );
  }
  for (const [key, group] of groups)
    if (group.seen < group.packets.length)
      failures.push({
        context: "missing complete publication sequence",
        key,
        expected: group.packets,
        seen: group.seen,
      });
  record.failures = failures;
  return failures;
}

export async function observeLibraryCase(
  session: LibrarySession,
  oracle: LibraryCase,
  projectRoot: string,
  progress: (record: LibraryRecord) => void,
  options: { libraryPath: string; batchWitness?: () => Promise<unknown> },
): Promise<LibraryRecord> {
  const expected = prepareLibraryExpectation(oracle, projectRoot);
  assert.ok(path.isAbsolute(options.libraryPath) && fs.lstatSync(options.libraryPath).isFile());
  assert.equal(fs.realpathSync(options.libraryPath), options.libraryPath);
  assert.equal(path.basename(options.libraryPath), "lib.dom.d.ts");
  const asset = fs.readFileSync(options.libraryPath);
  const actual: ObjectValue = {
    nativeGuard: {},
    initialPublications: [],
    responses: [],
    publicationSequences: [],
    errors: [],
    libraryCustody: { path: options.libraryPath, beforeSha256: digest(asset) },
  };
  const record: LibraryRecord = { expected, actual, failures: [] };
  const capture = () => progress(record);
  const guardPath = path.join(projectRoot, "src/NativeGuard.vue");
  const toggle = oracle.inputs.toggleBuffer ?? oracle.inputs.toggle;
  const input = [toggle, oracle.inputs.app];
  actual.initialDisk = names.map((file) => ({
    file,
    text: fs.readFileSync(path.join(projectRoot, file), "utf8"),
  }));
  const ask = async (method: string, position: unknown, extra: ObjectValue) => {
    const observation: ObjectValue = { method };
    actual.responses.push(observation);
    capture();
    try {
      observation.packet = await session.request(method, {
        textDocument: { uri: uri(projectRoot, names[0]) },
        position,
        ...extra,
      });
    } catch (error) {
      if (error && typeof error === "object" && "packet" in error)
        observation.packet = error.packet;
      throw error;
    } finally {
      capture();
    }
    return observation.packet.result;
  };
  const install = async (
    texts: string[],
    version: number,
    target: ObjectValue[],
    wanted: ObjectValue[],
  ) => {
    names.forEach((file, index) => fs.writeFileSync(path.join(projectRoot, file), texts[index]));
    for (const [index, file] of names.entries()) {
      const row: ObjectValue = {
        file,
        text: texts[index],
        disk: fs.readFileSync(path.join(projectRoot, file), "utf8"),
        ...(version === 3 ? { version } : {}),
      };
      target.push(row);
      capture();
      const opening = file === names[1] && version === 2 && !oracle.parentOpen;
      const packets = opening
        ? [await session.open(uri(projectRoot, file), texts[index], version)]
        : await session.changeWithPublications(uri(projectRoot, file), texts[index], version, 2);
      actual.publicationSequences.push({ file, version, opening, packets });
      row.diagnostics =
        packets.at(-1)?.params && (packets.at(-1)!.params as ObjectValue).diagnostics;
      capture();
      assert.deepEqual(packets, wanted[index].packets, "whole prompt/native publication sequence");
    }
  };
  capture();
  try {
    assert.deepEqual(
      actual.initialDisk,
      expected.initialDisk,
      "original authored disk before queries",
    );
    fs.mkdirSync(path.dirname(guardPath), { recursive: true });
    fs.writeFileSync(guardPath, libraryGuardSource);
    actual.nativeGuard.invalid = await session.open(
      uri(projectRoot, "src/NativeGuard.vue"),
      libraryGuardSource,
    );
    capture();
    assert.deepEqual(
      actual.nativeGuard.invalid,
      expected.nativeGuard.invalid,
      "whole native guard",
    );
    actual.nativeGuard.repaired = await session.changeWithPublications(
      uri(projectRoot, "src/NativeGuard.vue"),
      libraryGuardRepair,
      2,
      2,
    );
    capture();
    assert.deepEqual(
      actual.nativeGuard.repaired,
      expected.nativeGuard.repaired,
      "completed native guard repair",
    );
    for (const [index, file] of names.slice(0, oracle.parentOpen ? 2 : 1).entries()) {
      actual.initialPublications.push(await session.open(uri(projectRoot, file), input[index]));
      capture();
    }
    if (oracle.kind === "ambiguous") {
      actual.transaction = [];
      for (const name of ["update", 'up"date', "up'date", "up\\date"]) {
        const rename = await ask(
          "textDocument/rename",
          { line: 6, character: 9 },
          { newName: name },
        );
        actual.transaction.push({
          newName: name,
          rename,
          toggleDisk: fs.readFileSync(path.join(projectRoot, names[0]), "utf8"),
          appDisk: fs.readFileSync(path.join(projectRoot, names[1]), "utf8"),
          libraryBytesUnchanged: fs.readFileSync(options.libraryPath).equals(asset),
        });
        capture();
      }
    } else {
      const needle = oracle.kind === "cold" ? 'change", change' : 'hange", true';
      const offset = toggle.indexOf(needle);
      assert.ok(offset >= 0 && toggle.indexOf(needle, offset + 1) < 0);
      const position = utf16Position(toggle, offset);
      const transaction: ObjectValue = {};
      actual.transaction = transaction;
      transaction.rename = await ask("textDocument/rename", position, { newName: "update" });
      if (oracle.kind !== "partial")
        transaction.references = await ask("textDocument/references", position, {
          context: { includeDeclaration: true },
        });
      else
        assert.deepEqual(
          transaction.rename,
          expected.transaction.rename,
          "the provider returns the entire safe transaction",
        );
      let applied = input;
      const applicationErrors: ObjectValue[] = [];
      try {
        applied = applyLibraryTransaction(oracle, transaction.rename, projectRoot);
      } catch (error) {
        applicationErrors.push({ error: String(error) });
      }
      if (oracle.kind !== "partial") transaction.applicationErrors = applicationErrors;
      else if (applicationErrors.length) actual.errors.push(...applicationErrors);
      const target = oracle.kind === "partial" ? "partialFiles" : "files";
      transaction[target] = [];
      await install(applied, 2, transaction[target], expected.publicationSequences.slice(0, 2));
      capture();
      if (oracle.kind === "partial" && options.batchWitness) {
        actual.batchWitness = {
          scope:
            "Observed-only complete public batch bytes/status; no authored whole batch golden or transferred LSP/native qualification.",
          observation: await options.batchWitness(),
        };
        capture();
      }
      transaction.independentRepair = [];
      await install(
        expected.transaction.independentRepair.map((row: ObjectValue) => row.text),
        3,
        transaction.independentRepair,
        expected.publicationSequences.slice(2),
      );
    }
    actual.nativeGuard.disk = fs.readFileSync(guardPath, "utf8");
    actual.libraryCustody.afterSha256 = digest(fs.readFileSync(options.libraryPath));
    actual.libraryBytesUnchanged = fs.readFileSync(options.libraryPath).equals(asset);
    auditLibraryRecord(record, session.notifications);
    capture();
    assert.deepEqual(record.failures, [], "whole library22 contracts failed");
    return record;
  } catch (error) {
    actual.errors.push({
      error: String(error),
      ...(error && typeof error === "object" && "packet" in error ? { packet: error.packet } : {}),
    });
    capture();
    throw error;
  }
}
