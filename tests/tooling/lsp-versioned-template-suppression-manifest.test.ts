import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { asArray, asRecord, parseJson } from "../../tools/support/json-value.ts";
import {
  INITIALIZE_CAPABILITIES,
  initializeCapabilities,
  loadLspManifest,
} from "../differential/lsp-manifest.ts";
import {
  loadVersionedTemplateSuppression,
  VERSIONED_TEMPLATE_SUPPRESSION_ID,
} from "../differential/lsp-versioned-template-suppression.ts";
import type { FixtureData } from "../differential/lsp-types.ts";

const root = new URL("../_fixtures/differential/lsp/", import.meta.url);
const loaded = loadLspManifest(fileURLToPath(new URL("manifest.json", root)));
const current = loaded.cases.find((x) => x.id === VERSIONED_TEMPLATE_SUPPRESSION_ID);
assert(current);
const read = (name: string) => readFileSync(new URL(name, root));
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

void test("selected no-fix suppression preserves the whole original diagnostic and authored insertion", () => {
  const reference = read("code-action-diagnostics/references.json");
  assert.equal(hash(reference), "241817138323e3fbe957cae2d67f1ccd94782c2f71e3eb25987bdc901ed190e2");
  const diagnostic = asRecord(
    asArray(asRecord(parseJson(reference.toString("utf8"))).diagnostics)[0],
  );
  assert.equal(diagnostic.code, "a11y/alt-text");
  for (const name of ["App.vue.txt", "vize.config.json"]) {
    assert.deepEqual(
      read(`versioned-template-suppression-6876/${name}`),
      read(`code-action-diagnostics/${name}`),
    );
  }
  assert.deepEqual(current.data.request, {
    params: { range: diagnostic.range, context: { diagnostics: [diagnostic] } },
  });
  assert.deepEqual(current.requests[0].result, [
    {
      title: "Suppress with @vize:forget (a11y/alt-text)",
      kind: "quickfix",
      diagnostics: [diagnostic],
      edit: {
        documentChanges: [
          {
            textDocument: { uri: "${workspace}/App.vue", version: 1 },
            edits: [
              {
                range: { start: { line: 1, character: 0 }, end: { line: 1, character: 0 } },
                newText: "  <!-- @vize:forget a11y/alt-text -->\n",
              },
            ],
          },
        ],
      },
      isPreferred: false,
    },
  ]);
  assert.equal(current.data.workspaceEditDocumentChanges, true);
  const capabilities = structuredClone(INITIALIZE_CAPABILITIES);
  assert.deepEqual(initializeCapabilities(current), {
    ...capabilities,
    workspace: { workspaceEdit: { documentChanges: true } },
  });
  assert.deepEqual(INITIALIZE_CAPABILITIES, capabilities);
  assert.equal(loaded.cases.length, 82);
});

void test("closed one-suppression loader refuses altered identity, diagnostic, pin and whole-response metadata", () => {
  const expected = current.requests[0].result;
  const before = structuredClone({ data: current.data, expected });
  const load = (data: FixtureData, files = current.files, value = expected) =>
    loadVersionedTemplateSuppression(fileURLToPath(root), data, files, value);
  assert.deepEqual(load(current.data), current.requests);
  const mutations: ((data: FixtureData) => void)[] = [
    (data) => {
      data.id += "-foreign";
    },
    (data) => {
      data.method = "textDocument/hover";
    },
    (data) => {
      data.documentVersion = 2;
    },
    (data) => {
      Reflect.set(data, "workspaceEditDocumentChanges", false);
    },
    (data) => {
      delete data.workspaceEditDocumentChanges;
    },
    (data) => {
      data.provenance.fixCommit = "0".repeat(40);
    },
    (data) => {
      data.request = { params: { range: {}, context: { diagnostics: [] } } };
    },
    (data) => {
      data.entry = "Foreign.vue";
    },
    (data) => {
      data.initializationOptions = { editor: true, lint: false };
    },
    (data) => {
      data.files = [...(data.files ?? [])].reverse();
    },
    (data) => {
      data.expected.sha256 = "0".repeat(64);
    },
    (data) => {
      Reflect.set(data, "unknown", true);
    },
  ];
  for (const mutate of mutations) {
    const data = structuredClone(current.data);
    mutate(data);
    assert.throws(() => load(data), { name: "AssertionError" });
  }
  for (const files of [
    current.files.slice(0, 1),
    [...current.files].reverse(),
    current.files.map((x) => ({ ...x, bytes: Buffer.concat([x.bytes, Buffer.from(" ")]) })),
  ]) {
    assert.throws(() => load(current.data, files), { name: "AssertionError" });
  }
  const actions = asArray(expected),
    action = asRecord(actions[0]);
  for (const value of [
    [],
    [...actions, { title: "Fix: fabricated", isPreferred: true }],
    [{ ...action, isPreferred: true }],
    [{ ...action, diagnostics: [] }],
    [{ ...action, extra: true }],
    [{ ...action, edit: { changes: { "${workspace}/App.vue": [] } } }],
  ]) {
    assert.throws(() => load(current.data, current.files, value), { name: "AssertionError" });
  }
  assert.deepEqual({ data: current.data, expected }, before);
});
