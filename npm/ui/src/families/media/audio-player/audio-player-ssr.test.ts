import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import { AudioPlayerAudio, AudioPlayerPlayButton, AudioPlayerRoot } from "./audio-player.ts";

const SsrProbe = defineComponent({
  name: "AudioPlayerSsrProbe",
  setup: () => () =>
    h(AudioPlayerRoot, { id: "song" }, () => [
      h(AudioPlayerAudio, { src: "/song.ogg", preload: "auto" }),
      h(AudioPlayerPlayButton),
    ]),
});

test("renders byte-identical audio player markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /<audio id="song-media" src="\/song\.ogg" preload="auto"/);
  assert.match(left, /data-vize-ui="audio-player-audio"/);
  assert.match(left, /aria-controls="song-media"|aria-label="Play"/);
});

test("hydrates audio player markup without warnings or node replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverAudio = host.querySelector("audio");
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
    assert.ok(host.querySelector("audio") === serverAudio);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
