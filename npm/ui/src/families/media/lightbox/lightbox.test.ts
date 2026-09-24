import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type { LightboxRootExpose, LightboxSlotState } from "./lightbox.ts";
import LightboxClose from "./lightbox-close.vue";
import LightboxContent from "./lightbox-content.vue";
import LightboxCounter from "./lightbox-counter.vue";
import LightboxImage from "./lightbox-image.vue";
import LightboxItem from "./lightbox-item.vue";
import LightboxNext from "./lightbox-next.vue";
import LightboxPrevious from "./lightbox-previous.vue";
import LightboxRoot from "./lightbox-root.vue";
import {
  classifyLightboxSwipe,
  lightboxPreloadIndexes,
  resolveLightboxIndex,
  resolveLightboxMessages,
} from "./lightbox-state.ts";
import LightboxThumbnail from "./lightbox-thumbnail.vue";
import LightboxThumbnails from "./lightbox-thumbnails.vue";
import LightboxTrigger from "./lightbox-trigger.vue";
import { mountInteraction } from "../../../testing/mount.ts";

interface Photo {
  readonly src: string;
  readonly alt: string;
}

const photos: readonly Photo[] = ["Harbor", "Forest", "Dunes"].map((name) => ({
  src: `/photos/${name.toLowerCase()}.jpg`,
  alt: name,
}));

function mountLightbox(
  props: Record<string, unknown> = {},
  contentProps: Record<string, unknown> = {},
) {
  return mountInteraction(LightboxRoot, {
    props: { id: "gallery", items: photos, ...props },
    record: ["update:open", "update:index", "change"],
    slots: {
      default: (state: LightboxSlotState<Photo>) => [
        ...photos.map((photo, index) =>
          h(LightboxTrigger, { index, key: photo.src }, () => h("span", null, `Open ${photo.alt}`)),
        ),
        h(LightboxContent, { portalDisabled: true, ...contentProps }, () => [
          h(LightboxItem, null, () =>
            state.item === undefined
              ? null
              : h(LightboxImage, { src: state.item.src, alt: state.item.alt }),
          ),
          h(LightboxPrevious),
          h(LightboxNext),
          h(LightboxClose),
          h(LightboxCounter),
          h(LightboxThumbnails, null, () =>
            photos.map((photo, index) => h(LightboxThumbnail, { index, key: photo.src })),
          ),
          h("input", { "aria-label": "Caption", "data-caption": "" }),
        ]),
      ],
    },
  });
}

function part(root: ParentNode, name: string): HTMLElement {
  const element = root.querySelector<HTMLElement>(`[data-vize-ui="${name}"]`);
  assert.ok(element, `${name} must render`);
  return element;
}

function stage(root: ParentNode): HTMLElement {
  return part(root, "lightbox-content");
}

function key(target: Element, value: string): KeyboardEvent {
  const event = new KeyboardEvent("keydown", { key: value, bubbles: true, cancelable: true });
  target.dispatchEvent(event);
  return event;
}

function swipe(
  target: Element,
  from: [number, number],
  to: [number, number],
  pointerType = "touch",
) {
  const init = { bubbles: true, cancelable: true, pointerId: 7, isPrimary: true, pointerType };
  target.dispatchEvent(
    new PointerEvent("pointerdown", { ...init, clientX: from[0], clientY: from[1] }),
  );
  target.dispatchEvent(new PointerEvent("pointerup", { ...init, clientX: to[0], clientY: to[1] }));
}

