import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import AudioPlayerAudio from "../audio-player/audio-player-audio.vue";
import MediaPlayerCaptionsButton from "./media-player-captions-button.vue";
import MediaPlayerLoadingIndicator from "./media-player-loading-indicator.vue";
import MediaPlayerMuteButton from "./media-player-mute-button.vue";
import MediaPlayerPlayButton from "./media-player-play-button.vue";
import MediaPlayerPlaybackRateButton from "./media-player-playback-rate-button.vue";
import MediaPlayerRoot from "./media-player-root.vue";
import MediaPlayerSeekSlider from "./media-player-seek-slider.vue";
import MediaPlayerTimeDisplay from "./media-player-time-display.vue";
import MediaPlayerVolumeSlider from "./media-player-volume-slider.vue";

const SsrProbe = defineComponent({
  name: "MediaPlayerSsrProbe",
  setup: () => () =>
    h(MediaPlayerRoot, { ariaLabel: "Podcast", defaultMuted: true, defaultVolume: 0.4 }, () => [
      h(AudioPlayerAudio, { src: "/episode.mp3" }),
      h(MediaPlayerPlayButton),
      h(MediaPlayerMuteButton),
      h(MediaPlayerSeekSlider),
      h(MediaPlayerVolumeSlider),
      h(MediaPlayerTimeDisplay),
      h(MediaPlayerPlaybackRateButton),
      h(MediaPlayerCaptionsButton),
      h(MediaPlayerLoadingIndicator),
    ]),
});

test("renders byte-identical media player markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /^<div id="vize-v-\d+-media-player" role="group" aria-label="Podcast"/);
  assert.match(left, /data-state="paused"/);
  assert.match(left, /<audio id="vize-v-\d+-media-player-media"/);
  assert.match(left, /muted/);
  assert.match(left, /aria-label="Play"/);
  assert.match(left, /aria-label="Unmute"/);
  assert.match(left, /role="slider"[^>]*tabindex="-1"[^>]*aria-label="Seek"/);
  assert.match(left, /aria-valuenow="40" aria-valuetext="Muted"/);
  assert.match(left, /0:00/);
  assert.match(left, /role="status"/);
});

test("hydrates media player markup without warnings or node replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverMedia = host.querySelector("audio");
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);
  let mounted = false;
  try {
    app.mount(host);
    mounted = true;
    assert.ok(host.firstElementChild === serverRoot);
    assert.ok(host.querySelector("audio") === serverMedia);
    await nextTick();
    assert.equal(host.firstElementChild?.getAttribute("data-media-kind"), "audio");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
