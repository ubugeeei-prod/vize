import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import type { ScrollAreaExpose, ScrollAreaSlotState } from "./scroll-area.ts";
import ScrollArea from "./scroll-area.vue";

function getViewport(root: HTMLElement): HTMLDivElement {
  const viewport = root.querySelector('[data-vize-ui="scroll-area-viewport"]');
  assert.ok(viewport instanceof HTMLDivElement);
  return viewport;
}

function formatScrollAreaSlotState(state: ScrollAreaSlotState): string {
  const host = typeof state.as === "string" ? state.as : "component";
  return [
    host,
    state.orientation,
    state.dir,
    state.focusable ? "true" : "false",
    state.blockSize,
    state.inlineSize,
    state.maxBlockSize,
    state.maxInlineSize,
    state.overflowX,
    state.overflowY,
    state.overscrollBehavior,
    state.scrollBehavior,
    state.scrollbarGutter,
    state.scrollbarWidth,
    state.ariaLabel ?? "",
    state.ariaLabelledby ?? "",
    state.ariaDescribedby ?? "",
    state.labelled ? "true" : "false",
    state.described ? "true" : "false",
    state.state,
  ].join(":");
}

test("renders a vertical native viewport by default without generating ids or focus stops", async () => {
  const handle = mountInteraction(ScrollArea, {
    slots: { default: '<button type="button">Continue</button>' },
  });
  const root = handle.root();
  const viewport = getViewport(root);

  assert.equal(root.tagName, "DIV");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "scroll-area");
  assert.equal(root.getAttribute("data-state"), "scrollable");
  assert.equal(root.getAttribute("data-orientation"), "vertical");
  assert.equal(root.getAttribute("data-dir"), "ltr");
  assert.equal(root.getAttribute("dir"), "ltr");
  assert.equal(root.getAttribute("data-focusable"), "false");
  assert.equal(root.id, "");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-block-size"), "auto");
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-inline-size"), "auto");
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-max-block-size"), "none");
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-max-inline-size"), "none");
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-overflow-x"), "hidden");
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-overflow-y"), "auto");
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-overscroll-behavior"), "auto");
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-scroll-behavior"), "auto");
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-scrollbar-gutter"), "auto");
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-scrollbar-width"), "auto");

  assert.equal(viewport.getAttribute("part"), "viewport");
  assert.equal(viewport.getAttribute("data-vize-ui"), "scroll-area-viewport");
  assert.equal(viewport.getAttribute("data-orientation"), "vertical");
  assert.equal(viewport.getAttribute("data-dir"), "ltr");
  assert.equal(viewport.getAttribute("data-overflow-x"), "hidden");
  assert.equal(viewport.getAttribute("data-overflow-y"), "auto");
  assert.equal(viewport.getAttribute("role"), null);
  assert.equal(viewport.getAttribute("tabindex"), null);
  assert.equal(viewport.getAttribute("aria-label"), null);
  assert.equal(viewport.getAttribute("aria-labelledby"), null);
  assert.equal(viewport.getAttribute("aria-describedby"), null);
  assert.equal(viewport.id, "");
  assert.equal(await handle.tab(), handle.getByRole("button", { name: "Continue" }));
  handle.unmount();
});

