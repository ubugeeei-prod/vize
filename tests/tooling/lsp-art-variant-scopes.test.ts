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

test("art authored editor scopes isolate variants and shadow setup bindings without a type service", async () => {
  fs.mkdirSync(testOutputRoot, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(testOutputRoot, "art-variant-scopes-"));
  const session = new LspSession();
  try {
    fs.writeFileSync(path.join(workspace, "Child.vue"), "<template><slot /></template>\n");
    await session.initialize(workspace, { editor: true, lint: false, typecheck: false });
    for (const inline of [false, true]) {
      for (const newline of ["\n", "\r\n"]) {
        const ordinary = inline
          ? '<template><p v-for="ordinaryOnly in rows">{{ ordinaryOnly }}</p></template>\n'
          : "";
        const source = `<script setup lang="ts">
const rows = [1, 2]
const item = 'outside'
const slotValue = 'outside'
</script>
${ordinary}<art title="Scope" component="./Child.vue">
<variant name="First" default>
<div v-for="(item, firstOnly) in rows">🧭{{ item }}<Child v-slot="{ slotValue }">{{ slotValue }}</Child></div>
</variant>
<variant name="Second">
<section v-for="(item, secondOnly) in rows">🧭{{ item }}<span v-for="item in rows">{{ item }}</span><Child v-slot="{ slotValue }">{{ slotValue }}</Child></section>
</variant>
</art>
`.replaceAll("\n", newline);
        const filename = `${inline ? "Inline" : "Separate"}-${newline.length}.${inline ? "vue" : "art.vue"}`;
        const file = path.join(workspace, filename);
        const uri = pathToFileURL(file).href;
        fs.writeFileSync(file, source);
        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: inline ? "vue" : "art-vue", version: 1, text: source },
        });
        await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
          isDiagnosticsForUri(params, uri),
        );

        const cases = [
          {
            usage: source.indexOf("🧭{{ item }}") + "🧭{{ ".length,
            declaration: source.indexOf("(item, firstOnly)") + 1,
            name: "item",
            kind: "v-for",
            tag: '<div v-for="(item, firstOnly) in rows">',
            own: "firstOnly",
            sibling: "secondOnly",
          },
          {
            usage: source.lastIndexOf("🧭{{ item }}") + "🧭{{ ".length,
            declaration: source.indexOf("(item, secondOnly)") + 1,
            name: "item",
            kind: "v-for",
            tag: '<section v-for="(item, secondOnly) in rows">',
            own: "secondOnly",
            sibling: "firstOnly",
          },
          {
            usage: source.lastIndexOf("{{ item }}") + 3,
            declaration: source.lastIndexOf("item in rows"),
            name: "item",
            kind: "v-for",
            tag: '<span v-for="item in rows">',
            own: "secondOnly",
            sibling: "firstOnly",
          },
          {
            usage: source.indexOf("{{ slotValue }}") + 3,
            declaration: source.indexOf("{ slotValue }") + 2,
            name: "slotValue",
            kind: "v-slot",
            tag: '<Child v-slot="{ slotValue }">',
            own: "firstOnly",
            sibling: "secondOnly",
          },
          {
            usage: source.lastIndexOf("{{ slotValue }}") + 3,
            declaration: source.lastIndexOf("{ slotValue }") + 2,
            name: "slotValue",
            kind: "v-slot",
            tag: '<Child v-slot="{ slotValue }">',
            own: "secondOnly",
            sibling: "firstOnly",
          },
        ];
        for (const entry of cases) {
          const params = { textDocument: { uri }, position: offsetToPosition(source, entry.usage) };
          assert.deepEqual(await session.request("textDocument/definition", params), {
            uri,
            range: {
              start: offsetToPosition(source, entry.declaration),
              end: offsetToPosition(source, entry.declaration + entry.name.length),
            },
          });
          const loop = entry.kind === "v-for";
          assert.deepEqual(await session.request("textDocument/hover", params), {
            contents: {
              kind: "markdown",
              value: `**${entry.name}**\n\n_${entry.kind} scope binding_\n\n\`\`\`vue\n${entry.tag}\n\`\`\`\n\n${loop ? "Loop value alias declared by the nearest `v-for` and available to this template scope." : "Slot prop binding declared by the nearest scoped slot and available to this template subtree."}\n\n**Behavior**\n- Shadows outer template and script bindings with the same name.\n- ${loop ? "Type-backed sessions should refine this with the iterable item type when the backend answers." : "Type-backed sessions should refine this with the child component slot prop type when the backend answers."}`,
            },
          });
          const labels = completionLabels(await session.request("textDocument/completion", params));
          assert.ok(labels.includes(entry.name), `${filename}: missing ${entry.name}`);
          assert.ok(labels.includes(entry.own), `${filename}: missing ${entry.own}`);
          assert.ok(!labels.includes(entry.sibling), `${filename}: leaked ${entry.sibling}`);
          assert.ok(
            !labels.includes("ordinaryOnly"),
            `${filename}: ordinary template scope leaked`,
          );
        }
      }
    }
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
