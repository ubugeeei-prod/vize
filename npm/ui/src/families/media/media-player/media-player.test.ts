import assert from "node:assert/strict";

import { afterEach, beforeEach, test } from "vite-plus/test";
import { h, nextTick } from "vue";

import AudioPlayerAudio from "../audio-player/audio-player-audio.vue";
import type { MediaPlayerRootExpose, MediaPlayerSlotState } from "./media-player.ts";
import MediaPlayerCaptionsButton from "./media-player-captions-button.vue";
import MediaPlayerLoadingIndicator from "./media-player-loading-indicator.vue";
import MediaPlayerMuteButton from "./media-player-mute-button.vue";
import MediaPlayerPlayButton from "./media-player-play-button.vue";
import MediaPlayerPlaybackRateButton from "./media-player-playback-rate-button.vue";
import MediaPlayerRoot from "./media-player-root.vue";
import MediaPlayerSeekSlider from "./media-player-seek-slider.vue";
import { installFakeMedia } from "./media-player-test-utils.ts";
import type { FakeMedia } from "./media-player-test-utils.ts";
import MediaPlayerTimeDisplay from "./media-player-time-display.vue";
import MediaPlayerVolumeSlider from "./media-player-volume-slider.vue";
import { mountInteraction } from "../../../testing/mount.ts";

let fake: FakeMedia;

beforeEach(() => {
  fake = installFakeMedia();
});

afterEach(() => {
  fake.restore();
});

async function settle(): Promise<void> {
  await nextTick();
  await Promise.resolve();
  await nextTick();
}

function mountPlayer(props: Record<string, unknown> = {}) {
  return mountInteraction(MediaPlayerRoot, {
    props: { id: "podcast", ...props },
    record: [
      "update:volume",
      "update:muted",
      "update:playbackRate",
      "update:currentTime",
      "play",
      "pause",
      "ended",
      "playRejected",
      "error",
    ],
    slots: {
      default: (state: MediaPlayerSlotState) => [
        h("output", { "data-slot-state": state.state }),
        h(AudioPlayerAudio, { src: "/episode.mp3" }),
        h(MediaPlayerPlayButton),
        h(MediaPlayerMuteButton),
        h(MediaPlayerSeekSlider),
        h(MediaPlayerVolumeSlider),
        h(MediaPlayerTimeDisplay, { "data-testid": "current" }),
        h(MediaPlayerTimeDisplay, { mode: "duration", "data-testid": "duration" }),
        h(MediaPlayerTimeDisplay, { mode: "remaining", "data-testid": "remaining" }),
        h(MediaPlayerPlaybackRateButton),
        h(MediaPlayerCaptionsButton),
        h(MediaPlayerLoadingIndicator),
      ],
    },
  });
}

function mediaOf(root: HTMLElement): HTMLAudioElement {
  const media = root.querySelector("audio");
  assert.ok(media);
  return media;
}

function part(root: HTMLElement, name: string): HTMLElement {
  const element = root.querySelector<HTMLElement>(`[data-vize-ui="${name}"]`);
  assert.ok(element, `${name} must render`);
  return element;
}

function key(target: Element, value: string, init: KeyboardEventInit = {}): KeyboardEvent {
  const event = new KeyboardEvent("keydown", {
    key: value,
    bubbles: true,
    cancelable: true,
    ...init,
  });
  target.dispatchEvent(event);
  return event;
}

