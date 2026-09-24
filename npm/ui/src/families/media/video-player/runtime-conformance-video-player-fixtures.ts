import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  VideoPlayerFullscreenButton,
  VideoPlayerPictureInPictureButton,
  VideoPlayerRoot,
  VideoPlayerVideo,
} from "./video-player.ts";

function player(id: string) {
  return h(VideoPlayerRoot, { id }, () => [
    h(VideoPlayerVideo, { src: "/intro.mp4", poster: "/intro.jpg" }),
    h(VideoPlayerFullscreenButton),
    h(VideoPlayerPictureInPictureButton),
  ]);
}

function part(host: HTMLElement, name: string): Element {
  const element = host.querySelector(`[data-vize-ui="${name}"]`);
  assert.ok(element, `${name} must render`);
  return element;
}

export const videoPlayerRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "video-player",
    sourceFile: "families/media/video-player/video-player-video.vue",
    render: () => player("intro"),
    assertServerMarkup(html) {
      assert.match(html, /<video id="intro-media" src="\/intro\.mp4" poster="\/intro\.jpg"/);
      assert.match(html, /playsinline/);
    },
    assertHydratedDom(host) {
      const video = part(host, "video-player-video");
      assert.ok(video instanceof HTMLVideoElement);
      assert.equal(video.getAttribute("src"), "/intro.mp4");
    },
  },
  {
    name: "video-player-fullscreen-button",
    sourceFile: "families/media/video-player/video-player-fullscreen-button.vue",
    render: () => player("intro-fullscreen"),
    assertServerMarkup(html) {
      assert.match(html, /disabled aria-label="Enter fullscreen"/);
    },
    assertHydratedDom(host) {
      assert.ok(part(host, "video-player-fullscreen-button") instanceof HTMLButtonElement);
    },
  },
  {
    name: "video-player-picture-in-picture-button",
    sourceFile: "families/media/video-player/video-player-picture-in-picture-button.vue",
    render: () => player("intro-pip"),
    assertServerMarkup(html) {
      assert.match(html, /disabled aria-label="Enter picture-in-picture"/);
    },
    assertHydratedDom(host) {
      const button = part(host, "video-player-picture-in-picture-button");
      assert.equal(button.getAttribute("data-supported"), "false");
    },
  },
];
