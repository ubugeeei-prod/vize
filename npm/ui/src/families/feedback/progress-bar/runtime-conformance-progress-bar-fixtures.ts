import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
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

function finiteRangeFixture(
  name: string,
  min: number,
  max: number,
  value: number,
  percent: number,
  complete: boolean,
  invalid: boolean,
): RuntimeFixture {
  const state = complete ? "complete" : "loading";
  return {
    name,
    sourceFile: "families/feedback/progress-bar/progress-bar.vue",
    render: () => h(ProgressBar, { id: name, ariaLabel: "Finite progress", min, max, value }),
    assertServerMarkup(html) {
      assert.match(html, /^<div/);
      assert.ok(html.includes(`id="${name}"`));
      assert.ok(html.includes('role="progressbar"'));
      assert.ok(html.includes(`aria-valuemin="${min}"`));
      assert.ok(html.includes(`aria-valuemax="${max}"`));
      assert.ok(html.includes(`aria-valuenow="${value}"`));
      assert.ok(html.includes(`data-percent="${percent}"`));
      assert.ok(html.includes(`data-state="${state}"`));
      assert.ok(html.includes(`data-complete="${complete}"`));
      assert.ok(html.includes(`--vize-ui-progress-bar-percent:${percent}%`));
      assert.ok(html.includes(`--vize-ui-progress-bar-ratio:${percent / 100}`));
      assert.equal(html.includes('data-invalid="true"'), invalid);
      assert.doesNotMatch(html, /NaN|Infinity/);
    },
    assertHydratedDom(host) {
      const progress = host.querySelector('[data-vize-ui="progress-bar"]');
      assert.ok(progress instanceof HTMLElement);
      assert.equal(progress.id, name);
      assert.equal(progress.getAttribute("aria-valuemin"), String(min));
      assert.equal(progress.getAttribute("aria-valuemax"), String(max));
      assert.equal(progress.getAttribute("aria-valuenow"), String(value));
      assert.equal(progress.getAttribute("data-percent"), String(percent));
      assert.equal(progress.getAttribute("data-state"), state);
      assert.equal(progress.getAttribute("data-complete"), String(complete));
      assert.equal(progress.getAttribute("data-invalid"), invalid ? "true" : null);
      assert.equal(
        progress.style.getPropertyValue("--vize-ui-progress-bar-percent"),
        `${percent}%`,
      );
      assert.equal(
        progress.style.getPropertyValue("--vize-ui-progress-bar-ratio"),
        String(percent / 100),
      );
      assert.ok(progress.querySelector('[data-vize-ui="progress-bar-track"]'));
      assert.ok(progress.querySelector('[data-vize-ui="progress-bar-indicator"]'));
    },
  };
}

export const progressBarRuntimeFixtures: readonly RuntimeFixture[] = [
  progressBarRuntimeFixture,
  finiteRangeFixture(
    "progress-bar-wide-range",
    -Number.MAX_VALUE,
    Number.MAX_VALUE,
    0,
    50,
    false,
    false,
  ),
  finiteRangeFixture(
    "progress-bar-wide-endpoint",
    -Number.MAX_VALUE,
    Number.MAX_VALUE,
    Number.MAX_VALUE,
    100,
    true,
    false,
  ),
  finiteRangeFixture(
    "progress-bar-collapsed-positive",
    Number.MAX_VALUE,
    Number.MAX_VALUE,
    Number.MAX_VALUE,
    0,
    true,
    true,
  ),
  finiteRangeFixture(
    "progress-bar-collapsed-negative",
    -Number.MAX_VALUE,
    -Number.MAX_VALUE,
    -Number.MAX_VALUE,
    0,
    true,
    true,
  ),
];
