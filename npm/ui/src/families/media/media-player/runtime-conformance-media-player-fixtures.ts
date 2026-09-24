import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import AudioPlayerAudio from "../audio-player/audio-player-audio.vue";
import {
  MediaPlayerCaptionsButton,
  MediaPlayerLoadingIndicator,
  MediaPlayerMuteButton,
  MediaPlayerPlayButton,
  MediaPlayerPlaybackRateButton,
  MediaPlayerRoot,
  MediaPlayerSeekSlider,
  MediaPlayerTimeDisplay,
  MediaPlayerVolumeSlider,
} from "./media-player.ts";

function player(id: string) {
  return h(MediaPlayerRoot, { id, ariaLabel: "Briefing" }, () => [
    h(AudioPlayerAudio, { src: "/briefing.mp3" }),
    h(MediaPlayerPlayButton),
    h(MediaPlayerMuteButton),
    h(MediaPlayerSeekSlider),
    h(MediaPlayerVolumeSlider),
    h(MediaPlayerTimeDisplay, { mode: "remaining" }),
    h(MediaPlayerPlaybackRateButton),
    h(MediaPlayerCaptionsButton),
    h(MediaPlayerLoadingIndicator),
  ]);
}

function part(host: HTMLElement, name: string): Element {
  const element = host.querySelector(`[data-vize-ui="${name}"]`);
  assert.ok(element, `${name} must render`);
  return element;
}

function fixture(
  name: string,
  file: string,
  serverPattern: RegExp,
  check: (host: HTMLElement) => void,
): RuntimeFixture {
  return {
    name,
    sourceFile: `families/media/media-player/${file}.vue`,
    render: () => player(`${name}-fixture`),
    assertServerMarkup(html) {
      assert.match(html, serverPattern);
    },
    assertHydratedDom: check,
  };
}

export const mediaPlayerRuntimeFixtures: readonly RuntimeFixture[] = [
  fixture("media-player", "media-player-root", /role="group" aria-label="Briefing"/, (host) => {
    assert.equal(part(host, "media-player-root").getAttribute("data-state"), "paused");
  }),
  fixture("media-player-play-button", "media-player-play-button", /aria-label="Play"/, (host) => {
    assert.ok(part(host, "media-player-play-button") instanceof HTMLButtonElement);
  }),
  fixture("media-player-mute-button", "media-player-mute-button", /aria-label="Mute"/, (host) => {
    assert.ok(part(host, "media-player-mute-button") instanceof HTMLButtonElement);
  }),
  fixture(
    "media-player-seek-slider",
    "media-player-seek-slider",
    /role="slider" tabindex="-1"[^>]*aria-label="Seek"/,
    (host) => {
      assert.equal(part(host, "media-player-seek-thumb").getAttribute("role"), "slider");
    },
  ),
  fixture(
    "media-player-volume-slider",
    "media-player-volume-slider",
    /aria-valuenow="100" aria-valuetext="100%"/,
    (host) => {
      assert.equal(part(host, "media-player-volume-thumb").getAttribute("aria-valuemax"), "100");
    },
  ),
  fixture(
    "media-player-time-display",
    "media-player-time-display",
    /data-mode="remaining"[^>]*><!--\[-->0:00</,
    (host) => {
      assert.equal(part(host, "media-player-time-display").textContent, "0:00");
    },
  ),
  fixture(
    "media-player-playback-rate-button",
    "media-player-playback-rate-button",
    /aria-label="Playback speed 1×"/,
    (host) => {
      assert.equal(part(host, "media-player-playback-rate-button").getAttribute("data-rate"), "1");
    },
  ),
  fixture(
    "media-player-captions-button",
    "media-player-captions-button",
    /disabled aria-label="Show captions"/,
    (host) => {
      const button = part(host, "media-player-captions-button");
      assert.ok(button instanceof HTMLButtonElement);
      assert.equal(button.disabled, true);
    },
  ),
  fixture(
    "media-player-loading-indicator",
    "media-player-loading-indicator",
    /role="status" data-vize-ui="media-player-loading-indicator"/,
    (host) => {
      assert.equal(part(host, "media-player-loading-indicator").textContent, "");
    },
  ),
];
