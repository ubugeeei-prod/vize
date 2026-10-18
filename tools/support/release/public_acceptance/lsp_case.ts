import assert from "node:assert/strict";

type Json = ReturnType<typeof JSON.parse>;
const files = ["Child.vue", "Parent.vue", "Other.vue"];
const methods = ["prepareRename", "references", "definition", "rename"];
export function bindExpected(value: Json, uris: Record<string, string>): Json {
  if (typeof value === "string" && value.startsWith("$")) {
    assert.ok(files.includes(value.slice(1)), "unknown expected URI marker");
    return uris[value.slice(1)];
  }
  if (Array.isArray(value)) return value.map((entry) => bindExpected(entry, uris));
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value).map(([key, entry]) => {
        if (key.startsWith("$")) {
          assert.ok(files.includes(key.slice(1)), "unknown expected URI key");
          key = uris[key.slice(1)];
        }
        return [key, bindExpected(entry, uris)];
      }),
    );
  return value;
}
const publication = (uri: string, version: number, diagnostics: Json) => ({
  jsonrpc: "2.0",
  method: "textDocument/publishDiagnostics",
  params: { uri, version, diagnostics },
});
const reply = (id: number, result: Json) => ({ jsonrpc: "2.0", id, result });
/** One independent whole-envelope authority shared by case and lifecycle checks. */
export function diagnosticContract(uris: Record<string, string>) {
  const guardUri = uris["NativeGuard.vue"];
  const invalid = publication(guardUri, 1, [
    {
      range: { start: { line: 1, character: 6 }, end: { line: 1, character: 17 } },
      severity: 1,
      code: 2322,
      source: "vize/types",
      message: "Type 'string' is not assignable to type 'number'.",
    },
  ]);
  const repaired = publication(guardUri, 2, []);
  const allowed = [
    invalid,
    repaired,
    ...files.flatMap((file) => [1, 2, 3].map((version) => publication(uris[file], version, []))),
  ];
  return { invalid, repaired, allowed };
}
export function assertWholeCase(
  packet: Json,
  oracle: Json,
  uris: Record<string, string>,
  model: boolean,
) {
  assert.deepEqual(packet.oracle, oracle, "whole literal pre-query corpus");
  const actual = packet.actual;
  const { invalid, repaired, allowed } = diagnosticContract(uris);
  assert.deepEqual(actual.nativeGuard, {
    invalidDiagnostics: invalid,
    repairedDiagnostics: repaired,
    expectedInvalid: invalid,
  });
  assert.deepEqual(
    actual.initialDiagnostics,
    files.map((name) => publication(uris[name], 1, [])),
  );
  assert.deepEqual(
    actual.originalDiskAfterQueries,
    files.map((file) => ({ file, text: oracle.sources[file] })),
  );
  assert.equal(actual.applicationError, null);
  assert.equal(actual.selectedApplicationQueryIndex, 0);
  const installed = (version: number) =>
    files.map((file) => ({
      file,
      text: oracle.completeGoldenFiles[file],
      disk: oracle.completeGoldenFiles[file],
      version,
      diagnostics: publication(uris[file], version, []),
    }));
  assert.deepEqual(actual.appliedFiles, installed(2));
  assert.deepEqual(actual.independentGoldenFiles, installed(3));
  assert.equal(oracle.sources["Other.vue"], oracle.completeGoldenFiles["Other.vue"]);
  assert.ok(Array.isArray(actual.allDiagnosticPublications));
  for (const message of actual.allDiagnosticPublications)
    assert.ok(
      allowed.some((expected) => {
        try {
          assert.deepEqual(message, expected);
          return true;
        } catch {
          return false;
        }
      }),
      "whole diagnostic stream contains an unauthorized publication",
    );
  const queries = bindExpected(oracle, uris).queries;
  assert.equal(actual.observations.length, queries.length);
  let wanted;
  if (model) {
    wanted = queries;
    for (const [index, query] of queries.entries()) {
      const observation = actual.observations[index];
      assert.deepEqual(observation.query, query);
      assert.deepEqual(Object.keys(observation.replies).sort(), [...methods].sort());
      for (const method of methods)
        assert.ok(
          query.allowedWholeReplies[method].some((expected: Json) => {
            try {
              assert.deepEqual(observation.replies[method], expected);
              return true;
            } catch {
              return false;
            }
          }),
          `whole ${method} response`,
        );
    }
    assert.equal(packet.allDefinitionOriginsContracted, true);
  } else {
    const references = oracle.fullReferences.map((site: Json) => ({
      uri: uris[site.file],
      range: site.range,
    }));
    const changes: Record<string, Json[]> = {};
    for (const edit of oracle.fullEdits)
      (changes[uris[edit.file]] ??= []).push({ range: edit.range, newText: edit.newText });
    const site = oracle.definitionFromAllParentAndCallPositions[0];
    const definition = { uri: uris[site.file], range: site.range };
    wanted = queries.map((query: Json, index: number) => {
      const observation = actual.observations[index];
      const referencesReply = reply(2 + index * 3, references);
      const renameReply = reply(4 + index * 3, { changes });
      const definitions = [reply(3 + index * 3, definition), reply(3 + index * 3, [definition])];
      assert.deepEqual(observation.query, query);
      assert.deepEqual(observation.referencesReply, referencesReply);
      assert.deepEqual(observation.renameReply, renameReply);
      assert.equal(observation.definitionContracted, true);
      assert.ok(
        definitions.some((expected) => {
          try {
            assert.deepEqual(observation.definitionReply, expected);
            return true;
          } catch {
            return false;
          }
        }),
        "whole contracted definition response",
      );
      return {
        query,
        referencesReply,
        renameReply,
        definitionContracted: true,
        allowedWholeDefinitionResponses: definitions,
      };
    });
    assert.equal(packet.allDefinitionOriginsContracted, true);
  }
  assert.deepEqual(packet.expected, {
    queries: wanted,
    selectedApplicationQueryIndex: 0,
    version2: installed(2),
    independentVersion3: installed(3),
    allDefinitionOriginsContracted: true,
    allowedWholeDiagnosticPublications: allowed,
  });
  return actual.observations;
}
