import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import PaginationRoot from "./pagination.vue";
import PaginationEllipsis from "./pagination-ellipsis.vue";
import PaginationItem from "./pagination-item.vue";
import PaginationList from "./pagination-list.vue";
import PaginationNext from "./pagination-next.vue";
import PaginationPage from "./pagination-page.vue";
import PaginationPrevious from "./pagination-previous.vue";
import { createPaginationRange } from "./pagination-range.ts";
import type {
  PaginationControlExpose,
  PaginationEllipsisExpose,
  PaginationPageExpose,
  PaginationRootExpose,
  PaginationSlotState,
} from "./pagination.ts";
import { mountInteraction } from "../../../testing/mount.ts";

function renderPaginationTree(state: PaginationSlotState) {
  return h(PaginationList, null, {
    default: () => [
      h(PaginationItem, { key: "previous" }, () => h(PaginationPrevious, null, () => "Previous")),
      ...state.range.map((item) =>
        item.type === "page"
          ? h(PaginationItem, { key: item.key, page: item.page }, () =>
              h(PaginationPage, { page: item.page }, ({ state }) =>
                h("span", { "data-rendered-page": `${item.page}:${state}` }, String(item.page)),
              ),
            )
          : h(PaginationItem, { key: item.key }, () =>
              h(PaginationEllipsis, { position: item.position }, () => "..."),
            ),
      ),
      h(PaginationItem, { key: "next" }, () => h(PaginationNext, null, () => "Next")),
    ],
  });
}

function mountPagination(props: Record<string, unknown> = {}) {
  return mountInteraction(PaginationRoot, {
    props: { defaultValue: 2, pageCount: 4, ...props },
    record: ["update:modelValue", "change"],
    slots: {
      default: (state: PaginationSlotState) => [
        h(
          "output",
          { "data-range": "" },
          state.range
            .map((item) => (item.type === "page" ? String(item.page) : `ellipsis:${item.position}`))
            .join("|"),
        ),
        renderPaginationTree(state),
      ],
    },
  });
}

test("renders accessible pagination semantics with deterministic ids and range", () => {
  const handle = mountPagination({ defaultValue: 5, id: "docs-pages", pageCount: 10 });
  const root = handle.root();
  const list = root.querySelector<HTMLOListElement>('[data-vize-ui="pagination-list"]');
  const previous = root.querySelector<HTMLButtonElement>('[data-vize-ui="pagination-previous"]');
  const next = root.querySelector<HTMLButtonElement>('[data-vize-ui="pagination-next"]');
  const current = root.querySelector<HTMLButtonElement>(
    '[data-vize-ui="pagination-page"][aria-current="page"]',
  );
  const ellipses = root.querySelectorAll('[data-vize-ui="pagination-ellipsis"]');

  assert.equal(root.tagName, "NAV");
  assert.equal(root.id, "docs-pages");
  assert.equal(root.getAttribute("aria-label"), "Pagination");
  assert.equal(root.getAttribute("data-vize-ui"), "pagination");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-state"), "active");
  assert.equal(root.getAttribute("data-page"), "5");
  assert.equal(root.getAttribute("data-page-count"), "10");
  assert.equal(root.getAttribute("class"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.ok(list instanceof HTMLOListElement);
  assert.equal(list.id, "docs-pages-list");
  assert.equal(list.getAttribute("part"), "list");
  assert.equal(list.getAttribute("data-page"), "5");
  assert.equal(previous?.id, "docs-pages-previous");
  assert.equal(previous?.getAttribute("data-target-page"), "4");
  assert.equal(previous?.disabled, false);
  assert.equal(next?.id, "docs-pages-next");
  assert.equal(next?.getAttribute("data-target-page"), "6");
  assert.equal(next?.disabled, false);
  assert.ok(current);
  assert.equal(current.id, "docs-pages-page-5");
  assert.equal(current.disabled, false);
  assert.equal(current.getAttribute("aria-label"), "Page 5, current page");
  assert.equal(current.getAttribute("data-state"), "current");
  assert.equal(current.getAttribute("data-current"), "true");
  assert.equal(
    root.querySelector("[data-range]")?.textContent,
    "1|ellipsis:start|4|5|6|ellipsis:end|10",
  );
  assert.equal(ellipses.length, 2);
  assert.equal(ellipses[0]?.getAttribute("aria-label"), "More pages");
  assert.equal(ellipses[0]?.getAttribute("data-disabled"), "true");
  assert.equal(root.querySelector('[data-rendered-page="5:current"]')?.textContent, "5");
  handle.unmount();
});

test("clicks update uncontrolled page state and suppress current-page repeats", async () => {
  const handle = mountPagination({ defaultValue: 2, pageCount: 4 });

  await handle.click(handle.getByRole("button", { name: "Go to page 3" }));
  assert.equal(handle.root().getAttribute("data-page"), "3");
  assert.equal(
    handle
      .root()
      .querySelector('[data-vize-ui="pagination-page"][aria-current="page"]')
      ?.getAttribute("data-page"),
    "3",
  );

  await handle.click(handle.getByRole("button", { name: "Go to next page" }));
  assert.equal(handle.root().getAttribute("data-page"), "4");

  await handle.click(handle.getByRole("button", { name: "Page 4, current page" }));
  assert.equal(handle.root().getAttribute("data-page"), "4");

  await handle.click(handle.getByRole("button", { name: "Go to previous page" }));
  assert.equal(handle.root().getAttribute("data-page"), "3");
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[3], [4], [3]]);
  assert.deepEqual(
    handle.wrapper.emitted("change")?.map((payload) => payload.slice(0, 2)),
    [
      [3, 2],
      [4, 3],
      [3, 4],
    ],
  );
  assert.ok(handle.wrapper.emitted("change")?.[0]?.[2] instanceof MouseEvent);
  handle.unmount();
});