test("renders labelled controls wired to the registered media element", async () => {
  const handle = mountPlayer({ ariaLabel: "Episode 12" });
  const root = handle.root();
  await settle();

  assert.equal(root.id, "podcast");
  assert.equal(root.getAttribute("role"), "group");
  assert.equal(root.getAttribute("aria-label"), "Episode 12");
  assert.equal(root.getAttribute("data-vize-ui"), "media-player-root");
  assert.equal(root.getAttribute("data-state"), "paused");
  assert.equal(root.getAttribute("data-media-kind"), "audio");
  const media = mediaOf(root);
  assert.equal(media.id, "podcast-media");
  assert.equal(media.getAttribute("preload"), "metadata");
  const play = handle.getByRole("button", { name: "Play" });
  assert.equal(play.getAttribute("aria-controls"), "podcast-media");
  assert.ok(handle.getByRole("button", { name: "Mute" }));
  assert.ok(handle.getByRole("button", { name: "Playback speed 1×" }));
  const captions = handle.getByRole("button", { name: "Show captions" }) as HTMLButtonElement;
  assert.equal(captions.disabled, true, "captions need a caption track");
  const seek = handle.getByRole("slider", { name: "Seek" });
  assert.equal(seek.id, "podcast-seek");
  assert.equal(seek.getAttribute("tabindex"), "-1", "unknown durations cannot be sought");
  assert.equal(seek.getAttribute("aria-disabled"), "true");
  const volume = handle.getByRole("slider", { name: "Volume" });
  assert.equal(volume.getAttribute("aria-valuenow"), "100");
  assert.equal(volume.getAttribute("aria-valuetext"), "100%");
  assert.equal(handle.getByRole("status").textContent, "");
  handle.unmount();
});

test("play, pause, and replay follow native events and report rejected playback", async () => {
  const handle = mountPlayer();
  const root = handle.root();
  const media = mediaOf(root);
  await settle();

  await handle.click(handle.getByRole("button", { name: "Play" }));
  await settle();
  assert.equal(root.getAttribute("data-state"), "playing");
  assert.equal(root.querySelector("[data-slot-state]")?.getAttribute("data-slot-state"), "playing");
  await handle.click(handle.getByRole("button", { name: "Pause" }));
  await settle();
  assert.equal(root.getAttribute("data-state"), "paused");

  fake.setDuration(media, 60);
  fake.end(media);
  await settle();
  assert.equal(root.getAttribute("data-state"), "ended");
  await handle.click(handle.getByRole("button", { name: "Replay" }));
  await settle();
  assert.equal(root.getAttribute("data-state"), "playing");
  assert.equal(media.currentTime, 0);
  assert.equal(handle.wrapper.emitted("play")?.length, 2);
  assert.equal(handle.wrapper.emitted("pause")?.length, 2);
  assert.equal(handle.wrapper.emitted("ended")?.length, 1);

  media.pause();
  fake.rejectPlay = true;
  const exposed = handle.exposes<MediaPlayerRootExpose>();
  assert.equal(await exposed.play(), false);
  assert.ok(handle.wrapper.emitted("playRejected")?.[0]?.[0] instanceof DOMException);
  assert.equal(exposed.state, "paused");
  handle.unmount();
});

test("the seek slider exposes time value text, keyboard seeking, and buffered progress", async () => {
  const handle = mountPlayer();
  const root = handle.root();
  const media = mediaOf(root);
  await settle();
  fake.setDuration(media, 300);
  fake.advance(media, 83);
  fake.setBuffered(media, 150);
  await settle();

  const seek = handle.getByRole("slider", { name: "Seek" });
  assert.equal(seek.getAttribute("tabindex"), "0");
  assert.equal(seek.getAttribute("aria-valuemax"), "300");
  assert.equal(seek.getAttribute("aria-valuenow"), "83");
  assert.equal(seek.getAttribute("aria-valuetext"), "1:23 of 5:00");
  const track = part(root, "media-player-seek-slider");
  assert.match(track.getAttribute("style") ?? "", /--vize-ui-media-player-progress: 27\.67%/);
  assert.match(track.getAttribute("style") ?? "", /--vize-ui-media-player-buffered: 50%/);
  assert.equal(part(root, "media-player-time-display").textContent, "1:23");
  assert.equal(root.querySelector('[data-mode="duration"]')?.textContent, "5:00");
  assert.equal(root.querySelector('[data-mode="remaining"]')?.textContent, "-3:37");

  assert.equal(key(seek, "ArrowRight").defaultPrevented, true);
  assert.equal(media.currentTime, 88);
  key(seek, "ArrowLeft");
  key(seek, "ArrowDown");
  assert.equal(media.currentTime, 78);
  key(seek, "PageUp");
  assert.equal(media.currentTime, 108);
  key(seek, "End");
  assert.equal(media.currentTime, 300);
  key(seek, "Home");
  assert.equal(media.currentTime, 0);
  key(seek, "ArrowLeft");
  assert.equal(media.currentTime, 0, "seeking clamps at the start");
  assert.deepEqual(handle.wrapper.emitted("update:currentTime")?.at(-1), [0]);
  handle.unmount();
});

