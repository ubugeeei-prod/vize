import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";
import ProgressBar from "./progress-bar.vue";

export const progressBarRuntimeFixture: RuntimeFixture = {
  name: "progress-bar",
  sourceFile: "families/feedback/progress-bar/progress-bar.vue",
  render: () =>
    h(
      ProgressBar,
      {
        ariaLabel: "Upload progress",
        id: "upload-progress",
        max: 100,
        min: 20,
        value: 40,
        valueLabel: "25%",
      },
      {
        indicator: () => "25%",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<div/);
    assert.match(html, /id="upload-progress"/);
    assert.match(html, /role="progressbar"/);
    assert.match(html, /aria-label="Upload progress"/);
    assert.match(html, /aria-valuemin="20"/);
    assert.match(html, /aria-valuemax="100"/);
    assert.match(html, /aria-valuenow="40"/);
    assert.match(html, /aria-valuetext="25%"/);
    assert.match(html, /data-vize-ui="progress-bar"/);
    assert.match(html, /data-state="loading"/);
    assert.match(html, /data-percent="25"/);
    assert.match(html, /data-vize-ui="progress-bar-track"/);
    assert.match(html, /data-vize-ui="progress-bar-indicator"/);
    assert.match(html, /25%/);
  },
  assertHydratedDom(host) {
    const progress = host.querySelector('[data-vize-ui="progress-bar"]');
    assert.ok(progress instanceof HTMLElement);
    assert.equal(progress.id, "upload-progress");
    assert.equal(progress.getAttribute("role"), "progressbar");
    assert.equal(progress.getAttribute("aria-valuenow"), "40");
    assert.equal(progress.getAttribute("data-state"), "loading");
    assert.equal(progress.getAttribute("data-percent"), "25");
    assert.ok(progress.querySelector('[data-vize-ui="progress-bar-track"]'));
    assert.ok(progress.querySelector('[data-vize-ui="progress-bar-indicator"]'));
    assert.equal(progress.textContent, "25%25%");
  },
};
