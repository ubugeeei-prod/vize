import assert from "node:assert/strict";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { genHotReloadCode, genCSSModuleHotReloadCode, type HmrMetadata } from "./hotReload.ts";
import { hmrImportSnapshot } from "./module-output.ts";

const initial: HmrMetadata = {
  source: "source-1",
  script: "script-1",
  template: "template-1",
  options: "options-1",
  module: "module-1",
  styles: "styles-1",
  canRerender: true,
};

function session() {
  let data: Record<string, unknown> | undefined;
  let registered = false;
  const calls: string[] = [];
  const api = {
    createRecord: () => {
      const created = !registered;
      registered = true;
      return created;
    },
    reload: () => calls.push("reload"),
    rerender: () => calls.push("rerender"),
  };
  const dependency = {};
  return {
    calls,
    update(
      metadata: HmrMetadata | undefined = initial,
      component: Record<string, unknown> = { render() {} },
      imported: unknown = dependency,
      imports = '[["dependency", dependency]]',
    ) {
      let dispose!: (data: Record<string, unknown>) => void;
      const hot = {
        data,
        accept() {},
        dispose(callback: typeof dispose) {
          dispose = callback;
        },
      };
      runInNewContext(genHotReloadCode("test", metadata, imports), {
        module: { hot },
        _sfc_main: component,
        __VUE_HMR_RUNTIME__: api,
        dependency: imported,
      });
      data = {};
      dispose(data);
      return component;
    },
  };
}

void test("map/style-only reevaluation preserves state; subsequent template and script edits are classified independently", () => {
  const hmr = session();
  hmr.update();
  hmr.update({ ...initial, source: "style-edit" });
  assert.deepEqual(hmr.calls, []);
  const template = {
    ...initial,
    source: "template-edit",
    template: "template-2",
    module: "module-2",
  };
  hmr.update(template);
  assert.deepEqual(hmr.calls, ["rerender"]);
  hmr.update({ ...template, source: "script-edit", script: "script-2", module: "module-3" });
  assert.deepEqual(hmr.calls, ["rerender", "reload"]);
});

void test("dependency reevaluation reloads even when generated module text is unchanged", () => {
  const hmr = session();
  hmr.update();
  hmr.update();
  hmr.update({ ...initial, source: "simultaneous-style-edit" }, { render() {} }, {});
  assert.deepEqual(hmr.calls, ["reload", "reload"]);
});

void test("uninitialized circular imports do not prevent loading and disable state preservation", () => {
  const hmr = session();
  // Read a binding before its declaration, as a cyclic ESM import does.
  const uninitialized = '(() => { return [["cycle", cycle]]; let cycle; })()';
  hmr.update(initial, { render() {} }, undefined, uninitialized);
  hmr.update({ ...initial, source: "style-edit" }, { render() {} }, undefined, uninitialized);
  hmr.update({ ...initial, source: "another-style-edit" });
  assert.deepEqual(hmr.calls, ["reload", "reload"]);
});

void test("an uninitialized namespace export does not prevent loading", () => {
  const hmr = session();
  const imports = hmrImportSnapshot(
    "import * as dependency from './dependency'; console.log(dependency);",
  );
  assert.ok(imports);
  const dependency = {
    get value() {
      throw new ReferenceError("Cannot access 'value' before initialization");
    },
  };
  hmr.update(initial, { render() {} }, dependency, imports);
  hmr.update({ ...initial, source: "style-edit" }, { render() {} }, { value: 1 }, imports);
  assert.deepEqual(hmr.calls, ["reload"]);
});

void test("missing hashes, incompatible options/style structure and unsupported modes reload", () => {
  for (const change of [
    { script: undefined },
    { template: undefined },
    { options: "other" },
    { styles: "other" },
    { canRerender: false },
    { module: "changed-without-template-change" },
  ]) {
    const hmr = session();
    hmr.update();
    hmr.update({ ...initial, source: "edit", module: "changed", ...change });
    assert.deepEqual(hmr.calls, ["reload"]);
  }
});

void test("template edits that change setup bindings reload instead of retaining an incompatible setup", () => {
  const hmr = session();
  hmr.update(initial, {
    setup() {
      return { a: 1 };
    },
    render() {},
  });
  hmr.update(
    { ...initial, source: "edit", template: "template-2", module: "module-2" },
    {
      setup() {
        return { a: 1, b: 2 };
      },
      render() {},
    },
  );
  assert.deepEqual(hmr.calls, ["reload"]);
});