test("renders closed triggers that open the dialog at their item and restore focus", async () => {
  const handle = mountLightbox();
  const root = handle.root();
  assert.equal(root.getAttribute("data-vize-ui"), "lightbox-root");
  assert.equal(root.getAttribute("data-state"), "closed");
  assert.equal(root.querySelector('[data-vize-ui="lightbox-content"]'), null);
  const trigger = handle.getByRole("button", { name: "Open Forest" });
  assert.equal(trigger.getAttribute("aria-haspopup"), "dialog");
  assert.equal(trigger.getAttribute("aria-expanded"), "false");
  assert.equal(trigger.getAttribute("aria-controls"), "gallery-content");

  trigger.focus();
  await handle.click(trigger);
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "open");
  assert.equal(root.getAttribute("data-index"), "1");
  assert.equal(trigger.getAttribute("aria-expanded"), "true");
  const dialog = part(root, "dialog-content");
  assert.equal(dialog.getAttribute("role"), "dialog");
  assert.equal(dialog.getAttribute("aria-modal"), "true");
  assert.equal(dialog.getAttribute("aria-label"), "Media viewer");
  const item = part(root, "lightbox-item");
  assert.equal(item.id, "gallery-item-1");
  assert.equal(item.getAttribute("aria-roledescription"), "slide");
  assert.equal(item.getAttribute("aria-label"), "2 of 3");
  assert.equal(item.querySelector("img")?.getAttribute("src"), "/photos/forest.jpg");
  assert.equal(item.querySelector("img")?.getAttribute("loading"), "eager");
  const counter = part(root, "lightbox-counter");
  assert.equal(counter.getAttribute("aria-live"), "polite");
  assert.equal(counter.textContent, "2 of 3");
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[true]]);
  assert.deepEqual(handle.wrapper.emitted("change")?.[0], [1, 0, "trigger"]);

  await handle.click(handle.getByRole("button", { name: "Close" }));
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "closed");
  assert.ok(document.activeElement === trigger, "focus returns to the opening trigger");
  handle.unmount();
});

test("previous and next navigate, disable at the ends, and wrap with loop", async () => {
  const handle = mountLightbox({ defaultOpen: true });
  const root = handle.root();
  await nextTick();
  const previous = handle.getByRole("button", { name: "Previous item" }) as HTMLButtonElement;
  const next = handle.getByRole("button", { name: "Next item" }) as HTMLButtonElement;
  assert.equal(previous.disabled, true);
  await handle.click(next);
  await handle.click(next);
  assert.equal(root.getAttribute("data-index"), "2");
  assert.equal(next.disabled, true);
  assert.equal(part(root, "lightbox-counter").textContent, "3 of 3");
  await handle.click(previous);
  assert.equal(root.getAttribute("data-index"), "1");
  assert.deepEqual(
    handle.wrapper.emitted("change")?.map((payload) => payload[2]),
    ["next", "next", "previous"],
  );
  handle.unmount();

  const looping = mountLightbox({ defaultOpen: true, loop: true });
  await nextTick();
  await looping.click(looping.getByRole("button", { name: "Previous item" }));
  assert.equal(looping.root().getAttribute("data-index"), "2");
  await looping.click(looping.getByRole("button", { name: "Next item" }));
  assert.equal(looping.root().getAttribute("data-index"), "0");
  looping.unmount();
});

test("arrow, Home, and End keys navigate with reading direction and skip text fields", async () => {
  const handle = mountLightbox({ defaultOpen: true, dir: "rtl" });
  const root = handle.root();
  await nextTick();
  const content = stage(root);
  assert.equal(content.getAttribute("dir"), "rtl");
  assert.equal(key(content, "ArrowLeft").defaultPrevented, true);
  await nextTick();
  assert.equal(root.getAttribute("data-index"), "1");
  key(content, "ArrowRight");
  await nextTick();
  assert.equal(root.getAttribute("data-index"), "0");
  key(content, "End");
  await nextTick();
  assert.equal(root.getAttribute("data-index"), "2");
  key(content, "Home");
  await nextTick();
  assert.equal(root.getAttribute("data-index"), "0");
  const caption = content.querySelector("[data-caption]");
  assert.ok(caption);
  assert.equal(key(caption, "ArrowLeft").defaultPrevented, false);
  const modified = new KeyboardEvent("keydown", {
    key: "ArrowLeft",
    bubbles: true,
    cancelable: true,
    ctrlKey: true,
  });
  content.dispatchEvent(modified);
  assert.equal(modified.defaultPrevented, false);
  assert.equal(handle.wrapper.emitted("change")?.[0]?.[2], "keyboard");
  handle.unmount();
});

