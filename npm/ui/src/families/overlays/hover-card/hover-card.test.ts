import assert from "node:assert/strict";

import { afterEach, beforeEach, test, vi } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type { HoverCardRootExpose } from "./hover-card.ts";
import HoverCardContent from "./hover-card-content.vue";
import HoverCardRoot from "./hover-card-root.vue";
import HoverCardTrigger from "./hover-card-trigger.vue";
import { mountInteraction } from "../../../testing/mount.ts";

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

async function flush(ms = 0): Promise<void> {
  await nextTick();
  vi.advanceTimersByTime(ms);
  await nextTick();
  await nextTick();
}

function pointer(target: EventTarget, type: string, init: Partial<PointerEventInit> = {}): Event {
  const event = new PointerEvent(type, {
    bubbles: type === "pointerdown" || type === "pointermove",
    cancelable: true,
    composed: true,
    pointerType: "mouse",
    ...init,
  });
  target.dispatchEvent(event);
  return event;
}

function focusEvent(target: Element, type: "blur" | "focus", relatedTarget: Element | null = null) {
  target.dispatchEvent(new FocusEvent(type, { bubbles: false, relatedTarget }));
}

function content(): HTMLElement | null {
  const element = document.querySelector('[data-vize-ui="hover-card-content"]');
  assert.ok(element === null || element instanceof HTMLElement);
  return element;
}

function mountHoverCard(
  rootProps: Record<string, unknown> = {},
  triggerProps: Record<string, unknown> = {},
  contentProps: Record<string, unknown> = {},
) {
  return mountInteraction(HoverCardRoot, {
    props: { id: "profile", ...rootProps },
    slots: {
      default: () => [
        h(HoverCardTrigger, { href: "/users/ada", ...triggerProps }, () => "@ada"),
        h(HoverCardContent, { portalDisabled: true, ...contentProps }, ({ reason }) =>
          h("p", { "data-slot-reason": reason ?? "none" }, "Ada Lovelace"),
        ),
      ],
    },
  });
}

function triggerOf(handle: ReturnType<typeof mountHoverCard>): HTMLAnchorElement {
  const trigger = handle.getByRole("link", { name: "@ada" });
  assert.ok(trigger instanceof HTMLAnchorElement);
  return trigger;
}

test("hover opens after openDelay with link semantics and describedby wiring", async () => {
  const handle = mountHoverCard();
  const trigger = triggerOf(handle);

  assert.equal(trigger.id, "profile-trigger");
  assert.equal(trigger.getAttribute("href"), "/users/ada");
  assert.equal(trigger.getAttribute("data-state"), "closed");
  assert.equal(handle.root().tagName, "SPAN", "root stays valid inside phrasing content");

  pointer(trigger, "pointerenter");
  await flush(699);
  assert.equal(content(), null, "the card waits for the full open delay");
  await flush(1);
  const card = content();
  assert.ok(card);
  assert.equal(card.id, "profile-content");
  assert.equal(card.getAttribute("data-reason"), "hover");
  assert.equal(card.querySelector("[data-slot-reason]")?.getAttribute("data-slot-reason"), "hover");
  assert.equal(trigger.getAttribute("aria-describedby"), "profile-content");
  assert.equal(trigger.getAttribute("data-state"), "open");
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[true]]);

  handle.unmount();
});

test("leaving closes after closeDelay unless the pointer returns or enters the card", async () => {
  const handle = mountHoverCard({ openDelay: 0, closeDelay: 300 });
  const trigger = triggerOf(handle);

  pointer(trigger, "pointerenter");
  await flush();
  assert.ok(content());
  pointer(trigger, "pointerleave");
  await flush(200);
  pointer(trigger, "pointerenter");
  await flush(300);
  assert.ok(content(), "returning to the trigger cancels the pending close");

  pointer(trigger, "pointerleave");
  const card = content();
  assert.ok(card);
  pointer(card, "pointerenter");
  await flush(500);
  assert.ok(content(), "hovering the card keeps it open");
  pointer(card, "pointerleave");
  await flush(299);
  assert.ok(content());
  await flush(1);
  assert.equal(content(), null);
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[true], [false]]);

  handle.unmount();
});

test("pointer grace keeps the card open while travelling toward it", async () => {
  const handle = mountHoverCard({ openDelay: 0, closeDelay: 100 });
  const trigger = triggerOf(handle);

  pointer(trigger, "pointerenter");
  await flush();
  const card = content();
  assert.ok(card);
  card.getBoundingClientRect = () => new DOMRect(0, 100, 200, 100);

  pointer(trigger, "pointerleave", { clientX: 50, clientY: 10 });
  await flush(50);
  pointer(document, "pointermove", { clientX: 60, clientY: 60 });
  await flush(200);
  assert.ok(content(), "moving inside the safe triangle cancels the close");

  pointer(document, "pointermove", { clientX: 600, clientY: 20 });
  await flush(99);
  assert.ok(content());
  await flush(1);
  assert.equal(content(), null, "leaving the corridor closes after closeDelay");

  handle.unmount();
});

test("focus opens the card and blur closes it unless focus moves into the card", async () => {
  const handle = mountHoverCard({ openDelay: 100, closeDelay: 0 });
  const trigger = triggerOf(handle);

  focusEvent(trigger, "focus");
  await flush(100);
  const card = content();
  assert.ok(card);
  assert.equal(card.getAttribute("data-reason"), "focus");

  const inside = document.createElement("button");
  card.append(inside);
  focusEvent(trigger, "blur", inside);
  await flush();
  assert.ok(content(), "blurring into the card keeps it open");

  focusEvent(trigger, "blur", document.body);
  await flush();
  assert.equal(content(), null);

  handle.unmount();
});

