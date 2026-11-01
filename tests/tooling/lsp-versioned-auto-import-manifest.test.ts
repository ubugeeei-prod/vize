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
  VERSIONED_AUTO_IMPORT_ID,
  loadVersionedAutoImport,
} from "../differential/lsp-versioned-auto-import-manifest.ts";
import { lspWorkspace } from "../differential/lsp-workspace.ts";
import type { FixtureData } from "../differential/lsp-types.ts";

const pack = new URL("../_fixtures/differential/lsp/", import.meta.url);
const loaded = loadLspManifest(fileURLToPath(new URL("manifest.json", pack)));
const current = loaded.cases.find((row) => row.id === VERSIONED_AUTO_IMPORT_ID);
const original = loaded.cases.find(
  (row) => row.id === "lsp/regression/auto-import-quoted-symlink-3952",
);
assert(current && original);
const read = (name: string) => readFileSync(new URL(name, pack));
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

void test("the advertised import keeps the whole original source and changes only its version envelope", () => {
  for (const [name, sha] of [
    ["Page.vue.txt", "8e36c1d42e0c9ee8999c4b3259ae04580543a902409fc45938bf1d145aa62130"],
    ["MyButton.vue.txt", "b0ca633eb9dd0a5fbf4ea5598d08ad9f0b49af75169670d2c949df3104267dbb"],
  ] as const) {
    const before = read(`auto-import-quoted-symlink-3952/${name}`);
    assert.equal(hash(before), sha);
    assert.deepEqual(read(`versioned-auto-import-6876/${name}`), before);
  }
  const before = read("auto-import-quoted-symlink-3952/response.expected.json");
  assert.equal(hash(before), "d8616029b93c65704c175249c1dd008566635e23b1557089e51d5af3b73db6de");
  const actions = asArray(parseJson(before.toString("utf8")));
  assert.equal(actions.length, 1);
  const first = asRecord(actions[0]);
  const intended = [
    {
      ...first,
      edit: {
        documentChanges: [
          {
            textDocument: { uri: "${workspace}/app/Page.vue", version: 1 },
            edits: asRecord(asRecord(first.edit).changes)["${workspace}/app/Page.vue"],
          },
        ],
      },
    },
  ];
  assert.deepEqual(current.requests[0].result, intended);
  assert.deepEqual(current.data.request, original.data.request);
  assert.deepEqual(current.data.files, original.data.files);
  assert.deepEqual(current.data.workspaceSymlinks, original.data.workspaceSymlinks);
  assert.deepEqual(current.data.initializationOptions, original.data.initializationOptions);
  assert.equal(current.data.workspaceEditDocumentChanges, true);
  assert.equal(original.data.workspaceEditDocumentChanges, undefined);
  assert.deepEqual(lspWorkspace(current).mapping, lspWorkspace(original).mapping);
});

void test("only the exact supporting-client profile changes initialization without mutating defaults", () => {
  const before = structuredClone(INITIALIZE_CAPABILITIES);
  assert.deepEqual(initializeCapabilities(original), before);
  assert.deepEqual(initializeCapabilities(current), {
    ...before,
    workspace: { workspaceEdit: { documentChanges: true } },
  });
  assert.deepEqual(INITIALIZE_CAPABILITIES, before);
  assert.equal(loaded.cases.length, 82);
  assert.equal(loaded.cases.filter((row) => row.id !== VERSIONED_AUTO_IMPORT_ID).length, 81);
});

void test("the closed whole import contract refuses foreign metadata, altered pins and partial responses", () => {
  const expected = current.requests[0].result;
  const before = structuredClone({ data: current.data, expected });
  assert.deepEqual(
    loadVersionedAutoImport(current.data, current.files, expected),
    current.requests,
  );
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
      data.initializationOptions = { editor: true, lint: false };
    },
    (data) => {
      data.request = { params: { range: {}, context: { diagnostics: [] } } };
    },
    (data) => {
      data.workspaceSymlinks = [];
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
    assert.throws(() => loadVersionedAutoImport(data, current.files, expected), {
      name: "AssertionError",
    });
  }
  for (const files of [
    current.files.slice(0, 1),
    [...current.files].reverse(),
    current.files.map((file) => ({
      ...file,
      bytes: Buffer.concat([file.bytes, Buffer.from(" ")]),
    })),
  ]) {
    assert.throws(() => loadVersionedAutoImport(current.data, files, expected), {
      name: "AssertionError",
    });
  }
  const actions = asArray(expected);
  for (const response of [
    [],
    [...actions, { title: "Suppress with @vize:forget", isPreferred: false }],
    [{ ...asRecord(actions[0]), isPreferred: false }],
    [{ ...asRecord(actions[0]), extra: true }],
    [
      {
        ...asRecord(actions[0]),
        edit: {
          documentChanges: [
            { textDocument: { uri: "${workspace}/foreign.vue", version: 1 }, edits: [] },
          ],
        },
      },
    ],
    original.requests[0].result,
  ]) {
    assert.throws(() => loadVersionedAutoImport(current.data, current.files, response), {
      name: "AssertionError",
    });
  }
  assert.deepEqual({ data: current.data, expected }, before);
});
