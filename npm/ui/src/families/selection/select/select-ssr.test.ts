import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import type { Component } from "vue";
import { renderToString } from "vue/server-renderer";

import SelectContent from "./select-content.vue";
import SelectGroup from "./select-group.vue";
import SelectItem from "./select-item.vue";
import SelectItemIndicator from "./select-item-indicator.vue";
import SelectLabel from "./select-label.vue";
import SelectRoot from "./select-root.vue";
import SelectScrollButton from "./select-scroll-button.vue";
import SelectSeparator from "./select-separator.vue";
import SelectTrigger from "./select-trigger.vue";
import SelectValue from "./select-value.vue";
import SelectViewport from "./select-viewport.vue";
import SelectVirtualizer from "./select-virtualizer.vue";

interface Country {
  readonly code: string;
  readonly name: string;
}

const countries: readonly Country[] = [
  { code: "jp", name: "Japan" },
  { code: "fr", name: "France" },
  { code: "br", name: "Brazil" },
];

function probe(open: boolean): Component {
  return defineComponent({
    name: "SelectSsrProbe",
    setup: () => () =>
      h(
        SelectRoot<Country>,
        {
          by: "code",
          defaultOpen: open,
          defaultValue: countries[1],
          itemText: (country: Country) => country.name,
          items: countries,
          name: "country",
          placeholder: "Country",
          required: true,
        },
        () => [
          h(SelectTrigger, { ariaLabel: "Country" }, () => h(SelectValue)),
          h(SelectContent, null, () => [
            h(SelectScrollButton, { direction: "up" }, () => "▲"),
            h(SelectViewport, null, () => [
              h(SelectGroup, null, () => [
                h(SelectLabel, null, () => "Countries"),
                ...countries.map((country) =>
                  h(SelectItem<Country>, { key: country.code, value: country }, () => [
                    country.name,
                    h(SelectItemIndicator, null, () => "✓"),
                  ]),
                ),
              ]),
              h(SelectSeparator),
            ]),
            h(SelectScrollButton, { direction: "down" }, () => "▼"),
          ]),
        ],
      ),
  });
}

const VirtualProbe = defineComponent({
  name: "SelectVirtualSsrProbe",
  setup: () => () =>
    h(SelectRoot<number>, { defaultOpen: true, items: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] }, () => [
      h(SelectTrigger, { ariaLabel: "Numbers" }, () => h(SelectValue)),
      h(SelectContent, null, () =>
        h(SelectViewport, null, () =>
          h(
            SelectVirtualizer<number>,
            {
              estimateItemSize: 20,
              initialViewportHeight: 40,
              items: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
              overscan: 1,
            },
            {
              default: ({ index, item }: { readonly index: number; readonly item: number }) =>
                h(SelectItem<number>, { index, value: item }, () => `Number ${item}`),
            },
          ),
        ),
      ),
    ]),
});

async function assertHydrates(component: Component): Promise<void> {
  const serverHtml = await renderToString(createSSRApp(component));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverIds = [...host.querySelectorAll("[id]")].map((element) => element.id);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(component);
  let mounted = false;
  try {
    app.mount(host);
    mounted = true;
    assert.ok(host.firstElementChild === serverRoot, "hydration keeps the server root");
    assert.deepEqual(
      serverIds.filter((id) => host.querySelector(`[id="${id}"]`) === null),
      [],
      "every server id survives hydration",
    );
    await nextTick();
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
}

test("renders byte-identical closed Select markup with SSR labels and form mirror", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(probe(false))),
    renderToString(createSSRApp(probe(false))),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /data-vize-ui="select"/);
  assert.match(html, /role="combobox"/);
  assert.match(html, /aria-expanded="false"/);
  assert.match(html, /aria-haspopup="listbox"/);
  assert.match(html, /data-vize-ui="select-value"[^>]*><!--\[-->France<!--\]--><\/span>/);
  assert.match(html, /<select[^>]*data-vize-ui="select-native"[^>]*name="country"[^>]*required/);
  assert.match(html, /<option value="fr" selected[^>]*>fr<\/option>/);
  assert.doesNotMatch(html, /role="listbox"/);
});

test("renders byte-identical open Select markup in place before the portal hydrates", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(probe(true))),
    renderToString(createSSRApp(probe(true))),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /role="listbox"/);
  assert.match(html, /id="vize-v-\d+-select-listbox"/);
  assert.match(html, /role="group"/);
  assert.match(html, /role="option"[^>]*aria-selected="true"[^>]*data-state="checked"/);
  assert.match(html, /data-vize-ui="select-item-indicator"/);
  assert.match(html, /<div hidden aria-hidden="true" data-vize-ui="select-scroll-button"/);
});

test("hydrates closed and open Selects without mismatches", async () => {
  await assertHydrates(probe(false));
  await assertHydrates(probe(true));
});

test("server-renders and hydrates a deterministic virtual window", async () => {
  const html = await renderToString(createSSRApp(VirtualProbe));
  assert.equal(html, await renderToString(createSSRApp(VirtualProbe)));
  const options = html.match(/role="option"/g) ?? [];
  assert.equal(options.length, 3, "two visible rows plus one overscan row");
  assert.match(html, /data-count="10"/);
  await assertHydrates(VirtualProbe);
});
