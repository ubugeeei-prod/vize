// Exercise the exact command and default initialization profile emitted by the
// Zed extension against a real `vize lsp` process. The extension's Rust unit
// suite separately pins command discovery/configuration; this scenario pins
// every full LSP response the default Zed contract enables.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

import {
  prepareRealVueWorkspace,
  resolveRealServerPath,
} from "../editor-e2e/real-vue-workspace.mjs";
import { assertRefSurfaceHovers } from "./real-server-ref-surface.mjs";
import { isDiagnosticsForUri } from "../../tests/tooling/support/lsp/assertions.ts";
import { LspSession } from "../../tests/tooling/support/lsp/session.ts";

const expectedAuthoredSource = `<script setup lang="ts">
import Child from "./Child.vue";

const total = "3";
</script>

<template>
<Child  :count="total" />
</template>
`;

const zedRecommendedInitializationOptions = {
  editor: true,
  ecosystem: true,
  lint: true,
  typecheck: true,
};

const expectedDiagnostics = [
  {
    code: "vue/no-multi-spaces",
    codeDescription: { href: "https://eslint.vuejs.org/rules/no-multi-spaces.html" },
    message: "Multiple consecutive spaces",
    range: {
      end: { character: 8, line: 7 },
      start: { character: 6, line: 7 },
    },
    severity: 2,
    source: "vize/lint",
  },
  {
    code: 2322,
    message: "Type 'string' is not assignable to type 'number'.",
    range: {
      end: { character: 14, line: 7 },
      start: { character: 9, line: 7 },
    },
    severity: 1,
    source: "vize/types",
  },
];

const expectedCompletion = [
  {
    detail: " (const)",
    documentation: {
      kind: "markdown",
      value: "**Const**\n\nConstant binding (function, class, or literal).",
    },
    kind: 21,
    label: "Child",
    labelDetails: { detail: " (const)" },
    sortText: "0Child",
  },
  {
    detail: " (literal)",
    documentation: {
      kind: "markdown",
      value: "**Literal**\n\nLiteral constant value.",
    },
    kind: 21,
    label: "total",
    labelDetails: { detail: " (literal)" },
    sortText: "0total",
  },
];

const expectedHover = {
  contents: {
    kind: "markdown",
    value: '```typescript\nconst total: "3"\n```',
  },
  range: {
    end: { character: 11, line: 3 },
    start: { character: 6, line: 3 },
  },
};

function expectedCodeActions(uri) {
  return [
    {
      edit: {
        changes: {
          [uri]: [
            {
              newText: " ",
              range: {
                end: { character: 8, line: 7 },
                start: { character: 6, line: 7 },
              },
            },
          ],
        },
      },
      isPreferred: true,
      kind: "quickfix",
      title: "Fix: Replace multiple spaces with single space",
    },
    {
      edit: {
        changes: {
          [uri]: [
            {
              newText: "<!-- @vize:forget vue/no-multi-spaces -->\n",
              range: {
                end: { character: 0, line: 7 },
                start: { character: 0, line: 7 },
              },
            },
          ],
        },
      },
      isPreferred: false,
      kind: "quickfix",
      title: "Suppress with @vize:forget (vue/no-multi-spaces)",
    },
  ];
}

const expectedFormatting = [
  {
    newText: `<script setup lang="ts">
import Child from "./Child.vue";

const total = "3";
</script>

<template>
  <Child :count="total" />
</template>
`,
    range: {
      end: { character: 0, line: 9 },
      start: { character: 0, line: 0 },
    },
  },
];

const expectedSemanticTokens = { data: [7, 8, 6, 9, 0, 0, 8, 5, 8, 0] };

function expectedRename(uri) {
  return {
    changes: {
      [uri]: [
        {
          newText: "quantity",
          range: {
            end: { character: 11, line: 3 },
            start: { character: 6, line: 3 },
          },
        },
        {
          newText: "quantity",
          range: {
            end: { character: 21, line: 7 },
            start: { character: 16, line: 7 },
          },
        },
      ],
    },
  };
}

const sessionRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-zed-e2e-"));
const workspacePath = path.join(sessionRoot, "real-vue");
prepareRealVueWorkspace(workspacePath);

// LspSession's explicit binary override maps to the Zed extension's discovered
// `vize` command plus its default `["lsp"]` argument.
process.env.VIZE_LSP_BIN = resolveRealServerPath();
const session = new LspSession();

try {
  // Zed's fallback is the shared recommended profile. Formatting is the one
  // server-default capability intentionally left off, so opt into it exactly
  // as a Zed user would through `lsp.vize.initialization_options`.
  await session.initialize(workspacePath, {
    ...zedRecommendedInitializationOptions,
    formatting: true,
  });

  const documentPath = path.join(workspacePath, "src", "Scenario.vue");
  const uri = pathToFileURL(documentPath).href;
  const source = fs.readFileSync(documentPath, "utf8");
  assert.equal(source, expectedAuthoredSource, "the shared editor fixture changed");

  session.notify("textDocument/didOpen", {
    textDocument: {
      uri,
      languageId: "vue",
      version: 1,
      text: source,
    },
  });

  const diagnostics = await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) =>
      isDiagnosticsForUri(params, uri) && params.diagnostics.length === expectedDiagnostics.length,
    120_000,
  );
  assert.deepEqual(diagnostics.diagnostics, expectedDiagnostics);

  const completion = await session.request(
    "textDocument/completion",
    { textDocument: { uri }, position: { character: 16, line: 7 } },
    120_000,
  );
  assert.deepEqual(completion, expectedCompletion);

  const hover = await session.request(
    "textDocument/hover",
    { textDocument: { uri }, position: { character: 8, line: 3 } },
    120_000,
  );
  assert.deepEqual(hover, expectedHover);

  await assertRefSurfaceHovers({ isDiagnosticsForUri, session, workspacePath });

  const codeActions = await session.request(
    "textDocument/codeAction",
    {
      textDocument: { uri },
      range: {
        end: { character: 8, line: 7 },
        start: { character: 6, line: 7 },
      },
      context: { diagnostics: expectedDiagnostics },
    },
    120_000,
  );
  assert.deepEqual(codeActions, expectedCodeActions(uri));

  const formatting = await session.request(
    "textDocument/formatting",
    { textDocument: { uri }, options: { insertSpaces: true, tabSize: 2 } },
    120_000,
  );
  assert.deepEqual(formatting, expectedFormatting);

  const semanticTokens = await session.request(
    "textDocument/semanticTokens/full",
    { textDocument: { uri } },
    120_000,
  );
  assert.deepEqual(semanticTokens, expectedSemanticTokens);

  const rename = await session.request(
    "textDocument/rename",
    {
      textDocument: { uri },
      position: { character: 8, line: 3 },
      newName: "quantity",
    },
    120_000,
  );
  assert.deepEqual(rename, expectedRename(uri));

  console.log("zed extension-contract real-server scenario passed");
} finally {
  await session.shutdown();
  fs.rmSync(sessionRoot, { force: true, recursive: true });
}