test("pointer scrubbing seeks with capture and resumes playback afterwards", async () => {
  const handle = mountPlayer();
  const root = handle.root();
  const media = mediaOf(root);
  await settle();
  fake.setDuration(media, 200);
  await media.play();
  await settle();
  const track = part(root, "media-player-seek-slider");
  track.getBoundingClientRect = () => new DOMRect(0, 0, 400, 10);

  track.dispatchEvent(
    new PointerEvent("pointerdown", { button: 0, clientX: 100, pointerId: 3, cancelable: true }),
  );
  await settle();
  assert.equal(media.currentTime, 50);
  assert.equal(media.paused, true, "playback pauses while scrubbing");
  assert.equal(track.getAttribute("data-dragging"), "true");
  track.dispatchEvent(new PointerEvent("pointermove", { clientX: 300, pointerId: 3 }));
  assert.equal(media.currentTime, 150);
  track.dispatchEvent(new PointerEvent("pointermove", { clientX: 900, pointerId: 9 }));
  assert.equal(media.currentTime, 150, "other pointers are ignored");
  track.dispatchEvent(new PointerEvent("pointerup", { clientX: 300, pointerId: 3 }));
  await settle();
  assert.equal(media.paused, false, "playback resumes after scrubbing");
  assert.equal(track.hasAttribute("data-dragging"), false);
  handle.unmount();
});

test("volume and mute controls stay in sync with the element and controlled props", async () => {
  const handle = mountPlayer({ defaultVolume: 0.5 });
  const root = handle.root();
  const media = mediaOf(root);
  await settle();
  assert.equal(media.volume, 0.5);

  const volume = handle.getByRole("slider", { name: "Volume" });
  key(volume, "ArrowUp");
  await settle();
  assert.equal(media.volume, 0.55);
  key(volume, "PageDown");
  await settle();
  assert.equal(media.volume, 0.45);
  key(volume, "Home");
  await settle();
  assert.equal(media.volume, 0);
  key(volume, "End");
  await settle();
  assert.equal(media.volume, 1);

  await handle.click(handle.getByRole("button", { name: "Mute" }));
  await settle();
  assert.equal(media.muted, true);
  assert.equal(root.getAttribute("data-muted"), "true");
  assert.equal(volume.getAttribute("aria-valuetext"), "Muted");
  key(volume, "ArrowDown");
  await settle();
  assert.equal(media.muted, false, "raising or changing volume from a control unmutes");

  media.muted = true;
  media.volume = 0.3;
  await settle();
  assert.equal(root.getAttribute("data-muted"), "true", "native volume changes do not unmute");
  assert.deepEqual(handle.wrapper.emitted("update:volume")?.at(-1), [0.3]);

  const controlled = mountPlayer({ volume: 0.2, muted: false });
  const controlledMedia = mediaOf(controlled.root());
  await settle();
  key(controlled.getByRole("slider", { name: "Volume" }), "ArrowUp");
  await settle();
  assert.deepEqual(controlled.wrapper.emitted("update:volume"), [[0.25]]);
  assert.equal(controlledMedia.volume, 0.2, "controlled volume waits for the parent");
  await controlled.wrapper.setProps({ volume: 0.25 });
  assert.equal(controlledMedia.volume, 0.25);
  controlled.unmount();
  handle.unmount();
});

