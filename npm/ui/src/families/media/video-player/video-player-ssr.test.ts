import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  VideoPlayerFullscreenButton,
  VideoPlayerPictureInPictureButton,
  VideoPlayerPlayButton,
  VideoPlayerRoot,
  VideoPlayerVideo,
} from "./video-player.ts";

const SsrProbe = defineComponent({
  name: "VideoPlayerSsrProbe",
  setup: () => () =>
    h(VideoPlayerRoot, { defaultMuted: true }, () => [
      h(VideoPlayerVideo, { src: "/clip.mp4", poster: "/clip.jpg", autoplay: true }, () =>
        h("track", { kind: "captions", src: "/clip.vtt", srclang: "en" }),
      ),
      h(VideoPlayerPlayButton),
      h(VideoPlayerFullscreenButton),
      h(VideoPlayerPictureInPictureButton, { unsupported: "hide" }),
    ]),
});

test("renders byte-identical video player markup with unsupported platform controls", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /<video id="vize-v-\d+-media-player-media"/);
  assert.match(left, /src="\/clip\.mp4"/);
  assert.match(left, /poster="\/clip\.jpg"/);
  assert.match(left, /playsinline/);
  assert.match(left, /autoplay/);
  assert.match(left, /muted/);
  assert.match(left, /<track kind="captions"/);
  assert.match(left, /disabled aria-label="Enter fullscreen"[^>]*data-supported="false"/);
  assert.match(left, /hidden aria-label="Enter picture-in-picture"/);
});

test("hydrates video player markup without warnings or node replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverVideo = host.querySelector("video");
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
    assert.ok(host.querySelector("video") === serverVideo);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
