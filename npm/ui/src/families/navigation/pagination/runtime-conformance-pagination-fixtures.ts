import assert from "node:assert/strict";

import { h } from "vue";

import {
  Pagination,
  PaginationEllipsis,
  PaginationItem,
  PaginationList,
  PaginationNext,
  PaginationPage,
  PaginationPrevious,
} from "./pagination.ts";
import type { PaginationSlotState } from "./pagination.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

function renderPaginationFixture() {
  return h(
    Pagination,
    { defaultValue: 5, id: "results-pages", pageCount: 10 },
    {
      default: (state: PaginationSlotState) =>
        h(PaginationList, null, {
          default: () => [
            h(PaginationItem, null, () => h(PaginationPrevious, null, () => "Previous")),
            ...state.range.map((item) =>
              item.type === "page"
                ? h(PaginationItem, { key: item.key, page: item.page }, () =>
                    h(PaginationPage, { page: item.page }, () => String(item.page)),
                  )
                : h(PaginationItem, { key: item.key }, () =>
                    h(PaginationEllipsis, { position: item.position }, () => "..."),
                  ),
            ),
            h(PaginationItem, null, () => h(PaginationNext, null, () => "Next")),
          ],
        }),
    },
  );
}

function assertPaginationServerMarkup(html: string): void {
  assert.match(html, /^<nav/);
  assert.match(html, /id="results-pages"/);
  assert.match(html, /aria-label="Pagination"/);
  assert.match(html, /data-vize-ui="pagination"/);
  assert.match(html, /data-page="5"/);
  assert.match(html, /data-page-count="10"/);
  assert.match(html, /data-vize-ui="pagination-list"/);
  assert.match(html, /data-vize-ui="pagination-item"/);
  assert.match(html, /data-vize-ui="pagination-previous"/);
  assert.match(html, /data-target-page="4"/);
  assert.match(html, /id="results-pages-page-5"/);
  assert.match(html, /aria-current="page"/);
  assert.match(html, /data-current="true"/);
  assert.match(html, /data-vize-ui="pagination-ellipsis"/);
  assert.match(html, /data-vize-ui="pagination-next"/);
  assert.match(html, /data-target-page="6"/);
  assert.doesNotMatch(html, /class=/);
  assert.doesNotMatch(html, /style=/);
}

function assertPaginationHydratedDom(host: HTMLElement): void {
  const root = host.querySelector('[data-vize-ui="pagination"]');
  const list = host.querySelector('[data-vize-ui="pagination-list"]');
  const current = host.querySelector<HTMLButtonElement>(
    '[data-vize-ui="pagination-page"][aria-current="page"]',
  );
  const previous = host.querySelector<HTMLButtonElement>('[data-vize-ui="pagination-previous"]');
  const next = host.querySelector<HTMLButtonElement>('[data-vize-ui="pagination-next"]');
  const ellipsis = host.querySelector('[data-vize-ui="pagination-ellipsis"]');

  assert.ok(root instanceof HTMLElement);
  assert.equal(root.tagName, "NAV");
  assert.equal(root.id, "results-pages");
  assert.ok(list instanceof HTMLOListElement);
  assert.ok(current instanceof HTMLButtonElement);
  assert.equal(current.id, "results-pages-page-5");
  assert.equal(current.disabled, false);
  assert.equal(current.getAttribute("aria-label"), "Page 5, current page");
  assert.ok(previous instanceof HTMLButtonElement);
  assert.equal(previous.getAttribute("data-target-page"), "4");
  assert.ok(next instanceof HTMLButtonElement);
  assert.equal(next.getAttribute("data-target-page"), "6");
  assert.ok(ellipsis instanceof HTMLElement);
  assert.equal(ellipsis.getAttribute("aria-label"), "More pages");
}

export const paginationRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "pagination",
    sourceFile: "families/navigation/pagination/pagination.vue",
    render: renderPaginationFixture,
    assertServerMarkup: assertPaginationServerMarkup,
    assertHydratedDom: assertPaginationHydratedDom,
  },
  {
    name: "pagination-ellipsis",
    sourceFile: "families/navigation/pagination/pagination-ellipsis.vue",
    render: renderPaginationFixture,
    assertServerMarkup: assertPaginationServerMarkup,
    assertHydratedDom: assertPaginationHydratedDom,
  },
  {
    name: "pagination-item",
    sourceFile: "families/navigation/pagination/pagination-item.vue",
    render: renderPaginationFixture,
    assertServerMarkup: assertPaginationServerMarkup,
    assertHydratedDom: assertPaginationHydratedDom,
  },
  {
    name: "pagination-list",
    sourceFile: "families/navigation/pagination/pagination-list.vue",
    render: renderPaginationFixture,
    assertServerMarkup: assertPaginationServerMarkup,
    assertHydratedDom: assertPaginationHydratedDom,
  },
  {
    name: "pagination-next",
    sourceFile: "families/navigation/pagination/pagination-next.vue",
    render: renderPaginationFixture,
    assertServerMarkup: assertPaginationServerMarkup,
    assertHydratedDom: assertPaginationHydratedDom,
  },
  {
    name: "pagination-page",
    sourceFile: "families/navigation/pagination/pagination-page.vue",
    render: renderPaginationFixture,
    assertServerMarkup: assertPaginationServerMarkup,
    assertHydratedDom: assertPaginationHydratedDom,
  },
  {
    name: "pagination-previous",
    sourceFile: "families/navigation/pagination/pagination-previous.vue",
    render: renderPaginationFixture,
    assertServerMarkup: assertPaginationServerMarkup,
    assertHydratedDom: assertPaginationHydratedDom,
  },
];