test("Escape and outside pointer-down dismiss immediately", async () => {
  const handle = mountHoverCard({ defaultOpen: true });
  const trigger = triggerOf(handle);
  await flush();
  assert.ok(content());

  const escape = new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true });
  trigger.dispatchEvent(escape);
  await flush();
  assert.equal(escape.defaultPrevented, true);
  assert.equal(content(), null);
  handle.unmount();

  const outside = document.createElement("button");
  document.body.append(outside);
  const dismissed: string[] = [];
  const second = mountHoverCard(
    { defaultOpen: true },
    {},
    { onDismiss: (event: { reason: string }) => dismissed.push(event.reason) },
  );
  await flush();
  assert.ok(content());
  pointer(outside, "pointerdown");
  await flush();
  assert.equal(content(), null);
  assert.equal(dismissed.length, 1);
  second.unmount();
  outside.remove();
});

test("touch is ignored by default and keeps native link activation", async () => {
  const handle = mountHoverCard({ openDelay: 0 });
  const trigger = triggerOf(handle);

  pointer(trigger, "pointerenter", { pointerType: "touch" });
  pointer(trigger, "pointerdown", { pointerType: "touch" });
  await flush(1000);
  assert.equal(content(), null);
  const click = new MouseEvent("click", { bubbles: true, cancelable: true });
  trigger.dispatchEvent(click);
  assert.equal(click.defaultPrevented, false);

  handle.unmount();
});

test("long-press touch opens the card and suppresses the follow-up click", async () => {
  const handle = mountHoverCard({ touchBehavior: "long-press", longPressDelay: 400 });
  const trigger = triggerOf(handle);

  pointer(trigger, "pointerdown", { pointerType: "touch", clientX: 5, clientY: 5 });
  pointer(trigger, "pointermove", { pointerType: "touch", clientX: 30, clientY: 5 });
  await flush(400);
  assert.equal(content(), null, "moving past the touch slop cancels the long press");

  pointer(trigger, "pointerdown", { pointerType: "touch", clientX: 5, clientY: 5 });
  const menu = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });
  trigger.dispatchEvent(menu);
  assert.equal(menu.defaultPrevented, true);
  await flush(400);
  const card = content();
  assert.ok(card);
  assert.equal(card.getAttribute("data-reason"), "long-press");
  pointer(trigger, "pointerup", { pointerType: "touch" });
  const click = new MouseEvent("click", { bubbles: true, cancelable: true });
  trigger.dispatchEvent(click);
  assert.equal(click.defaultPrevented, true);
  const nextClick = new MouseEvent("click", { bubbles: true, cancelable: true });
  trigger.dispatchEvent(nextClick);
  assert.equal(nextClick.defaultPrevented, false, "only the long-press click is suppressed");

  handle.unmount();
});

test("disabled roots ignore intent and close an open card", async () => {
  const handle = mountHoverCard({ openDelay: 0, defaultOpen: true });
  const trigger = triggerOf(handle);
  await flush();
  assert.ok(content());

  await handle.wrapper.setProps({ disabled: true });
  await flush();
  assert.equal(content(), null);
  assert.equal(handle.root().getAttribute("data-disabled"), "true");
  assert.equal(trigger.getAttribute("data-disabled"), "true");
  pointer(trigger, "pointerenter");
  focusEvent(trigger, "focus");
  await flush(1000);
  assert.equal(content(), null);

  handle.unmount();
});

test("controlled open follows the parent after emitting requests", async () => {
  const handle = mountHoverCard({ open: false, openDelay: 0 });
  const trigger = triggerOf(handle);

  pointer(trigger, "pointerenter");
  await flush();
  assert.equal(content(), null);
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[true]]);
  await handle.wrapper.setProps({ open: true });
  await flush();
  assert.ok(content());

  handle.unmount();
});

test("root exposes delays and programmatic scheduling", async () => {
  let root: HoverCardRootExpose | null = null;
  const Probe = defineComponent({
    name: "HoverCardExposeProbe",
    setup: () => () =>
      h(
        HoverCardRoot,
        {
          openDelay: 50,
          closeDelay: -5,
          ref: (value) => {
            root = value as HoverCardRootExpose | null;
          },
        },
        () => [
          h(HoverCardTrigger, { as: "button", type: "button" }, () => "Preview"),
          h(HoverCardContent, { portalDisabled: true }, () => "Preview card"),
        ],
      ),
  });
  const handle = mountInteraction(Probe);
  if (root === null) assert.fail("HoverCardRoot must expose its API");
  const exposed: HoverCardRootExpose = root;

  assert.equal(exposed.openDelay, 50);
  assert.equal(exposed.closeDelay, 0, "invalid delays normalize to zero");
  assert.match(exposed.contentId, /-hover-card-content$/);
  assert.equal(exposed.scheduleOpen(), true);
  assert.equal(exposed.cancelPending(), true);
  await flush(100);
  assert.equal(exposed.open, false);
  assert.equal(exposed.scheduleOpen(), true);
  await flush(50);
  assert.equal(exposed.open, true);
  assert.equal(exposed.reason, "programmatic");
  assert.equal(exposed.scheduleClose(), true);
  await flush();
  assert.equal(exposed.open, false);
  assert.equal(exposed.setOpen(true), true);
  await flush();
  assert.equal(exposed.state, "open");
  assert.ok(handle.getByRole("button", { name: "Preview" }));

  handle.unmount();
});

test("trigger and content require a HoverCard root", () => {
  assert.throws(() => mountInteraction(HoverCardTrigger), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(HoverCardContent), /VIZE_UI_CONTEXT_MISSING/);
});
