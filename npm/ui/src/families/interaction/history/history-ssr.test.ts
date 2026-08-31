import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, shallowRef } from "vue";
import { renderToString } from "vue/server-renderer";

import { useHistory } from "./history.ts";

const SsrProbe = defineComponent({
  name: "HistorySsrProbe",
  setup() {
    const value = shallowRef(0);
    const history = useHistory();
    const increment = () => {
      const before = value.value;
      value.value += 1;
      history.pushSnapshot({
        before,
        after: value.value,
        apply: (next) => {
          value.value = next;
        },
        label: "Increment",
      });
    };
    return () =>
      h("div", { "data-value": value.value, "data-can-undo": String(history.canUndo.value) }, [
        h("button", { type: "button", "data-action": "increment", onClick: increment }, "Add"),
        h(
          "button",
          {
            type: "button",
            "data-action": "undo",
            disabled: !history.canUndo.value,
            onClick: () => history.undo(),
          },
          "Undo",
        ),
      ]);
  },
});

test("renders byte-identical history markup with no request-global timelines", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(
    outputs[0],
    '<div data-value="0" data-can-undo="false">' +
      '<button type="button" data-action="increment">Add</button>' +
      '<button type="button" data-action="undo" disabled>Undo</button></div>',
  );
});

test("hydrates undoable interactions without replacing the server host", async () => {
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
    const increment = target.querySelector<HTMLButtonElement>('[data-action="increment"]')!;
    const undo = target.querySelector<HTMLButtonElement>('[data-action="undo"]')!;

    increment.click();
    await nextTick();
    assert.equal(target.dataset.value, "1");
    assert.equal(undo.disabled, false);

    undo.click();
    await nextTick();
    assert.equal(target.dataset.value, "0");
    assert.equal(undo.disabled, true);
    assert.deepEqual(diagnostics, []);
  } finally {
    console.warn = originalWarn;
    console.error = originalError;
    app.unmount();
    host.remove();
  }
});
