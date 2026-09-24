import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useMediaStream, useUserMedia } from "./use-user-media.ts";
import type {
  MediaDeviceInfoLike,
  MediaStreamLike,
  MediaStreamTrackLike,
  UserMediaHost,
} from "./use-user-media.ts";

class FakeTrack extends EventTarget implements MediaStreamTrackLike {
  readonly kind: string;
  readyState = "live";

  constructor(kind: string) {
    super();
    this.kind = kind;
  }

  stop(): void {
    this.readyState = "ended";
  }

  end(): void {
    this.readyState = "ended";
    this.dispatchEvent(new Event("ended"));
  }
}

class FakeStream implements MediaStreamLike {
  readonly tracks = [new FakeTrack("audio"), new FakeTrack("video")];

  getTracks(): readonly MediaStreamTrackLike[] {
    return this.tracks;
  }

  get stopped(): boolean {
    return this.tracks.every((track) => track.readyState === "ended");
  }
}

interface Pending {
  readonly constraints: MediaStreamConstraints | undefined;
  readonly resolve: (stream: FakeStream) => void;
  readonly reject: (error: unknown) => void;
}

class FakeDevices extends EventTarget implements UserMediaHost {
  readonly pending: Pending[] = [];
  list: MediaDeviceInfoLike[] = [];
  enumerations = 0;

  getUserMedia(constraints?: MediaStreamConstraints): Promise<MediaStreamLike> {
    return new Promise((resolve, reject) => this.pending.push({ constraints, resolve, reject }));
  }

  enumerateDevices(): Promise<readonly MediaDeviceInfoLike[]> {
    this.enumerations += 1;
    return Promise.resolve(this.list);
  }

  grant(index = -1): FakeStream {
    const stream = new FakeStream();
    this.pending.at(index)?.resolve(stream);
    return stream;
  }
}

function named(name: string): Error {
  const error = new Error(name);
  error.name = name;
  return error;
}

async function flush(): Promise<void> {
  for (let index = 0; index < 4; index += 1) await Promise.resolve();
}

void test("starts with the default constraints and stops every track", async () => {
  const host = new FakeDevices();
  const media = useUserMedia({ host });
  assert.equal(media.supported.value, true);
  assert.equal(media.status.value, "idle");

  const started = media.start();
  assert.equal(media.status.value, "requesting");
  assert.deepEqual(host.pending[0]?.constraints, { audio: true, video: true });
  const stream = host.grant();
  assert.equal(await started, stream);
  assert.equal(media.status.value, "active");
  assert.equal(media.stream.value, stream);
  assert.equal(await media.start(), stream, "an active stream is reused");

  media.stop();
  assert.equal(stream.stopped, true);
  assert.equal(media.status.value, "idle");
  assert.equal(media.stream.value, undefined);
});

void test("classifies acquisition failures", async () => {
  const cases: [string, string][] = [
    ["NotAllowedError", "permission-denied"],
    ["NotFoundError", "not-found"],
    ["NotReadableError", "not-readable"],
    ["OverconstrainedError", "overconstrained"],
    ["AbortError", "aborted"],
    ["TypeError", "invalid-constraints"],
    ["WeirdError", "failed"],
  ];
  for (const [name, code] of cases) {
    const host = new FakeDevices();
    const media = useUserMedia({ host });
    const started = media.start();
    host.pending[0]?.reject(named(name));
    assert.equal(await started, undefined);
    assert.equal(media.status.value, "error");
    assert.equal(media.error.value?.code, code);
  }
});

void test("stops a stream that resolves after a newer request", async () => {
  const host = new FakeDevices();
  const media = useUserMedia({ host });
  const first = media.start();
  media.stop();
  const second = media.start();
  const stale = host.grant(0);
  const fresh = host.grant(1);
  assert.equal(await first, undefined);
  assert.equal(stale.stopped, true);
  assert.equal(await second, fresh);
  assert.equal(media.stream.value, fresh);
});

void test("follows enabled and restarts on constraint changes", async () => {
  const host = new FakeDevices();
  const enabled = ref(false);
  const constraints = ref<MediaStreamConstraints>({ video: true });
  const media = useUserMedia({ host, enabled, constraints });

  enabled.value = true;
  await nextTick();
  const first = host.grant();
  await flush();
  assert.equal(media.status.value, "active");

  constraints.value = { video: { width: 640 } };
  await nextTick();
  assert.equal(first.stopped, true);
  assert.deepEqual(host.pending[1]?.constraints, { video: { width: 640 } });
  host.grant();
  await flush();

  enabled.value = false;
  await nextTick();
  assert.equal(media.status.value, "idle");
});

void test("returns to idle when every track ended", async () => {
  const host = new FakeDevices();
  const media = useUserMedia({ host });
  const started = media.start();
  const stream = host.grant();
  await started;
  stream.tracks[0]?.end();
  assert.equal(media.status.value, "active");
  stream.tracks[1]?.end();
  assert.equal(media.status.value, "idle");
});

void test("lists devices and follows devicechange", async () => {
  const host = new FakeDevices();
  host.list = [{ deviceId: "a", kind: "videoinput", label: "", groupId: "g" }];
  const scope = effectScope();
  const media = scope.run(() => useUserMedia({ host, listDevices: true }));
  assert.ok(media);
  await flush();
  assert.equal(media.devices.value.length, 1);

  host.list = [];
  host.dispatchEvent(new Event("devicechange"));
  await flush();
  assert.equal(media.devices.value.length, 0);

  scope.stop();
  const before = host.enumerations;
  host.dispatchEvent(new Event("devicechange"));
  assert.equal(host.enumerations, before);
});

void test("stops tracks with the scope", async () => {
  const host = new FakeDevices();
  const scope = effectScope();
  const media = scope.run(() => useUserMedia({ host }));
  assert.ok(media);
  const started = media.start();
  const stream = host.grant();
  await started;
  scope.stop();
  assert.equal(stream.stopped, true);
});

void test("wraps arbitrary sources through useMediaStream", async () => {
  const stream = new FakeStream();
  const requested: number[] = [];
  const media = useMediaStream({
    source: {
      request: (fps: number) => {
        requested.push(fps);
        return Promise.resolve(stream);
      },
    },
    constraints: 30,
  });
  assert.equal(await media.start(), stream);
  assert.deepEqual(requested, [30]);
  assert.equal(await media.restart(), stream);
  assert.deepEqual(requested, [30, 30]);
});

void test("a missing source without a window stays idle", async () => {
  const media = useUserMedia({ host: null });
  assert.equal(media.supported.value, false);
  assert.equal(await media.start(), undefined);
  assert.equal(media.status.value, "idle");
  assert.equal(media.error.value, undefined);
});

void test("server rendering touches no device", async () => {
  const state = await renderComposableOnServer(() => {
    const media = useUserMedia({ enabled: true, listDevices: true });
    return { supported: media.supported, status: media.status, devices: media.devices };
  });
  assert.equal(state, '{"supported":false,"status":"idle","devices":[]}');
});
