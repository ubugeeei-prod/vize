import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import ListboxGrid from "./listbox-grid.vue";
import ListboxGridItem from "./listbox-grid-item.vue";

const icons = ["home", "star", "bell", "gear"] as const;
type Icon = (typeof icons)[number];

const SsrProbe = defineComponent({
  name: "ListboxGridSsrProbe",
  setup: () => () =>
    h(
      ListboxGrid<Icon, true>,
      { ariaLabel: "Icons", columns: 2, defaultValue: ["star"], multiple: true, name: "icons" },
      () =>
        icons.map((icon) =>
          h(ListboxGridItem<Icon>, { ariaLabel: icon, key: icon, value: icon }, () => icon),
        ),
    ),
});

test("renders byte-identical ListboxGrid markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /role="listbox"/);
  assert.match(html, /aria-multiselectable="true"/);
  assert.match(html, /data-columns="2"/);
  assert.match(html, /id="vize-v-\d+-listbox-grid-option"/);
  assert.match(
    html,
    /aria-label="star"[^>]*data-vize-ui="listbox-grid-item"[^>]*data-state="checked"/,
  );
  assert.match(html, /type="hidden"[^>]*name="icons"[^>]*value="star"/);
});

test("hydrates ListboxGrid without mismatches or node replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);
  let mounted = false;
  try {
    app.mount(host);
    mounted = true;
    await nextTick();
    assert.ok(host.firstElementChild === serverRoot);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
