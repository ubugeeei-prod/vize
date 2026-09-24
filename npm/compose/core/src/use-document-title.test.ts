import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useDocumentTitle } from "./use-document-title.ts";
import type { DocumentTitleHost } from "./use-document-title.ts";

class FakeDocument implements DocumentTitleHost {
  title: string;
  readonly observers = new Set<() => void>();

  constructor(title = "Original") {
    this.title = title;
  }

  readonly observeTitle = (callback: () => void): (() => void) => {
    this.observers.add(callback);
    return () => this.observers.delete(callback);
  };

  setExternally(title: string): void {
    this.title = title;
    for (const callback of this.observers) callback();
  }
}

void test("seeds an owned ref from the current title", () => {
  const host = new FakeDocument("Home");
  const { title, supported } = useDocumentTitle(undefined, { host });
  assert.equal(supported.value, true);
  assert.equal(title.value, "Home");

  title.value = "About";
  assert.equal(host.title, "About");
});

void test("applies string and function templates", () => {
  const host = new FakeDocument();
  const { title } = useDocumentTitle("Docs", { host, template: "%s | Vize" });
  assert.equal(host.title, "Docs | Vize");
  title.value = "API";
  assert.equal(host.title, "API | Vize");

  useDocumentTitle("x", { host, template: (value) => value.toUpperCase() });
  assert.equal(host.title, "X");
});

void test("follows refs and getters, ignoring nullish titles", async () => {
  const host = new FakeDocument();
  const source = ref<string | null>("A");
  const bound = useDocumentTitle(source, { host });
  assert.equal(bound.title, source, "a writable ref is used as-is");
  assert.equal(host.title, "A");
  source.value = null;
  assert.equal(host.title, "A");

  const page = ref("One");
  useDocumentTitle(() => `Page ${page.value}`, { host });
  assert.equal(host.title, "Page One");
  page.value = "Two";
  await nextTick();
  assert.equal(host.title, "Page Two");
});

void test("observes external changes without template feedback loops", () => {
  const host = new FakeDocument();
  const scope = effectScope();
  const controls = scope.run(() =>
    useDocumentTitle("Mine", { host, observe: true, template: "%s | Site" }),
  );
  assert.ok(controls);
  assert.equal(host.title, "Mine | Site");

  host.setExternally("Mine | Site");
  assert.equal(controls.title.value, "Mine", "own writes are not read back");
  host.setExternally("Other");
  assert.equal(controls.title.value, "Other");
  assert.equal(host.title, "Other | Site");

  scope.stop();
  assert.equal(host.observers.size, 0);
});

void test("restores the previous or computed title on dispose", () => {
  const host = new FakeDocument("Start");
  const scope = effectScope();
  scope.run(() => useDocumentTitle("Temporary", { host, restoreOnDispose: "previous" }));
  assert.equal(host.title, "Temporary");
  scope.stop();
  assert.equal(host.title, "Start");

  const custom = effectScope();
  custom.run(() =>
    useDocumentTitle("Now", {
      host,
      restoreOnDispose: (original, current) => `${original} (was ${current})`,
    }),
  );
  custom.stop();
  assert.equal(host.title, "Start (was Now)");
});

void test("keeps state without a document", () => {
  const { title, supported } = useDocumentTitle("x", { host: null });
  assert.equal(supported.value, false);
  assert.equal(title.value, "x");
});

void test("server rendering never touches the document", async () => {
  const state = await renderComposableOnServer(() => {
    const controls = useDocumentTitle("Server", { template: "%s | Vize", observe: true });
    return { title: controls.title, supported: controls.supported };
  });
  assert.equal(state, '{"title":"Server","supported":false}');
});
