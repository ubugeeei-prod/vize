import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  MenubarContent,
  MenubarItem,
  MenubarMenu,
  MenubarRoot,
  MenubarTrigger,
} from "./menubar.ts";

const Probe = defineComponent({
  setup: () => () =>
    h("div", [
      h(MenubarRoot, { defaultValue: "edit", ariaLabel: "App" }, () => [
        h(MenubarMenu, { value: "file" }, () => [
          h(MenubarTrigger, null, () => "File"),
          h(MenubarContent, null, () => h(MenubarItem, null, () => "New")),
        ]),
        h(MenubarMenu, { value: "edit" }, () => [
          h(MenubarTrigger, null, () => "Edit"),
          h(MenubarContent, null, () => h(MenubarItem, null, () => "Undo")),
        ]),
      ]),
    ]),
});

test("renders identical menubar markup across SSR requests and hydrates cleanly", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /role="menubar"/);
  assert.match(left, /data-value="edit"[^>]*data-highlighted|aria-expanded="true"/);
  assert.equal(left.match(/tabindex="0"/g)?.length, 1);

  const host = document.createElement("div");
  host.innerHTML = left;
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  try {
    app.mount(host);
    for (let index = 0; index < 4; index++) await nextTick();
    assert.deepEqual(diagnostics, []);
    assert.equal(document.querySelectorAll('[role="menu"]').length, 1);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
  }
});
