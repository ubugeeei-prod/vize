import assert from "node:assert/strict";
import { test } from "node:test";
import { PatternSession } from "./support/upstream/pattern-lsp.ts";
import { firstLocation, hoverToText, offsetToPosition } from "./support/lsp/assertions.ts";

for (const [receiver, inline] of [
  ["slots", false],
  ["$slots", false],
  ["slots", true],
  ["$slots", true],
] as const) {
  test(`${receiver} exposes ${inline ? "inline" : "named"} slot documentation and navigation after incomplete edits`, async () => {
    const editor = new PatternSession();
    const slotType = `{
  /** **Invoice summary** shown above the total.
   * @param props Customer-visible invoice data.
   * @example
   * summary({ total: 1200 })
   */
  summary(props: { total: number }): any;
}`;
    const source = (member: string) => `<!-- @inferTemplateDollarSlots true -->
<script setup lang="ts">
import { useSlots } from 'vue';
${inline ? `defineSlots<${slotType}>();` : `type InvoiceSlots = ${slotType};\ndefineSlots<InvoiceSlots>();`}
const slots = useSlots();
${receiver === "slots" ? `slots${member}` : "void slots;"}
</script><template>
<slot name="summary" :total="1200" />
${receiver === "$slots" ? `{{ $slots${member} }}` : ""}
</template>`;
    type Item = { label: string; documentation?: { kind: string; value: string } };
    try {
      await editor.initialize();
      for (const member of [
        ".su",
        ".",
        "?.",
        ...(receiver === "slots" ? [".({ total: 1200 })"] : []),
      ]) {
        const text = source(member);
        await editor.update(text);
        const expression = `${receiver}${member.split("(")[0]}`;
        const response = (await editor.request(
          "completion",
          text.lastIndexOf(expression) + expression.length,
        )) as Item[] | { items: Item[] };
        assert.ok(response, `completion must survive the incomplete ${expression} expression`);
        const items = Array.isArray(response) ? response : response.items;
        const item = items.find((item) => item.label === "summary");
        assert.ok(item, JSON.stringify(items));
        const resolved = (await editor.session.request("completionItem/resolve", item)) as Item;
        assert.equal(resolved.documentation?.kind, "markdown");
        assert.match(resolved.documentation?.value ?? "", /```typescript\n.*summary/s);
        assert.match(resolved.documentation?.value ?? "", /\*\*Invoice summary\*\*/);
        assert.match(resolved.documentation?.value ?? "", /Customer-visible invoice data/);
        assert.match(resolved.documentation?.value ?? "", /summary\(\{ total: 1200 \}\)/);
      }
      const text = source(".summary({ total: 1200 })");
      assert.deepEqual(await editor.update(text), []);
      const start = text.lastIndexOf(`${receiver}.summary`) + receiver.length + 1;
      const receiverHover = hoverToText(
        (await editor.request("hover", start - receiver.length)) as Parameters<
          typeof hoverToText
        >[0],
      );
      assert.doesNotMatch(
        receiverHover,
        /__vize/i,
        "private aliases must not hide useful slot types",
      );
      assert.match(receiverHover, inline ? /summary.*total: number/s : /InvoiceSlots/);
      if (receiver === "slots") {
        const hookHover = hoverToText(
          (await editor.request("hover", text.indexOf("useSlots();") + 2)) as Parameters<
            typeof hoverToText
          >[0],
        );
        assert.doesNotMatch(hookHover, /__vize/i);
        assert.match(hookHover, inline ? /summary.*total: number/s : /InvoiceSlots/);
      }
      const hover = (await editor.request("hover", start + 2)) as Parameters<
        typeof hoverToText
      >[0] & { range: unknown };
      assert.match(hoverToText(hover), /```typescript\n.*summary/s);
      assert.match(hoverToText(hover), /\*\*Invoice summary\*\*/);
      assert.deepEqual(hover?.range, {
        start: offsetToPosition(text, start),
        end: offsetToPosition(text, start + 7),
      });
      const declaration = text.indexOf("summary(props:");
      assert.deepEqual(
        firstLocation(
          (await editor.request("definition", start + 2)) as Parameters<typeof firstLocation>[0],
        ),
        {
          uri: editor.uri,
          range: {
            start: offsetToPosition(text, declaration),
            end: offsetToPosition(text, declaration + 7),
          },
        },
      );
    } finally {
      await editor.close();
    }
  });
}