test("volume pointer input maps the track width to the volume", async () => {
  const handle = mountPlayer({ dir: "rtl" });
  const root = handle.root();
  const media = mediaOf(root);
  await settle();
  const track = part(root, "media-player-volume-slider");
  track.getBoundingClientRect = () => new DOMRect(0, 0, 100, 10);
  track.dispatchEvent(new PointerEvent("pointerdown", { button: 0, clientX: 25, pointerId: 1 }));
  await settle();
  assert.equal(media.volume, 0.75, "rtl tracks grow from the right");
  track.dispatchEvent(new PointerEvent("pointermove", { clientX: 100, pointerId: 1 }));
  track.dispatchEvent(new PointerEvent("pointerup", { clientX: 100, pointerId: 1 }));
  await settle();
  assert.equal(media.volume, 0);
  handle.unmount();
});

test("playback rate cycles through rates and mirrors native rate changes", async () => {
  const handle = mountPlayer();
  const root = handle.root();
  const media = mediaOf(root);
  await settle();
  await handle.click(handle.getByRole("button", { name: "Playback speed 1×" }));
  await settle();
  assert.equal(media.playbackRate, 1.25);
  assert.ok(handle.getByRole("button", { name: "Playback speed 1.25×" }));
  media.playbackRate = 3;
  await settle();
  assert.deepEqual(handle.wrapper.emitted("update:playbackRate")?.at(-1), [3]);
  await handle.click(handle.getByRole("button", { name: "Playback speed 3×" }));
  await settle();
  assert.equal(media.playbackRate, 0.5, "unknown rates restart the cycle");
  assert.equal(handle.exposes<MediaPlayerRootExpose>().setPlaybackRate(0), false);
  handle.unmount();
});

test("captions toggle between hidden and the last or first caption track", async () => {
  const handle = mountPlayer();
  const root = handle.root();
  const media = mediaOf(root);
  const tracks = fake.state(media).tracks;
  tracks.add({ kind: "chapters", label: "Chapters", language: "en" });
  tracks.add({ kind: "captions", label: "English", language: "en" });
  tracks.add({ kind: "subtitles", label: "Français", language: "fr" });
  await settle();

  const exposed = handle.exposes<MediaPlayerRootExpose>();
  assert.equal(exposed.textTracks.length, 3);
  await handle.click(handle.getByRole("button", { name: "Show captions" }));
  await settle();
  assert.equal(exposed.captionTrack, 1);
  assert.equal(root.getAttribute("data-captions"), "true");
  assert.equal(exposed.setCaptionTrack(2), true);
  await settle();
  assert.equal(exposed.captionTrack, 2);
  assert.equal(tracks.items[1]?.mode, "disabled");
  await handle.click(handle.getByRole("button", { name: "Hide captions" }));
  await settle();
  assert.equal(exposed.captionTrack, -1);
  assert.equal(tracks.items[0]?.mode, "disabled", "non-caption tracks are untouched");
  assert.equal(exposed.toggleCaptions(), true);
  await settle();
  assert.equal(exposed.captionTrack, 2, "the last caption track is restored");
  handle.unmount();
});

test("root keyboard shortcuts control playback, time, volume, and captions", async () => {
  const handle = mountPlayer({ defaultVolume: 0.5 });
  const root = handle.root();
  const media = mediaOf(root);
  fake.state(media).tracks.add({ kind: "captions", label: "English", language: "en" });
  await settle();
  fake.setDuration(media, 100);
  fake.advance(media, 50);
  await settle();
  const time = part(root, "media-player-time-display");

  assert.equal(key(time, "k").defaultPrevented, true);
  await settle();
  assert.equal(media.paused, false);
  key(time, " ");
  await settle();
  assert.equal(media.paused, true);
  key(time, "l");
  assert.equal(media.currentTime, 60);
  key(time, "J");
  assert.equal(media.currentTime, 50);
  key(time, "ArrowRight");
  assert.equal(media.currentTime, 55);
  key(time, "ArrowLeft");
  assert.equal(media.currentTime, 50);
  key(time, "7");
  assert.equal(media.currentTime, 70);
  key(time, "ArrowUp");
  await settle();
  assert.equal(media.volume, 0.55);
  key(time, "ArrowDown");
  await settle();
  assert.equal(media.volume, 0.5);
  key(time, "m");
  await settle();
  assert.equal(media.muted, true);
  key(time, "c");
  await settle();
  assert.equal(handle.exposes<MediaPlayerRootExpose>().captionTrack, 0);
  assert.equal(key(time, "f").defaultPrevented, false, "unsupported fullscreen is not handled");
  assert.equal(key(time, "k", { ctrlKey: true }).defaultPrevented, false);
  assert.equal(key(time, "x").defaultPrevented, false);
  handle.unmount();
});

