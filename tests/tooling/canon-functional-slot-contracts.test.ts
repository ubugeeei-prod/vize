import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { PatternSession } from "./support/upstream/pattern-lsp.ts";
import {
  check,
  compareIdentity,
  diagnosticIdentity,
} from "./support/upstream/vue-language-tools.ts";
import { vueTscDiagnostics } from "./support/vue-tsc-oracle.ts";
import { firstLocation, hoverToText, offsetToPosition } from "./support/lsp/assertions.ts";

async function compare(editor: PatternSession, source: string, errors: number): Promise<void> {
  fs.writeFileSync(editor.file, source);
  const expected = vueTscDiagnostics(editor.directory).sort(compareIdentity);
  assert.equal(expected.length, errors, JSON.stringify(expected));
  assert.deepEqual(
    (await check(editor.directory)).map(diagnosticIdentity).sort(compareIdentity),
    expected,
  );
  assert.deepEqual(
    (await editor.update(source))
      .map((d) => ({
        file: "App.vue",
        line: d.range.start.line + 1,
        column: d.range.start.character + 1,
        code: Number(d.code),
      }))
      .sort(compareIdentity),
    expected,
  );
}

for (const [name, slots, directive] of [
  ["static optional", "item?(props: { value: string }): void;", "#item"],
  ["string index", "[name: string]: (props: { value: string }) => void;", "#item"],
  ["dynamic string index", "[name: string]: (props: { value: string }) => void;", "#[name]"],
  [
    "template literal index",
    "[name: `item-${string}`]: (props: { value: string }) => void;",
    "#[`item-${name}`]",
  ],
] as const) {
  test(`functional component ${name} slot keeps its declared payload`, async () => {
    const editor = new PatternSession();
    const valid = `<script setup lang="ts">
declare const name: 'item';
declare const Comp: (props: {}, context: { slots: { ${slots} } }) => {};
</script><template><Comp><template ${directive}="{ value }">
{{ value.toUpperCase() }}
</template></Comp></template>`;
    try {
      await editor.initialize();
      await compare(editor, valid, 0);
      await compare(editor, valid.replace("value.toUpperCase()", "value.toFixed()"), 1);
      await compare(editor, valid, 0);
    } finally {
      await editor.close();
    }
  });
}