test("controlled page wins until the parent accepts the request", async () => {
  const handle = mountPagination({ modelValue: 2, pageCount: 5 });

  await handle.click(handle.getByRole("button", { name: "Go to page 4" }));
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[4]]);
  assert.deepEqual(handle.wrapper.emitted("change")?.[0]?.slice(0, 2), [4, 2]);
  assert.equal(handle.root().getAttribute("data-page"), "2");
  assert.equal(
    handle
      .root()
      .querySelector('[data-vize-ui="pagination-page"][aria-current="page"]')
      ?.getAttribute("data-page"),
    "2",
  );

  await handle.wrapper.setProps({ modelValue: 4 });
  assert.equal(handle.root().getAttribute("data-page"), "4");
  assert.equal(
    handle
      .root()
      .querySelector('[data-vize-ui="pagination-page"][aria-current="page"]')
      ?.getAttribute("data-page"),
    "4",
  );
  handle.unmount();
});

test("disabled roots and boundary controls suppress activation and tab focus", async () => {
  const boundary = mountPagination({ defaultValue: 1, pageCount: 2 });
  const previous = boundary.getByRole("button", {
    name: "Go to previous page",
  }) as HTMLButtonElement;

  assert.equal(previous.disabled, true);
  assert.equal(previous.getAttribute("data-disabled"), "true");
  await boundary.click(previous);
  await boundary.press(previous, "Enter");
  assert.equal(boundary.wrapper.emitted("update:modelValue"), undefined);
  assert.notEqual(await boundary.tab(), previous);
  boundary.unmount();

  const disabled = mountPagination({ defaultValue: 1, disabled: true, pageCount: 3 });
  const next = disabled.getByRole("button", { name: "Go to next page" }) as HTMLButtonElement;
  const page = disabled.getByRole("button", { name: "Page 1, current page" }) as HTMLButtonElement;

  assert.equal(disabled.root().getAttribute("data-state"), "disabled");
  assert.equal(next.disabled, true);
  assert.equal(page.disabled, true);
  await disabled.click(next);
  await disabled.press(next, " ");
  assert.equal(disabled.wrapper.emitted("update:modelValue"), undefined);
  assert.equal(await disabled.tab(), null);
  disabled.unmount();
});

