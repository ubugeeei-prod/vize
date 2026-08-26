import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, shallowRef } from "vue";
import { renderToString } from "vue/server-renderer";

import { useCommandRouter } from "./command.ts";

const SsrProbe = defineComponent({
  name: "CommandSsrProbe",
  setup() {
    const output = shallowRef("idle");
    const router = useCommandRouter<"save">();
    router.register({
      id: "save",
      title: "Save Document",
      run: () => {
        output.value = "saved";
      },
    });
    return () =>
      h(
        "button",
        {
          type: "button",
          disabled: !router.isEnabled("save"),
          "data-output": output.value,
          "data-commands": router.commands.value.length,
          onClick: () => router.execute("save", undefined, { source: "menu" }),
        },
        router.commands.value[0]?.title ?? "",
      );
  },
});

test("renders byte-identical command markup with no request-global state", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(
    outputs[0],
    '<button type="button" data-output="idle" data-commands="1">Save Document</button>',
  );
});

test("hydrates command dispatch without replacing the server host", async () => {
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
    const target = host.firstElementChild as HTMLButtonElement;
    target.click();
    await nextTick();
    assert.equal(target.dataset.output, "saved");
    assert.deepEqual(diagnostics, []);
  } finally {
    console.warn = originalWarn;
    console.error = originalError;
    app.unmount();
    host.remove();
  }
});
