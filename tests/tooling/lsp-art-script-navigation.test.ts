import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

// The quoted member mirrors a generated typed-router declaration.
const declarations = `declare function useRoute(name: '/art'): {
  params: { id: number; 'page': number }
}
`;

const page = (body: string) => `<script setup lang="ts">
// 雪😀
const route = useRoute('/art')
const page: number = route.params.page
</script>
${body}
`;

test("art script navigation answers from the typed setup projection", async () => {
  fs.mkdirSync(testOutputRoot, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(testOutputRoot, "art-script-navigation-"));
  const session = new LspSession();
  try {
    fs.writeFileSync(
      path.join(workspace, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: { strict: true, target: "ESNext", module: "ESNext", noEmit: true },
        include: ["./**/*"],
      }),
    );
    // A tsconfig-included ambient declaration is shared with the private
    // diagnostic mirror project, whose files must never reach the client.
    const declarationFile = path.join(workspace, "route.d.ts");
    fs.writeFileSync(declarationFile, declarations);
    const declarationUri = pathToFileURL(declarationFile).href;
    await session.initialize(workspace, { editor: true, lint: false, typecheck: true });

    const open = async (relative: string, text: string) => {
      const file = path.join(workspace, relative);
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, text);
      const uri = pathToFileURL(file).href;
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "art-vue", version: 1, text },
      });
      const published = (await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, uri),
      )) as { diagnostics: unknown[]; version?: number };
      return { uri, published };
    };
    const query = async (uri: string, source: string) => {
      const position = offsetToPosition(
        source,
        source.indexOf("route.params.page") + "route.params.p".length,
      );
      const references = async (includeDeclaration: boolean) =>
        session.request("textDocument/references", {
          textDocument: { uri },
          position,
          context: { includeDeclaration },
        });
      return {
        hover: await session.request("textDocument/hover", { textDocument: { uri }, position }),
        definition: await session.request("textDocument/definition", {
          textDocument: { uri },
          position,
        }),
        references: await references(true),
        usages: await references(false),
      };
    };
    const range = (source: string, offset: number, length: number) => ({
      start: offsetToPosition(source, offset),
      end: offsetToPosition(source, offset + length),
    });

    const variants = page(
      `<art title="Router"><variant name="Default"><div>{{ route.params.page.toFixed() }}</div></variant></art>`,
    );
    const proper = await open("src/pages/[id=int].art.vue", variants);
    assert.deepEqual(proper.published.diagnostics, []);
    const declaration = declarations.indexOf("'page'");
    const script = {
      uri: proper.uri,
      range: range(variants, variants.indexOf("route.params.page") + "route.params.".length, 4),
    };
    const template = {
      uri: proper.uri,
      range: range(variants, variants.lastIndexOf("route.params.page") + "route.params.".length, 4),
    };
    const expected = {
      hover: {
        contents: { kind: "markdown", value: "```typescript\n(property) 'page': number\n```" },
        range: script.range,
      },
      definition: { uri: declarationUri, range: range(declarations, declaration, 6) },
      references: [
        { uri: declarationUri, range: range(declarations, declaration + 1, 4) },
        script,
        template,
      ],
      usages: [script, template],
    };
    assert.deepEqual(await query(proper.uri, variants), expected);

    // Reopening the same version must not reuse a retired projection.
    session.notify("textDocument/didClose", { textDocument: { uri: proper.uri } });
    session.notify("textDocument/didOpen", {
      textDocument: { uri: proper.uri, languageId: "art-vue", version: 1, text: variants },
    });
    assert.deepEqual(await query(proper.uri, variants), expected);

    // An ordinary SFC template is not an Art form: refuse it explicitly.
    const plainSource = page(`<template><div>{{ route.params.page.toFixed() }}</div></template>`);
    const plain = await open("src/pages/plain/[id=int].art.vue", plainSource);
    assert.deepEqual(plain.published.diagnostics, [
      {
        range: range(plainSource, plainSource.indexOf("<template>"), "<template>".length),
        severity: 1,
        code: "invalid-art",
        source: "vize/types",
        message:
          "Art files need an `<art>` block with `<variant>` templates; this `<template>` is not type checked.",
      },
    ]);
    assert.deepEqual(await query(plain.uri, plainSource), {
      hover: null,
      definition: null,
      references: null,
      usages: null,
    });
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
