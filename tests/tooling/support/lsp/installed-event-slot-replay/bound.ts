import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";
import { utf16Offset, type Position } from "../installed-alias-replay/apply.ts";
import type { InstalledAliasSession, Packet } from "../installed-alias-replay/protocol.ts";

export const boundFiles = Object.freeze(["Child.vue", "Parent.vue", "Other.vue"] as const);
export const boundCasesSha256 = "1e548804413bd7bc623333a559213a8089a9c4ff04181219e32b82301b5ab462";
export const boundGuardSource =
  "<script setup lang=\"ts\">\nconst nativeGuard: number = 'wrong';\n</script>\n<template>{{ nativeGuard }}</template>\n";
export const boundGuardRepair = boundGuardSource.replace("'wrong'", "1");
const stockConfig =
  "../../../../../crates/vize/tests/fixtures/content_mapper_project/tsconfig.json";
export const boundTsconfig = fs.readFileSync(new URL(stockConfig, import.meta.url), "utf8");
assert.equal(
  createHash("sha256").update(boundTsconfig).digest("hex"),
  "fb30a305647e8df4e8c4d944b83f99b279c6cddaad1f9ff7379dc3fbff65d4fd",
);
export const boundVizeConfig = freeze({
  experimentals: { patternedTemplate: false },
  typeChecker: { checkFallthroughAttrs: false, optionsApi: false },
  lsp: { lint: false, typecheck: true, hover: true, crossFile: true },
});
type BoundFile = (typeof boundFiles)[number];
type Range = { start: Position; end: Position };
type Site = { file: BoundFile; range: Range };
type Query = { file: BoundFile; position: Position };
type Files = Record<BoundFile, string>;
type ObjectValue = Record<string, any>;
export type BoundOracle = {
  form: string;
  newline: "\n" | "\r\n";
  newName: string;
  sources: Files;
  fullReferences: Site[];
  queries: Query[];
  fullEdits: (Site & { newText: string })[];
  completeGoldenFiles: Files;
  expectedVersion2Diagnostics: Record<BoundFile, unknown[]>;
  expectedIndependentVersion3Diagnostics: Record<BoundFile, unknown[]>;
  definitionFromAllParentAndCallPositions: Site[];
  completeNegativePreservation: string;
};
export type BoundRecord = { expected: ObjectValue; actual: ObjectValue; failures: ObjectValue[] };
type BoundSession = InstalledAliasSession & {
  requestObserved(method: string, params?: unknown): Promise<Packet>;
};
function freeze<T>(value: T): T {
  if (value && typeof value === "object") {
    Object.values(value).forEach(freeze);
    Object.freeze(value);
  }
  return value;
}
/** The byte-authenticated pre-query oracle, never a provider result, supplies all expectations. */
export function loadFrozenBoundCases(fixtureRoot: string): readonly BoundOracle[] {
  const bytes = fs.readFileSync(path.join(fixtureRoot, "cases.json.txt"));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), boundCasesSha256);
  const document = JSON.parse(bytes.toString("utf8"));
  for (const oracle of document.cases) {
    for (const name of boundFiles) {
      for (const prefix of ["input", "updated"]) {
        const field = prefix === "input" ? "sources" : "completeGoldenFiles";
        const source = fs.readFileSync(
          path.join(fixtureRoot, oracle.form, `${prefix}-${name}.txt`),
          "utf8",
        );
        assert.equal(source.replaceAll("\n", oracle.newline), oracle[field][name]);
      }
    }
  }
  return freeze(document.cases);
}
const uri = (root: string, name: string) => pathToFileURL(path.join(root, "src", name)).href;
const publication = (root: string, name: string, version: number, diagnostics: unknown) => ({
  jsonrpc: "2.0",
  method: "textDocument/publishDiagnostics",
  params: { uri: uri(root, name), version, diagnostics },
});
const reply = (id: number, result: unknown) => ({ jsonrpc: "2.0", id, result });
const guardDiagnostic = JSON.parse(
  '[{"range":{"start":{"line":1,"character":6},"end":{"line":1,"character":17}},"severity":1,"code":2322,"source":"vize/types","message":"Type \'string\' is not assignable to type \'number\'."}]',
);

