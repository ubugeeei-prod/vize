import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import {
  completionLabels,
  isDiagnosticsForUri,
  offsetToPosition,
} from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

type CompletionResponse = Parameters<typeof completionLabels>[0];

test("Pug authored scopes preserve local identity, nearest shadowing and exact hover excerpts", async () => {
  fs.mkdirSync(testOutputRoot, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(testOutputRoot, "pug-authored-scopes-"));
  const session = new LspSession();
  try {
    fs.writeFileSync(path.join(workspace, "Child.vue"), "<template><slot /></template>\n");
    await session.initialize(workspace, { editor: true, lint: false, typecheck: false });
    for (const language of ["pug", "jade", "PUG", "JADE"]) {
      for (const newline of ["\n", "\r\n"]) {
        for (const quote of ['"', "'"]) {
          const context = `${language} newline=${JSON.stringify(newline)} quote=${quote}`;
          const source = `<script setup lang="ts">
const item = 'outside'
const rows = [1, 2]
</script>
<template lang="${language}">
section
  div(v-for=${quote}(item, outerIndex) in rows${quote})
    p(:title=${quote}item${quote}) 🧭{{ item }}
    Child(v-slot=${quote}{ item, slotOnly }${quote})
      p {{ item }}
      em(v-for=${quote}(item, innerIndex) in rows${quote}) {{ item }}
  div(v-for=${quote}(item, siblingOnly) in rows${quote}) {{ item }}
  li(
    :title=${quote}1 < 2 && true${quote}
    v-for=${quote}(item, multilineOnly) in rows${quote}
  ) 🧭{{ item }}
  p {{ item }}
</template>
`.replaceAll("\n", newline);
          const file = path.join(
            workspace,
            `${language}-${newline.length}-${quote === '"' ? "double" : "single"}.vue`,
          );
          const uri = pathToFileURL(file).href;
          fs.writeFileSync(file, source);
          session.notify("textDocument/didOpen", {
            textDocument: { uri, languageId: "vue", version: 1, text: source },
          });
          const diagnostics = await session.waitForNotification(
            "textDocument/publishDiagnostics",
            (params) => isDiagnosticsForUri(params, uri),
          );
          assert.deepEqual(
            diagnostics,
            { uri, version: 1, diagnostics: [] },
            `diagnostics: ${context}`,
          );
          const entries = [
            {
              usage: source.indexOf("🧭{{ item") + "🧭{{ ".length,
              declaration: source.indexOf("(item, outerIndex)") + 1,
              kind: "v-for",
              tag: `div(v-for=${quote}(item, outerIndex) in rows${quote})`,
              locals: ["item", "outerIndex"],
            },
            {
              usage: source.indexOf("p {{ item") + "p {{ ".length,
              declaration: source.indexOf("{ item, slotOnly }") + 2,
              kind: "v-slot",
              tag: `Child(v-slot=${quote}{ item, slotOnly }${quote})`,
              locals: ["item", "outerIndex", "slotOnly"],
            },
            {
              usage: source.indexOf("{{ item }}", source.indexOf("em(v-for")) + 3,
              declaration: source.indexOf("(item, innerIndex)") + 1,
              kind: "v-for",
              tag: `em(v-for=${quote}(item, innerIndex) in rows${quote})`,
              locals: ["innerIndex", "item", "outerIndex", "slotOnly"],
            },
            {
              usage: source.indexOf("{{ item }}", source.indexOf("(item, siblingOnly)")) + 3,
              declaration: source.indexOf("(item, siblingOnly)") + 1,
              kind: "v-for",
              tag: `div(v-for=${quote}(item, siblingOnly) in rows${quote})`,
              locals: ["item", "siblingOnly"],
            },
            {
              usage: source.lastIndexOf("🧭{{ item") + "🧭{{ ".length,
              declaration: source.indexOf("(item, multilineOnly)") + 1,
              kind: "v-for",
              tag: `li(${newline}    :title=${quote}1 < 2 && true${quote}${newline}    v-for=${quote}(item, multilineOnly) in rows${quote}${newline}  )`,
              locals: ["item", "multilineOnly"],
            },
          ];
          for (const entry of entries) {
            const entryContext = `${context} ${entry.kind} at ${entry.usage}`;
            const params = {
              textDocument: { uri },
              position: offsetToPosition(source, entry.usage),
            };
            assert.deepEqual(
              await session.request("textDocument/definition", params),
              {
                uri,
                range: {
                  start: offsetToPosition(source, entry.declaration),
                  end: offsetToPosition(source, entry.declaration + 4),
                },
              },
              `definition: ${entryContext}`,
            );
            const loop = entry.kind === "v-for";
            assert.deepEqual(
              await session.request("textDocument/hover", params),
              {
                contents: {
                  kind: "markdown",
                  value: `**item**\n\n_${entry.kind} scope binding_\n\n\`\`\`pug\n${entry.tag}\n\`\`\`\n\n${loop ? "Loop value alias declared by the nearest `v-for` and available to this template scope." : "Slot prop binding declared by the nearest scoped slot and available to this template subtree."}\n\n**Behavior**\n- Shadows outer template and script bindings with the same name.\n- ${loop ? "Type-backed sessions should refine this with the iterable item type when the backend answers." : "Type-backed sessions should refine this with the child component slot prop type when the backend answers."}`,
                },
                range: {
                  start: offsetToPosition(source, entry.usage),
                  end: offsetToPosition(source, entry.usage + 4),
                },
              },
              `hover: ${entryContext}`,
            );
            const names = [
              "item",
              "outerIndex",
              "innerIndex",
              "slotOnly",
              "siblingOnly",
              "multilineOnly",
            ];
            const labels = completionLabels(
              (await session.request("textDocument/completion", params)) as CompletionResponse,
            );
            assert.deepEqual(
              labels.filter((label) => names.includes(label)).sort(),
              entry.locals,
              `complete named scope vector: ${entryContext}`,
            );
          }
          const boundary =
            source.indexOf(`:title=${quote}item${quote}`) + `:title=${quote}item`.length;
          const boundaryLabels = completionLabels(
            (await session.request("textDocument/completion", {
              textDocument: { uri },
              position: offsetToPosition(source, boundary),
            })) as CompletionResponse,
          );
          assert.deepEqual(
            boundaryLabels
              .filter((label) =>
                [
                  "item",
                  "outerIndex",
                  "innerIndex",
                  "slotOnly",
                  "siblingOnly",
                  "multilineOnly",
                ].includes(label),
              )
              .sort(),
            ["item", "outerIndex"],
            `closed directive value caret retains its entire local scope: ${context}`,
          );
          const outerUse = source.lastIndexOf("{{ item }}") + 3;
          assert.deepEqual(
            await session.request("textDocument/definition", {
              textDocument: { uri },
              position: offsetToPosition(source, outerUse),
            }),
            {
              uri,
              range: {
                start: offsetToPosition(source, source.indexOf("item =")),
                end: offsetToPosition(source, source.indexOf("item =") + 4),
              },
            },
            `dedented script binding: ${context}`,
          );
          session.notify("textDocument/didClose", { textDocument: { uri } });
        }
      }
    }
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