test("renders an RTL labelled region with native scrolling hooks", () => {
  const handle = mountInteraction(ScrollArea, {
    attrs: {
      "data-owner": "consumer",
      id: "release-log",
    },
    props: {
      ariaDescribedby: " scroll-help ",
      ariaLabelledby: " scroll-title ",
      as: "section",
      blockSize: 240,
      dir: "rtl",
      focusable: true,
      inlineSize: "min(100%, 34rem)",
      maxBlockSize: "60vh",
      maxInlineSize: "100%",
      orientation: "both",
      overscrollBehavior: "contain",
      scrollBehavior: "smooth",
      scrollbarGutter: "stable both-edges",
      scrollbarWidth: "thin",
    },
    slots: {
      default: '<h2 id="scroll-title">Updates</h2><p id="scroll-help">Scrollable release notes</p>',
    },
  });
  const root = handle.root();
  const viewport = getViewport(root);

  assert.equal(root.tagName, "SECTION");
  assert.equal(root.id, "release-log");
  assert.equal(root.getAttribute("data-owner"), "consumer");
  assert.equal(root.getAttribute("data-orientation"), "both");
  assert.equal(root.getAttribute("data-dir"), "rtl");
  assert.equal(root.getAttribute("dir"), "rtl");
  assert.equal(root.getAttribute("data-focusable"), "true");
  assert.equal(root.getAttribute("data-overscroll-behavior"), "contain");
  assert.equal(root.getAttribute("data-scroll-behavior"), "smooth");
  assert.equal(root.getAttribute("data-scrollbar-gutter"), "stable both-edges");
  assert.equal(root.getAttribute("data-scrollbar-width"), "thin");
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-block-size"), "240px");
  assert.equal(
    root.style.getPropertyValue("--vize-ui-scroll-area-inline-size"),
    "min(100%, 34rem)",
  );
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-max-block-size"), "60vh");
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-max-inline-size"), "100%");
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-overflow-x"), "auto");
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-overflow-y"), "auto");

  assert.equal(viewport.getAttribute("role"), "region");
  assert.equal(viewport.getAttribute("tabindex"), "0");
  assert.equal(viewport.getAttribute("aria-labelledby"), "scroll-title");
  assert.equal(viewport.getAttribute("aria-describedby"), "scroll-help");
  assert.equal(viewport.getAttribute("dir"), "rtl");
  assert.equal(viewport.getAttribute("data-dir"), "rtl");
  assert.equal(viewport.getAttribute("data-overflow-x"), "auto");
  assert.equal(viewport.getAttribute("data-overflow-y"), "auto");
  assert.equal(viewport.textContent, "UpdatesScrollable release notes");
  handle.unmount();
});

test("passes slot state and exposes live viewport state", async () => {
  const handle = mountInteraction(ScrollArea, {
    props: {
      ariaLabel: "Timeline",
      blockSize: "12rem",
      focusable: true,
      orientation: "horizontal",
      scrollbarWidth: "none",
    },
    slots: {
      default: formatScrollAreaSlotState,
    },
  });
  const exposed = handle.exposes<ScrollAreaExpose>();
  const root = handle.root();
  const viewport = getViewport(root);

  assert.ok(exposed.root === root);
  assert.ok(exposed.viewport === viewport);
  assert.equal(exposed.as, "div");
  assert.equal(exposed.orientation, "horizontal");
  assert.equal(exposed.dir, "ltr");
  assert.equal(exposed.focusable, true);
  assert.equal(exposed.blockSize, "12rem");
  assert.equal(exposed.inlineSize, "auto");
  assert.equal(exposed.maxBlockSize, "none");
  assert.equal(exposed.maxInlineSize, "none");
  assert.equal(exposed.overflowX, "auto");
  assert.equal(exposed.overflowY, "hidden");
  assert.equal(exposed.scrollbarWidth, "none");
  assert.equal(exposed.ariaLabel, "Timeline");
  assert.equal(exposed.ariaLabelledby, undefined);
  assert.equal(exposed.ariaDescribedby, undefined);
  assert.equal(exposed.labelled, true);
  assert.equal(exposed.described, false);
  assert.equal(
    viewport.textContent,
    "div:horizontal:ltr:true:12rem:auto:none:none:auto:hidden:auto:auto:auto:none:Timeline:::true:false:scrollable",
  );

  await handle.wrapper.setProps({
    ariaDescribedby: "details",
    ariaLabel: undefined,
    ariaLabelledby: "heading",
    dir: "rtl",
    focusable: false,
    inlineSize: 480,
    maxInlineSize: "90vw",
    orientation: "vertical",
    overscrollBehavior: "none",
    scrollbarGutter: "stable",
    scrollbarWidth: "thin",
  });

  assert.equal(exposed.orientation, "vertical");
  assert.equal(exposed.dir, "rtl");
  assert.equal(exposed.focusable, false);
  assert.equal(exposed.inlineSize, "480px");
  assert.equal(exposed.maxInlineSize, "90vw");
  assert.equal(exposed.overflowX, "hidden");
  assert.equal(exposed.overflowY, "auto");
  assert.equal(exposed.overscrollBehavior, "none");
  assert.equal(exposed.scrollbarGutter, "stable");
  assert.equal(exposed.scrollbarWidth, "thin");
  assert.equal(exposed.ariaLabel, undefined);
  assert.equal(exposed.ariaLabelledby, "heading");
  assert.equal(exposed.ariaDescribedby, "details");
  assert.equal(exposed.labelled, true);
  assert.equal(exposed.described, true);
  assert.equal(viewport.getAttribute("tabindex"), null);
  assert.equal(viewport.getAttribute("aria-label"), null);
  assert.equal(viewport.getAttribute("aria-labelledby"), "heading");
  assert.equal(viewport.getAttribute("aria-describedby"), "details");
  assert.equal(
    viewport.textContent,
    "div:vertical:rtl:false:12rem:480px:none:90vw:hidden:auto:none:auto:stable:thin::heading:details:true:true:scrollable",
  );
  handle.unmount();
});

