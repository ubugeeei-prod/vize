import assert from "node:assert/strict";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { renderAutocompleteTree } from "./autocomplete-fixture-tree.ts";

export const autocompleteRuntimeFixtures: readonly RuntimeFixture[] = ["autocomplete-root.vue"].map(
  (file) => ({
    name: file.replace(".vue", ""),
    sourceFile: `families/selection/autocomplete/${file}`,
    render: renderAutocompleteTree,
    assertServerMarkup: (html: string) => {
      assert.match(html, /^<div id="site-search"/);
      assert.match(html, /data-vize-ui-preset="autocomplete"/);
    },
    assertHydratedDom: (host: HTMLElement) => {
      const root = host.querySelector('[data-vize-ui-preset="autocomplete"]');
      assert.ok(root instanceof HTMLDivElement);
      assert.equal(root.getAttribute("data-history-count"), "2");
    },
  }),
);
