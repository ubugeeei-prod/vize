import assert from "node:assert/strict";

import { afterEach, test, vi } from "vite-plus/test";
import { h, nextTick } from "vue";

import Pager from "./pager.vue";
import PagerPage from "./pager-page.vue";
import PagerTab from "./pager-tab.vue";
import PagerTabList from "./pager-tab-list.vue";
import PagerViewport from "./pager-viewport.vue";
import type { PagerExpose, PagerSlotState } from "./pager-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

type Page = "all" | "unread" | "starred";
const pages: readonly Page[] = ["all", "unread", "starred"];

afterEach(() => {
  vi.useRealTimers();
});

function mountPager(props: Record<string, unknown> = {}) {
  const handle = mountInteraction(Pager, {
    props: { pages, id: "mail", ...props },
    record: ["update:modelValue", "change"],
    slots: {
      default: (state: PagerSlotState<Page>) => [
        h(PagerTabList, { ariaLabel: "Mailboxes" }, () =>
          pages.map((page) => h(PagerTab, { key: page, page }, () => page)),
        ),
        h(PagerViewport, null, () =>
          pages.map((page) => h(PagerPage, { key: page, page }, () => `${page} list`)),
        ),
        h("output", `${state.active}:${state.index}/${state.count}`),
      ],
    },
  });
  const viewport = handle.root().querySelector<HTMLElement>('[data-vize-ui="pager-viewport"]');
  assert.ok(viewport);
  Object.defineProperty(viewport, "clientWidth", { value: 300, configurable: true });
  const scrolls: { left: number; behavior: string }[] = [];
  viewport.scrollTo = ((options: ScrollToOptions) => {
    scrolls.push({ left: options.left ?? 0, behavior: options.behavior ?? "auto" });
    viewport.scrollLeft = options.left ?? 0;
  }) as typeof viewport.scrollTo;
  return { handle, viewport, scrolls };
}

async function settle(): Promise<void> {
  await nextTick();
  await nextTick();
}

function tabs(root: HTMLElement): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>('[role="tab"]')];
}

test("renders APG tabs wired to scroll-snap pages with inert off-screen panels", async () => {
  const { handle } = mountPager({ defaultValue: "unread" });
  await settle();
  const root = handle.root();
  const [all, unread] = tabs(root);
  const panels = [...root.querySelectorAll<HTMLElement>('[role="tabpanel"]')];

  assert.equal(root.querySelector('[role="tablist"]')?.getAttribute("aria-label"), "Mailboxes");
  assert.equal(unread?.getAttribute("aria-selected"), "true");
  assert.equal(unread?.tabIndex, 0);
  assert.equal(all?.tabIndex, -1);
  assert.equal(unread?.getAttribute("aria-controls"), "mail-panel-unread");
  assert.equal(panels[1]?.getAttribute("aria-labelledby"), "mail-tab-unread");
  assert.equal(panels[0]?.hasAttribute("inert"), true);
  assert.equal(panels[1]?.hasAttribute("inert"), false);
  assert.equal(root.querySelector("output")?.textContent, "unread:1/3");
  handle.unmount();
});

test("selecting a tab scrolls the viewport smoothly and emits typed changes", async () => {
  const { handle, scrolls } = mountPager();
  await settle();
  const [, , starred] = tabs(handle.root());
  starred?.click();
  await settle();
  assert.deepEqual(scrolls.at(-1), { left: 600, behavior: "smooth" });
  assert.equal(handle.root().getAttribute("data-page"), "starred");
  assert.deepEqual(handle.recorded().at(-1), {
    event: "change",
    payload: ["starred", "all", "tab"],
  });
  handle.unmount();
});

test("arrow keys, Home, and End move the selected segment with wrapping", async () => {
  const { handle } = mountPager();
  await settle();
  const [all, , starred] = tabs(handle.root());
  const list = handle.root().querySelector('[role="tablist"]');
  assert.ok(list);
  all?.focus();
  const press = async (key: string) => {
    list.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }));
    await settle();
  };
  await press("ArrowLeft");
  assert.ok(document.activeElement === starred, "ArrowLeft wraps to the last page");
  await press("Home");
  assert.equal(handle.root().getAttribute("data-page"), "all");
  await press("End");
  await press("ArrowRight");
  assert.equal(handle.root().getAttribute("data-page"), "all");
  assert.ok(
    handle
      .recorded()
      .filter((entry) => entry.event === "change")
      .every((entry) => entry.payload[2] === "keyboard"),
  );
  handle.unmount();
});

test("user swipes settle on the nearest page while programmatic scrolls are ignored", async () => {
  vi.useFakeTimers();
  const { handle, viewport } = mountPager();
  await settle();
  viewport.scrollLeft = 320;
  viewport.dispatchEvent(new Event("scroll"));
  vi.advanceTimersByTime(119);
  await nextTick();
  assert.equal(handle.root().getAttribute("data-page"), "all", "still scrolling");
  vi.advanceTimersByTime(1);
  await nextTick();
  assert.equal(handle.root().getAttribute("data-page"), "unread");
  assert.deepEqual(handle.recorded().at(-1), {
    event: "change",
    payload: ["unread", "all", "scroll"],
  });

  viewport.scrollLeft = 600;
  viewport.dispatchEvent(new Event("scrollend"));
  await nextTick();
  assert.equal(handle.root().getAttribute("data-page"), "starred", "scrollend settles immediately");

  const api = handle.exposes<PagerExpose<Page>>();
  assert.equal(api.previous(), true);
  await nextTick();
  viewport.dispatchEvent(new Event("scrollend"));
  await nextTick();
  assert.equal(api.active, "unread", "the programmatic scroll does not re-select");
  assert.equal(api.next(), true);
  assert.equal(api.next(), false, "no wrap at the end");
  handle.unmount();
});

test("controlled pages scroll when the parent changes them and reduced motion scrolls instantly", async () => {
  const original = window.matchMedia;
  window.matchMedia = ((query: string) => ({
    matches: query.includes("reduce"),
    media: query,
  })) as unknown as typeof window.matchMedia;
  try {
    const { handle, scrolls } = mountPager({ modelValue: "all" });
    await settle();
    tabs(handle.root())[1]?.click();
    await settle();
    assert.equal(handle.root().getAttribute("data-page"), "all", "controlled value wins");
    await handle.wrapper.setProps({ modelValue: "starred" });
    await settle();
    assert.deepEqual(scrolls.at(-1), { left: 600, behavior: "auto" });
    handle.unmount();
  } finally {
    window.matchMedia = original;
  }
});

test("rejects empty page lists and parts outside a Pager", () => {
  const warn = console.warn;
  console.warn = () => undefined;
  try {
    assert.throws(() => mountInteraction(Pager, { props: { pages: [] } }), /VIZE_UI_PAGER_PAGES/);
    for (const [part, props] of [
      [PagerTabList, {}],
      [PagerTab, { page: "a" }],
      [PagerViewport, {}],
      [PagerPage, { page: "a" }],
    ] as const) {
      assert.throws(() => mountInteraction(part, { props }), /VIZE_UI_CONTEXT_MISSING: Pager/);
    }
  } finally {
    console.warn = warn;
  }
});
