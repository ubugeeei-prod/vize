import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import InteractionHooksExample from "./interaction-hooks-example.vue";

test("renders byte-identical interaction output without server DOM access", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(InteractionHooksExample)),
    renderToString(createSSRApp(InteractionHooksExample)),
  ]);

  assert.equal(outputs[0], outputs[1]);
  assert.match(outputs[0], /^<button/);
  assert.match(outputs[0], /data-vize-ui="interaction-hooks-example"/);
  assert.match(outputs[0], /Activated 0 times/);
  assert.match(outputs[0], /Shortcut 0/);
  assert.doesNotMatch(outputs[0], /data-modality=/);
  assert.doesNotMatch(outputs[0], /onClick|pointerenter|function/);
});

test("hydrates composed handlers without replacing the host", async () => {
  const serverHtml = await renderToString(createSSRApp(InteractionHooksExample));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverButton = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(InteractionHooksExample);

  try {
    app.mount(host);
    assert.equal(host.firstElementChild, serverButton);
    const button = host.querySelector("button");
    button?.dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 0 }));
    button?.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "k", ctrlKey: true }));
    document.dispatchEvent(
      new PointerEvent("pointerdown", { bubbles: true, pointerType: "mouse" }),
    );
    await nextTick();
    assert.equal(button?.textContent?.replace(/\s+/g, " ").trim(), "Activated 1 times Shortcut 1");
    assert.equal(button?.getAttribute("data-modality"), "pointer");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
