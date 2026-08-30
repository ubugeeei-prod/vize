import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h } from "vue";

import PaginationRoot from "./pagination.vue";
import PaginationItem from "./pagination-item.vue";
import PaginationList from "./pagination-list.vue";
import PaginationPage from "./pagination-page.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("out-of-range page controls stay disabled without duplicating valid page ids", () => {
  const handle = mountInteraction(PaginationRoot, {
    props: { defaultValue: 3, id: "edge-pages", pageCount: 3 },
    slots: {
      default: () =>
        h(PaginationList, null, () => [
          h(PaginationItem, { page: 3 }, () => h(PaginationPage, { page: 3 }, () => "3")),
          h(PaginationItem, { page: 8 }, () => h(PaginationPage, { page: 8 }, () => "8")),
          h(PaginationItem, { page: 0 }, () => h(PaginationPage, { page: 0 }, () => "0")),
        ]),
    },
  });
  const valid = handle.getByRole("button", { name: "Page 3, current page" });
  const overflow = handle.getByRole("button", { name: "Go to page 8" }) as HTMLButtonElement;
  const underflow = handle.getByRole("button", { name: "Go to page 0" }) as HTMLButtonElement;

  assert.equal(valid.id, "edge-pages-page-3");
  assert.equal(overflow.id, "edge-pages-page-after-8");
  assert.equal(underflow.id, "edge-pages-page-before-0");
  assert.equal(new Set([valid.id, overflow.id, underflow.id]).size, 3);
  assert.equal(overflow.disabled, true);
  assert.equal(underflow.disabled, true);
  assert.equal(overflow.getAttribute("data-page"), "8");
  assert.equal(underflow.getAttribute("data-page"), "0");
  handle.unmount();
});
