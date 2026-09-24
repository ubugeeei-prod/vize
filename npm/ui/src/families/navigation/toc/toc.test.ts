import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import { collectTocEntries } from "./toc.ts";
import type { TocRootExpose } from "./toc.ts";
import TocItem from "./toc-item.vue";
import TocLink from "./toc-link.vue";
import TocList from "./toc-list.vue";
import TocRoot from "./toc-root.vue";
import { mountInteraction } from "../../../testing/mount.ts";

interface Section {
  readonly element: HTMLElement;
  top: number;
}

function createSections(ids: readonly string[]): Section[] {
  return ids.map((id, index) => {
    const element = document.createElement("h2");
    element.id = id;
    element.textContent = id;
    document.body.append(element);
    const section: Section = { element, top: index * 500 };
    element.getBoundingClientRect = () => new DOMRect(0, section.top, 800, 40);
    element.scrollIntoView = () => undefined;
    return section;
  });
}

function removeSections(sections: readonly Section[]): void {
  for (const section of sections) section.element.remove();
}

function mountToc(props: Record<string, unknown> = {}) {
  return mountInteraction(TocRoot, {
    props,
    record: ["update:activeId", "navigate"],
    slots: {
      default: () =>
        h(TocList, null, () => [
          h(TocItem, { targetId: "install" }, () =>
            h(TocLink, { targetId: "install" }, () => "Install"),
          ),
          h(TocItem, null, () => [
            h(TocLink, { targetId: "usage" }, () => "Usage"),
            h(TocList, { level: 2 }, () =>
              h(TocItem, null, () => h(TocLink, { targetId: "usage-advanced" }, () => "Advanced")),
            ),
          ]),
        ]),
    },
  });
}

async function frame(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 40));
  await nextTick();
}

test("renders a labelled navigation landmark with fragment links", () => {
  const handle = mountToc({ defaultActiveId: "usage" });
  const nav = handle.root();
  assert.equal(nav.tagName, "NAV");
  assert.equal(nav.getAttribute("aria-label"), "Table of contents");
  assert.equal(nav.getAttribute("data-state"), "active");
  const usage = handle.getByRole("link", { name: "Usage" });
  assert.equal(usage.getAttribute("href"), "#usage");
  assert.equal(usage.getAttribute("aria-current"), "location");
  assert.equal(handle.getByRole("link", { name: "Install" }).getAttribute("aria-current"), null);
  const nested = nav.querySelector('[data-vize-ui="toc-list"][data-level="2"]');
  assert.ok(nested instanceof HTMLOListElement);
  handle.unmount();
});

test("tracks the section in view and marks links and items active", async () => {
  const sections = createSections(["install", "usage", "usage-advanced"]);
  const handle = mountToc({ offset: 80 });
  await frame();
  assert.equal(
    handle.getByRole("link", { name: "Install" }).getAttribute("aria-current"),
    "location",
  );
  for (const section of sections) section.top -= 520;
  window.dispatchEvent(new Event("scroll"));
  await frame();
  const usage = handle.getByRole("link", { name: "Usage" });
  assert.equal(usage.getAttribute("aria-current"), "location");
  assert.equal(usage.closest("li")?.getAttribute("data-active"), "true");
  assert.equal(handle.root().getAttribute("data-active-id"), "usage");
  assert.deepEqual(
    handle.wrapper.emitted("update:activeId")?.map(([id]) => id),
    ["install", "usage"],
  );
  handle.unmount();
  removeSections(sections);
});

test("controlled activeId and disabled tracking leave ownership to the parent", async () => {
  const sections = createSections(["install", "usage", "usage-advanced"]);
  const handle = mountToc({ activeId: "usage-advanced", track: false });
  await frame();
  assert.equal(
    handle.getByRole("link", { name: "Advanced" }).getAttribute("aria-current"),
    "location",
  );
  assert.equal(handle.wrapper.emitted("update:activeId"), undefined);
  await handle.wrapper.setProps({ activeId: "install" });
  assert.equal(
    handle.getByRole("link", { name: "Install" }).getAttribute("aria-current"),
    "location",
  );
  handle.unmount();
  removeSections(sections);
});

