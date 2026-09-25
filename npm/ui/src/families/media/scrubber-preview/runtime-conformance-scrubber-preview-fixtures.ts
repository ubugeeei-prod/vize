import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  ScrubberPreviewRoot,
  ScrubberPreviewThumbnail,
  ScrubberPreviewTime,
  ScrubberPreviewTrack,
} from "./scrubber-preview.ts";

const sprite = {
  src: "/thumbs/sheet.jpg",
  columns: 4,
  rows: 4,
  interval: 10,
  width: 160,
  height: 90,
};

function preview(time: number | null) {
  return h(ScrubberPreviewRoot, { duration: 600, sprite, time }, () => [
    h(ScrubberPreviewTrack, null, () => h("div", { "data-seek-bar": "" })),
    h(ScrubberPreviewThumbnail),
    h(ScrubberPreviewTime),
  ]);
}

export const scrubberPreviewRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "scrubber-preview",
    sourceFile: "families/media/scrubber-preview/scrubber-preview-root.vue",
    render: () => preview(null),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="scrubber-preview-root"[^>]*data-state="idle"/);
      assert.match(html, /data-kind="sprite"/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="scrubber-preview-root"]');
      assert.ok(root instanceof HTMLDivElement);
      assert.equal(root.getAttribute("data-state"), "idle");
    },
  },
  {
    name: "scrubber-preview-track",
    sourceFile: "families/media/scrubber-preview/scrubber-preview-track.vue",
    render: () => preview(null),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="scrubber-preview-track" part="track"/);
    },
    assertHydratedDom(host) {
      assert.ok(host.querySelector('[data-vize-ui="scrubber-preview-track"] [data-seek-bar]'));
    },
  },
  {
    name: "scrubber-preview-thumbnail",
    sourceFile: "families/media/scrubber-preview/scrubber-preview-thumbnail.vue",
    render: () => preview(125),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="scrubber-preview-thumbnail"[^>]*data-status="ready"/);
      assert.match(html, /--vize-ui-scrubber-preview-x:0px/);
      assert.match(html, /--vize-ui-scrubber-preview-y:-270px/);
    },
    assertHydratedDom(host) {
      const thumbnail = host.querySelector('[data-vize-ui="scrubber-preview-thumbnail"]');
      assert.ok(thumbnail instanceof HTMLDivElement);
      assert.equal(thumbnail.hidden, false);
    },
  },
  {
    name: "scrubber-preview-time",
    sourceFile: "families/media/scrubber-preview/scrubber-preview-time.vue",
    render: () => preview(125),
    assertServerMarkup(html) {
      assert.match(
        html,
        /data-vize-ui="scrubber-preview-time"[^>]*>(?:<!--\[-->)?2:05(?:<!--\]-->)?<\/span>/,
      );
    },
    assertHydratedDom(host) {
      assert.equal(
        host.querySelector('[data-vize-ui="scrubber-preview-time"]')?.textContent,
        "2:05",
      );
    },
  },
];