test("scoped slot completion and hover retain documentation and navigation during incomplete edits", async () => {
  const editor = new PatternSession();
  const source = (expression: string) => `<script setup lang="ts">
declare const Comp: (props: {}, context: { slots: {
  item(props: {
    /** **Invoice** total shown to the customer.
     * @example
     * total.toFixed(2)
     */
    total: number;
    currency: 'JPY' | 'USD';
  }): void;
} }) => {};
</script><template><Comp><template #item="invoice">
{{ ${expression} }}
</template></Comp></template>`;
  type Item = Record<string, unknown> & {
    label: string;
    documentation?: { kind: string; value: string };
  };
  try {
    await editor.initialize();
    for (const expression of ["invoice.to", "invoice.", "invoice?."]) {
      const text = source(expression);
      await editor.update(text);
      const response = (await editor.request(
        "completion",
        text.lastIndexOf(expression) + expression.length,
      )) as Item[] | { items: Item[] };
      const items = Array.isArray(response) ? response : response.items;
      const item = items.find((item) => item.label === "total");
      assert.ok(item, JSON.stringify(items));
      const resolved = (await editor.session.request("completionItem/resolve", item)) as Item;
      assert.equal(resolved.documentation?.kind, "markdown");
      assert.match(resolved.documentation?.value ?? "", /```typescript\n.*total: number/s);
      assert.match(resolved.documentation?.value ?? "", /\*\*Invoice\*\* total/);
      assert.match(resolved.documentation?.value ?? "", /total\.toFixed\(2\)/);
    }
    const text = source("invoice.total.toFixed(2)");
    assert.deepEqual(await editor.update(text), []);
    const start = text.lastIndexOf("total.toFixed");
    const hover = (await editor.request("hover", start + 2)) as Parameters<
      typeof hoverToText
    >[0] & { range: unknown };
    assert.match(hoverToText(hover), /```typescript\n.*total: number/s);
    assert.match(hoverToText(hover), /\*\*Invoice\*\* total/);
    assert.deepEqual(hover?.range, {
      start: offsetToPosition(text, start),
      end: offsetToPosition(text, start + "total".length),
    });
    const target = firstLocation(
      (await editor.request("definition", start + 2)) as Parameters<typeof firstLocation>[0],
    );
    assert.deepEqual(target, {
      uri: editor.uri,
      range: {
        start: offsetToPosition(text, text.indexOf("total: number")),
        end: offsetToPosition(text, text.indexOf("total: number") + "total".length),
      },
    });
  } finally {
    await editor.close();
  }
});

for (const [kind, declaration] of [
  [
    "constructor",
    "new <T>(props: { items: T[] }) => { $props: typeof props; $slots: { default(props: { item: T }): void } }",
  ],
  [
    "functional",
    "<T>(props: { items: T[] }, context?: { slots: { default(props: { item: T }): void } }) => import('vue').VNode & { __ctx?: { slots: { default(props: { item: T }): void } } }",
  ],
  [
    "published functional declaration",
    "<T>(props: NonNullable<Awaited<typeof setup>>['props'], context?: Pick<NonNullable<Awaited<typeof setup>>, 'slots'>, expose?: (value: T) => void, setup?: Promise<{ props: { items: T[] }; slots: { default(props: { item: T }): void } }>) => import('vue').VNode & { __ctx?: Awaited<typeof setup> }",
  ],
] as const) {
  test(`generic ${kind} slots infer payloads from authored props`, async () => {
    const editor = new PatternSession();
    const valid = `<script setup lang="ts">
declare const Comp: ${declaration};
const invoices = [{ total: 1200, currency: 'JPY' }];
</script><template><Comp :items="invoices" v-slot="{ item }">
{{ item.total.toFixed(2) }} {{ item.currency.toUpperCase() }}
</Comp></template>`;
    try {
      await editor.initialize();
      await compare(editor, valid, 0);
      await compare(editor, valid.replace("item.total.toFixed(2)", "item.total.toUpperCase()"), 1);
      await compare(
        editor,
        valid.replace("item.currency.toUpperCase()", "item.currency.toFixed()"),
        1,
      );
      await compare(editor, valid, 0);
    } finally {
      await editor.close();
    }
  });
}

for (const additional of ["", "unrelated(props: { other: boolean }): void;"]) {
  test(`functional component slot payloads follow dynamic names${additional ? " without unrelated slots" : ""}`, async () => {
    const editor = new PatternSession();
    const valid = `<script setup lang="ts">
declare const Comp: (props: {}, context: { slots: {
  foo(props: { foo: string }): void;
  bar(props: { bar: string }): void;
  ${additional}
} }) => {};
</script><template>
<Comp>
  <template v-for="name in (['foo', 'bar'] as const)" #[name]="slot">
    {{ 'foo' in slot ? slot.foo.toUpperCase() : slot.bar.toUpperCase() }}
  </template>
</Comp>
</template>`;
    try {
      await editor.initialize();
      await compare(editor, valid, 0);
      await compare(editor, valid.replace("slot.foo.toUpperCase()", "slot.foo.toFixed()"), 1);
      await compare(editor, valid.replace("slot.bar.toUpperCase()", "slot.bar.toFixed()"), 1);
      await compare(editor, valid, 0);
    } finally {
      await editor.close();
    }
  });
}

for (const directive of ["slot", "prop"] as const) {
  test(`incomplete dynamic ${directive} names keep documented completion and recover typed payloads`, async () => {
    const editor = new PatternSession();
    const source = (name: string) => `<script setup lang="ts">
declare const Comp: (props: { item?: string }, context: { slots: { item(props: { total: number }): void } }) => {};
const names = {
  /** **Primary** invoice slot shown in the summary. */
  current: 'item' as const,
};
</script><template><Comp ${directive === "prop" ? `:[${name}]="'details'"` : ""}><template ${directive === "slot" ? `#[${name}]` : "#item"}="invoice">
{{ invoice.total.toFixed(2) }}
</template></Comp></template>`;
    try {
      await editor.initialize();
      const incomplete = source("names.");
      await editor.update(incomplete);
      type Item = { label: string; documentation?: { kind: string; value: string } };
      const response = (await editor.request(
        "completion",
        incomplete.lastIndexOf("names.") + "names.".length,
      )) as Item[] | { items: Item[] };
      const items = Array.isArray(response) ? response : response.items;
      const item = items.find((item) => item.label === "current");
      assert.ok(item, JSON.stringify(items));
      const resolved = (await editor.session.request("completionItem/resolve", item)) as Item;
      assert.equal(resolved.documentation?.kind, "markdown");
      assert.match(resolved.documentation?.value ?? "", /\*\*Primary\*\* invoice slot/);
      assert.match(resolved.documentation?.value ?? "", /```typescript\n/);
      const repaired = source("names.current");
      assert.deepEqual(await editor.update(repaired), []);
      const hover = (await editor.request(
        "hover",
        repaired.lastIndexOf("current") + 2,
      )) as Parameters<typeof hoverToText>[0];
      assert.match(hoverToText(hover), /\*\*Primary\*\* invoice slot/);
      assert.deepEqual(
        firstLocation(
          (await editor.request("definition", repaired.lastIndexOf("current") + 2)) as Parameters<
            typeof firstLocation
          >[0],
        ),
        {
          uri: editor.uri,
          range: {
            start: offsetToPosition(repaired, repaired.indexOf("current:")),
            end: offsetToPosition(repaired, repaired.indexOf("current:") + "current".length),
          },
        },
      );
    } finally {
      await editor.close();
    }
  });
}
