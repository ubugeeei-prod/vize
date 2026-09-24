import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref, shallowRef } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useMediaControls } from "./use-media-controls.ts";
import type {
  MediaElementLike,
  MediaSourceCandidate,
  PictureInPictureDocumentLike,
  TextTrackLike,
  TextTrackListLike,
  TimeRangesLike,
} from "./use-media-controls.ts";

class FakeRanges implements TimeRangesLike {
  readonly ranges: readonly (readonly [number, number])[];

  constructor(ranges: readonly (readonly [number, number])[]) {
    this.ranges = ranges;
  }

  get length(): number {
    return this.ranges.length;
  }

  start(index: number): number {
    return this.ranges[index]?.[0] ?? 0;
  }

  end(index: number): number {
    return this.ranges[index]?.[1] ?? 0;
  }
}

class FakeTrackList extends EventTarget implements TextTrackListLike {
  [index: number]: TextTrackLike;
  length = 0;

  add(track: TextTrackLike): void {
    this[this.length] = track;
    this.length += 1;
  }
}

class FakeMedia extends EventTarget implements MediaElementLike {
  currentTime = 0;
  volume = 1;
  muted = false;
  playbackRate = 1;
  src = "";
  duration = Number.NaN;
  paused = true;
  ended = false;
  seeking = false;
  buffered: TimeRangesLike = new FakeRanges([]);
  readonly textTracks = new FakeTrackList();
  blockPlay = false;
  writes: string[] = [];
  listeners = 0;
  readonly playable = new Set<string>(["video/mp4"]);

  override addEventListener(type: string, listener: EventListener): void {
    this.listeners += 1;
    super.addEventListener(type, listener);
  }

  override removeEventListener(type: string, listener: EventListener): void {
    this.listeners -= 1;
    super.removeEventListener(type, listener);
  }

  play(): Promise<void> {
    if (this.blockPlay) return Promise.reject(new Error("NotAllowedError"));
    this.paused = false;
    this.emit("play");
    return Promise.resolve();
  }

  pause(): void {
    this.paused = true;
    this.emit("pause");
  }

  canPlayType(type: string): string {
    return this.playable.has(type) ? "maybe" : "";
  }

  requestPictureInPicture(): Promise<unknown> {
    return Promise.resolve({});
  }

  emit(type: string): void {
    this.dispatchEvent(new Event(type));
  }
}

void test("reads the element state when it attaches", () => {
  const element = new FakeMedia();
  element.currentTime = 12;
  element.volume = 0.5;
  element.duration = 60;
  element.buffered = new FakeRanges([[0, 30]]);
  const media = useMediaControls(element, { document: null });

  assert.equal(media.currentTime.value, 12);
  assert.equal(media.volume.value, 0.5);
  assert.equal(media.duration.value, 60);
  assert.deepEqual(media.buffered.value, [[0, 30]]);
  assert.equal(media.playing.value, false);
});

void test("writes refs to the element and follows element events without echo", () => {
  const element = new FakeMedia();
  const media = useMediaControls(element, { document: null });

  media.currentTime.value = 5;
  media.volume.value = 0.2;
  media.muted.value = true;
  media.rate.value = 2;
  assert.equal(element.currentTime, 5);
  assert.equal(element.volume, 0.2);
  assert.equal(element.muted, true);
  assert.equal(element.playbackRate, 2);

  let assignments = 0;
  Object.defineProperty(element, "currentTime", {
    get: () => 9,
    set: () => {
      assignments += 1;
    },
  });
  element.emit("timeupdate");
  assert.equal(media.currentTime.value, 9);
  assert.equal(assignments, 0, "element-driven updates are not written back");
});

void test("plays and pauses through the playing ref", async () => {
  const element = new FakeMedia();
  const media = useMediaControls(element, { document: null });

  media.playing.value = true;
  await Promise.resolve();
  assert.equal(element.paused, false);
  media.playing.value = false;
  assert.equal(element.paused, true);

  element.blockPlay = true;
  assert.equal(await media.play(), false);
  assert.ok(media.error.value instanceof Error);
  assert.equal(media.playing.value, false);
});

