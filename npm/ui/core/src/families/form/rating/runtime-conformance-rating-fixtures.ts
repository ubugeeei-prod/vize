import assert from "node:assert/strict";

import { h } from "vue";

import Rating from "./rating.vue";
import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";

export const ratingRuntimeFixture: RuntimeFixture = {
  name: "rating",
  sourceFile: "families/form/rating/rating.vue",
  render: () =>
    h(
      Rating,
      {
        ariaLabel: "Movie score",
        defaultValue: 4,
        id: "movie-rating",
        name: "score",
        required: true,
      },
      {
        item: ({ value }: { value: number }) => String(value),
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<span/);
    assert.match(html, /id="movie-rating"/);
    assert.match(html, /role="radiogroup"/);
    assert.match(html, /aria-label="Movie score"/);
    assert.match(html, /aria-required="true"/);
    assert.match(html, /data-vize-ui="rating"/);
    assert.match(html, /data-state="selected"/);
    assert.match(html, /data-value="4"/);
    assert.match(html, /type="radio"/);
    assert.match(html, /name="score"/);
    assert.match(html, /value="4"/);
    assert.match(html, /checked/);
  },
  assertHydratedDom(host) {
    const rating = host.querySelector('[data-vize-ui="rating"]');
    const checked = host.querySelector<HTMLInputElement>(
      '[data-vize-ui="rating-control"][value="4"]',
    );
    assert.ok(rating instanceof HTMLSpanElement);
    assert.ok(checked instanceof HTMLInputElement);
    assert.equal(rating.getAttribute("role"), "radiogroup");
    assert.equal(rating.getAttribute("data-value"), "4");
    assert.equal(checked.checked, true);
    assert.equal(checked.name, "score");
    assert.equal(checked.required, true);
  },
};
