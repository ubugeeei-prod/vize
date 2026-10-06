import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import NativeSelect from "./native-select.vue";

for (const value of ["", "done"] as const) {
  test(`single select retains complete synchronous SSR state for ${JSON.stringify(value)}`, async () => {
    const root = defineComponent({
      setup: () => () =>
        h("form", [
          h(NativeSelect, {
            id: "production-single-select",
            name: "status",
            defaultValue: value,
            options: [
              { value: "", label: "Select status" },
              { value: "todo", label: "Todo" },
              { value: "done", label: "Done" },
            ],
          }),
        ]),
    });
    const html = await renderToString(createSSRApp(root));
    const host = document.createElement("div");
    host.innerHTML = html;
    document.body.append(host);
    const form = host.querySelector("form");
    const select = host.querySelector("select");
    assert.ok(form instanceof HTMLFormElement);
    assert.ok(select instanceof HTMLSelectElement);
    const originalOptions = [...select.options];
    const state = () => ({
      value: select.value,
      multiple: select.multiple,
      selected: [...select.selectedOptions].map((option) => option.value),
      options: [...select.options].map((option) => ({
        value: option.value,
        selected: option.selected,
      })),
      form: [...new FormData(form).entries()],
    });
    const expected = {
      value,
      multiple: false,
      selected: [value],
      options: [
        { value: "", selected: value === "" },
        { value: "todo", selected: false },
        { value: "done", selected: value === "done" },
      ],
      form: [["status", value]],
    };
    assert.deepEqual(state(), expected);
    const diagnostics: string[] = [];
    const warn = console.warn;
    const error = console.error;
    console.warn = (...args: unknown[]) => diagnostics.push(args.map(String).join(" "));
    console.error = (...args: unknown[]) => diagnostics.push(args.map(String).join(" "));
    const app = createSSRApp(root);
    try {
      app.mount(host);
      assert.ok(host.firstElementChild === form);
      assert.ok(host.querySelector("select") === select);
      assert.ok(originalOptions.every((option, index) => option === select.options[index]));
      assert.deepEqual(state(), expected);
      assert.deepEqual(diagnostics, []);
    } finally {
      app.unmount();
      host.remove();
      console.warn = warn;
      console.error = error;
    }
  });
}
