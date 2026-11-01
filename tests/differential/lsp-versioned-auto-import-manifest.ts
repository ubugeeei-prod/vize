import assert from "node:assert/strict";
import { sha256 } from "./harness.mjs";
import type { FixtureData, LspFixture } from "./lsp-types.ts";

export const VERSIONED_AUTO_IMPORT_ID = "lsp/regression/versioned-auto-import-6876";

/** Closed advertised-client successor; the original unversioned fixture stays intact. */
export function loadVersionedAutoImport(
  data: FixtureData,
  files: { runtimePath: string; bytes: Buffer }[],
  expected: unknown,
): LspFixture["requests"] {
  const request = {
    params: {
      range: { start: { line: 5, character: 7 }, end: { line: 5, character: 7 } },
      context: { diagnostics: [] },
    },
  };
  const pins = [
    {
      source: "Page.vue.txt",
      sha256: "8e36c1d42e0c9ee8999c4b3259ae04580543a902409fc45938bf1d145aa62130",
      runtimePath: "app/Page.vue",
    },
    {
      source: "MyButton.vue.txt",
      sha256: "b0ca633eb9dd0a5fbf4ea5598d08ad9f0b49af75169670d2c949df3104267dbb",
      runtimePath: "owner's-components/MyButton.vue",
    },
  ];
  assert.deepEqual(data, {
    version: 1,
    id: VERSIONED_AUTO_IMPORT_ID,
    provenance: {
      fixCommit: "066242c3ae956ac9fbc65ed16cbc2ca81256aedb",
      inputOrigin: "Exact unchanged auto-import-quoted-symlink-3952 Page and component sources.",
      contractOrigin:
        "Whole original one-action oracle d8616029b93c65704c175249c1dd008566635e23b1557089e51d5af3b73db6de; only the advertised versioned edit envelope changes.",
    },
    documentVersion: 1,
    workspaceEditDocumentChanges: true,
    initializationOptions: { editor: true, lint: true, typecheck: false, codeActions: true },
    files: pins,
    workspaceSymlinks: [{ path: "app/MyButton.vue", target: "owner's-components/MyButton.vue" }],
    entry: "app/Page.vue",
    method: "textDocument/codeAction",
    request,
    expected: {
      source: "response.expected.json",
      sha256: "520dfa41229df94168f666fe769d9ff9af478c8a28be9a63cc2c267b6f434a07",
    },
  });
  assert.deepEqual(
    files.map((file) => ({ runtimePath: file.runtimePath, sha256: sha256(file.bytes) })),
    pins.map(({ runtimePath, sha256 }) => ({ runtimePath, sha256 })),
  );
  assert.deepEqual(expected, [
    {
      title: "Auto-import `MyButton` from `../owner's-components/MyButton.vue`",
      kind: "quickfix",
      edit: {
        documentChanges: [
          {
            textDocument: {
              uri: "${workspace}/app/Page.vue",
              version: 1,
            },
            edits: [
              {
                range: {
                  start: {
                    line: 1,
                    character: 0,
                  },
                  end: {
                    line: 1,
                    character: 0,
                  },
                },
                newText: 'import MyButton from "../owner\'s-components/MyButton.vue"\n',
              },
            ],
          },
        ],
      },
      isPreferred: true,
    },
  ]);
  return [{ ...request, result: expected }];
}
