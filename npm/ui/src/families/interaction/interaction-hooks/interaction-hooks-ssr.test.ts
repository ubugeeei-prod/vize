import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, ref } from "vue";
import { renderToString } from "vue/server-renderer";

import { useInteractionHooks } from "./interaction-hooks.ts";

const SsrProbe = defineComponent({
  name: "InteractionHooksSsrProbe",
  setup() {
    const activations = ref(0);
    const interactions = useInteractionHooks({
      focusWithin: {},
      press: { onPress: () => activations.value++ },
    });
    return () =>
      h(
        "button",
        {
          ...interactions.interactionProps,
          "data-focused": String(interactions.isFocused.value),
          "data-hovered": String(interactions.isHovered.value),
          "data-pressed": String(interactions.isPressed.value),
          type: "button",
        },
        `Activate ${activations.value}`,
      );
  },
});

test("renders byte-identical interaction output without server DOM access", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);

  assert.equal(outputs[0], outputs[1]);
  assert.match(
    outputs[0],
    /^<button data-focused="false" data-hovered="false" data-pressed="false" type="button">/,
  );
  assert.doesNotMatch(outputs[0], /onClick|pointerenter|function/);
});

test("hydrates composed handlers without replacing the host", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverButton = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);

  try {
    app.mount(host);
    assert.equal(host.firstElementChild, serverButton);
    host
      .querySelector("button")
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 0 }));
    await nextTick();
    assert.equal(host.querySelector("button")?.textContent, "Activate 1");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
