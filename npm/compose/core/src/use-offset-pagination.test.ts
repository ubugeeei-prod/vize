import assert from "node:assert/strict";
import { test } from "node:test";
import { nextTick, shallowRef } from "vue";

import { type OffsetPaginationInfo, useOffsetPagination } from "./use-offset-pagination.ts";

void test("navigates within the page count", () => {
  const pager = useOffsetPagination({ total: 45, pageSize: 10 });
  assert.equal(pager.pageCount.value, 5);
  assert.equal(pager.isFirstPage.value, true);
  pager.prev();
  assert.equal(pager.currentPage.value, 1);
  pager.next();
  assert.equal(pager.currentPage.value, 2);
  assert.equal(pager.offset.value, 10);
  pager.currentPage.value = 99;
  assert.equal(pager.currentPage.value, 5);
  assert.equal(pager.isLastPage.value, true);
  pager.next();
  assert.equal(pager.currentPage.value, 5);
});

void test("clamps when total shrinks or the page size grows, and binds refs two-way", () => {
  const total = shallowRef(100);
  const page = shallowRef(10);
  const pageSize = shallowRef(10);
  const pager = useOffsetPagination({ total, page, pageSize });
  total.value = 30;
  assert.equal(pager.currentPage.value, 3);
  pager.currentPageSize.value = 50;
  assert.equal(pageSize.value, 50);
  assert.equal(pager.pageCount.value, 1);
  assert.equal(pager.currentPage.value, 1);
  pager.currentPageSize.value = 0;
  assert.equal(pager.currentPageSize.value, 1);
  pager.currentPage.value = 2;
  assert.equal(page.value, 2);
  total.value = 0;
  assert.equal(pager.pageCount.value, 1);
});

void test("reports changes through callbacks", async () => {
  const calls: [string, OffsetPaginationInfo][] = [];
  const total = shallowRef(50);
  const pager = useOffsetPagination({
    total,
    onPageChange: (info) => calls.push(["page", info]),
    onPageSizeChange: (info) => calls.push(["size", info]),
    onPageCountChange: (info) => calls.push(["count", info]),
  });
  pager.next();
  await nextTick();
  pager.currentPageSize.value = 25;
  await nextTick();
  assert.deepEqual(calls.map(([kind]) => kind).sort(), ["count", "page", "size"]);
  assert.deepEqual(calls.at(-1)?.[1], { currentPage: 2, currentPageSize: 25, pageCount: 2 });
});

void test("an unknown total never ends", () => {
  const pager = useOffsetPagination();
  pager.currentPage.value = 1_000;
  assert.equal(pager.currentPage.value, 1_000);
  assert.equal(pager.isLastPage.value, false);
});
