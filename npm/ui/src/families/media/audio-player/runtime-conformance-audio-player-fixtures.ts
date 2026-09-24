import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { AudioPlayerAudio, AudioPlayerPlayButton, AudioPlayerRoot } from "./audio-player.ts";

export const audioPlayerRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "audio-player",
    sourceFile: "families/media/audio-player/audio-player-audio.vue",
    render: () =>
      h(AudioPlayerRoot, { id: "theme" }, () => [
        h(AudioPlayerAudio, { src: "/theme.ogg", loop: true }),
        h(AudioPlayerPlayButton),
      ]),
    assertServerMarkup(html) {
      assert.match(html, /<audio id="theme-media" src="\/theme\.ogg" preload="metadata" loop/);
      assert.match(html, /data-vize-ui="audio-player-audio"/);
    },
    assertHydratedDom(host) {
      const audio = host.querySelector('[data-vize-ui="audio-player-audio"]');
      assert.ok(audio instanceof HTMLAudioElement);
      assert.equal(audio.loop, true);
    },
  },
];