void test("tracks buffering, seeking, stalls, and the end", () => {
  const element = new FakeMedia();
  const media = useMediaControls(element, { document: null });

  element.emit("waiting");
  assert.equal(media.waiting.value, true);
  element.emit("canplay");
  assert.equal(media.waiting.value, false);
  element.emit("seeking");
  assert.equal(media.seeking.value, true);
  element.emit("seeked");
  assert.equal(media.seeking.value, false);
  element.emit("stalled");
  assert.equal(media.stalled.value, true);
  element.emit("playing");
  assert.equal(media.stalled.value, false);
  void element.play();
  element.emit("ended");
  assert.equal(media.ended.value, true);
  assert.equal(media.playing.value, false);
  element.duration = 42;
  element.emit("durationchange");
  assert.equal(media.duration.value, 42);
});

void test("chooses the first playable source", async () => {
  const element = new FakeMedia();
  const source = ref<string | readonly MediaSourceCandidate[]>([
    { src: "/a.webm", type: "video/webm" },
    { src: "/a.mp4", type: "video/mp4" },
  ]);
  useMediaControls(element, { src: source, document: null });
  assert.equal(element.src, "/a.mp4");
  source.value = "/b.mp4";
  await nextTick();
  assert.equal(element.src, "/b.mp4");
});

void test("lists and selects text tracks", () => {
  const element = new FakeMedia();
  element.textTracks.add({
    id: "en",
    kind: "subtitles",
    label: "English",
    language: "en",
    mode: "disabled",
  });
  element.textTracks.add({
    id: "ja",
    kind: "subtitles",
    label: "日本語",
    language: "ja",
    mode: "showing",
  });
  const media = useMediaControls(element, { document: null });

  assert.equal(media.tracks.value.length, 2);
  assert.equal(media.selectedTrack.value, 1);
  media.selectTrack(0);
  assert.equal(media.selectedTrack.value, 0);
  assert.equal(element.textTracks[1]?.mode, "disabled");
  media.selectTrack(-1);
  assert.equal(media.selectedTrack.value, -1);
});

void test("toggles picture-in-picture", async () => {
  const element = new FakeMedia();
  let exited = 0;
  const document: PictureInPictureDocumentLike = {
    pictureInPictureEnabled: true,
    exitPictureInPicture: () => {
      exited += 1;
      return Promise.resolve();
    },
  };
  const media = useMediaControls(element, { document });
  assert.equal(media.supportsPictureInPicture.value, true);
  assert.equal(await media.togglePictureInPicture(), true);
  assert.equal(await media.togglePictureInPicture(), false);
  assert.equal(exited, 1);
  element.emit("enterpictureinpicture");
  assert.equal(media.isPictureInPicture.value, true);
});

void test("rebinds listeners when the target changes and detaches with the scope", async () => {
  const first = new FakeMedia();
  const second = new FakeMedia();
  second.volume = 0.3;
  const target = shallowRef<FakeMedia | null>(first);
  const scope = effectScope();
  const media = scope.run(() => useMediaControls(target, { document: null }));
  assert.ok(media);
  assert.ok(first.listeners > 0);

  target.value = second;
  await nextTick();
  assert.equal(first.listeners, 0);
  assert.equal(media.volume.value, 0.3);

  scope.stop();
  assert.equal(second.listeners, 0);
});

void test("server rendering keeps neutral defaults", async () => {
  const state = await renderComposableOnServer(() => {
    const media = useMediaControls(null, { src: "/clip.mp4" });
    return {
      playing: media.playing,
      currentTime: media.currentTime,
      volume: media.volume,
      duration: media.duration,
      pip: media.supportsPictureInPicture,
    };
  });
  assert.equal(state, '{"playing":false,"currentTime":0,"volume":1,"duration":0,"pip":false}');
});
