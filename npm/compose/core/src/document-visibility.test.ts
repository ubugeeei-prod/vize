import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { useDocumentVisibility } from "./document-visibility.ts";
import type { DocumentVisibilityHost, PageVisibilityState } from "./document-visibility.ts";

class ObservableVisibilityHost extends EventTarget implements DocumentVisibilityHost {
  visibilityState: PageVisibilityState | "prerender" = "visible";
  hidden = false;
  listeners = 0;

  override addEventListener(
    event: "visibilitychange",
    listener: EventListener,
    options?: boolean | AddEventListenerOptions,
  ): void {
    this.listeners += 1;
    super.addEventListener(event, listener, options);
  }

  override removeEventListener(
    event: "visibilitychange",
    listener: EventListener,
    options?: boolean | EventListenerOptions,
  ): void {
    this.listeners -= 1;
    super.removeEventListener(event, listener, options);
  }

  setState(state: PageVisibilityState | "prerender"): void {
    this.visibilityState = state;
    this.hidden = state !== "visible";
    this.dispatchEvent(new Event("visibilitychange"));
  }
}

void test("uses the configured server fallback without a document host", () => {
  const visibility = useDocumentVisibility({
    host: () => undefined,
    ssrState: "hidden",
  });

  assert.equal(visibility.supported.value, false);
  assert.equal(visibility.state.value, "hidden");
  assert.equal(visibility.hidden.value, true);
  assert.equal(visibility.visible.value, false);
});

void test("tracks a concrete document host and cleans up with the scope", () => {
  const host = new ObservableVisibilityHost();
  const scope = effectScope();
  const visibility = scope.run(() => useDocumentVisibility({ host }));

  assert.ok(visibility);
  assert.equal(host.listeners, 1);
  assert.equal(visibility.supported.value, true);
  assert.equal(visibility.state.value, "visible");

  host.setState("hidden");
  assert.equal(visibility.state.value, "hidden");
  assert.equal(visibility.hidden.value, true);

  scope.stop();
  assert.equal(host.listeners, 0);
  assert.equal(visibility.supported.value, false);
  host.setState("visible");
  assert.equal(visibility.state.value, "hidden");
});

void test("rebinds reactive hosts and restores the fallback while unsupported", async () => {
  const first = new ObservableVisibilityHost();
  const second = new ObservableVisibilityHost();
  second.setState("hidden");
  const host = ref<DocumentVisibilityHost | null>(first);
  const visibility = useDocumentVisibility({ host, ssrState: "hidden" });

  assert.equal(first.listeners, 1);
  assert.equal(visibility.state.value, "visible");

  host.value = second;
  await nextTick();
  assert.equal(first.listeners, 0);
  assert.equal(second.listeners, 1);
  assert.equal(visibility.state.value, "hidden");

  host.value = null;
  await nextTick();
  assert.equal(second.listeners, 0);
  assert.equal(visibility.supported.value, false);
  assert.equal(visibility.state.value, "hidden");
});

void test("normalizes future browser states to hidden", () => {
  const host = new ObservableVisibilityHost();
  const visibility = useDocumentVisibility({ host });

  host.setState("prerender");
  host.hidden = false;
  host.dispatchEvent(new Event("visibilitychange"));
  assert.equal(visibility.state.value, "hidden");
  assert.equal(visibility.visible.value, false);
});
