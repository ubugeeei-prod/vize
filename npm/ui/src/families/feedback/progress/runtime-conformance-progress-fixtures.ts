import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import Progress from "./progress.vue";

export const progressRuntimeFixture: RuntimeFixture = {
  name: "progress",
  sourceFile: "families/feedback/progress/progress.vue",
  render: () =>
    h(
      Progress,
      {
        ariaLabel: "Upload progress",
        id: "upload-progress",
        max: 100,
        value: 40,
      },
      {
        default: () => "40%",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<progress/);
    assert.match(html, /id="upload-progress"/);
    assert.match(html, /value="40"/);
    assert.match(html, /max="100"/);
    assert.match(html, /aria-label="Upload progress"/);
    assert.match(html, /data-vize-ui="progress"/);
    assert.match(html, /data-state="loading"/);
    assert.match(html, /40%/);
  },
  assertHydratedDom(host) {
    const progress = host.querySelector('[data-vize-ui="progress"]');
    assert.ok(progress instanceof HTMLProgressElement);
    assert.equal(progress.id, "upload-progress");
    assert.equal(progress.value, 40);
    assert.equal(progress.max, 100);
    assert.equal(progress.getAttribute("data-state"), "loading");
    assert.equal(progress.textContent, "40%");
  },
};
