import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, ref } from "vue";
import { renderToString } from "vue/server-renderer";

import { useHover } from "./hover.ts";

const SsrProbe = defineComponent({
  name: "HoverSsrProbe",
  setup() {
    const starts = ref(0);
    const hover = useHover({ onHoverStart: () => starts.value++ });
    return () =>
      h(
        "div",
        { ...hover.hoverProps, "data-hovered": String(hover.isHovered.value) },
        `Hover ${starts.value}`,
      );
  },
});

function pointer(type: string): PointerEvent {
  return new PointerEvent(type, { pointerId: 2, pointerType: "mouse" });
}

test("renders byte-identical hover SSR output without DOM access or handlers", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(outputs[0], '<div data-hovered="false">Hover 0</div>');
  assert.doesNotMatch(outputs[0], /pointerenter|function/);
});

test("hydrates hover state without replacement or diagnostics", async () => {
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
    const target = host.firstElementChild!;
    target.dispatchEvent(pointer("pointerenter"));
    await nextTick();
    assert.equal(target.getAttribute("data-hovered"), "true");
    assert.equal(target.textContent, "Hover 1");
    target.dispatchEvent(pointer("pointerleave"));
    await nextTick();
    assert.equal(target.getAttribute("data-hovered"), "false");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