test("emits native scroll events and exposes focus and scroll methods", async () => {
  const handle = mountInteraction(ScrollArea, {
    props: {
      ariaLabel: "Activity",
      focusable: true,
    },
    record: ["scroll"],
    slots: { default: "<p>Activity stream</p>" },
  });
  const exposed = handle.exposes<ScrollAreaExpose>();
  const viewport = getViewport(handle.root());
  const calls: Array<readonly ["by" | "to", ScrollToOptions | undefined]> = [];
  viewport.scrollTo = (options?: ScrollToOptions) => {
    calls.push(["to", options]);
  };
  viewport.scrollBy = (options?: ScrollToOptions) => {
    calls.push(["by", options]);
  };

  exposed.focus();
  assert.ok(handle.activeElement() === viewport);
  exposed.scrollTo({ top: 48 });
  exposed.scrollBy({ left: 12, behavior: "auto" });
  assert.deepEqual(calls, [
    ["to", { top: 48 }],
    ["by", { left: 12, behavior: "auto" }],
  ]);

  const event = new Event("scroll", { bubbles: true, cancelable: false });
  viewport.dispatchEvent(event);
  await nextTick();

  assert.deepEqual(handle.recorded(), [{ event: "scroll", payload: [event] }]);
  handle.unmount();
});

test("renders a consumer component root without dropping scroll hooks", () => {
  const CustomRoot = defineComponent({
    name: "ScrollAreaCustomRoot",
    setup(_, { attrs, slots }) {
      return () => h("main", attrs, slots.default?.());
    },
  });
  const handle = mountInteraction(ScrollArea, {
    attrs: { id: "custom-scroll-area" },
    props: {
      ariaLabel: "Messages",
      as: CustomRoot,
      blockSize: "16rem",
      orientation: "vertical",
    },
    slots: { default: "<p>Messages</p>" },
  });
  const root = handle.root();
  const viewport = getViewport(root);

  assert.equal(root.tagName, "MAIN");
  assert.equal(root.id, "custom-scroll-area");
  assert.equal(root.getAttribute("data-vize-ui"), "scroll-area");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-block-size"), "16rem");
  assert.equal(viewport.getAttribute("role"), "region");
  assert.equal(viewport.getAttribute("aria-label"), "Messages");
  assert.equal(viewport.textContent, "Messages");
  handle.unmount();
});