test("exposes typed state and imperative page controls", async () => {
  let rootExpose: PaginationRootExpose | null = null;
  let listExpose: {
    readonly focus: (options?: FocusOptions) => void;
    readonly listId: string;
  } | null = null;
  let pageExpose: PaginationPageExpose | null = null;
  let previousExpose: PaginationControlExpose | null = null;
  let nextExpose: PaginationControlExpose | null = null;
  let ellipsisExpose: PaginationEllipsisExpose | null = null;
  const Probe = defineComponent({
    name: "PaginationExposeProbe",
    setup: () => () =>
      h(
        PaginationRoot,
        {
          defaultValue: 1,
          id: "exposed-pages",
          pageCount: 3,
          ref: (value) => {
            rootExpose = value as PaginationRootExpose | null;
          },
        },
        () =>
          h(
            PaginationList,
            {
              ref: (value) => {
                listExpose = value as typeof listExpose;
              },
            },
            () => [
              h(PaginationItem, () =>
                h(
                  PaginationPrevious,
                  {
                    ref: (value) => {
                      previousExpose = value as PaginationControlExpose | null;
                    },
                  },
                  () => "Previous",
                ),
              ),
              h(PaginationItem, { page: 1 }, () => h(PaginationPage, { page: 1 }, () => "1")),
              h(PaginationItem, { page: 2 }, () =>
                h(
                  PaginationPage,
                  {
                    page: 2,
                    ref: (value) => {
                      pageExpose = value as PaginationPageExpose | null;
                    },
                  },
                  () => "2",
                ),
              ),
              h(PaginationItem, () =>
                h(PaginationEllipsis, {
                  ref: (value) => {
                    ellipsisExpose = value as PaginationEllipsisExpose | null;
                  },
                }),
              ),
              h(PaginationItem, () =>
                h(
                  PaginationNext,
                  {
                    ref: (value) => {
                      nextExpose = value as PaginationControlExpose | null;
                    },
                  },
                  () => "Next",
                ),
              ),
            ],
          ),
      ),
  });
  const handle = mountInteraction(Probe);

  if (
    !rootExpose ||
    !listExpose ||
    !pageExpose ||
    !previousExpose ||
    !nextExpose ||
    !ellipsisExpose
  ) {
    assert.fail("Pagination refs must expose root, list, page, controls, and ellipsis state");
  }
  assert.equal(rootExpose.page, 1);
  assert.equal(rootExpose.nextPage, 2);
  assert.equal(rootExpose.canPrevious, false);
  assert.equal(listExpose.listId, "exposed-pages-list");
  assert.equal(pageExpose.current, false);
  assert.equal(previousExpose.disabled, true);
  assert.equal(nextExpose.targetPage, 2);
  assert.equal(ellipsisExpose.disabled, true);

  rootExpose.focus();
  assert.ok(
    handle.activeElement() === handle.getByRole("button", { name: "Page 1, current page" }),
  );
  assert.equal(rootExpose.setPage(2), true);
  await nextTick();
  assert.equal(rootExpose.page, 2);
  assert.equal(pageExpose.current, true);
  assert.equal(nextExpose.select(), true);
  await nextTick();
  assert.equal(rootExpose.page, 3);
  assert.equal(previousExpose.select(), true);
  await nextTick();
  assert.equal(rootExpose.page, 2);
  assert.equal(rootExpose.reset(), true);
  await nextTick();
  assert.equal(rootExpose.page, 1);
  listExpose.focus();
  assert.ok(
    handle.activeElement() === handle.getByRole("button", { name: "Page 1, current page" }),
  );
  handle.unmount();
});

test("compound parts require a matching root provider", () => {
  assert.throws(() => mountInteraction(PaginationList), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(PaginationItem), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(
    () => mountInteraction(PaginationPage, { props: { page: 1 } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
  assert.throws(() => mountInteraction(PaginationPrevious), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(PaginationNext), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(PaginationEllipsis), /VIZE_UI_CONTEXT_MISSING/);
});

test("range helper expands one-page gaps without ellipsis", () => {
  assert.deepEqual(
    createPaginationRange({ boundaryCount: 1, page: 3, pageCount: 5, siblingCount: 1 }),
    [
      { key: "page-1", page: 1, type: "page" },
      { key: "page-2", page: 2, type: "page" },
      { key: "page-3", page: 3, type: "page" },
      { key: "page-4", page: 4, type: "page" },
      { key: "page-5", page: 5, type: "page" },
    ],
  );
});
