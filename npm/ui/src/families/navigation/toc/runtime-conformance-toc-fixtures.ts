import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { TocItem, TocLink, TocList, TocRoot } from "./toc.ts";
import type { TocLinkSlotState } from "./toc-types.ts";

const label = ({ active, targetId }: TocLinkSlotState) => `${targetId}:${active}`;

function renderToc() {
  return h(
    TocRoot,
    { ariaLabel: "On this page", defaultActiveId: "setup" },
    {
      default: () =>
        h(TocList, null, {
          default: () =>
            ["overview", "setup"].map((id) =>
              h(
                TocItem,
                { key: id, targetId: id },
                { default: () => h(TocLink, { targetId: id }, { default: label }) },
              ),
            ),
        }),
    },
  );
}

function fixture(name: string, file: string, marker: string): RuntimeFixture {
  return {
    name,
    sourceFile: `families/navigation/toc/${file}`,
    render: renderToc,
    assertServerMarkup(html) {
      assert.match(html, new RegExp(`data-vize-ui="${marker}"`));
      assert.match(html, /aria-label="On this page"/);
      assert.match(html, /aria-current="location"/);
      assert.match(html, /setup:true/);
      assert.match(html, /overview:false/);
    },
    assertHydratedDom(host) {
      const current = host.querySelector('[aria-current="location"]');
      assert.ok(current instanceof HTMLAnchorElement);
      assert.equal(current.getAttribute("href"), "#setup");
      assert.ok(host.querySelector(`[data-vize-ui="${marker}"]`) instanceof HTMLElement);
    },
  };
}

export const tocRuntimeFixtures: readonly RuntimeFixture[] = [
  fixture("toc", "toc-root.vue", "toc"),
  fixture("toc-list", "toc-list.vue", "toc-list"),
  fixture("toc-item", "toc-item.vue", "toc-item"),
  fixture("toc-link", "toc-link.vue", "toc-link"),
];