test("scrollBehavior drives smooth navigation and updates the fragment", async () => {
  const sections = createSections(["install", "usage", "usage-advanced"]);
  const scrolled: ScrollIntoViewOptions[] = [];
  const usageSection = sections[1];
  assert.ok(usageSection);
  usageSection.element.scrollIntoView = (options?: boolean | ScrollIntoViewOptions) => {
    if (typeof options === "object") scrolled.push(options);
  };
  const handle = mountToc({ scrollBehavior: "smooth" });
  const usage = handle.getByRole("link", { name: "Usage" });
  const click = new MouseEvent("click", { bubbles: true, cancelable: true, button: 0 });
  usage.dispatchEvent(click);
  await nextTick();
  assert.equal(click.defaultPrevented, true);
  assert.deepEqual(scrolled, [{ behavior: "smooth", block: "start" }]);
  assert.equal(window.location.hash, "#usage");
  assert.equal(usage.getAttribute("aria-current"), "location");
  assert.equal(handle.wrapper.emitted("navigate")?.[0]?.[0], "usage");

  const modified = new MouseEvent("click", { bubbles: true, cancelable: true, metaKey: true });
  handle.getByRole("link", { name: "Install" }).dispatchEvent(modified);
  assert.equal(modified.defaultPrevented, false, "modified clicks keep native behavior");
  handle.unmount();
  removeSections(sections);
});

test("native fragment navigation is untouched without scrollBehavior", async () => {
  const handle = mountToc();
  const click = new MouseEvent("click", { bubbles: true, cancelable: true, button: 0 });
  handle.getByRole("link", { name: "Install" }).dispatchEvent(click);
  await nextTick();
  assert.equal(click.defaultPrevented, false);
  handle.unmount();
});

test("exposes scrollTo and refresh", async () => {
  const sections = createSections(["install", "usage", "usage-advanced"]);
  let toc: TocRootExpose | null = null;
  const Probe = defineComponent({
    name: "TocExposeProbe",
    setup: () => () =>
      h(
        TocRoot,
        {
          ariaLabelledby: "toc-heading",
          ref: (value: unknown) => {
            toc = value as TocRootExpose | null;
          },
        },
        () => [
          h(TocLink, { targetId: "install" }, () => "Install"),
          h(TocLink, { targetId: "usage" }, () => "Usage"),
        ],
      ),
  });
  const handle = mountInteraction(Probe);
  if (toc === null) assert.fail("toc must expose controls");
  const expose: TocRootExpose = toc;
  assert.equal(handle.root().getAttribute("aria-label"), null);
  assert.equal(handle.root().getAttribute("aria-labelledby"), "toc-heading");
  assert.equal(expose.scrollTo("usage"), true);
  await nextTick();
  assert.equal(expose.activeId, "usage");
  assert.equal(expose.scrollTo("missing"), false);
  expose.refresh();
  assert.equal(expose.activeId, "install");
  handle.unmount();
  removeSections(sections);
});

test("collectTocEntries reads headings with ids in document order", () => {
  const container = document.createElement("article");
  container.innerHTML = [
    '<h2 id="a">  First   heading </h2>',
    "<h2>No id</h2>",
    '<h3 id="b">Second</h3>',
    '<h3 id="c" data-toc-ignore>Hidden</h3>',
    '<h4 id="d">Deep</h4>',
  ].join("");
  assert.deepEqual(collectTocEntries(container), [
    { id: "a", level: 2, text: "First heading" },
    { id: "b", level: 3, text: "Second" },
  ]);
  assert.deepEqual(
    collectTocEntries(container, { selector: "h4" }).map((entry) => entry.level),
    [4],
  );
});

test("compound parts require a matching root provider", () => {
  assert.throws(
    () => mountInteraction(TocLink, { props: { targetId: "x" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
  assert.throws(() => mountInteraction(TocItem), /VIZE_UI_CONTEXT_MISSING/);
});
