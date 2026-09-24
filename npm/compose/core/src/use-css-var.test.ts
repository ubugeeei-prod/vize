import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useCssVar } from "./use-css-var.ts";
import type { CssVarHost, CssVarObserver, CssVarStyle, CssVarTarget } from "./use-css-var.ts";

class FakeStyle implements CssVarStyle {
  readonly properties = new Map<string, string>();
  writes = 0;

  getPropertyValue(name: string): string {
    return this.properties.get(name) ?? "";
  }

  setProperty(name: string, value: string | null): void {
    this.writes += 1;
    if (value === null) this.properties.delete(name);
    else this.properties.set(name, value);
  }

  removeProperty(name: string): string {
    const previous = this.getPropertyValue(name);
    this.properties.delete(name);
    return previous;
  }
}

class FakeElement implements CssVarTarget {
  readonly style = new FakeStyle();
  /** Stylesheet-provided values, consulted when no inline value exists. */
  readonly sheet = new Map<string, string>();
}

class FakeObserver implements CssVarObserver {
  static instances: FakeObserver[] = [];
  readonly callback: () => void;
  connected = false;

  constructor(callback: () => void) {
    this.callback = callback;
    FakeObserver.instances.push(this);
  }

  observe(): void {
    this.connected = true;
  }

  disconnect(): void {
    this.connected = false;
  }
}

function createHost(root = new FakeElement()): CssVarHost {
  return {
    documentElement: root,
    getComputedStyle: (target) => ({
      getPropertyValue: (name) => {
        const inline = target.style.getPropertyValue(name);
        if (inline !== "") return ` ${inline}`;
        return target instanceof FakeElement ? (target.sheet.get(name) ?? "") : "";
      },
    }),
    createObserver: (callback) => new FakeObserver(callback),
  };
}

void test("reads the computed value of the root element by default", () => {
  const root = new FakeElement();
  root.sheet.set("--accent", "#0af");
  const accent = useCssVar("--accent", undefined, { host: createHost(root) });

  assert.equal(accent.supported.value, true);
  assert.equal(accent.value.value, "#0af");
  assert.equal(root.style.writes, 0, "read values are not frozen into inline style");
});

void test("writes assignments as inline style and removes empty values", () => {
  const element = new FakeElement();
  const variable = useCssVar("--gap", element, { host: createHost(), initialValue: "4px" });

  assert.equal(variable.value.value, "4px");
  variable.value.value = "8px";
  assert.equal(element.style.getPropertyValue("--gap"), "8px");
  variable.value.value = "";
  assert.equal(element.style.properties.has("--gap"), false);
});

void test("re-reads when the reactive name or target changes", async () => {
  const first = new FakeElement();
  const second = new FakeElement();
  first.sheet.set("--a", "1");
  second.sheet.set("--a", "2");
  second.sheet.set("--b", "3");
  const target = ref<CssVarTarget>(first);
  const name = ref("--a");
  const variable = useCssVar(name, target, { host: createHost() });

  assert.equal(variable.value.value, "1");
  target.value = second;
  await nextTick();
  assert.equal(variable.value.value, "2");
  name.value = "--b";
  await nextTick();
  assert.equal(variable.value.value, "3");
});

void test("observes attribute changes when requested and disconnects with the scope", () => {
  FakeObserver.instances = [];
  const element = new FakeElement();
  const scope = effectScope();
  const variable = scope.run(() =>
    useCssVar("--x", element, { host: createHost(), observe: true }),
  );
  assert.ok(variable);
  const observer = FakeObserver.instances[0];
  assert.equal(observer?.connected, true);

  element.sheet.set("--x", "changed");
  observer?.callback();
  assert.equal(variable.value.value, "changed");

  scope.stop();
  assert.equal(observer?.connected, false);
});

void test("rejects names that are not custom properties", () => {
  assert.throws(
    () => useCssVar("color", undefined, { host: null }),
    /VIZE_COMPOSE_CSS_VAR_INVALID_NAME/,
  );
});

void test("keeps the initial value without a capability", () => {
  const variable = useCssVar("--x", undefined, { host: null, initialValue: "red" });
  assert.equal(variable.supported.value, false);
  assert.equal(variable.value.value, "red");
  variable.value.value = "blue";
  assert.equal(variable.value.value, "blue");
});

void test("server rendering keeps the initial value", async () => {
  const state = await renderComposableOnServer(() => {
    const variable = useCssVar("--accent", undefined, { initialValue: "#000" });
    return { value: variable.value, supported: variable.supported };
  });
  assert.equal(state, '{"value":"#000","supported":false}');
});
