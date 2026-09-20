import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { workspace } from "./support/upstream/vue-language-tools.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  firstLocation,
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "./support/lsp/assertions.ts";

type Item = Record<string, unknown>;
const api = `const api = {
  /**
   * Format the total shown on an invoice.
   *
   * Supports **JPY** and **USD** currencies.
   * @param amount Amount before currency formatting.
   * @param currency Currency used on the invoice.
   * @returns A localized invoice total.
   * @example
   * api.formatTotal(1200, 'JPY')
   */
  formatTotal(amount: number, currency: 'JPY' | 'USD' = 'JPY'): string {
    return amount.toFixed() + currency;
  }
};
function getApi() { return api; }`;
const surfaces = {
  templateWithIncompleteScript: (expression: string) =>
    `<script setup lang="ts">\n${api}\nconst pending = api.\n</script>\n<template><p>{{ ${expression} }}</p></template>`,
  template: (expression: string) =>
    `<script setup lang="ts">\n${api}\n</script>\n<template><p>{{ ${expression} }}</p></template>`,
  sfcTsx: (expression: string) =>
    `<script setup lang="tsx">\n${api}\nconst view = <div>{${expression}}</div>;\n</script>`,
  tsx: (expression: string) => `${api}\nconst view = <div>{${expression}}</div>;\n`,
};

for (const [surface, source] of Object.entries(surfaces)) {
  test(`${surface} authoring keeps documented completion, rich hover and exact navigation across incomplete edits`, async () => {
    const directory = workspace("rich-authoring-");
    const file = path.join(directory, surface === "tsx" ? "App.tsx" : "App.vue");
    const uri = pathToFileURL(file).href;
    const session = new LspSession();
    let version = 0;
    let text = "";
    fs.writeFileSync(file, source("api.formatTotal(1200)"));
    fs.writeFileSync(
      path.join(directory, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          skipLibCheck: true,
          target: "ESNext",
          module: "ESNext",
          moduleResolution: "Bundler",
          jsx: "preserve",
        },
        include: ["*.vue", "*.tsx"],
      }),
    );
    fs.writeFileSync(
      path.join(directory, "vize.config.json"),
      JSON.stringify({
        lsp: { editor: true, typecheck: true, lint: false },
        typeChecker: { jsxTypecheck: true },
      }),
    );
    async function update(expression: string) {
      text = source(expression);
      version++;
      if (version === 1)
        session.notify("textDocument/didOpen", {
          textDocument: {
            uri,
            languageId: surface === "tsx" ? "typescriptreact" : "vue",
            version,
            text,
          },
        });
      else
        session.notify("textDocument/didChange", {
          textDocument: { uri, version },
          contentChanges: [{ text }],
        });
      await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, uri) && params.version === version,
      );
    }
    function request(method: string, offset: number) {
      return session.request(`textDocument/${method}`, {
        textDocument: { uri },
        position: offsetToPosition(text, offset),
      });
    }
    try {
      await session.initialize(directory, { editor: true, typecheck: true, lint: false });
      for (const expression of ["api.for", "api.", "getApi().", "api?."]) {
        await update(expression);
        const response = (await request(
          "completion",
          text.lastIndexOf(expression) + expression.length,
        )) as Item[] | { items: Item[] } | null;
        const items = Array.isArray(response) ? response : (response?.items ?? []);
        const item = items.find((item) => item.label === "formatTotal");
        assert.ok(item, `${surface}/${expression}: ${JSON.stringify(items)}`);
        const resolved = (await session.request("completionItem/resolve", item)) as Item;
        assert.match(String(resolved.detail), /formatTotal\(amount: number, currency\?:/);
        const documentation = resolved.documentation as
          | { kind?: string; value?: string }
          | undefined;
        assert.equal(documentation?.kind, "markdown", JSON.stringify(resolved));
        assert.match(documentation?.value ?? "", /```typescript\n.*formatTotal/s);
        assert.match(documentation?.value ?? "", /Format the total shown on an invoice/);
        assert.match(documentation?.value ?? "", /Amount before currency formatting/);
        assert.match(documentation?.value ?? "", /A localized invoice total/);
        assert.match(documentation?.value ?? "", /api\.formatTotal\(1200, 'JPY'\)/);
        const declarationHover = (await request(
          "hover",
          text.indexOf("formatTotal(amount") + 2,
        )) as Parameters<typeof hoverToText>[0];
        assert.match(hoverToText(declarationHover), /Format the total shown on an invoice/);
      }
      await update("api.formatTotal(1200)");
      const offset = text.lastIndexOf("formatTotal") + 2;
      const hover = (await request("hover", offset)) as Parameters<typeof hoverToText>[0];
      const hoverText = hoverToText(hover);
      assert.match(hoverText, /```typescript/);
      assert.match(hoverText, /Format the total shown on an invoice/);
      assert.match(hoverText, /Amount before currency formatting/);
      assert.match(hoverText, /A localized invoice total/);
      const target = firstLocation(
        (await request("definition", offset)) as Parameters<typeof firstLocation>[0],
      );
      assert.deepEqual(target, {
        uri,
        range: {
          start: offsetToPosition(text, text.indexOf("formatTotal(amount")),
          end: offsetToPosition(text, text.indexOf("formatTotal(amount") + "formatTotal".length),
        },
      });
    } finally {
      await session.shutdown();
      fs.rmSync(directory, { recursive: true, force: true });
    }
  });
}
