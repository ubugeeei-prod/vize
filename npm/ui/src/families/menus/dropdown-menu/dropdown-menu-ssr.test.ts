import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuRoot,
  DropdownMenuTrigger,
} from "./dropdown-menu.ts";

const Probe = defineComponent({
  setup: () => () =>
    h("div", [
      h(DropdownMenuRoot, { defaultOpen: true }, () => [
        h(DropdownMenuTrigger, null, () => "File"),
        h(DropdownMenuContent, null, () => [
          h(DropdownMenuItem, null, () => "New"),
          h(DropdownMenuItem, null, () => "Open"),
        ]),
      ]),
    ]),
});

test("renders identical dropdown markup across SSR requests and hydrates cleanly", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-vize-ui="dropdown-menu-trigger"/);
  assert.match(left, /data-menu-kind="dropdown-menu"/);

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
    assert.equal(document.querySelectorAll('[role="menuitem"]').length, 2);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
  }
});
