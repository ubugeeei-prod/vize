import assert from "node:assert/strict";
import { sha256 } from "./harness.mjs";
import type { FixtureData, LspFixture } from "./lsp-types.ts";

export const AUTO_IMPORT_SYMLINK_ID = "lsp/regression/auto-import-quoted-symlink-3952";

/** This authored auto-import has one action; it is not a lint suppression fixture. */
export function loadAutoImportSymlink(
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
    id: AUTO_IMPORT_SYMLINK_ID,
    provenance: { fixCommit: "e7bccbcf6314c0dfda9f46e76c2311043d5fb408" },
    documentVersion: 1,
    initializationOptions: { editor: true, lint: true, typecheck: false, codeActions: true },
    files: pins,
    workspaceSymlinks: [{ path: "app/MyButton.vue", target: "owner's-components/MyButton.vue" }],
    entry: "app/Page.vue",
    method: "textDocument/codeAction",
    request,
    expected: {
      source: "response.expected.json",
      sha256: "d8616029b93c65704c175249c1dd008566635e23b1557089e51d5af3b73db6de",
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
        changes: {
          "${workspace}/app/Page.vue": [
            {
              range: { start: { line: 1, character: 0 }, end: { line: 1, character: 0 } },
              newText: 'import MyButton from "../owner\'s-components/MyButton.vue"\n',
            },
          ],
        },
      },
      isPreferred: true,
    },
  ]);
  return [{ ...request, result: expected }];
}
