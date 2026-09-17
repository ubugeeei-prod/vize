import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import { offsetToPosition } from "./support/lsp/assertions.ts";

type Item = Record<string, unknown>;
const documentation = "Produce the greeting shown in the welcome banner.";
const script = `/** ${documentation} */
function greetVisitor(name: string): string { return "Hello " + name; }
const greeting = greetVis`;
const sources = {
  setup: `<script setup lang="ts">\n${script}\n</script>`,
  normal: `<script lang="ts">\n${script}\n</script>`,
  inline: `<script setup lang="ts">${script.replaceAll("\n", " ").replace("const greeting", 'const marker = "\u{1f600}"; const greeting')};</script>`,
  trailingComment: `<script setup lang="ts">const marker = "\u{1f600}"; ${script.replaceAll("\n", " ")};</script>`,
  crlf: `<script setup lang="ts">\r\n${script.replaceAll("\n", "\r\n")}\r\n</script>`,
  template: `<script setup lang="ts">
function createApi() { return {
  /** ${documentation} */
  greetVisitor(name: string): string { return "Hello " + name; }
}; }
const api = createApi();
</script>
<template><p>{{ api.greetVis }}</p></template>`,
};

test("completion resolves authored documentation through the real server", async (t) => {
  fs.mkdirSync(testOutputRoot, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(testOutputRoot, "completion-resolve-"));
  const appPath = path.join(workspace, "App.vue");
  fs.writeFileSync(appPath, sources.setup);
  fs.writeFileSync(
    path.join(workspace, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        target: "ESNext",
        module: "ESNext",
        moduleResolution: "Bundler",
      },
      include: ["**/*.vue"],
    }),
  );
  const uri = pathToFileURL(appPath).href;
  const session = new LspSession();
  const open = (text: string) =>
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text },
    });
  const close = () => session.notify("textDocument/didClose", { textDocument: { uri } });
  const complete = async (text: string): Promise<Item> => {
    const response = (await session.request("textDocument/completion", {
      textDocument: { uri },
      position: offsetToPosition(text, text.lastIndexOf("greetVis") + "greetVis".length),
    })) as Item[] | { items: Item[] };
    const items = Array.isArray(response) ? response : response.items;
    const item = items.find((candidate) => candidate.label === "greetVisitor");
    assert.ok(item, JSON.stringify(items));
    assert.ok(item.data, "the candidate must retain deferred resolution data");
    return item;
  };
  const resolve = async (item: Item) =>
    (await session.request("completionItem/resolve", item)) as Item;
  const assertResolved = (item: Item, resolved: Item, expected: string | undefined) => {
    assert.equal(resolved.label, "greetVisitor");
    const doc = resolved.documentation;
    const value = typeof doc === "string" ? doc : (doc as { value: string } | undefined)?.value;
    assert.equal(value, expected);
    assert.match(String(resolved.detail), /greetVisitor\(name: string\): string/);
    assert.equal(
      String(resolved.detail).split("\n").length,
      1,
      "one authored declaration, not duplicate projections",
    );
    assert.deepEqual(
      { ...resolved, documentation: item.documentation, detail: item.detail },
      { ...item, documentation: item.documentation, detail: item.detail },
      "resolution may only enrich detail and documentation, never insertion fields",
    );
  };
  try {
    await session.initialize(workspace);
    for (const [name, source] of Object.entries(sources)) {
      await t.test(name, async () => {
        open(source);
        const item = await complete(source);
        // TypeScript treats a same-line comment after a statement as trailing
        // trivia, not documentation for the following declaration.
        assertResolved(
          item,
          await resolve(item),
          name === "trailingComment" ? undefined : documentation,
        );
        close();
      });
    }
    await t.test(
      "unsaved edits invalidate old items and expose current documentation",
      async () => {
        open(sources.setup);
        const old = await complete(sources.setup);
        const updatedDocumentation = "Use the greeting from the unsaved buffer.";
        const updated = sources.setup.replace(documentation, updatedDocumentation);
        session.notify("textDocument/didChange", {
          textDocument: { uri, version: 2 },
          contentChanges: [{ text: updated }],
        });
        const current = await complete(updated);
        assert.deepEqual(await resolve(old), old);
        const updatedResult = await resolve(current);
        assertResolved(current, updatedResult, updatedDocumentation);
        close();
        assert.deepEqual(await resolve(current), current);
        open(sources.setup);
        const reopened = await complete(sources.setup);
        assert.deepEqual(
          await resolve(old),
          old,
          "reused client version must not revive old items",
        );
        assertResolved(reopened, await resolve(reopened), documentation);
        close();
      },
    );
    await t.test("foreign and malformed items remain unchanged", async () => {
      for (const item of [
        { label: "other", detail: "authored detail", data: { extension: true } },
        { label: "other", data: { vizeCompletion: { uri: false, revision: 1, item: {} } } },
      ])
        assert.deepEqual(await resolve(item), item);
    });
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
