import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { AudioVisualizer, AudioVisualizerBars } from "./audio-visualizer.ts";

export const audioVisualizerRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "audio-visualizer",
    sourceFile: "families/media/audio-visualizer/audio-visualizer.vue",
    render: () => h(AudioVisualizer, { fftSize: 32 }),
    assertServerMarkup(html) {
      assert.match(html, /aria-hidden="true" data-vize-ui="audio-visualizer-root"/);
      assert.match(html, /data-state="idle"/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="audio-visualizer-root"]');
      assert.ok(root instanceof HTMLDivElement);
      assert.equal(root.getAttribute("aria-hidden"), "true");
    },
  },
  {
    name: "audio-visualizer-bars",
    sourceFile: "families/media/audio-visualizer/audio-visualizer-bars.vue",
    render: () =>
      h(AudioVisualizer, { ariaLabel: "Level", fftSize: 32 }, () =>
        h(AudioVisualizerBars, { count: 2, scale: "linear" }),
      ),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="audio-visualizer-bars" part="bars" data-count="2"/);
      assert.equal(html.match(/data-vize-ui="audio-visualizer-bar"/g)?.length, 2);
    },
    assertHydratedDom(host) {
      assert.equal(host.querySelectorAll('[data-vize-ui="audio-visualizer-bar"]').length, 2);
    },
  },
];
