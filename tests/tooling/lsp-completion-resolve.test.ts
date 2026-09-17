import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
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
  escaped: `<script setup lang="ts">\n${script.replace("function greetVisitor", "function greet\\u0056isitor")}\n</script>`,
  tsx: `<script setup lang="tsx">\n${script.replace("const greeting", "const node = <div />; const greeting")}\n</script>`,
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
  const vuePackage = path.dirname(createRequire(import.meta.url).resolve("vue/package.json"));
  fs.mkdirSync(path.join(workspace, "node_modules"));
  const linkType = process.platform === "win32" ? "junction" : "dir";
  fs.symlinkSync(vuePackage, path.join(workspace, "node_modules/vue"), linkType);
  fs.symlinkSync(
    path.join(path.dirname(vuePackage), "@vue"),
    path.join(workspace, "node_modules/@vue"),
    linkType,
  );
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
  const open = (text: string, target = uri) =>
    session.notify("textDocument/didOpen", {
      textDocument: { uri: target, languageId: "vue", version: 1, text },
    });
  const close = (target = uri) =>
    session.notify("textDocument/didClose", { textDocument: { uri: target } });
  const complete = async (
    text: string,
    label = "greetVisitor",
    prefix = "greetVis",
    target = uri,
  ): Promise<Item> => {
    const response = (await session.request("textDocument/completion", {
      textDocument: { uri: target },
      position: offsetToPosition(text, text.lastIndexOf(prefix) + prefix.length),
    })) as Item[] | { items: Item[] };
    const items = Array.isArray(response) ? response : response.items;
    assert.deepEqual(
      items.filter((item) => /^__Vize|^__vize/.test(String(item.label))).map((item) => item.label),
      label.startsWith("__vize") ? [label] : [],
      "only authored helpers may become script candidates",
    );
    const item = items.find((candidate) => candidate.label === label);
    assert.ok(item, JSON.stringify(items));
    assert.ok(item.data, "the candidate must retain deferred resolution data");
    return item;
  };
  const resolve = async (item: Item) =>
    (await session.request("completionItem/resolve", item)) as Item;
  const assertResolved = (
    item: Item,
    resolved: Item,
    expected: string | undefined,
    declarationName = String(item.label),
  ) => {
    assert.equal(resolved.label, item.label);
    const doc = resolved.documentation;
    const value = typeof doc === "string" ? doc : (doc as { value: string } | undefined)?.value;
    assert.equal(value, expected);
    assert.ok(
      String(resolved.detail).includes(`${declarationName}(name: string): string`),
      String(resolved.detail),
    );
    assert.equal(
      String(resolved.detail).split(`${declarationName}(name: string): string`).length - 1,
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
          name === "escaped" ? "greet\\u0056isitor" : undefined,
        );
        close();
      });
    }
    await t.test(
      "authored helper-like names survive while out-of-scope ones cannot admit generated bindings",
      async () => {
        const label = "__vizeDirectiveValue";
        const prefix = "__vizeDirectiveVal";
        const source = sources.setup
          .replaceAll("greetVisitor", label)
          .replace("= greetVis", `= ${prefix}`);
        open(source);
        const item = await complete(source, label, prefix);
        assertResolved(item, await resolve(item), documentation);
        close();
        const nested = sources.setup.replace(
          script,
          `function outside() { const ${label} = 1; }\n${script}`,
        );
        open(nested);
        const ordinary = await complete(nested);
        assertResolved(ordinary, await resolve(ordinary), documentation);
        close();
        const member = sources.template
          .replaceAll("greetVisitor", label)
          .replace(
            "const api = createApi();",
            `const api = createApi(); const greeting = api.${prefix};`,
          )
          .replace("<template><p>{{ api.greetVis }}</p></template>", "<template />");
        open(member);
        const memberItem = await complete(member, label, prefix);
        assertResolved(memberItem, await resolve(memberItem), documentation);
        close();
      },
    );
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
    await t.test("non-file SFCs retain checker-backed script fallback", async () => {
      const target = "untitled://workspace/Unsaved.vue";
      open(sources.setup, target);
      const item = await complete(sources.setup, "greetVisitor", "greetVis", target);
      assertResolved(item, await resolve(item), documentation);
      close(target);
    });
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
