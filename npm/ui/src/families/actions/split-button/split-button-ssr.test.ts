import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  SplitButtonAction,
  SplitButtonContent,
  SplitButtonItem,
  SplitButtonRoot,
  SplitButtonTrigger,
} from "./split-button.ts";

function probe(rootProps: Record<string, unknown> = {}) {
  return defineComponent({
    setup: () => () =>
      h("div", [
        h(SplitButtonRoot, { ariaLabel: "Save options", ...rootProps }, () => [
          h(SplitButtonAction, null, () => "Save"),
          h(SplitButtonTrigger, { ariaLabel: "More save options" }),
          h(SplitButtonContent, null, () => [
            h(SplitButtonItem, null, () => "Draft"),
            h(SplitButtonItem, null, () => "Schedule"),
          ]),
        ]),
      ]),
  });
}

test("renders identical split button markup across SSR requests and hydrates cleanly", async () => {
  const Probe = probe({ defaultOpen: true });
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /role="group"/);
  assert.match(left, /aria-label="Save options"/);
  assert.match(left, /data-vize-ui="split-button-action"/);
  assert.match(left, /data-vize-ui="split-button-trigger"/);
  assert.match(left, /aria-haspopup="menu"/);
  assert.match(left, /data-menu-kind="split-button"/);

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
    const trigger = host.querySelector('[data-vize-ui="split-button-trigger"]');
    const menu = document.querySelector('[role="menu"]');
    assert.equal(menu?.getAttribute("aria-labelledby"), trigger?.id);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
  }
});

test("a closed or disabled split button renders no menu on the server", async () => {
  const closed = await renderToString(createSSRApp(probe()));
  assert.match(closed, /aria-expanded="false"/);
  assert.doesNotMatch(closed, /role="menu"/);
  assert.doesNotMatch(closed, /aria-controls=/);

  const disabled = await renderToString(createSSRApp(probe({ defaultOpen: true, disabled: true })));
  assert.match(disabled, /aria-disabled="true"/);
  assert.equal(disabled.match(/<button[^>]* disabled/g)?.length, 2);
  assert.doesNotMatch(disabled, /role="menu"/);
});
