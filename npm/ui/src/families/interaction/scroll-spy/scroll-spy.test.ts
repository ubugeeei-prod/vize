import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope, nextTick, shallowRef } from "vue";

import { createScrollSpy, useScrollSpy } from "./scroll-spy.ts";

interface Section {
  readonly element: HTMLElement;
  top: number;
  height: number;
}

function createSections(ids: readonly string[], spacing = 400): Section[] {
  return ids.map((id, index) => {
    const element = document.createElement("section");
    element.id = id;
    document.body.append(element);
    const section: Section = { element, top: index * spacing, height: spacing };
    element.getBoundingClientRect = () => new DOMRect(0, section.top, 800, section.height);
    return section;
  });
}

function scrollBy(sections: readonly Section[], delta: number): void {
  for (const section of sections) section.top -= delta;
}

function cleanup(sections: readonly Section[]): void {
  for (const section of sections) section.element.remove();
}

test("activates the last target whose top crossed the activation line", async () => {
  const sections = createSections(["intro", "usage", "api"]);
  const changes: [string | null, string | null, string][] = [];
  const spy = createScrollSpy({
    ids: ["intro", "usage", "api"],
    offset: 64,
    onActiveChange: (id, previous, reason) => changes.push([id, previous, reason]),
  });
  await nextTick();
  spy.refresh();
  assert.equal(spy.activeId.value, "intro");
  assert.deepEqual(spy.visibleIds.value, ["intro", "usage"]);

  scrollBy(sections, 380);
  window.dispatchEvent(new Event("scroll"));
  await new Promise((resolve) => setTimeout(resolve, 40));
  assert.equal(spy.activeId.value, "usage", "scroll events are batched per frame");
  assert.deepEqual(changes, [
    ["intro", null, "scroll"],
    ["usage", "intro", "scroll"],
  ]);
  spy.dispose();
  cleanup(sections);
});

test("keeps null above the first target and honours initialActiveId before measuring", async () => {
  const sections = createSections(["first", "second"]);
  for (const section of sections) section.top += 200;
  const spy = createScrollSpy({ ids: ["first", "second"], initialActiveId: "second" });
  assert.equal(spy.activeId.value, "second", "the server value is used until measured");
  await nextTick();
  spy.refresh();
  assert.equal(spy.activeId.value, null);
  spy.dispose();
  cleanup(sections);
});

test("scrolling a container to its end activates the last visible target", async () => {
  const sections = createSections(["a", "b", "c"], 100);
  const root = document.createElement("div");
  document.body.append(root);
  root.getBoundingClientRect = () => new DOMRect(0, 0, 800, 250);
  Object.defineProperty(root, "scrollHeight", { configurable: true, value: 300 });
  Object.defineProperty(root, "clientHeight", { configurable: true, value: 250 });
  const spy = createScrollSpy({ ids: ["a", "b", "c"], root });
  await nextTick();
  spy.refresh();
  assert.equal(spy.activeId.value, "a");
  scrollBy(sections, 50);
  root.scrollTop = 50;
  Object.defineProperty(root, "scrollTop", { configurable: true, value: 50 });
  root.dispatchEvent(new Event("scroll"));
  spy.refresh();
  assert.equal(spy.activeId.value, "c", "the end of the container activates the last target");
  spy.dispose();
  root.remove();
  cleanup(sections);
});

test("scrollTo navigates immediately and reactive ids rebind tracking", async () => {
  const sections = createSections(["one", "two", "three"]);
  const scrolled: string[] = [];
  for (const section of sections) {
    section.element.scrollIntoView = () => scrolled.push(section.element.id);
  }
  const ids = shallowRef<readonly string[]>(["one", "two"]);
  const changes: string[] = [];
  const spy = createScrollSpy({
    ids,
    onActiveChange: (_id, _previous, reason) => changes.push(reason),
  });
  assert.equal(spy.scrollTo("two"), true);
  assert.equal(spy.activeId.value, "two");
  assert.deepEqual(scrolled, ["two"]);
  assert.equal(spy.scrollTo("missing"), false);
  ids.value = ["one", "two", "three"];
  await nextTick();
  scrollBy(sections, 900);
  spy.refresh();
  assert.equal(spy.activeId.value, "three");
  assert.deepEqual(changes, ["navigation", "scroll"]);
  spy.dispose();
  spy.dispose();
  assert.throws(() => spy.refresh(), /VIZE_UI_SCROLL_SPY_DISPOSED/);
  cleanup(sections);
});

test("disabled spies keep the last value and useScrollSpy disposes with its scope", async () => {
  const sections = createSections(["x", "y"]);
  const disabled = shallowRef(false);
  const scope = effectScope();
  const spy = scope.run(() => useScrollSpy({ ids: ["x", "y"], isDisabled: disabled }));
  assert.ok(spy);
  await nextTick();
  spy.refresh();
  assert.equal(spy.activeId.value, "x");
  disabled.value = true;
  scrollBy(sections, 500);
  spy.refresh();
  assert.equal(spy.activeId.value, "x");
  scope.stop();
  assert.throws(() => spy.refresh(), /VIZE_UI_SCROLL_SPY_DISPOSED/);
  assert.throws(() => useScrollSpy({ ids: [] }), /VIZE_UI_SCROLL_SPY_SETUP/);
  cleanup(sections);
});
