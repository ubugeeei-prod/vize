import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, ref } from "vue";
import { renderToString } from "vue/server-renderer";

import { useMove } from "./move.ts";

const SsrProbe = defineComponent({
  name: "MoveSsrProbe",
  setup() {
    const x = ref(0);
    const move = useMove({ onMove: (event) => (x.value += event.deltaX) });
    return () =>
      h(
        "div",
        { ...move.moveProps, "data-moving": String(move.isMoving.value), tabindex: 0 },
        `Move ${x.value}`,
      );
  },
});

function pointer(type: string, x: number): PointerEvent {
  const event = new PointerEvent(type, {
    bubbles: true,
    button: 0,
    clientX: x,
    clientY: 0,
    isPrimary: true,
    pointerId: 2,
    pointerType: "mouse",
  });
  Object.defineProperties(event, {
    pageX: { value: x },
    pageY: { value: 0 },
  });
  return event;
}

test("renders byte-identical move SSR output without DOM access or serialized handlers", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(outputs[0], '<div data-moving="false" tabindex="0">Move 0</div>');
  assert.doesNotMatch(outputs[0], /pointerdown|function/);
});

test("hydrates move state without replacement or diagnostics", async () => {
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
    target.dispatchEvent(pointer("pointerdown", 1));
    document.dispatchEvent(pointer("pointermove", 4));
    await nextTick();
    assert.equal(target.getAttribute("data-moving"), "true");
    assert.equal(target.textContent, "Move 3");
    document.dispatchEvent(pointer("pointerup", 4));
    await nextTick();
    assert.equal(target.getAttribute("data-moving"), "false");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
