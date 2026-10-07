import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

type Item = { label: string; kind?: number; detail?: string; sortText?: string };
const corpus = path.join(root, "tests/_fixtures/differential/lsp/template-expression-globals-8015");
const original = fs.readFileSync(path.join(corpus, "Comp.vue.txt"), "utf8");
const expected = JSON.parse(
  fs.readFileSync(path.join(corpus, "globals.expected.json"), "utf8"),
) as Item[];
const globalLabels = new Set(expected.map((item) => item.label));

function items(response: unknown): Item[] {
  if (Array.isArray(response)) return response as Item[];
  return (response as { items?: Item[] } | null)?.items ?? [];
}

function assertGlobals(actual: Item[]): void {
  assert.deepEqual(
    actual.filter((item) => globalLabels.has(item.label)),
    expected,
  );
  for (const excluded of ["window", "document", "require", "Promise", "_ctx", "_cache"])
    assert.ok(!actual.some((item) => item.label === excluded), JSON.stringify(actual));
}

for (const typecheck of [false, true]) {
  test(`Vue template expression globals retain original scopes with typecheck=${typecheck}`, async () => {
    fs.mkdirSync(testOutputRoot, { recursive: true });
    const directory = fs.mkdtempSync(path.join(testOutputRoot, "template-globals-8015-"));
    for (const name of ["Child", "Comp", "List", "App"])
      fs.copyFileSync(path.join(corpus, `${name}.vue.txt`), path.join(directory, `${name}.vue`));
    fs.writeFileSync(
      path.join(directory, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: { strict: true, moduleResolution: "Bundler", module: "ESNext" },
        include: ["*.vue"],
      }),
    );
    const dependencies = path.join(root, "tests/node_modules");
    if (fs.existsSync(dependencies))
      fs.symlinkSync(dependencies, path.join(directory, "node_modules"), "dir");
    const session = new LspSession();
    const uri = pathToFileURL(path.join(directory, "Comp.vue")).href;
    let source = original;
    let version = 1;
    try {
      await session.initialize(directory, { editor: true, lint: false, typecheck });
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version, text: source },
      });
      const settled = () =>
        session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) => isDiagnosticsForUri(params, uri) && params.version === version,
        );
      await settled();
      const complete = async (marker: string): Promise<Item[]> => {
        const offset = source.indexOf(marker);
        assert.ok(offset >= 0, marker);
        return items(
          await session.request("textDocument/completion", {
            textDocument: { uri },
            position: offsetToPosition(source, offset + marker.length),
          }),
        );
      };
      const update = async (text: string): Promise<void> => {
        source = text;
        version++;
        session.notify("textDocument/didChange", {
          textDocument: { uri, version },
          contentChanges: [{ text: source }],
        });
        await settled();
      };
      const originals: Item[][] = [];
      for (const marker of ["$e", "{{ St", "{{ Ma"]) {
        const actual = await complete(marker);
        assertGlobals(actual);
        for (const name of ["Child", "emit", "value"])
          assert.equal(actual.filter((item) => item.label === name).length, 1);
        assert.equal(
          actual.filter((item) => item.label === "$event").length,
          Number(marker === "$e"),
        );
        originals.push(actual);
      }
      const global = originals[0].find((item) => item.label === "$emit")!;
      assert.deepEqual(await session.request("completionItem/resolve", global), global);

      await update(`<!-- unsaved 東京 🧭 -->\r\n${original.replaceAll("\n", "\r\n")}`);
      for (const [index, marker] of ["$e", "{{ St", "{{ Ma"].entries())
        assert.deepEqual(await complete(marker), originals[index]);
      await update(original);
      for (const [index, marker] of ["$e", "{{ St", "{{ Ma"].entries())
        assert.deepEqual(await complete(marker), originals[index]);

      await update(`<script setup lang="ts">
const String = 'local';
defineProps<{ Math: number; rows: string[] }>();
</script>
<template><p v-for="Array in rows">{{ St }}</p></template>`);
      const shadowed = await complete("{{ St");
      for (const name of ["String", "Math", "Array"]) {
        const matching = shadowed.filter((item) => item.label === name);
        assert.equal(matching.length, 1, JSON.stringify(shadowed));
        assert.notDeepEqual(
          matching[0],
          expected.find((item) => item.label === name),
        );
      }
      assert.equal(shadowed.find((item) => item.label === "Math")?.detail, "prop: number");
      assert.equal(shadowed.find((item) => item.label === "Array")?.detail, "Local v-for binding");

      for (const [text, marker] of [
        ["<template><p>{{ St }}</p></template>", "{{ St"],
        ["<template><p :title= /></template>", ":title="],
        ["<template><button @click= /></template>", "@click="],
      ]) {
        await update(text);
        assertGlobals(await complete(marker));
      }
      for (const [text, marker] of [
        ['<template><p title="St" /></template>', 'title="St'],
        ["<template><p cl /></template>", "<p cl"],
        ["<template><!-- St --></template>", "<!-- St"],
      ]) {
        await update(text);
        assert.deepEqual(
          (await complete(marker)).filter((item) => globalLabels.has(item.label)),
          [],
        );
      }
      session.notify("textDocument/didClose", { textDocument: { uri } });
      assert.deepEqual(await session.request("completionItem/resolve", global), global);
    } finally {
      await session.shutdown();
      fs.rmSync(directory, { recursive: true, force: true });
    }
  });
}
