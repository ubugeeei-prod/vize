// Exercise the exact command and initialization profile from the packaged
// Helix languages.toml against a real `vize lsp` process. Helix 25.07.1 has no
// supported headless editing API, so the official binary separately validates
// config loading and command resolution with `hx --health`.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import toml from "@iarna/toml";

import {
  prepareRealVueWorkspace,
  resolveRealServerPath,
} from "../editor-e2e/real-vue-workspace.mjs";
import { isDiagnosticsForUri } from "../../tests/tooling/support/lsp/assertions.ts";
import { LspSession } from "../../tests/tooling/support/lsp/session.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const helixConfigPath = path.join(root, "editors", "helix", "languages.toml");
const parsedConfig = toml.parse(fs.readFileSync(helixConfigPath, "utf8"));
const helixServer = parsedConfig["language-server"].vize;
const helixRecommendedOptions = {
  editor: true,
  ecosystem: true,
  lint: true,
  typecheck: true,
};

assert.equal(helixServer.command, "vize");
assert.deepEqual(helixServer.args, ["lsp"]);
assert.deepEqual(helixServer.config, helixRecommendedOptions);

const expectedAuthoredSource = `<script setup lang="ts">
import Child from "./Child.vue";

const total = "3";
</script>

<template>
<Child  :count="total" />
</template>
`;

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

const sessionRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-helix-e2e-"));
const workspacePath = path.join(sessionRoot, "real-vue");
prepareRealVueWorkspace(workspacePath);

process.env.VIZE_LSP_BIN = resolveRealServerPath();
const session = new LspSession();

try {
  const initialization = await session.initialize(workspacePath, helixRecommendedOptions);
  assert.ok(initialization && typeof initialization === "object");
  const capabilities = initialization.capabilities;
  assert.equal(capabilities.documentFormattingProvider, undefined);

  const documentPath = path.join(workspacePath, "src", "Scenario.vue");
  const uri = pathToFileURL(documentPath).href;
  const source = fs.readFileSync(documentPath, "utf8");
  assert.equal(source, expectedAuthoredSource, "the shared editor fixture changed");

  session.notify("textDocument/didOpen", {
    textDocument: { uri, languageId: "vue", version: 1, text: source },
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

  console.log("helix package-contract real-server scenario passed");
} finally {
  await session.shutdown();
  fs.rmSync(sessionRoot, { force: true, recursive: true });
}