test("Escape closes through the dialog layer", async () => {
  const handle = mountLightbox({ defaultOpen: true });
  await nextTick();
  key(part(handle.root(), "dialog-content"), "Escape");
  await nextTick();
  assert.equal(handle.root().getAttribute("data-state"), "closed");
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[false]]);
  handle.unmount();
});

test("touch swipes navigate by direction and a downward swipe closes", async () => {
  const handle = mountLightbox({ defaultOpen: true });
  const root = handle.root();
  await nextTick();
  swipe(stage(root), [200, 100], [100, 110]);
  await nextTick();
  assert.equal(root.getAttribute("data-index"), "1");
  assert.equal(handle.wrapper.emitted("change")?.[0]?.[2], "swipe");
  swipe(stage(root), [100, 100], [200, 100]);
  await nextTick();
  assert.equal(root.getAttribute("data-index"), "0");
  swipe(stage(root), [100, 100], [120, 105]);
  swipe(stage(root), [100, 100], [0, 100], "mouse");
  await nextTick();
  assert.equal(root.getAttribute("data-index"), "0", "short and mouse gestures are ignored");
  swipe(stage(root), [100, 100], [100, 220]);
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "closed");
  handle.unmount();

  const pinned = mountLightbox({ defaultOpen: true, closeOnSwipeDown: false });
  await nextTick();
  swipe(stage(pinned.root()), [100, 100], [100, 300]);
  const cancelled = stage(pinned.root());
  cancelled.dispatchEvent(
    new PointerEvent("pointerdown", {
      pointerId: 3,
      isPrimary: true,
      pointerType: "touch",
      clientX: 200,
    }),
  );
  cancelled.dispatchEvent(new PointerEvent("pointercancel", { pointerId: 3 }));
  cancelled.dispatchEvent(
    new PointerEvent("pointerup", { pointerId: 3, pointerType: "touch", clientX: 0 }),
  );
  await nextTick();
  assert.equal(pinned.root().getAttribute("data-state"), "open");
  assert.equal(pinned.root().getAttribute("data-index"), "0");
  pinned.unmount();
});

test("thumbnails form a roving tablist that selects and focuses items", async () => {
  const handle = mountLightbox({ defaultOpen: true });
  const root = handle.root();
  await nextTick();
  const tablist = handle.getByRole("tablist", { name: "Choose item" });
  const tabs = [...tablist.querySelectorAll<HTMLButtonElement>('[role="tab"]')];
  assert.equal(tabs[0]?.getAttribute("aria-selected"), "true");
  assert.equal(tabs[0]?.getAttribute("tabindex"), "0");
  assert.equal(tabs[1]?.getAttribute("tabindex"), "-1");
  assert.equal(tabs[1]?.getAttribute("aria-label"), "Show item 2 of 3");
  assert.equal(tabs[1]?.getAttribute("aria-controls"), "gallery-item-0");
  await handle.click(tabs[2] as HTMLButtonElement);
  assert.equal(root.getAttribute("data-index"), "2");
  tabs[2]?.focus();
  const home = key(tabs[2] as HTMLButtonElement, "Home");
  await nextTick();
  await nextTick();
  assert.equal(home.defaultPrevented, true);
  assert.equal(root.getAttribute("data-index"), "0", "the stage does not double-handle keys");
  assert.ok(document.activeElement === tabs[0]);
  key(tabs[0] as HTMLButtonElement, "ArrowRight");
  await nextTick();
  await nextTick();
  assert.equal(root.getAttribute("data-index"), "1");
  key(tabs[1] as HTMLButtonElement, "End");
  await nextTick();
  key(tabs[2] as HTMLButtonElement, "ArrowRight");
  await nextTick();
  assert.equal(root.getAttribute("data-index"), "2");
  key(tabs[2] as HTMLButtonElement, "ArrowLeft");
  await nextTick();
  assert.equal(root.getAttribute("data-index"), "1");
  assert.equal(key(tabs[1] as HTMLButtonElement, "Enter").defaultPrevented, false);
  handle.unmount();
});

