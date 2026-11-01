import assert from "node:assert/strict";
import { asArray, asRecord, parseJson } from "../../tools/support/json-value.ts";
import { readPinnedArtifact, sha256 } from "./harness.mjs";
import type { FixtureData, LspFixture } from "./lsp-types.ts";

export const VERSIONED_TEMPLATE_SUPPRESSION_ID =
  "lsp/regression/versioned-template-suppression-6876";

/** One selected no-fix diagnostic, not a generic fix-and-suppression exception. */
export function loadVersionedTemplateSuppression(
  packRoot: string,
  data: FixtureData,
  files: { runtimePath: string; bytes: Buffer }[],
  expected: unknown,
): LspFixture["requests"] {
  const originalDiagnostic = {
    path: "code-action-diagnostics/references.json",
    sha256: "241817138323e3fbe957cae2d67f1ccd94782c2f71e3eb25987bdc901ed190e2",
  };
  const reference = asRecord(
    parseJson(readPinnedArtifact(packRoot, originalDiagnostic).toString("utf8")),
  );
  const diagnostic = asRecord(asArray(reference.diagnostics)[0]);
  const request = { params: { range: diagnostic.range, context: { diagnostics: [diagnostic] } } };
  const pins = [
    {
      source: "App.vue.txt",
      runtimePath: "App.vue",
      sha256: "7dcf16fe1a2d823b8d63ef69e16f7016bd0f6054158acc8cdeda0a6c3ffb6e19",
    },
    {
      source: "vize.config.json",
      runtimePath: "vize.config.json",
      sha256: "1d297caf631fad17f18d5ba9577567b7ccb77db5f299df777c88469ae604f75a",
    },
  ];
  assert.deepEqual(data, {
    version: 1,
    id: VERSIONED_TEMPLATE_SUPPRESSION_ID,
    provenance: {
      fixCommit: "3cd259fd698baa8c38518d3542b7333816a84189",
      originalDiagnostic,
      inputOrigin:
        "Exact existing selected no-fix diagnostic caller App/config; no new reporter-authored witness claim.",
      contractOrigin:
        "Authored full supporting-client one-suppression envelope; original diagnostic, range, indentation and LF text retained. Hosted observation is separate.",
    },
    documentVersion: 1,
    workspaceEditDocumentChanges: true,
    initializationOptions: { editor: true, lint: true, typecheck: false, codeActions: true },
    files: pins,
    entry: "App.vue",
    method: "textDocument/codeAction",
    request,
    expected: {
      source: "response.expected.json",
      sha256: "9336ed7587e516187fc0b381af2c7f9a2380b221041b13ccb74ff6becd351231",
    },
  });
  assert.deepEqual(
    files.map((file) => ({ runtimePath: file.runtimePath, sha256: sha256(file.bytes) })),
    pins.map(({ runtimePath, sha256 }) => ({ runtimePath, sha256 })),
  );
  assert.deepEqual(expected, [
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
  return [{ ...request, result: expected }];
}
