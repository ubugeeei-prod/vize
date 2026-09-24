import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useStyleTag } from "./use-style-tag.ts";
import type { StyleTagElement, StyleTagHost } from "./use-style-tag.ts";

class FakeStyle implements StyleTagElement {
  textContent: string | null = null;
  readonly attributes = new Map<string, string>();

  setAttribute(name: string, value: string): void {
    this.attributes.set(name, value);
  }
}

class FakeHead implements StyleTagHost {
  readonly styles: FakeStyle[] = [];

  findStyle(id: string): StyleTagElement | undefined {
    return this.styles.find((style) => style.attributes.get("id") === id);
  }

  createStyle(): StyleTagElement {
    return new FakeStyle();
  }

  append(element: StyleTagElement): void {
    if (element instanceof FakeStyle) this.styles.push(element);
  }

  remove(element: StyleTagElement): void {
    const index = this.styles.findIndex((style) => style === element);
    if (index !== -1) this.styles.splice(index, 1);
  }
}

void test("injects a configured style element and follows css changes", async () => {
  const host = new FakeHead();
  const color = ref("red");
  const style = useStyleTag(() => `a { color: ${color.value} }`, {
    host,
    id: "links",
    media: "screen",
    nonce: "n",
  });

  assert.equal(style.id, "links");
  assert.equal(style.loaded.value, true);
  const element = host.styles[0];
  assert.equal(element?.textContent, "a { color: red }");
  assert.equal(element?.attributes.get("media"), "screen");
  assert.equal(element?.attributes.get("nonce"), "n");

  color.value = "blue";
  await nextTick();
  assert.equal(element?.textContent, "a { color: blue }");
  style.css.value = "b {}";
  assert.equal(element?.textContent, "b {}");
});

void test("adopts an existing element with the same id", () => {
  const host = new FakeHead();
  const server = new FakeStyle();
  server.setAttribute("id", "theme");
  server.textContent = ":root{}";
  host.styles.push(server);

  const style = useStyleTag(":root{}", { host, id: "theme" });
  assert.equal(style.loaded.value, true);
  assert.equal(host.styles.length, 1);
  assert.equal(server.textContent, ":root{}");
});

void test("loads manually and unloads with the scope", () => {
  const host = new FakeHead();
  const scope = effectScope();
  const style = scope.run(() => useStyleTag("x{}", { host, immediate: false }));
  assert.ok(style);
  assert.equal(host.styles.length, 0);
  assert.equal(style.load(), true);
  assert.equal(style.load(), true);
  assert.equal(host.styles.length, 1);
  assert.equal(host.styles[0]?.attributes.get("id"), "vize-style");

  scope.stop();
  assert.equal(host.styles.length, 0);
  assert.equal(style.loaded.value, false);
});

void test("reports no capability", () => {
  const style = useStyleTag("x{}", { host: null });
  assert.equal(style.loaded.value, false);
  assert.equal(style.load(), false);
});

void test("server rendering injects nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const style = useStyleTag("a{}", { id: "s" });
    return { id: style.id, css: style.css, loaded: style.loaded };
  });
  assert.equal(state, '{"id":"s","css":"a{}","loaded":false}');
});
