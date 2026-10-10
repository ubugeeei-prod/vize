import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  applyBoundReply,
  auditBoundRecord,
  boundCasesSha256,
  boundFiles,
  boundGuardRepair,
  boundGuardSource,
  boundTsconfig,
  loadFrozenBoundCases,
  prepareBoundExpectation,
  type BoundRecord,
} from "./bound.ts";

const fixtureRoot = fileURLToPath(
  new URL(
    "../../../../_fixtures/differential/lsp/event-rename/4075/bound-emitter/",
    import.meta.url,
  ),
);
const projectRoot = path.join(tmpdir(), "bound pure authored project");
const cases = loadFrozenBoundCases(fixtureRoot);
const childUri = pathToFileURL(path.join(projectRoot, "src/Child.vue")).href;
const parentUri = pathToFileURL(path.join(projectRoot, "src/Parent.vue")).href;

test("byte-authenticated original20 retains its complete order, 660 cursors and independent source/golden bytes", () => {
  assert.equal(
    createHash("sha256")
      .update(fs.readFileSync(path.join(fixtureRoot, "cases.json.txt")))
      .digest("hex"),
    boundCasesSha256,
  );
  assert.equal(cases.length, 20);
  assert.deepEqual(
    cases.map((oracle) => [oracle.form, oracle.newline, oracle.newName]),
    ["record", "call-signature", "runtime-array", "runtime-object", "generic"].flatMap((form) => [
      [form, "\n", "nextEvent"],
      [form, "\n", "next-event"],
      [form, "\r\n", "nextEvent"],
      [form, "\r\n", "next-event"],
    ]),
  );
  assert.equal(
    cases.reduce((count, oracle) => count + oracle.queries.length, 0),
    660,
  );
  assert.ok(
    Object.isFrozen(cases) &&
      Object.isFrozen(cases[0].sources) &&
      Object.isFrozen(cases[0].fullReferences[0].range),
  );
  assert.throws(() => Object.assign(cases[0].sources, { "Other.vue": "changed" }), TypeError);
  assert.equal(boundGuardRepair, boundGuardSource.replace("'wrong'", "1"));
  assert.equal(
    createHash("sha256").update(boundTsconfig).digest("hex"),
    "fb30a305647e8df4e8c4d944b83f99b279c6cddaad1f9ff7379dc3fbff65d4fd",
  );
});

