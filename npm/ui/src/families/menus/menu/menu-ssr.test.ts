import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  MenuArrow,
  MenuCheckboxItem,
  MenuContent,
  MenuGroup,
  MenuItem,
  MenuItemIndicator,
  MenuLabel,
  MenuRadioGroup,
  MenuRadioItem,
  MenuRoot,
  MenuSeparator,
  MenuSub,
  MenuSubContent,
  MenuSubTrigger,
  MenuTrigger,
} from "./menu.ts";

const MenuSsrProbe = defineComponent({
  name: "MenuSsrProbe",
  setup: () => () =>
    h("div", [
      h(MenuRoot, { defaultOpen: true }, () => [
        h(MenuTrigger, null, () => "Actions"),
        h(MenuContent, null, () => [
          h(MenuGroup, null, () => [
            h(MenuLabel, null, () => "File"),
            h(MenuItem, null, () => "New"),
            h(MenuItem, { disabled: true }, () => "Delete"),
          ]),
          h(MenuSeparator),
          h(MenuCheckboxItem, { defaultValue: true }, () => [
            h(MenuItemIndicator, null, () => "✓"),
            "Wrap",
          ]),
          h(MenuRadioGroup<string>, { defaultValue: "a" }, () => [
            h(MenuRadioItem<string>, { value: "a" }, () => "A"),
            h(MenuRadioItem<string>, { value: "b" }, () => "B"),
          ]),
          h(MenuSub, { defaultOpen: true }, () => [
            h(MenuSubTrigger, null, () => "More"),
            h(MenuSubContent, null, () => h(MenuItem, null, () => "Nested")),
          ]),
          h(MenuArrow),
        ]),
      ]),
    ]),
});

test("renders deterministic open menu markup on the server", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(MenuSsrProbe)),
    renderToString(createSSRApp(MenuSsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /aria-haspopup="menu"/);
  assert.match(left, /aria-expanded="true"/);
  assert.match(left, /role="menu"/);
  assert.match(left, /role="menuitemcheckbox"[^>]*aria-checked="true"/);
  assert.match(left, /role="menuitemradio"[^>]*aria-checked="true"/);
  assert.match(left, /aria-disabled="true"/);
  assert.match(left, /data-vize-ui="menu-sub-content"/);
  assert.match(left, /data-vize-ui="menu-arrow"/);
  assert.doesNotMatch(left, /inert|data-vize-scroll-locked|data-highlighted/);
});

test("hydrates without mismatches and then activates focus", async () => {
  const serverHtml = await renderToString(createSSRApp(MenuSsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(MenuSsrProbe);
  try {
    app.mount(host);
    for (let index = 0; index < 4; index++) await nextTick();
    assert.equal(host.firstElementChild, serverRoot);
    assert.deepEqual(diagnostics, []);
    assert.equal(document.querySelectorAll('[role="menu"]').length, 2);
    const content = document.querySelector('[data-vize-ui="menu-content"]');
    const trigger = document.querySelector('[data-vize-ui="menu-trigger"]');
    assert.equal(trigger?.getAttribute("aria-controls"), content?.id);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
