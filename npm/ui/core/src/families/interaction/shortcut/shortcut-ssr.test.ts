import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, shallowRef } from "vue";
import { renderToString } from "vue/server-renderer";

import { formatShortcut, useShortcutRegistry } from "./shortcut.ts";

const SsrProbe = defineComponent({
  name: "ShortcutSsrProbe",
  setup() {
    const count = shallowRef(0);
    const registry = useShortcutRegistry({ platform: "standard", target: null });
    registry.register({
      shortcut: "Mod+K",
      description: "Open palette",
      handler: () => {
        count.value += 1;
      },
    });
    return () =>
      h(
        "div",
        {
          ...registry.shortcutProps,
          "data-count": count.value,
          "data-pending": registry.pendingSequence.value.length,
          tabindex: 0,
        },
        formatShortcut("Mod+K", { platform: "standard" }),
      );
  },
});

test("renders byte-identical shortcut markup without listeners or timers", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(outputs[0], '<div data-count="0" data-pending="0" tabindex="0">Ctrl+K</div>');
  assert.doesNotMatch(outputs[0]!, /keydown|function/);
});

test("hydrates shortcut dispatch without replacing the server host", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverTarget = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);

  try {
    app.mount(host);
    assert.equal(host.firstElementChild, serverTarget);
    const target = host.firstElementChild as HTMLElement;
    target.dispatchEvent(
      new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "k", ctrlKey: true }),
    );
    await nextTick();
    assert.equal(target.dataset.count, "1");
    assert.equal(target.dataset.pending, "0");
    assert.deepEqual(diagnostics, []);
  } finally {
    console.warn = originalWarn;
    console.error = originalError;
    app.unmount();
    host.remove();
  }
});
