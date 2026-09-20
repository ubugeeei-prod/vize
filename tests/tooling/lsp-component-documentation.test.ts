import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { hoverToText, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { PatternSession } from "./support/upstream/pattern-lsp.ts";

const childSource = `<script setup lang="ts">
interface Props {
  /** Heading shown above the invoice total. */
  title: string;
  /** Visual emphasis for overdue invoices. */
  tone?: 'muted' | 'strong';
  /** **Billing** identifier printed on the receipt.
   * @example
   * \`\`\`vue
   * <Invoice billing-code="INV-42" />
   * \`\`\`
   */
  billingCode?: string;
}
defineProps<Props>();
</script><template><h2>{{ title }}</h2></template>`;

function parent(attribute: string): string {
  return `<script setup lang="ts">
import Invoice from './Invoice.vue';
</script><template><Invoice title="April" ${attribute} /></template>`;
}

type Item = Record<string, unknown> & {
  label: string;
  documentation?: { kind: string; value: string };
};

async function complete(
  editor: PatternSession,
  attribute: string,
  label: string,
  offset = editor.source.lastIndexOf(attribute) + attribute.length,
): Promise<Item> {
  const response = (await editor.request("completion", offset)) as Item[] | { items: Item[] };
  const items = Array.isArray(response) ? response : response.items;
  const item = items.find((item) => item.label === label);
  assert.ok(item, JSON.stringify(items));
  assert.ok(
    item.data,
    `component ${label} at ${attribute} must support deferred resolution: ${JSON.stringify(item)}`,
  );
  return item;
}

async function resolve(editor: PatternSession, item: Item): Promise<Item> {
  const resolved = (await editor.session.request("completionItem/resolve", item)) as Item;
  const { detail: _detail, documentation: _documentation, ...insertion } = item;
  const {
    detail: _resolvedDetail,
    documentation: _resolvedDocumentation,
    ...resolvedInsertion
  } = resolved;
  assert.deepEqual(
    resolvedInsertion,
    insertion,
    "documentation must preserve Vue labels, snippets, and authored edits",
  );
  assert.equal(resolved.documentation?.kind, "markdown");
  assert.match(resolved.documentation?.value ?? "", /```typescript\n/);
  return resolved;
}

test("component props retain authored Markdown, exact hover ranges, and Vue completion edits", async () => {
  const editor = new PatternSession();
  try {
    await editor.initialize();
    fs.writeFileSync(path.join(editor.directory, "Invoice.vue"), childSource);
    for (const [attribute, prefix, label, description, type] of [
      ['tone="muted"', "to", "tone", /Visual emphasis for overdue invoices/, /muted.*strong/],
      ['billing-code="INV-42"', "bil", "billing-code", /\*\*Billing\*\* identifier/, /string/],
      [":billing-code=\"'INV-42'\"", ":bil", "billingCode", /\*\*Billing\*\* identifier/, /string/],
    ] as const) {
      const source = parent(attribute);
      assert.deepEqual(await editor.update(source), []);
      const name = attribute.split("=")[0].replace(/^:/, "");
      const start = source.lastIndexOf(name);
      const hover = (await editor.request("hover", start + 2)) as Parameters<
        typeof hoverToText
      >[0] & { range: unknown };
      assert.match(hoverToText(hover), description);
      assert.match(hoverToText(hover), type);
      assert.deepEqual(hover?.range, {
        start: offsetToPosition(source, start),
        end: offsetToPosition(source, start + name.length),
      });
      if (label !== "tone") assert.match(hoverToText(hover), /```vue\n.*billing-code="INV-42"/);

      await editor.update(parent(prefix));
      const item = await complete(editor, prefix, label);
      const resolved = await resolve(editor, item);
      assert.match(resolved.documentation?.value ?? "", description);
      assert.match(resolved.documentation?.value ?? "", type);
      if (label !== "tone")
        assert.match(resolved.documentation?.value ?? "", /```vue\n.*billing-code="INV-42"/);
      assert.deepEqual(
        await editor.session.request("completionItem/resolve", { ...item, label: "different" }),
        { ...item, label: "different" },
      );
      await editor.update(source);
      assert.deepEqual(
        await editor.session.request("completionItem/resolve", item),
        item,
        "an edit invalidates deferred documentation",
      );
    }
  } finally {
    await editor.close();
  }
});

test("component documentation is available before typing an attribute name", async () => {
  const editor = new PatternSession();
  try {
    await editor.initialize();
    fs.writeFileSync(path.join(editor.directory, "Invoice.vue"), childSource);
    for (const prefix of ["", ":", "v-bind:"]) {
      const source = parent(prefix).replace('title="April" ', "");
      await editor.update(source);
      const offset = source.lastIndexOf("<Invoice ") + "<Invoice ".length + prefix.length;
      const item = await complete(editor, prefix, "tone", offset);
      const resolved = await resolve(editor, item);
      assert.match(resolved.documentation?.value ?? "", /Visual emphasis for overdue invoices/);
    }
  } finally {
    await editor.close();
  }
});

test("component hover and completion refresh documentation from an unsaved child", async () => {
  const editor = new PatternSession();
  try {
    await editor.initialize();
    const childPath = path.join(editor.directory, "Invoice.vue");
    fs.writeFileSync(childPath, childSource);
    await editor.update(parent("to"));
    const initial = await resolve(editor, await complete(editor, "to", "tone"));
    assert.match(initial.documentation?.value ?? "", /Visual emphasis for overdue invoices/);

    const childUri = pathToFileURL(childPath).href;
    editor.session.notify("textDocument/didOpen", {
      textDocument: {
        uri: childUri,
        languageId: "vue",
        version: 1,
        text: childSource.replace(
          "Visual emphasis for overdue invoices.",
          "**Updated** emphasis for pending invoices.",
        ),
      },
    });
    await editor.session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, childUri) && params.version === 1,
    );
    const updated = await resolve(editor, await complete(editor, "to", "tone"));
    assert.match(
      updated.documentation?.value ?? "",
      /\*\*Updated\*\* emphasis for pending invoices/,
    );
    assert.doesNotMatch(updated.documentation?.value ?? "", /overdue invoices/);
    const source = parent('tone="muted"');
    await editor.update(source);
    const hover = (await editor.request("hover", source.lastIndexOf('tone="') + 2)) as Parameters<
      typeof hoverToText
    >[0];
    assert.match(hoverToText(hover), /\*\*Updated\*\* emphasis for pending invoices/);
    assert.equal(
      fs.readFileSync(childPath, "utf8"),
      childSource,
      "the editor buffer must override unchanged disk contents",
    );
  } finally {
    await editor.close();
  }
});

test("imported prop interfaces provide typed and documented completion", async () => {
  const editor = new PatternSession();
  try {
    await editor.initialize();
    fs.writeFileSync(
      path.join(editor.directory, "invoice.ts"),
      `export interface InvoiceProps {
  /** **Payment** state used for the invoice badge. */
  paymentState?: 'pending' | 'paid';
}`,
    );
    fs.writeFileSync(
      path.join(editor.directory, "Invoice.vue"),
      `<script setup lang="ts">
import type { InvoiceProps } from './invoice';
defineProps<InvoiceProps>();
</script><template />`,
    );
    const source = parent("pay").replace('title="April" ', "");
    await editor.update(source);
    const resolved = await resolve(editor, await complete(editor, "pay", "payment-state"));
    assert.match(
      resolved.documentation?.value ?? "",
      /\*\*Payment\*\* state used for the invoice badge/,
    );
    assert.match(resolved.documentation?.value ?? "", /pending.*paid|paid.*pending/);
  } finally {
    await editor.close();
  }
});
