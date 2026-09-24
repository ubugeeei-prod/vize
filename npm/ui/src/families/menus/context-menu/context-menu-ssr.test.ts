import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuRoot,
  ContextMenuTrigger,
} from "./context-menu.ts";

const Probe = defineComponent({
  setup: () => () =>
    h("div", [
      h(ContextMenuRoot, { defaultOpen: true }, () => [
        h(ContextMenuTrigger, null, () => "Right-click here"),
        h(ContextMenuContent, { ariaLabel: "Actions" }, () =>
          h(ContextMenuItem, null, () => "Cut"),
        ),
      ]),
    ]),
});

test("renders identical context-menu markup across SSR requests and hydrates cleanly", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-vize-ui="context-menu-trigger"/);
  assert.match(left, /data-menu-kind="context-menu"/);
  assert.doesNotMatch(left, /aria-description/);

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
    assert.equal(document.querySelectorAll('[role="menuitem"]').length, 1);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
  }
});
