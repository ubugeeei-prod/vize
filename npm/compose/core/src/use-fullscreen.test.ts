import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useFullscreen } from "./use-fullscreen.ts";
import type { FullscreenElementLike, FullscreenHost } from "./use-fullscreen.ts";

class FakeDocument extends EventTarget implements FullscreenHost {
  fullscreenElement: unknown = null;
  fullscreenEnabled = true;
  readonly documentElement: FakeElement;
  listeners = 0;

  constructor() {
    super();
    this.documentElement = new FakeElement(this);
  }

  override addEventListener(type: string, listener: EventListener): void {
    this.listeners += 1;
    super.addEventListener(type, listener);
  }

  override removeEventListener(type: string, listener: EventListener): void {
    this.listeners -= 1;
    super.removeEventListener(type, listener);
  }

  exitFullscreen(): Promise<void> {
    this.fullscreenElement = null;
    this.dispatchEvent(new Event("fullscreenchange"));
    return Promise.resolve();
  }
}

class FakeElement implements FullscreenElementLike {
  readonly document: FakeDocument;
  reject = false;
  navigationUI: string | undefined;

  constructor(document: FakeDocument) {
    this.document = document;
  }

  requestFullscreen(options?: { navigationUI?: "auto" | "hide" | "show" }): Promise<void> {
    if (this.reject) return Promise.reject(new TypeError("no gesture"));
    this.navigationUI = options?.navigationUI;
    this.document.fullscreenElement = this;
    this.document.dispatchEvent(new Event("fullscreenchange"));
    return Promise.resolve();
  }
}

void test("enters, exits, and toggles the document element by default", async () => {
  const host = new FakeDocument();
  const fullscreen = useFullscreen(undefined, { host, navigationUI: "hide" });

  assert.equal(fullscreen.supported.value, true);
  assert.equal(await fullscreen.enter(), true);
  assert.equal(host.documentElement.navigationUI, "hide");
  assert.equal(fullscreen.isFullscreen.value, true);
  assert.equal(await fullscreen.toggle(), false);
  assert.equal(fullscreen.isFullscreen.value, false);
});

void test("tracks fullscreenchange for its own target only", async () => {
  const host = new FakeDocument();
  const video = new FakeElement(host);
  const other = new FakeElement(host);
  const fullscreen = useFullscreen(video, { host });

  await other.requestFullscreen();
  assert.equal(fullscreen.isFullscreen.value, false);
  assert.equal(await fullscreen.exit(), false, "does not exit someone else's fullscreen");
  assert.equal(host.fullscreenElement, other);

  await video.requestFullscreen();
  assert.equal(fullscreen.isFullscreen.value, true);
  host.fullscreenElement = null;
  host.dispatchEvent(new Event("fullscreenchange"));
  assert.equal(fullscreen.isFullscreen.value, false, "Escape is reflected");
});

void test("supports WebKit-prefixed hosts", async () => {
  class WebKitDocument extends EventTarget implements FullscreenHost {
    webkitFullscreenElement: unknown = null;
    webkitFullscreenEnabled = true;
    webkitExitFullscreen(): void {
      this.webkitFullscreenElement = null;
      this.dispatchEvent(new Event("webkitfullscreenchange"));
    }
  }
  const host = new WebKitDocument();
  const element: FullscreenElementLike = {
    webkitRequestFullscreen: () => {
      host.webkitFullscreenElement = element;
      host.dispatchEvent(new Event("webkitfullscreenchange"));
    },
  };
  const fullscreen = useFullscreen(element, { host });

  assert.equal(await fullscreen.enter(), true);
  assert.equal(await fullscreen.exit(), false);
});

void test("reports rejected requests and disabled fullscreen", async () => {
  const host = new FakeDocument();
  host.documentElement.reject = true;
  const fullscreen = useFullscreen(undefined, { host });
  assert.equal(await fullscreen.enter(), false);

  host.fullscreenEnabled = false;
  const disabled = useFullscreen(undefined, { host });
  assert.equal(disabled.supported.value, false);
  assert.equal(await disabled.enter(), false);
});

void test("rebinds reactive targets and cleans up with the scope", async () => {
  const host = new FakeDocument();
  const first = new FakeElement(host);
  const second = new FakeElement(host);
  const target = ref<FakeElement | null>(first);
  const scope = effectScope();
  const fullscreen = scope.run(() => useFullscreen(target, { host, autoExit: true }));
  assert.ok(fullscreen);

  await second.requestFullscreen();
  target.value = second;
  await nextTick();
  assert.equal(fullscreen.isFullscreen.value, true);

  scope.stop();
  await Promise.resolve();
  assert.equal(host.listeners, 0);
  assert.equal(host.fullscreenElement, null, "autoExit leaves fullscreen");
});

void test("server rendering reports no support", async () => {
  const state = await renderComposableOnServer(() => {
    const fullscreen = useFullscreen();
    return { supported: fullscreen.supported, isFullscreen: fullscreen.isFullscreen };
  });
  assert.equal(state, '{"supported":false,"isFullscreen":false}');
});