test("shortcuts ignore editable targets, focused buttons, and opt-out roots", async () => {
  const handle = mountInteraction(MediaPlayerRoot, {
    slots: {
      default: () => [
        h(AudioPlayerAudio, { src: "/a.mp3" }),
        h("input", { type: "text", "data-testid": "note" }),
        h(MediaPlayerPlayButton),
      ],
    },
  });
  const root = handle.root();
  const media = mediaOf(root);
  await settle();
  const input = root.querySelector("input");
  assert.ok(input);
  assert.equal(key(input, "k").defaultPrevented, false);
  const button = handle.getByRole("button", { name: "Play" });
  assert.equal(key(button, " ").defaultPrevented, false, "Space activates the focused button");
  await settle();
  assert.equal(media.paused, true);
  assert.equal(key(button, "k").defaultPrevented, true);
  handle.unmount();

  const disabled = mountPlayer({ keyboardShortcuts: false });
  await settle();
  assert.equal(
    key(part(disabled.root(), "media-player-time-display"), "k").defaultPrevented,
    false,
  );
  disabled.unmount();
});

test("loading and error states surface through data attributes, status, and emits", async () => {
  const handle = mountPlayer();
  const root = handle.root();
  const media = mediaOf(root);
  await settle();
  await media.play();
  fake.wait(media);
  await settle();
  assert.equal(root.getAttribute("data-loading"), "true");
  assert.equal(handle.getByRole("status").textContent, "Loading");
  media.dispatchEvent(new Event("canplay"));
  await settle();
  assert.equal(root.hasAttribute("data-loading"), false);
  fake.fail(media, 4);
  await settle();
  assert.deepEqual(handle.wrapper.emitted("error"), [[4]]);
  assert.equal(handle.exposes<MediaPlayerRootExpose>().error, 4);
  handle.unmount();
});

test("messages localize every label and aria-label overrides are honored", async () => {
  const handle = mountInteraction(MediaPlayerRoot, {
    props: {
      messages: {
        play: "再生",
        mute: "ミュート",
        seek: "シーク",
        seekValueText: (current: string, duration: string) => `${current} / ${duration}`,
        playbackRate: (rate: number) => `速度 ${rate}`,
      },
    },
    slots: {
      default: () => [
        h(AudioPlayerAudio, { src: "/a.mp3" }),
        h(MediaPlayerPlayButton),
        h(MediaPlayerMuteButton, { ariaLabel: null }, () => "Sound"),
        h(MediaPlayerSeekSlider),
        h(MediaPlayerPlaybackRateButton, { ariaLabel: "Speed" }),
        h(MediaPlayerVolumeSlider, { ariaLabel: "Loudness" }),
      ],
    },
  });
  const media = mediaOf(handle.root());
  await settle();
  fake.setDuration(media, 65);
  await settle();
  assert.ok(handle.getByRole("button", { name: "再生" }));
  assert.ok(handle.getByRole("button", { name: "Sound" }));
  assert.ok(handle.getByRole("button", { name: "Speed" }));
  assert.equal(
    handle.getByRole("slider", { name: "シーク" }).getAttribute("aria-valuetext"),
    "0:00 / 1:05",
  );
  assert.ok(handle.getByRole("slider", { name: "Loudness" }));
  handle.unmount();
});