test("controlled open and index win until the parent accepts the request", async () => {
  const handle = mountLightbox({ open: false, index: 0 });
  const root = handle.root();
  await handle.click(handle.getByRole("button", { name: "Open Dunes" }));
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[true]]);
  assert.deepEqual(handle.wrapper.emitted("update:index"), [[2]]);
  assert.equal(root.getAttribute("data-state"), "closed");
  assert.equal(root.getAttribute("data-index"), "0");
  await handle.wrapper.setProps({ open: true, index: 2 });
  assert.equal(root.getAttribute("data-state"), "open");
  assert.equal(part(root, "lightbox-counter").textContent, "3 of 3");
  await handle.wrapper.setProps({ index: 99 });
  assert.equal(root.getAttribute("data-index"), "2", "out-of-range indexes clamp");
  handle.unmount();
});

test("preloads neighbouring images on the client while open", async () => {
  const created: string[] = [];
  const previousImage = globalThis.Image;
  class FakeImage {
    decoding = "auto";
    set src(value: string) {
      created.push(value);
    }
  }
  globalThis.Image = FakeImage as unknown as typeof Image;
  try {
    const handle = mountLightbox({
      loop: true,
      preload: 1,
      getPreloadSrc: (photo: Photo) => (photo.alt === "Harbor" ? undefined : photo.src),
    });
    await nextTick();
    assert.deepEqual(created, [], "closed viewers never preload");
    handle.exposes<LightboxRootExpose<Photo>>().openAt(1);
    await nextTick();
    assert.deepEqual(created, ["/photos/dunes.jpg"]);
    handle.exposes<LightboxRootExpose<Photo>>().next();
    await nextTick();
    assert.deepEqual(
      created,
      ["/photos/dunes.jpg", "/photos/forest.jpg"],
      "each source warms once",
    );
    handle.unmount();
  } finally {
    globalThis.Image = previousImage;
  }
});

test("localized messages label every control", async () => {
  const handle = mountLightbox(
    {
      defaultOpen: true,
      messages: {
        dialog: "写真ビューア",
        previous: "前へ",
        next: "次へ",
        close: "閉じる",
        thumbnails: "写真を選択",
        slide: "スライド",
        counter: (position: number, count: number) => `${count} 枚中 ${position} 枚目`,
        thumbnail: (position: number) => `${position} 枚目を表示`,
      },
    },
    {},
  );
  await nextTick();
  const root = handle.root();
  assert.equal(part(root, "dialog-content").getAttribute("aria-label"), "写真ビューア");
  assert.ok(handle.getByRole("button", { name: "前へ" }));
  assert.ok(handle.getByRole("button", { name: "次へ" }));
  assert.ok(handle.getByRole("button", { name: "閉じる" }));
  assert.ok(handle.getByRole("tablist", { name: "写真を選択" }));
  assert.equal(part(root, "lightbox-counter").textContent, "3 枚中 1 枚目");
  assert.equal(part(root, "lightbox-item").getAttribute("aria-roledescription"), "スライド");
  assert.equal(root.querySelector('[role="tab"]')?.getAttribute("aria-label"), "1 枚目を表示");
  handle.unmount();

  const labelled = mountLightbox({ defaultOpen: true }, { ariaLabelledby: "caption-id" });
  await nextTick();
  const dialog = part(labelled.root(), "dialog-content");
  assert.equal(dialog.getAttribute("aria-labelledby"), "caption-id");
  assert.equal(dialog.hasAttribute("aria-label"), false);
  labelled.unmount();
});

