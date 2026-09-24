import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useScriptTag } from "./use-script-tag.ts";
import type { ScriptTagElement, ScriptTagHost } from "./use-script-tag.ts";

class FakeScript extends EventTarget implements ScriptTagElement {
  readonly attributes = new Map<string, string>();

  getAttribute(name: string): string | null {
    return this.attributes.get(name) ?? null;
  }

  setAttribute(name: string, value: string): void {
    this.attributes.set(name, value);
  }

  fire(type: "load" | "error"): void {
    this.dispatchEvent(new Event(type));
  }
}

class FakeHead implements ScriptTagHost {
  readonly scripts: FakeScript[] = [];
  created = 0;

  findScript(src: string): ScriptTagElement | undefined {
    return this.scripts.find((script) => script.getAttribute("src") === src);
  }

  createScript(): ScriptTagElement {
    this.created += 1;
    return new FakeScript();
  }

  append(element: ScriptTagElement): void {
    if (element instanceof FakeScript) this.scripts.push(element);
  }

  remove(element: ScriptTagElement): void {
    const index = this.scripts.findIndex((script) => script === element);
    if (index !== -1) this.scripts.splice(index, 1);
  }
}

void test("injects a configured script and reports loading", async () => {
  const host = new FakeHead();
  const loaded: ScriptTagElement[] = [];
  const script = useScriptTag("/a.js", {
    host,
    type: "module",
    crossOrigin: "anonymous",
    nonce: "n",
    attributes: { "data-x": "1" },
    onLoaded: (element) => loaded.push(element),
  });

  assert.equal(script.status.value, "loading");
  const element = host.scripts[0];
  assert.ok(element);
  assert.equal(element.getAttribute("src"), "/a.js");
  assert.equal(element.getAttribute("type"), "module");
  assert.equal(element.getAttribute("async"), "");
  assert.equal(element.getAttribute("crossorigin"), "anonymous");
  assert.equal(element.getAttribute("nonce"), "n");
  assert.equal(element.getAttribute("data-x"), "1");

  const result = script.load();
  element.fire("load");
  assert.deepEqual(await result, { status: "loaded", element });
  assert.equal(script.status.value, "loaded");
  assert.deepEqual(loaded, [element]);
});

void test("reports load errors", async () => {
  const host = new FakeHead();
  const script = useScriptTag("/broken.js", { host, immediate: false });
  const result = script.load();
  host.scripts[0]?.fire("error");
  assert.equal((await result).status, "error");
  assert.equal(script.status.value, "error");
});

void test("reuses existing tags instead of duplicating them", async () => {
  const host = new FakeHead();
  const first = useScriptTag("/shared.js", { host });
  const second = useScriptTag("/shared.js", { host });
  assert.equal(host.created, 1);
  const pending = second.load();
  host.scripts[0]?.fire("load");
  assert.equal((await pending).status, "loaded");
  assert.equal(first.status.value, "loaded");

  const third = useScriptTag("/shared.js", { host });
  assert.equal(third.status.value, "loaded", "a loaded tag resolves immediately");

  const server = new FakeHead();
  const tag = new FakeScript();
  tag.setAttribute("src", "/ssr.js");
  server.scripts.push(tag);
  const reused = useScriptTag("/ssr.js", { host: server });
  assert.equal(reused.status.value, "loaded", "unmarked tags are assumed executed");
});

void test("swaps scripts when the reactive src changes", async () => {
  const host = new FakeHead();
  const src = ref("/one.js");
  const script = useScriptTag(src, { host });
  const stale = script.load();
  src.value = "/two.js";
  await nextTick();
  assert.deepEqual(
    host.scripts.map((element) => element.getAttribute("src")),
    ["/two.js"],
  );
  assert.deepEqual(await stale, { status: "cancelled" });
});

void test("removes created scripts with the scope unless disabled", () => {
  const host = new FakeHead();
  const scope = effectScope();
  scope.run(() => useScriptTag("/a.js", { host }));
  scope.stop();
  assert.equal(host.scripts.length, 0);

  const keep = effectScope();
  keep.run(() => useScriptTag("/b.js", { host, removeOnDispose: false }));
  keep.stop();
  assert.equal(host.scripts.length, 1);
});

void test("manual loading and unloading", async () => {
  const host = new FakeHead();
  const script = useScriptTag("/m.js", { host, immediate: false });
  assert.equal(script.status.value, "idle");
  const first = script.load();
  assert.equal(script.load(), first, "concurrent loads share one promise");
  script.unload();
  assert.equal(script.status.value, "idle");
  assert.equal(host.scripts.length, 0);

  const again = script.load();
  host.scripts[0]?.fire("load");
  assert.equal((await again).status, "loaded");
});

void test("server rendering injects nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const script = useScriptTag("/a.js");
    return { status: script.status };
  });
  assert.equal(state, '{"status":"idle"}');
  const unsupported = useScriptTag("/a.js", { host: null, immediate: false });
  assert.deepEqual(await unsupported.load(), { status: "unsupported" });
});
