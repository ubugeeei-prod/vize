import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import type { Component } from "vue";
import { renderToString } from "vue/server-renderer";

import SelectContent from "../select/select-content.vue";
import SelectItem from "../select/select-item.vue";
import ComboboxAnchor from "./combobox-anchor.vue";
import ComboboxChip from "./combobox-chip.vue";
import ComboboxChipRemove from "./combobox-chip-remove.vue";
import ComboboxCreateItem from "./combobox-create-item.vue";
import ComboboxEmpty from "./combobox-empty.vue";
import ComboboxInput from "./combobox-input.vue";
import ComboboxLoading from "./combobox-loading.vue";
import ComboboxRoot from "./combobox-root.vue";
import ComboboxTrigger from "./combobox-trigger.vue";
import type { ComboboxSlotState } from "./combobox-types.ts";

type Language = string;
const languages: readonly Language[] = ["TypeScript", "Rust", "Go"];

function probe(open: boolean, multiple: boolean): Component {
  return defineComponent({
    name: "ComboboxSsrProbe",
    setup: () => () =>
      h(
        ComboboxRoot<Language, boolean>,
        {
          createOption: (text: string) => text,
          defaultOpen: open,
          defaultValue: multiple ? ["Rust"] : "Rust",
          items: languages,
          multiple,
          name: "language",
        },
        {
          default: (state: ComboboxSlotState<Language>) => [
            h(ComboboxAnchor, null, () => [
              ...state.selected.map((language) =>
                h(ComboboxChip<Language>, { key: language, value: language }, () => [
                  language,
                  h(ComboboxChipRemove, null, () => "×"),
                ]),
              ),
              h(ComboboxInput, { ariaLabel: "Language" }),
              h(ComboboxTrigger, null, () => "▾"),
            ]),
            h(SelectContent, null, () => [
              h(ComboboxLoading, null, () => "Loading"),
              ...state.filteredItems.map((language) =>
                h(SelectItem<Language>, { key: language, value: language }, () => language),
              ),
              h(ComboboxCreateItem, null, () => "Create"),
              h(ComboboxEmpty, null, () => "Nothing"),
            ]),
          ],
        },
      ),
  });
}

async function assertHydrates(component: Component): Promise<void> {
  const serverHtml = await renderToString(createSSRApp(component));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
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
    assert.ok(host.firstElementChild === serverRoot);
    await nextTick();
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
}

test("renders byte-identical single Combobox markup with the selected label", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(probe(false, false))),
    renderToString(createSSRApp(probe(false, false))),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /data-vize-ui="combobox"/);
  assert.match(html, /<input[^>]*role="combobox"[^>]*value="Rust"/);
  assert.match(html, /aria-autocomplete="list"/);
  assert.match(html, /aria-expanded="false"/);
  assert.match(
    html,
    /<input type="hidden" data-vize-ui="combobox-native"[^>]*name="language"[^>]*value="Rust"/,
  );
});

test("renders byte-identical open multiple Combobox markup with chips", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(probe(true, true))),
    renderToString(createSSRApp(probe(true, true))),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /data-vize-ui="combobox-chip"/);
  assert.match(html, /aria-label="Remove Rust"/);
  assert.match(html, /role="listbox"[^>]*aria-multiselectable="true"/);
  assert.match(html, /data-vize-ui="combobox-item"/);
  assert.match(html, /data-vize-ui="combobox-create-item"/);
});

test("hydrates closed single and open multiple Comboboxes without mismatches", async () => {
  await assertHydrates(probe(false, false));
  await assertHydrates(probe(true, true));
});
