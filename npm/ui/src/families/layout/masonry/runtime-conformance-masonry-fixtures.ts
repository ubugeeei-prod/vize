import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { Masonry } from "./masonry.ts";
import type { MasonryItemSlotState } from "./masonry-types.ts";

interface Photo {
  readonly title: string;
  readonly height: number;
}

// Instantiation expressions pin the generic SFC so `h()` checks props against real item types.
const PhotoMasonry = Masonry<Photo>;

const photos: readonly Photo[] = ["Dunes", "Harbor", "Forest", "Glacier"].map((title, index) => ({
  title,
  height: 100 + index * 40,
}));

export const masonryRuntimeFixture: RuntimeFixture = {
  name: "masonry",
  sourceFile: "families/layout/masonry/masonry.vue",
  render: () =>
    h(
      PhotoMasonry,
      {
        items: photos,
        columns: 2,
        gap: 12,
        estimateHeight: (photo: Photo) => photo.height,
        getKey: (photo: Photo) => photo.title,
      },
      { item: ({ item }: MasonryItemSlotState<Photo>) => h("figure", h("figcaption", item.title)) },
    ),
  assertServerMarkup(html) {
    assert.match(html, /data-vize-ui="masonry"/);
    assert.match(html, /data-columns="2"/);
    assert.equal(html.match(/data-part="column"/g)?.length, 2);
    assert.match(html, /<figcaption>Dunes<\/figcaption>/);
  },
  assertHydratedDom(host) {
    const root = host.querySelector('[data-vize-ui="masonry"]');
    assert.ok(root instanceof HTMLElement);
    assert.equal(root.getAttribute("role"), null);
    assert.equal(root.querySelectorAll("figure").length, 4);
  },
};