export function prepareBoundExpectation(oracle: BoundOracle, projectRoot: string): ObjectValue {
  assert.ok(path.isAbsolute(projectRoot));
  const location = (site: Site) => ({ uri: uri(projectRoot, site.file), range: site.range });
  const references = oracle.fullReferences.map(location);
  const changes: Record<string, unknown[]> = {};
  for (const edit of oracle.fullEdits)
    (changes[uri(projectRoot, edit.file)] ??= []).push({
      range: edit.range,
      newText: edit.newText,
    });
  const definition = location(oracle.definitionFromAllParentAndCallPositions[0]);
  const files = (version: number) =>
    boundFiles.map((file) => ({
      file,
      text: oracle.completeGoldenFiles[file],
      disk: oracle.completeGoldenFiles[file],
      version,
      diagnostics: publication(projectRoot, file, version, []),
    }));
  const invalid = publication(projectRoot, "NativeGuard.vue", 1, guardDiagnostic);
  const repaired = publication(projectRoot, "NativeGuard.vue", 2, []);
  return freeze({
    nativeGuard: {
      invalidDiagnostics: invalid,
      repairedDiagnostics: repaired,
      expectedInvalid: invalid,
    },
    initialDiagnostics: boundFiles.map((file) => publication(projectRoot, file, 1, [])),
    originalDiskAfterQueries: boundFiles.map((file) => ({ file, text: oracle.sources[file] })),
    queries: oracle.queries.map((query, index) => ({
      query,
      referencesReply: reply(2 + 3 * index, references),
      renameReply: reply(4 + 3 * index, { changes }),
      definitionContracted: index >= 8,
      allowedWholeDefinitionResponses:
        index >= 8 ? [reply(3 + 3 * index, definition), reply(3 + 3 * index, [definition])] : null,
    })),
    selectedApplicationQueryIndex: 0,
    version2: files(2),
    independentVersion3: files(3),
    declarationOriginDefinitionsObservedOnly: 8,
    allowedWholeDiagnosticPublications: [
      invalid,
      repaired,
      ...boundFiles.flatMap((file) => [
        publication(projectRoot, file, 1, []),
        publication(projectRoot, file, 2, oracle.expectedVersion2Diagnostics[file]),
        publication(projectRoot, file, 3, oracle.expectedIndependentVersion3Diagnostics[file]),
      ]),
    ],
  });
}
function object(value: unknown): ObjectValue {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as ObjectValue;
}
/** Apply the complete selected reply atomically, preserving the original Rust refusal boundary. */
export function applyBoundReply(oracle: BoundOracle, projectRoot: string, packet: Packet): Files {
  assert.ok(!("error" in packet), "whole rename error");
  assert.ok("result" in packet, "missing whole rename result");
  if (packet.result === null) return { ...oracle.sources };
  const workspace = object(packet.result),
    entries: ObjectValue = Object.create(null);
  if (isDeepStrictEqual(Object.keys(workspace), ["changes"]))
    Object.assign(entries, object(workspace.changes));
  else {
    assert.deepEqual(Object.keys(workspace), ["documentChanges"]);
    assert.ok(Array.isArray(workspace.documentChanges));
    for (const document of workspace.documentChanges) {
      assert.deepEqual(Object.keys(object(document)).toSorted(), ["edits", "textDocument"]);
      const target = object(document.textDocument).uri;
      assert.equal(typeof target, "string");
      assert.ok(!Object.hasOwn(entries, target), "duplicate document transaction");
      entries[target] = document.edits;
    }
  }
  assert.ok(
    Object.keys(entries).every((target) =>
      boundFiles.some((file) => uri(projectRoot, file) === target),
    ),
    "unowned edit target",
  );
  const result = { ...oracle.sources };
  for (const file of boundFiles) {
    const source = oracle.sources[file],
      edits = entries[uri(projectRoot, file)] ?? [];
    assert.ok(Array.isArray(edits));
    const spans = edits
      .map((edit: unknown) => {
        const value = object(edit);
        assert.deepEqual(Object.keys(value).toSorted(), ["newText", "range"]);
        assert.equal(typeof value.newText, "string");
        const start = utf16Offset(source, value.range.start),
          end = utf16Offset(source, value.range.end);
        assert.ok(start <= end, "reversed edit");
        return { start, end, text: value.newText as string };
      })
      .sort((left, right) => left.start - right.start || left.end - right.end);
    assert.ok(
      spans.every((span, index) => index === 0 || spans[index - 1].end <= span.start),
      "overlapping actual edits",
    );
    for (const span of spans.toReversed())
      result[file] = result[file].slice(0, span.start) + span.text + result[file].slice(span.end);
  }
  return result;
}
/** Call again after shutdown: duplicates remain intact and late publications are checked too. */
export function auditBoundRecord(
  record: BoundRecord,
  notifications: readonly Packet[],
): ObjectValue[] {
  record.actual.allDiagnosticPublications = notifications.filter(
    (packet) => packet.method === "textDocument/publishDiagnostics",
  );
  const failures: ObjectValue[] = [];
  const check = (context: string, actual: unknown, expected: unknown) => {
    if (!isDeepStrictEqual(actual, expected)) failures.push({ context, actual, expected });
  };
  const allowed = (packet: unknown, candidates: readonly unknown[]) =>
    candidates.some((candidate) => isDeepStrictEqual(candidate, packet));
  const { actual, expected } = record;
  for (const key of "nativeGuard initialDiagnostics originalDiskAfterQueries selectedApplicationQueryIndex".split(
    " ",
  ))
    check(key, actual[key], expected[key]);
  check("whole application refusal", actual.applicationError, null);
  check("actual all-file version2", actual.appliedFiles, expected.version2);
  check(
    "independent all-file version3",
    actual.independentGoldenFiles,
    expected.independentVersion3,
  );
  check("complete query count", actual.observations.length, 33);
  for (const [index, wanted] of expected.queries.entries()) {
    const observed = actual.observations[index];
    for (const key of ["query", "referencesReply", "renameReply", "definitionContracted"])
      check(`query${index} ${key}`, observed?.[key], wanted[key]);
    check(
      `query${index} retained definition response id`,
      observed?.definitionReply?.id,
      3 + 3 * index,
    );
    if (
      wanted.definitionContracted &&
      !allowed(observed?.definitionReply, wanted.allowedWholeDefinitionResponses)
    )
      failures.push({
        context: `query${index} whole contracted definition`,
        actual: observed?.definitionReply,
        allowedWholeResponses: wanted.allowedWholeDefinitionResponses,
      });
  }
  for (const packet of actual.allDiagnosticPublications)
    if (!allowed(packet, expected.allowedWholeDiagnosticPublications))
      failures.push({
        context: "complete diagnostic publication stream",
        actual: packet,
        allowedWholePublications: expected.allowedWholeDiagnosticPublications,
      });
  record.failures = failures;
  return failures;
}

