import assert from "node:assert/strict";
import { test } from "node:test";

import { effectScope, nextTick, shallowRef } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useVisualViewport } from "./use-visual-viewport.ts";
import type { VirtualKeyboardHost, VisualViewportWindowHost } from "./use-visual-viewport.ts";

class FakeViewport extends EventTarget {
  width = 390;
  height = 844;
  offsetTop = 0;
  offsetLeft = 0;
  scale = 1;
  resize(height: number, offsetTop = 0): void {
    this.height = height;
    this.offsetTop = offsetTop;
    this.dispatchEvent(new Event("resize"));
  }
}

class FakeKeyboard extends EventTarget implements VirtualKeyboardHost {
  overlaysContent = false;
  boundingRect = { height: 0 };
  show(height: number): void {
    this.boundingRect = { height };
    this.dispatchEvent(new Event("geometrychange"));
  }
}

function windowWith(
  viewport: FakeViewport | null,
  keyboard?: FakeKeyboard,
): VisualViewportWindowHost {
  return { innerHeight: 844, visualViewport: viewport, navigator: { virtualKeyboard: keyboard } };
}

void test("reports geometry and the occluded keyboard height from visualViewport", async () => {
  const viewport = new FakeViewport();
  const scope = effectScope();
  const state = scope.run(() => useVisualViewport({ host: windowWith(viewport) }));
  assert.ok(state);
  assert.equal(state.isSupported.value, true);
  assert.equal(state.height.value, 844);
  assert.equal(state.keyboardOpen.value, false);

  viewport.resize(500);
  assert.equal(state.keyboardHeight.value, 344);
  assert.equal(state.keyboardOpen.value, true);
  viewport.resize(744, 100);
  assert.equal(state.keyboardHeight.value, 0, "scrolled visual viewports are not keyboards");
  viewport.resize(780);
  assert.equal(state.keyboardHeight.value, 64);
  assert.equal(state.keyboardOpen.value, false, "toolbar-sized changes stay below the threshold");
  viewport.scale = 2;
  viewport.dispatchEvent(new Event("scroll"));
  assert.equal(state.scale.value, 2);

  scope.stop();
  viewport.resize(300);
  assert.equal(state.keyboardHeight.value, 64, "listeners are removed with the scope");
  await nextTick();
});

void test("opts into the VirtualKeyboard API and restores overlaysContent on cleanup", () => {
  const keyboard = new FakeKeyboard();
  const scope = effectScope();
  const state = scope.run(() =>
    useVisualViewport({
      host: windowWith(null, keyboard),
      overlaysContent: true,
      keyboardThreshold: 10,
    }),
  );
  assert.ok(state);
  assert.equal(keyboard.overlaysContent, true);
  assert.equal(state.isSupported.value, true);
  keyboard.show(300);
  assert.equal(state.keyboardHeight.value, 300);
  assert.equal(state.keyboardOpen.value, true);
  scope.stop();
  assert.equal(keyboard.overlaysContent, false);
});

void test("unsupported or swapped hosts reset to deterministic fallbacks", async () => {
  const host = shallowRef<VisualViewportWindowHost | null>(windowWith(new FakeViewport()));
  const scope = effectScope();
  const state = scope.run(() => useVisualViewport({ host }));
  assert.ok(state);
  assert.equal(state.width.value, 390);
  host.value = windowWith(null);
  await nextTick();
  assert.equal(state.isSupported.value, false);
  assert.equal(state.width.value, 0);
  assert.equal(state.scale.value, 1);
  scope.stop();
});

void test("server rendering reports zeros without touching host globals", async () => {
  const html = await renderComposableOnServer(() => {
    const state = useVisualViewport();
    return {
      height: state.height,
      keyboardHeight: state.keyboardHeight,
      keyboardOpen: state.keyboardOpen,
      supported: state.isSupported,
    };
  });
  assert.equal(html, '{"height":0,"keyboardHeight":0,"keyboardOpen":false,"supported":false}');
});