test("controls honor preventDefault and exposes typed imperative controls", async () => {
  const blocked = (event: MouseEvent) => event.preventDefault();
  const handle = mountInteraction(LightboxRoot, {
    props: { items: photos, defaultOpen: true },
    slots: {
      default: () => [
        h(LightboxTrigger, { index: 2, onClick: blocked }, () => "Open"),
        h(LightboxContent, { portalDisabled: true }, () => [
          h(LightboxNext, { onClick: blocked }),
          h(LightboxClose, { onClick: blocked }),
          h(LightboxThumbnail, { index: 2, onClick: blocked }),
        ]),
      ],
    },
  });
  await nextTick();
  for (const button of handle.root().querySelectorAll("button")) {
    button.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
  }
  await nextTick();
  const exposed = handle.exposes<LightboxRootExpose<Photo>>();
  assert.equal(exposed.open, true);
  assert.equal(exposed.index, 0);
  assert.equal(exposed.count, 3);
  assert.equal(exposed.item?.alt, "Harbor");
  assert.equal(exposed.items.length, 3);
  assert.equal(exposed.canGoPrevious, false);
  assert.equal(exposed.canGoNext, true);
  assert.equal(exposed.state, "open");
  assert.equal(exposed.next(), true);
  assert.equal(exposed.previous(), true);
  assert.equal(exposed.previous(), false);
  assert.equal(exposed.goTo(2), true);
  assert.equal(exposed.goTo(2), false);
  assert.equal(exposed.close(), true);
  assert.equal(exposed.close(), false);
  assert.equal(exposed.openAt(0), true);
  handle.unmount();

  const empty = mountInteraction(LightboxRoot, { props: { items: [] } });
  const emptyExposed = empty.exposes<LightboxRootExpose<Photo>>();
  assert.equal(emptyExposed.item, undefined);
  assert.equal(emptyExposed.goTo(1), false);
  assert.equal(emptyExposed.next(), false);
  empty.unmount();
});

test("teleports the viewer to the document body by default", async () => {
  const handle = mountInteraction(LightboxRoot, {
    props: { items: photos, defaultOpen: true },
    slots: { default: () => h(LightboxContent, null, () => h(LightboxCounter)) },
  });
  await nextTick();
  await nextTick();
  const counter = document.body.querySelector('[data-vize-ui="lightbox-counter"]');
  assert.ok(counter);
  assert.equal(handle.root().contains(counter), false);
  handle.unmount();
});

test("compound parts require a matching root provider", () => {
  for (const [component, props] of [
    [LightboxContent, {}],
    [LightboxItem, {}],
    [LightboxPrevious, {}],
    [LightboxNext, {}],
    [LightboxClose, {}],
    [LightboxCounter, {}],
    [LightboxThumbnails, {}],
    [LightboxThumbnail, { index: 0 }],
    [LightboxTrigger, { index: 0 }],
  ] as const) {
    assert.throws(() => mountInteraction(component, { props }), /VIZE_UI_CONTEXT_MISSING/);
  }
});

test("resolves indexes, preload neighbours, swipes, and messages", () => {
  assert.equal(resolveLightboxIndex(4, 3, false), 2);
  assert.equal(resolveLightboxIndex(-1, 3, true), 2);
  assert.equal(resolveLightboxIndex(Number.NaN, 3, false), 0);
  assert.equal(resolveLightboxIndex(2, 0, true), 0);
  assert.deepEqual(lightboxPreloadIndexes(0, 5, 2, false), [1, 2]);
  assert.deepEqual(lightboxPreloadIndexes(0, 5, 1, true), [1, 4]);
  assert.deepEqual(lightboxPreloadIndexes(0, 2, 3, true), [1]);
  assert.deepEqual(lightboxPreloadIndexes(1, 3, Number.NaN, false), []);
  const base = { threshold: 50, dir: "ltr", closeOnSwipeDown: true } as const;
  assert.equal(classifyLightboxSwipe({ ...base, deltaX: -80, deltaY: 10 }), "next");
  assert.equal(classifyLightboxSwipe({ ...base, deltaX: 80, deltaY: 10 }), "previous");
  assert.equal(classifyLightboxSwipe({ ...base, dir: "rtl", deltaX: 80, deltaY: 10 }), "next");
  assert.equal(classifyLightboxSwipe({ ...base, deltaX: -20, deltaY: 5 }), "none");
  assert.equal(classifyLightboxSwipe({ ...base, deltaX: 5, deltaY: 90 }), "close");
  assert.equal(classifyLightboxSwipe({ ...base, deltaX: 5, deltaY: -90 }), "none");
  assert.equal(
    classifyLightboxSwipe({ ...base, closeOnSwipeDown: false, deltaX: 5, deltaY: 90 }),
    "none",
  );
  assert.equal(resolveLightboxMessages(undefined).counter(2, 9), "2 of 9");
  assert.equal(resolveLightboxMessages({ close: "Done" }).close, "Done");
});
