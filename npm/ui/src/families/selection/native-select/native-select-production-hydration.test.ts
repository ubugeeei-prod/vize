import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, version } from "vue";
import { renderToString } from "vue/server-renderer";

import NativeSelect from "./native-select.vue";

const require = createRequire(import.meta.url);
const providerManifest = path.join(path.dirname(require.resolve("happy-dom")), "../package.json");
const provider = JSON.parse(fs.readFileSync(providerManifest, "utf8"));
assert.equal(provider.name, "happy-dom");
assert.equal(provider.version, "20.11.2");

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
    // HappyDOM20.11.2's parser uses selected-count minus one as an option index.
    // Its auto-selected first blank + authored third selected option chooses todo.
    // This characterizes that provider, not the HTML-standard selection contract.
    const parsedExpected =
      value === ""
        ? expected
        : {
            value: "todo",
            multiple: false,
            selected: ["todo"],
            options: [
              { value: "", selected: false },
              { value: "todo", selected: true },
              { value: "done", selected: false },
            ],
            form: [["status", "todo"]],
          };
    console.log(
      JSON.stringify({
        scope: "native-select-single-ssr-provider-custody",
        value,
        vueVersion: version,
        providerManifest,
        provider,
        html,
        parsedState: state(),
        optionAttributes: originalOptions.map((option) => ({
          value: option.value,
          selected: option.getAttribute("selected"),
          html: option.outerHTML,
        })),
        render: NativeSelect.render?.toString() ?? null,
        ssrRender: NativeSelect.ssrRender?.toString() ?? null,
      }),
    );
    assert.deepEqual(
      originalOptions.map((option) => option.hasAttribute("selected")),
      [false, false, value === "done"],
    );
    assert.deepEqual(state(), parsedExpected);
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
