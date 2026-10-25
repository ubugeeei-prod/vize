import assert from "node:assert/strict";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { compileTurbopackSfc } from "./loader.ts";

function session() {
  let data: Record<string, unknown> | undefined;
  let registered = false;
  const calls: string[] = [];
  const setup = () => ({});
  return {
    calls,
    update(source: string, currentSetup = setup) {
      const output = compileTurbopackSfc("/fixture/FixtureCounter.vue", source, {
        root: "/fixture",
        ssr: false,
        isProduction: false,
        styleLanguages: ["css"],
      });
      const start = output.code.indexOf("/* hot reload */");
      assert.notEqual(start, -1);
      // Execute the real native adapter's generated HMR block. Imports and the
      // component's render/setup bodies remain the actual host test's concern.
      const code = output.code
        .slice(start, output.code.lastIndexOf("export default"))
        .replaceAll("import.meta.turbopackHot", "hot");
      const component = {
        setup: currentSetup,
        render() {},
        __cssModules: { $style: { button: source } },
      };
      let dispose!: (value: Record<string, unknown>) => void;
      runInNewContext(code, {
        hot: {
          data,
          accept() {},
          dispose(callback: typeof dispose) {
            dispose = callback;
          },
        },
        __vize_component__: component,
        __VUE_HMR_RUNTIME__: {
          createRecord() {
            const first = !registered;
            registered = true;
            return first;
          },
          reload() {
            calls.push("reload");
          },
          rerender() {
            calls.push("rerender");
          },
        },
      });
      data = {};
      dispose(data);
      return component;
    },
  };
}

void test("host CSS maps preserve state across native map metadata changes while dependency and script edits reload", () => {
  const initial =
    "<script setup>const initial = 2;</script><template>{{initial}}</template><style module>.button{color:red}</style>";
  const hmr = session();
  const first = hmr.update(initial);
  const captured = first.__cssModules.$style;
  const styleEdit = initial.replace("color:red", "color:blue");
  const next = hmr.update(styleEdit);
  assert.deepEqual(hmr.calls, ["rerender"]);
  assert.equal(next.__cssModules.$style, captured);
  assert.equal(captured.button, styleEdit);
  hmr.update(styleEdit);
  assert.deepEqual(
    hmr.calls,
    ["rerender", "reload"],
    "unchanged-source dependency reevaluation remains conservative",
  );
  const templateEdit = styleEdit.replace("{{initial}}", "updated {{initial}}");
  hmr.update(templateEdit);
  hmr.update(templateEdit.replace("initial = 2", "initial = 4"));
  assert.deepEqual(hmr.calls, ["rerender", "reload", "rerender", "reload"]);
});

void test("missing script identity uses emitted code and changed setup reloads", () => {
  const templateOnly = "<template><div/></template><style module>.button{color:red}</style>";
  const missing = session();
  missing.update(templateOnly);
  const styleOnly = templateOnly.replace("red", "blue");
  missing.update(styleOnly);
  missing.update(styleOnly.replace("<div/>", "<span/>"));
  assert.deepEqual(missing.calls, ["rerender", "reload"]);
  const initial =
    "<script setup>const initial = 2;</script><template>{{initial}}</template><style module>.button{color:red}</style>";
  const changed = session();
  changed.update(initial);
  changed.update(initial.replace("red", "blue"), () => ({ changed: true }));
  assert.deepEqual(changed.calls, ["reload"]);
});
