import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  loadBindingOccurrences,
  BINDING_SESSION,
} from "../differential/lsp-binding-occurrences-manifest.ts";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { inspectLspObservation } from "../differential/lsp-report.ts";
import { referenceObservation, rewriteFrames } from "./support/lsp/differential-reference.ts";
import type { JsonRpcMessage } from "../differential/lsp-types.ts";

const pack = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../_fixtures/differential/lsp",
);
const loaded = loadLspManifest(path.join(pack, "manifest.json"));
const fixture = loaded.cases.find((row) => row.id === BINDING_SESSION);
assert(fixture);
const expected: unknown[] = JSON.parse(
  fs.readFileSync(path.join(pack, fixture.inputs.root, fixture.data.expected.source), "utf8"),
);

function object(value: unknown): Record<string, unknown> {
  assert(value && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}
function array(message: JsonRpcMessage): unknown[] {
  assert(Array.isArray(message.result));
  return message.result;
}
function result(message: JsonRpcMessage, index: number) {
  return object(array(message)[index]);
}

void test("7992 registers all three highlights, three unchanged references and the whole lens array", () => {
  assert.equal(loaded.cases.length, 17);
  assert.deepEqual(
    fixture.requests.map((row) => row.method),
    [
      "textDocument/documentHighlight",
      "textDocument/documentHighlight",
      "textDocument/documentHighlight",
      "textDocument/references",
      "textDocument/references",
      "textDocument/references",
      "textDocument/codeLens",
    ],
  );
  assert.deepEqual(
    fixture.requests.map((row) => (row.result as unknown[]).length),
    [2, 2, 4, 2, 2, 4, 3],
  );
  assert.equal(
    fixture.files[0].bytes.toString("utf8"),
    fs.readFileSync(path.join(pack, fixture.inputs.root, "Field.vue"), "utf8"),
  );
  assert(
    inspectLspObservation(fixture, referenceObservation(fixture)).every(
      (row) => row.state === "equal",
    ),
  );
});

void test("7992 admission refuses a dropped plan, wrong binding, reference context or redirected lens", () => {
  const changed = (mutate: (rows: unknown[]) => void) => {
    const rows = structuredClone(expected);
    mutate(rows);
    assert.throws(() => loadBindingOccurrences(pack, fixture.data, rows));
  };
  changed((rows) => rows.pop());
  changed((rows) => {
    object(object(object(rows[0]).params).position).line = 2;
  });
  changed((rows) => {
    object(object(object(rows[3]).params).context).includeDeclaration = false;
  });
  changed((rows) => {
    object(rows[6]).method = "textDocument/documentHighlight";
  });
});

void test("whole RPC comparison rejects false ranges, kinds, lens totals and optional-field drift", () => {
  const mutations: ((messages: JsonRpcMessage[]) => void)[] = [
    (messages) => {
      array(messages[2]).push({
        range: { start: { line: 7, character: 3 }, end: { line: 7, character: 8 } },
        kind: 2,
      });
    },
    (messages) => {
      result(messages[2], 0).kind = 2;
    },
    (messages) => {
      result(messages[5], 0).uri = "${workspace}/Field.vue.stale";
    },
    (messages) => {
      object(result(messages[8], 0).command).title = "3 template/style references";
    },
    (messages) => {
      object(result(messages[8], 0).command).arguments = [];
    },
    (messages) => {
      messages[8].result = null;
    },
  ];
  for (const mutate of mutations) {
    const observation = referenceObservation(fixture);
    rewriteFrames(observation, "server", mutate);
    assert(inspectLspObservation(fixture, observation).some((row) => row.state === "different"));
  }
});
