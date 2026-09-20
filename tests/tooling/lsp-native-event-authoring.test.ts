import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { firstLocation, hoverToText, offsetToPosition } from "./support/lsp/assertions.ts";
import { PatternSession } from "./support/upstream/pattern-lsp.ts";
import { check, diagnosticIdentity } from "./support/upstream/vue-language-tools.ts";
import { vueTscDiagnostics } from "./support/vue-tsc-oracle.ts";

const script = `<script setup lang="ts">
import { defineComponent } from 'vue';
interface Invoice {
  /** **Invoice total** in the customer's currency.
   * @example
   * total.toFixed(2)
   */
  total: number;
}
const invoice: Invoice = { total: 42 };
const Native = defineComponent({ emits: { 'row-picked': (_value: Invoice) => true } });
declare const Generic: new <T>(props: { value: T }) => {
  $props: { value: T };
  $emit: {
    (event: 'rowPicked', value: T): void;
    (event: 'reset'): void;
  };
};
</script>`;

function source(component: string, expression: string): string {
  const props = component === "Generic" ? ':value="invoice" ' : "";
  return `${script}\n<template><${component} ${props}@row-picked="${expression}" /></template>`;
}

for (const component of ["Native", "Generic"]) {
  test(`${component} event payloads retain types and documentation through incomplete edits`, async () => {
    const editor = new PatternSession();
    try {
      await editor.initialize();
      for (const expression of [
        "$event.total.toFixed(2)",
        "$event.missing()",
        "$event.total.toFixed(2)",
      ]) {
        const text = source(component, expression);
        fs.writeFileSync(editor.file, text);
        const expected = vueTscDiagnostics(editor.directory);
        assert.equal(expected.length, expression.includes("missing") ? 1 : 0);
        if (expected.length) assert.equal(expected[0].code, 2339);
        assert.deepEqual((await check(editor.directory)).map(diagnosticIdentity), expected);
        assert.deepEqual(
          (await editor.update(text)).map((d) => ({
            file: "App.vue",
            line: d.range.start.line + 1,
            column: d.range.start.character + 1,
            code: Number(d.code),
          })),
          expected,
        );
      }
      const text = editor.source;
      const start = text.lastIndexOf("total.toFixed");
      const hover = (await editor.request("hover", start + 2)) as Parameters<
        typeof hoverToText
      >[0] & { range: unknown };
      assert.match(hoverToText(hover), /\*\*Invoice total\*\*/);
      assert.match(hoverToText(hover), /```typescript\n/);
      assert.match(hoverToText(hover), /total: number/);
      assert.deepEqual(hover.range, {
        start: offsetToPosition(text, start),
        end: offsetToPosition(text, start + "total".length),
      });
      const declaration = text.indexOf("total: number");
      assert.deepEqual(
        firstLocation(
          (await editor.request("definition", start + 2)) as Parameters<typeof firstLocation>[0],
        ),
        {
          uri: editor.uri,
          range: {
            start: offsetToPosition(text, declaration),
            end: offsetToPosition(text, declaration + "total".length),
          },
        },
      );

      await editor.update(source(component, "$event."));
      type Item = { label: string; data?: unknown };
      const completion = (await editor.request(
        "completion",
        editor.source.lastIndexOf("$event.") + "$event.".length,
      )) as Item[] | { items: Item[] };
      assert.ok(completion, "completion must survive the incomplete event expression");
      const items = Array.isArray(completion) ? completion : completion.items;
      const item = items.find((item) => item.label === "total");
      assert.ok(item, JSON.stringify(completion));
      const resolved = (await editor.session.request("completionItem/resolve", item)) as {
        documentation: { kind: string; value: string };
      };
      assert.equal(resolved.documentation.kind, "markdown");
      assert.match(resolved.documentation.value, /\*\*Invoice total\*\*/);
      assert.match(resolved.documentation.value, /```typescript\n/);
      assert.match(resolved.documentation.value, /total\.toFixed\(2\)/);
      assert.deepEqual(await editor.update(source(component, "$event.total.toFixed(2)")), []);
    } finally {
      await editor.close();
    }
  });
}
