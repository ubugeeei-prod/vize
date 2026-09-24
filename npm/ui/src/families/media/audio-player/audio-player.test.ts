import assert from "node:assert/strict";

import { afterEach, beforeEach, test } from "vite-plus/test";
import { h, nextTick, ref } from "vue";

import type { MediaPlayerRootExpose, MediaPlayerSlotState } from "../media-player/media-player.ts";
import { installFakeMedia } from "../media-player/media-player-test-utils.ts";
import type { FakeMedia } from "../media-player/media-player-test-utils.ts";
import {
  AudioPlayer,
  AudioPlayerAudio,
  AudioPlayerPlayButton,
  AudioPlayerRoot,
  AudioPlayerSeekSlider,
  AudioPlayerTimeDisplay,
} from "./audio-player.ts";
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

function mountAudio(audioProps: Record<string, unknown> = {}) {
  return mountInteraction(AudioPlayerRoot, {
    props: { id: "song", ariaLabelledby: "song-title" },
    slots: {
      default: (state: MediaPlayerSlotState) => [
        h("h2", { id: "song-title" }, "Nocturne"),
        h(AudioPlayerAudio, { src: "/nocturne.ogg", ...audioProps }, () =>
          h("source", { src: "/nocturne.mp3", type: "audio/mpeg" }),
        ),
        h(AudioPlayerPlayButton, { ariaLabel: null }, () => (state.paused ? "Play" : "Pause")),
        h(AudioPlayerSeekSlider),
        h(AudioPlayerTimeDisplay, { mode: "duration" }),
      ],
    },
  });
}

test("renders a native audio element registered as audio media", async () => {
  const handle = mountAudio({ preload: "none", loop: true, controls: true });
  const root = handle.root();
  await settle();
  const audio = root.querySelector("audio");
  assert.ok(audio);
  assert.equal(root.getAttribute("role"), "group");
  assert.equal(root.getAttribute("aria-labelledby"), "song-title");
  assert.equal(root.getAttribute("data-media-kind"), "audio");
  assert.equal(audio.id, "song-media");
  assert.equal(audio.getAttribute("data-vize-ui"), "audio-player-audio");
  assert.equal(audio.getAttribute("src"), "/nocturne.ogg");
  assert.equal(audio.getAttribute("preload"), "none");
  assert.equal(audio.hasAttribute("loop"), true);
  assert.equal(audio.hasAttribute("controls"), true);
  assert.equal(audio.querySelector("source")?.getAttribute("type"), "audio/mpeg");
  assert.ok(handle.exposes<MediaPlayerRootExpose>().media === audio);
  handle.unmount();
});

test("slot-labelled controls drive audio playback and time display", async () => {
  const handle = mountAudio();
  const root = handle.root();
  await settle();
  const audio = root.querySelector("audio");
  assert.ok(audio);
  fake.setDuration(audio, 245);
  await settle();
  assert.equal(root.querySelector('[data-mode="duration"]')?.textContent, "4:05");
  await handle.click(handle.getByRole("button", { name: "Play" }));
  await settle();
  assert.ok(handle.getByRole("button", { name: "Pause" }));
  assert.equal(audio.paused, false);
  handle.unmount();
});

test("unsafe audio sources are dropped and marked", async () => {
  const handle = mountAudio({ src: "vbscript:msgbox" });
  await settle();
  const audio = handle.root().querySelector("audio");
  assert.equal(audio?.hasAttribute("src"), false);
  assert.equal(audio?.getAttribute("data-invalid-src"), "true");
  handle.unmount();
});

test("unmounting the audio part unregisters it from the root", async () => {
  const present = ref(true);
  const handle = mountInteraction(AudioPlayer, {
    props: { id: "toggle" },
    slots: { default: () => (present.value ? [h(AudioPlayerAudio, { src: "/a.mp3" })] : []) },
  });
  await settle();
  const exposed = handle.exposes<MediaPlayerRootExpose>();
  assert.equal(handle.root().getAttribute("data-media-kind"), "audio");
  present.value = false;
  await settle();
  assert.equal(handle.root().hasAttribute("data-media-kind"), false);
  assert.equal(exposed.media, null);
  assert.equal(AudioPlayer, AudioPlayerRoot);
  handle.unmount();
});
