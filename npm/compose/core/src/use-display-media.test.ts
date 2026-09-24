import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useDisplayMedia } from "./use-display-media.ts";
import type { DisplayMediaHost } from "./use-display-media.ts";
import type { MediaStreamLike, MediaStreamTrackLike } from "./use-user-media.ts";

class FakeTrack extends EventTarget implements MediaStreamTrackLike {
  readonly kind = "video";
  readyState = "live";

  stop(): void {
    this.readyState = "ended";
  }

  end(): void {
    this.readyState = "ended";
    this.dispatchEvent(new Event("ended"));
  }
}

class FakeStream implements MediaStreamLike {
  readonly track = new FakeTrack();

  getTracks(): readonly MediaStreamTrackLike[] {
    return [this.track];
  }
}

class FakeDisplay implements DisplayMediaHost {
  readonly requests: (DisplayMediaStreamOptions | undefined)[] = [];
  failure: unknown;

  getDisplayMedia(options?: DisplayMediaStreamOptions): Promise<MediaStreamLike> {
    this.requests.push(options);
    return this.failure === undefined
      ? Promise.resolve(new FakeStream())
      : Promise.reject(this.failure);
  }
}

void test("requests display capture with reactive constraints", async () => {
  const host = new FakeDisplay();
  const audio = ref(false);
  const screen = useDisplayMedia({ host, audio });
  assert.equal(screen.supported.value, true);

  await screen.start();
  assert.deepEqual(host.requests[0], { video: true, audio: false });
  assert.equal(screen.status.value, "active");

  audio.value = true;
  await nextTick();
  await Promise.resolve();
  assert.deepEqual(host.requests[1], { video: true, audio: true });
});

void test("returns to idle when the user stops sharing", async () => {
  const host = new FakeDisplay();
  const screen = useDisplayMedia({ host });
  const stream = await screen.start();
  assert.ok(stream instanceof FakeStream);
  stream.track.end();
  assert.equal(screen.status.value, "idle");
  assert.equal(screen.stream.value, undefined);
});

void test("reports a dismissed picker as permission denied", async () => {
  const host = new FakeDisplay();
  const error = new Error("dismissed");
  error.name = "NotAllowedError";
  host.failure = error;
  const screen = useDisplayMedia({ host });
  assert.equal(await screen.start(), undefined);
  assert.equal(screen.error.value?.code, "permission-denied");
});

void test("follows enabled and stops with the scope", async () => {
  const host = new FakeDisplay();
  const enabled = ref(true);
  const scope = effectScope();
  const screen = scope.run(() => useDisplayMedia({ host, enabled }));
  assert.ok(screen);
  await Promise.resolve();
  await Promise.resolve();
  const stream = screen.stream.value;
  assert.ok(stream instanceof FakeStream);
  scope.stop();
  assert.equal(stream.track.readyState, "ended");
});

void test("server rendering captures nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const screen = useDisplayMedia({ enabled: true });
    return { supported: screen.supported, status: screen.status };
  });
  assert.equal(state, '{"supported":false,"status":"idle"}');
});