void test("identical JavaScript keeps the map-only guard even without template hash metadata", () => {
  const hmr = session();
  hmr.update({ ...initial, template: undefined });
  hmr.update({ ...initial, source: "style-edit", template: undefined });
  assert.deepEqual(hmr.calls, []);
  hmr.update({ ...initial, source: "code-edit", template: undefined, module: "changed" });
  assert.deepEqual(hmr.calls, ["reload"]);
});

void test("CSS module bindings remain shared across repeated SFC reevaluations and dependency callbacks", () => {
  const hmr = session();
  const originalExports = Object.freeze({ button: "old", removed: "old-class" });
  const bindings: Record<string, Record<string, string>> = {
    $style: originalExports,
    theme: Object.freeze({ label: "old-theme" }),
  };
  hmr.update(initial, { __cssModules: bindings, render() {} });
  // useCssModule() returns the inner mapping; setup retains it across rerenders.
  const styles = bindings.$style;
  const theme = bindings.theme;
  const latest = hmr.update(
    { ...initial, source: "style-edit" },
    {
      __cssModules: {
        $style: Object.freeze({ button: "new", added: "new-class" }),
        theme: Object.freeze({ label: "new-theme" }),
      },
      render() {},
    },
  );
  assert.equal(latest.__cssModules, bindings);
  assert.equal(bindings.$style, styles);
  assert.equal(bindings.theme, theme);
  assert.equal(styles.button, "new");
  assert.equal(styles.added, "new-class");
  assert.equal("removed" in styles, false);
  assert.equal(theme.label, "new-theme");
  assert.deepEqual(originalExports, { button: "old", removed: "old-class" });
  let callback!: () => void;
  let render: unknown;
  const context = {
    module: {
      hot: {
        accept(_request: string, handler: () => void) {
          callback = handler;
        },
      },
    },
    _sfc_main: latest,
    __VUE_HMR_RUNTIME__: {
      rerender(_id: string, nextRender: unknown) {
        render = nextRender;
      },
    },
    css: Object.freeze({ button: "third" }),
  };
  runInNewContext(genCSSModuleHotReloadCode("test", '"style"', "css", "$style"), context);
  callback();
  assert.equal(bindings.$style, styles);
  assert.equal(styles.button, "third");
  assert.equal("added" in styles, false);
  assert.equal(render, latest.render);
  context.css = Object.freeze({ button: "fourth" });
  callback();
  assert.equal(styles.button, "fourth");
  assert.equal(theme.label, "new-theme");
});

void test("HMR history is local to each browser module lifecycle", () => {
  const a = session();
  const b = session();
  a.update();
  a.update({ ...initial, source: "style-edit" });
  b.update({ ...initial, source: "other-client", script: "other-script" });
  assert.deepEqual(a.calls, []);
  assert.deepEqual(b.calls, []);
});

void test("import snapshots copy namespace exports and exclude Vue helpers and SFC styles", () => {
  const expression = hmrImportSnapshot(`import { ref } from 'vue';
import './App.vue?vue&type=style&index=0&lang=css';
import * as helpers from './helpers'; import child from './Child.vue';
console.log(helpers, child);`);
  assert.ok(expression);
  const helpers = { value: 1 };
  const child = {};
  const snapshot = runInNewContext(expression, { helpers, child }) as [string, unknown][];
  helpers.value = 2;
  assert.deepEqual(
    Array.from(snapshot, ([key, value]) => [key, value]),
    [
      ["./helpers:helpers:value", 1],
      ["./Child.vue:child", child],
    ],
  );
  assert.equal(hmrImportSnapshot("import './side-effect';"), null);
  assert.equal(hmrImportSnapshot("import {"), null);
});

void test("HMR metadata does not turn TypeScript-only imports into runtime dependencies", () => {
  assert.equal(
    hmrImportSnapshot(`import { BadgeType } from './types';
interface Props { badges: BadgeType[] }
const props: Props = { badges: [] }; console.log(props);`),
    "[]",
  );
  assert.equal(
    hmrImportSnapshot(`import type { BadgeType } from './types';
import { value } from './dependency'; const count: BadgeType = value; console.log(count);`),
    '[["./dependency:value", value]]',
  );
});