test("controls honor preventDefault and stay disabled without media", async () => {
  const blocked = (event: MouseEvent) => event.preventDefault();
  const handle = mountInteraction(MediaPlayerRoot, {
    slots: {
      default: () => [
        h(AudioPlayerAudio, { src: "/a.mp3" }),
        h(MediaPlayerPlayButton, { onClick: blocked }),
        h(MediaPlayerMuteButton, { onClick: blocked }),
        h(MediaPlayerPlaybackRateButton, { onClick: blocked }),
      ],
    },
  });
  const media = mediaOf(handle.root());
  await settle();
  for (const button of handle.root().querySelectorAll("button")) {
    button.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
  }
  await settle();
  assert.equal(media.paused, true);
  assert.equal(media.muted, false);
  assert.equal(media.playbackRate, 1);
  handle.unmount();

  const empty = mountInteraction(MediaPlayerRoot, {
    slots: { default: () => [h(MediaPlayerPlayButton), h(MediaPlayerMuteButton)] },
  });
  for (const button of empty.root().querySelectorAll("button")) {
    assert.equal(button.disabled, true);
  }
  assert.equal(await empty.exposes<MediaPlayerRootExpose>().togglePlay(), false);
  empty.unmount();
});

test("synchronized currentTime seeks when the parent moves it", async () => {
  const handle = mountPlayer({ currentTime: 12 });
  const media = mediaOf(handle.root());
  await settle();
  fake.setDuration(media, 100);
  assert.equal(media.currentTime, 12, "the initial position is applied on registration");
  await handle.wrapper.setProps({ currentTime: 40 });
  assert.equal(media.currentTime, 40);
  fake.advance(media, 40.3);
  await handle.wrapper.setProps({ currentTime: 40.3 });
  assert.equal(media.currentTime, 40.3);
  assert.deepEqual(handle.wrapper.emitted("update:currentTime")?.at(-1), [40.3]);
  handle.unmount();
});

test("exposes typed state and imperative controls", async () => {
  const handle = mountPlayer();
  const exposed = handle.exposes<MediaPlayerRootExpose>();
  const media = mediaOf(handle.root());
  await settle();
  fake.setDuration(media, 90);
  await settle();
  assert.ok(exposed.element === handle.root());
  assert.ok(exposed.media === media);
  assert.equal(exposed.duration, 90);
  assert.equal(await exposed.togglePlay(), true);
  assert.equal(exposed.paused, false);
  exposed.pause();
  assert.equal(exposed.paused, true);
  exposed.seek(30);
  exposed.seekBy(-40);
  assert.equal(exposed.currentTime, 0);
  exposed.seek(500);
  assert.equal(exposed.currentTime, 90);
  assert.equal(exposed.setVolume(2), false, "volumes clamp to 1, the current value");
  assert.equal(exposed.setVolume(0.4), true);
  assert.equal(exposed.volume, 0.4);
  assert.equal(exposed.setMuted(true), true);
  assert.equal(exposed.muted, true);
  assert.equal(exposed.toggleCaptions(), false, "no caption tracks");
  assert.equal(await exposed.toggleFullscreen(), false);
  assert.equal(await exposed.togglePictureInPicture(), false, "audio cannot float");
  assert.equal(exposed.fullscreen, false);
  handle.unmount();
});

test("compound parts require a matching root provider", () => {
  for (const [part, props] of [
    [MediaPlayerPlayButton, {}],
    [MediaPlayerMuteButton, {}],
    [MediaPlayerSeekSlider, {}],
    [MediaPlayerVolumeSlider, {}],
    [MediaPlayerTimeDisplay, {}],
    [MediaPlayerPlaybackRateButton, {}],
    [MediaPlayerCaptionsButton, {}],
    [MediaPlayerLoadingIndicator, {}],
    [AudioPlayerAudio, {}],
  ] as const) {
    assert.throws(
      () => mountInteraction(part, { props }),
      /VIZE_UI_CONTEXT_MISSING: MediaPlayer requires a matching provider/,
    );
  }
});