export async function observeBoundCase(
  session: BoundSession,
  oracle: BoundOracle,
  projectRoot: string,
  progress: (record: BoundRecord) => void,
): Promise<BoundRecord> {
  const expected = prepareBoundExpectation(oracle, projectRoot);
  const actual: ObjectValue = {
    nativeGuard: { expectedInvalid: expected.nativeGuard.expectedInvalid },
    initialDiagnostics: [],
    observations: [],
    selectedApplicationQueryIndex: 0,
    applicationError: null,
    appliedFiles: [],
    independentGoldenFiles: [],
  };
  const record: BoundRecord = { expected, actual, failures: [] };
  const capture = () => progress(record);
  capture();
  const guardPath = path.join(projectRoot, "src/NativeGuard.vue"),
    guardUri = uri(projectRoot, "NativeGuard.vue");
  fs.writeFileSync(guardPath, boundGuardSource);
  actual.nativeGuard.invalidDiagnostics = await session.open(guardUri, boundGuardSource);
  capture();
  fs.writeFileSync(guardPath, boundGuardRepair);
  actual.nativeGuard.repairedDiagnostics = await session.change(guardUri, boundGuardRepair, 2);
  capture();
  for (const file of boundFiles) {
    actual.initialDiagnostics.push(
      await session.open(uri(projectRoot, file), oracle.sources[file]),
    );
    capture();
  }
  for (const [index, query] of oracle.queries.entries()) {
    const observation: ObjectValue = { query, definitionContracted: index >= 8 };
    actual.observations.push(observation);
    capture();
    const params = {
      textDocument: { uri: uri(projectRoot, query.file) },
      position: query.position,
    };
    const ask = async (key: string, method: string, value: unknown, observed = false) => {
      try {
        observation[key] = await (observed
          ? session.requestObserved(method, value)
          : session.request(method, value));
        capture();
      } catch (error) {
        const packet =
          error && typeof error === "object" && "packet" in error ? error.packet : undefined;
        if (packet !== undefined) observation[key] = packet;
        capture();
        throw error;
      }
    };
    await ask("referencesReply", "textDocument/references", {
      ...params,
      context: { includeDeclaration: true },
    });
    await ask("definitionReply", "textDocument/definition", params, index < 8);
    await ask("renameReply", "textDocument/rename", { ...params, newName: oracle.newName });
  }
  actual.originalDiskAfterQueries = boundFiles.map((file) => ({
    file,
    text: fs.readFileSync(path.join(projectRoot, "src", file), "utf8"),
  }));
  capture();
  let files: Files;
  try {
    files = applyBoundReply(oracle, projectRoot, actual.observations[0].renameReply);
  } catch (error) {
    actual.applicationError = String(error);
    files = { ...oracle.sources };
  }
  capture();
  const install = async (sources: Files, version: number, target: ObjectValue[]) => {
    for (const file of boundFiles)
      fs.writeFileSync(path.join(projectRoot, "src", file), sources[file]);
    for (const file of boundFiles) {
      const entry: ObjectValue = {
        file,
        text: sources[file],
        disk: fs.readFileSync(path.join(projectRoot, "src", file), "utf8"),
        version,
      };
      target.push(entry);
      capture();
      entry.diagnostics = await session.change(uri(projectRoot, file), sources[file], version);
      capture();
    }
  };
  await install(files, 2, actual.appliedFiles);
  // Complete actual reply/application/V2 is persisted before the independent V3.
  await install(oracle.completeGoldenFiles, 3, actual.independentGoldenFiles);
  auditBoundRecord(record, session.notifications);
  capture();
  assert.deepEqual(record.failures, [], "whole bound-event contracts failed");
  return record;
}
