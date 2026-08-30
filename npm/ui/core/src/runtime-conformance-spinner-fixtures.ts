import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";
import Spinner from "./spinner.vue";

export const spinnerRuntimeFixture: RuntimeFixture = {
  name: "spinner",
  sourceFile: "spinner.vue",
  render: () =>
    h(
      Spinner,
      {
        ariaLabel: "Syncing profile",
        id: "profile-spinner",
      },
      {
        default: () => "Syncing",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<span/);
    assert.match(html, /id="profile-spinner"/);
    assert.match(html, /role="status"/);
    assert.match(html, /aria-label="Syncing profile"/);
    assert.match(html, /aria-live="polite"/);
    assert.match(html, /aria-atomic="true"/);
    assert.match(html, /data-vize-ui="spinner"/);
    assert.match(html, /data-state="loading"/);
    assert.match(html, /data-progress-state="none"/);
    assert.match(html, /Syncing/);
    assert.doesNotMatch(html, /class=/);
    assert.doesNotMatch(html, /style=/);
    assert.doesNotMatch(html, /tabindex=/);
  },
  assertHydratedDom(host) {
    const spinner = host.querySelector('[data-vize-ui="spinner"]');

    assert.ok(spinner instanceof HTMLElement);
    assert.equal(spinner.tagName, "SPAN");
    assert.equal(spinner.id, "profile-spinner");
    assert.equal(spinner.getAttribute("role"), "status");
    assert.equal(spinner.getAttribute("aria-live"), "polite");
    assert.equal(spinner.getAttribute("data-state"), "loading");
    assert.equal(spinner.getAttribute("data-progress-state"), "none");
    assert.equal(spinner.textContent, "Syncing");
  },
};
