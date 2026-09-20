import assert from "node:assert/strict";
import { test } from "node:test";
import { PatternSession } from "./support/upstream/pattern-lsp.ts";
import { firstLocation, hoverToText, offsetToPosition } from "./support/lsp/assertions.ts";

for (const receiver of ["attrs", "$attrs"]) {
  test(`${receiver} preserves authored attribute documentation through incomplete edits`, async () => {
    const editor = new PatternSession();
    const source = (member: string) => `<!-- @inferTemplateDollarAttrs true -->
<script setup lang="ts">
import { useAttrs } from 'vue';
declare module 'vue' {
  interface ComponentCustomProperties {
    $attrs: {
      /** **Invoice total** in minor currency units.
       * @example
       * invoiceTotal: 1200
       */
      invoiceTotal: number;
    }
  }
}
const attrs = useAttrs();
${receiver === "attrs" ? `attrs${member};` : "void attrs;"}
</script><template>${receiver === "$attrs" ? `{{ $attrs${member} }}` : ""}</template>`;
    type Item = { label: string; documentation?: { kind: string; value: string } };
    try {
      await editor.initialize();
      for (const member of [".in", ".", "?."]) {
        const text = source(member);
        await editor.update(text);
        const result = (await editor.request(
          "completion",
          text.lastIndexOf(`${receiver}${member}`) + receiver.length + member.length,
        )) as Item[] | { items: Item[] };
        assert.ok(result, `completion must survive ${receiver}${member}`);
        const items = Array.isArray(result) ? result : result.items;
        const item = items.find((item) => item.label === "invoiceTotal");
        assert.ok(item, JSON.stringify(items));
        const resolved = (await editor.session.request("completionItem/resolve", item)) as Item;
        assert.equal(resolved.documentation?.kind, "markdown");
        assert.match(resolved.documentation.value, /```typescript\n.*invoiceTotal: number/s);
        assert.match(resolved.documentation.value, /\*\*Invoice total\*\*/);
        assert.match(resolved.documentation.value, /```tsx\ninvoiceTotal: 1200/);
      }
      const text = source(".invoiceTotal");
      assert.deepEqual(await editor.update(text), []);
      const start = text.lastIndexOf(`${receiver}.invoiceTotal`) + receiver.length + 1;
      const hover = (await editor.request("hover", start + 2)) as Parameters<
        typeof hoverToText
      >[0] & { range: unknown };
      assert.match(hoverToText(hover), /```typescript\n.*invoiceTotal: number/s);
      assert.match(hoverToText(hover), /\*\*Invoice total\*\*/);
      assert.deepEqual(hover?.range, {
        start: offsetToPosition(text, start),
        end: offsetToPosition(text, start + 12),
      });
      const declaration = text.indexOf("invoiceTotal: number");
      assert.deepEqual(
        firstLocation(
          (await editor.request("definition", start + 2)) as Parameters<typeof firstLocation>[0],
        ),
        {
          uri: editor.uri,
          range: {
            start: offsetToPosition(text, declaration),
            end: offsetToPosition(text, declaration + 12),
          },
        },
      );
      const valueHover = hoverToText(
        (await editor.request("hover", start - receiver.length)) as Parameters<
          typeof hoverToText
        >[0],
      );
      assert.doesNotMatch(valueHover, /__vize/i);
    } finally {
      await editor.close();
    }
  });
}

test("inherited native attrs expose usable public types in hook and variable hover", async () => {
  const editor = new PatternSession();
  const source = `<!-- @inferTemplateDollarAttrs true -->
<!-- @fallthroughAttributes true -->
<script setup lang="ts">
import { useAttrs } from 'vue';
const attrs = useAttrs();
attrs.href;
</script><template><a>{{ $attrs.href }}</a></template>`;
  try {
    await editor.initialize();
    assert.deepEqual(await editor.update(source), []);
    for (const [text, offset] of [
      ["attrs.href", 1],
      ["$attrs.href", 2],
      ["useAttrs();", 2],
    ] as const) {
      const hover = hoverToText(
        (await editor.request("hover", source.lastIndexOf(text) + offset)) as Parameters<
          typeof hoverToText
        >[0],
      );
      assert.doesNotMatch(hover, /__vize/i, "hover must not expose generated type aliases");
      assert.match(hover, /Attrs/);
      assert.match(
        hover,
        text === "useAttrs();" ? /NativeElements\["a"\]/ : /AnchorHTMLAttributes/,
      );
    }
  } finally {
    await editor.close();
  }
});