test("altered JSON and independent readable source/golden files fail before any query", () => {
  const temporary = fs.mkdtempSync(path.join(tmpdir(), "bound-oracle-custody-"));
  try {
    fs.cpSync(fixtureRoot, temporary, { recursive: true });
    const file = path.join(temporary, "cases.json.txt"),
      original = fs.readFileSync(file);
    fs.writeFileSync(file, Buffer.concat([original, Buffer.from(" ")]));
    assert.throws(() => loadFrozenBoundCases(temporary));
    fs.writeFileSync(file, original);
    for (const prefix of ["input", "updated"]) {
      const input = path.join(temporary, "record", `${prefix}-Other.vue.txt`),
        bytes = fs.readFileSync(input);
      fs.writeFileSync(input, Buffer.concat([bytes, Buffer.from(" ")]));
      assert.throws(() => loadFrozenBoundCases(temporary));
      fs.writeFileSync(input, bytes);
    }
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

test("whole expectations fix IDs, URI ownership, four ordered references, both legal definition shapes and absence of declaration contracts", () => {
  for (const oracle of cases) {
    const expected = prepareBoundExpectation(oracle, projectRoot);
    assert.ok(Object.isFrozen(expected) && Object.isFrozen(expected.queries));
    assert.equal(expected.queries.length, 33);
    assert.equal(expected.allowedWholeDiagnosticPublications.length, 11);
    assert.equal(expected.declarationOriginDefinitionsObservedOnly, 8);
    assert.equal(expected.selectedApplicationQueryIndex, 0);
    for (const [index, query] of expected.queries.entries()) {
      assert.equal(query.referencesReply.id, 2 + 3 * index);
      assert.equal(query.renameReply.id, 4 + 3 * index);
      assert.equal(query.referencesReply.result.length, 4);
      assert.deepEqual(
        query.referencesReply.result.map((site: { uri: string }) => site.uri),
        [childUri, childUri, parentUri, parentUri],
      );
      if (index < 8) assert.equal(query.allowedWholeDefinitionResponses, null);
      else {
        assert.deepEqual(
          query.allowedWholeDefinitionResponses.map((packet: { id: number }) => packet.id),
          [3 + 3 * index, 3 + 3 * index],
        );
        assert.deepEqual(query.allowedWholeDefinitionResponses[1].result, [
          query.allowedWholeDefinitionResponses[0].result,
        ]);
      }
    }
  }
});

test("complete authored query0 edits apply to every LF/CRLF golden while all negative sources remain unchanged", () => {
  for (const oracle of cases) {
    const expected = prepareBoundExpectation(oracle, projectRoot),
      packet = expected.queries[0].renameReply;
    const before = JSON.stringify({ oracle, packet });
    assert.deepEqual(applyBoundReply(oracle, projectRoot, packet), oracle.completeGoldenFiles);
    assert.equal(JSON.stringify({ oracle, packet }), before);
    assert.equal(oracle.sources["Other.vue"], oracle.completeGoldenFiles["Other.vue"]);
    assert.equal(
      applyBoundReply(oracle, projectRoot, { result: null })["Child.vue"],
      oracle.sources["Child.vue"],
    );
  }
});

test("whole atomic edit refusal rejects foreign operations, overlaps, metadata and invalid authored UTF16 without mutating inputs", () => {
  const oracle = cases[0],
    packet = prepareBoundExpectation(oracle, projectRoot).queries[0].renameReply;
  const malformed = [
    { error: { code: -1, message: "whole error" } },
    {},
    { result: { changes: {}, documentChanges: [] } },
    { result: { changes: { "file:///foreign/Child.vue": [] } } },
    { result: { changes: Object.fromEntries([["__proto__", []]]) } },
    { result: { documentChanges: [{ kind: "create", uri: childUri }] } },
    {
      result: {
        documentChanges: [
          { textDocument: { uri: childUri }, edits: [] },
          { textDocument: { uri: childUri }, edits: [] },
        ],
      },
    },
  ];
  for (const result of malformed) assert.throws(() => applyBoundReply(oracle, projectRoot, result));
  const textEdit = packet.result.changes[childUri][0];
  for (const edits of [
    [textEdit, textEdit],
    [{ ...textEdit, annotationId: "unexpected" }],
    [{ ...textEdit, range: { start: textEdit.range.end, end: textEdit.range.start } }],
  ]) {
    const altered = { result: { changes: { [childUri]: edits } } },
      before = JSON.stringify(altered);
    assert.throws(() => applyBoundReply(oracle, projectRoot, altered));
    assert.equal(JSON.stringify(altered), before);
  }
  assert.throws(
    () =>
      applyBoundReply(oracle, projectRoot, {
        result: {
          changes: {
            [parentUri]: [
              {
                range: { start: { line: 6, character: 11 }, end: { line: 6, character: 12 } },
                newText: "",
              },
            ],
          },
        },
      }),
    /surrogate/,
  );
  assert.deepEqual(applyBoundReply(oracle, projectRoot, packet), oracle.completeGoldenFiles);
});

// These are pure audit inputs derived from independently authored contracts;
// no process, mocked transport or provider is constructed.
function auditInput(): BoundRecord {
  const expected = prepareBoundExpectation(cases[0], projectRoot);
  return {
    expected,
    actual: {
      nativeGuard: expected.nativeGuard,
      initialDiagnostics: expected.initialDiagnostics,
      originalDiskAfterQueries: expected.originalDiskAfterQueries,
      applicationError: null,
      selectedApplicationQueryIndex: 0,
      appliedFiles: expected.version2,
      independentGoldenFiles: expected.independentVersion3,
      observations: expected.queries.map((query: Record<string, any>, index: number) => ({
        query: query.query,
        referencesReply: query.referencesReply,
        renameReply: query.renameReply,
        definitionContracted: query.definitionContracted,
        definitionReply:
          index < 8
            ? {
                jsonrpc: "2.0",
                id: 3 + 3 * index,
                error: { code: -1, message: "uncontracted observation" },
              }
            : query.allowedWholeDefinitionResponses[0],
      })),
    },
    failures: [],
  };
}

test("pure audit preserves every uncontracted whole definition including errors and rejects missing observations or contracted errors", () => {
  const record = auditInput(),
    publications = record.expected.allowedWholeDiagnosticPublications;
  assert.deepEqual(auditBoundRecord(record, publications), []);
  delete record.actual.observations[0].definitionReply;
  assert.ok(
    auditBoundRecord(record, publications).some(
      (failure) => failure.context === "query0 retained definition response id",
    ),
  );
  const contracted = auditInput();
  contracted.actual.observations[8].definitionReply = { jsonrpc: "2.0", id: 27, result: null };
  assert.ok(
    auditBoundRecord(contracted, publications).some(
      (failure) => failure.context === "query8 whole contracted definition",
    ),
  );
  const unknown = auditInput();
  unknown.actual.observations[32].referencesReply = {
    ...unknown.actual.observations[32].referencesReply,
    extra: true,
  };
  assert.ok(
    auditBoundRecord(unknown, publications).some(
      (failure) => failure.context === "query32 referencesReply",
    ),
  );
});

test("complete notification membership retains duplicates and fails late foreign URI/version/diagnostic packets as a whole", () => {
  const record = auditInput(),
    allowed = record.expected.allowedWholeDiagnosticPublications;
  const duplicateStream = [...allowed, allowed[2], allowed[2]],
    before = JSON.stringify(duplicateStream);
  assert.deepEqual(auditBoundRecord(record, duplicateStream), []);
  assert.equal(record.actual.allDiagnosticPublications.length, 13);
  assert.equal(JSON.stringify(duplicateStream), before);
  for (const params of [
    { ...allowed[2].params, uri: "file:///foreign/Child.vue" },
    { ...allowed[2].params, version: 99 },
    { ...allowed[2].params, diagnostics: [{ code: 2322 }] },
  ]) {
    const foreign = { ...allowed[2], params };
    assert.ok(
      auditBoundRecord(record, [...duplicateStream, foreign]).some(
        (failure) => failure.context === "complete diagnostic publication stream",
      ),
    );
    assert.equal(record.actual.allDiagnosticPublications.at(-1), foreign);
  }
  const dropped = auditInput();
  dropped.actual.appliedFiles = dropped.actual.appliedFiles.slice(0, 2);
  assert.ok(
    auditBoundRecord(dropped, allowed).some(
      (failure) => failure.context === "actual all-file version2",
    ),
  );
  assert.deepEqual(boundFiles, ["Child.vue", "Parent.vue", "Other.vue"]);
});
