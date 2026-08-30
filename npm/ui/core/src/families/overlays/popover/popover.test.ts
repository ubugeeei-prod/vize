import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type { PopoverRootExpose } from "./popover.ts";
import PopoverArrow from "./popover-arrow.vue";
import PopoverContent from "./popover-content.vue";
import PopoverRoot from "./popover-root.vue";
import PopoverTrigger from "./popover-trigger.vue";
import { mountInteraction } from "../../../testing/mount.ts";

async function settlePopover(): Promise<void> {
  await nextTick();
  await nextTick();
}

function popoverContent(scope: ParentNode = document.body): HTMLElement | null {
  const content = scope.querySelector('[data-vize-ui="popover-content"]');
  assert.ok(content == null || content instanceof HTMLElement);
  return content;
}

function dispatchPointerDown(target: Element): void {
  const ViewPointer = target.ownerDocument.defaultView?.PointerEvent;
  const event = ViewPointer
    ? new ViewPointer("pointerdown", {
        bubbles: true,
        cancelable: true,
        composed: true,
        pointerType: "mouse",
      })
    : new MouseEvent("pointerdown", { bubbles: true, cancelable: true });
  target.dispatchEvent(event);
}

function dispatchEscape(target: Element): KeyboardEvent {
  const event = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key: "Escape",
  });
  target.dispatchEvent(event);
  return event;
}

function mountPopover(
  rootProps: Record<string, unknown> = {},
  triggerProps: Record<string, unknown> = {},
  contentProps: Record<string, unknown> = {},
) {
  return mountInteraction(PopoverRoot, {
    props: { id: "filters", ...rootProps },
    slots: {
      default: () => [
        h(PopoverTrigger, { ariaLabel: "Filters", ...triggerProps }, () => "Filters"),
        h(PopoverContent, { portalDisabled: true, ...contentProps }, ({ align, side }) => [
          h("button", { type: "button" }, "Today"),
          h("output", { "data-placement": `${side}:${align}` }, "Selected filters"),
          h(PopoverArrow, null, () => h("span", "Arrow")),
        ]),
      ],
    },
  });
}

test("opens uncontrolled content from the trigger with popover ARIA and data hooks", async () => {
  const handle = mountPopover({}, {}, { placement: "bottom-start" });
  const trigger = handle.getByRole("button", { name: "Filters" }) as HTMLButtonElement;

  assert.equal(handle.root().getAttribute("data-state"), "closed");
  assert.equal(trigger.id, "filters-trigger");
  assert.equal(trigger.getAttribute("aria-haspopup"), "dialog");
  assert.equal(trigger.getAttribute("aria-expanded"), "false");
  assert.equal(trigger.getAttribute("aria-controls"), "filters-content");
  assert.equal(popoverContent(handle.root()), null);

  trigger.focus();
  await handle.click(trigger);
  await settlePopover();
  const content = popoverContent(handle.root());
  const arrow = handle.root().querySelector('[data-vize-ui="popover-arrow"]');

  assert.ok(content);
  assert.equal(handle.root().getAttribute("data-state"), "open");
  assert.equal(trigger.getAttribute("aria-expanded"), "true");
  assert.equal(trigger.getAttribute("data-state"), "open");
  assert.equal(content.id, "filters-content");
  assert.equal(content.getAttribute("role"), "dialog");
  assert.equal(content.getAttribute("aria-modal"), null);
  assert.equal(content.getAttribute("part"), "content");
  assert.equal(content.getAttribute("data-state"), "open");
  assert.equal(content.getAttribute("data-modal"), "false");
  assert.equal(content.getAttribute("data-side"), "bottom");
  assert.equal(content.getAttribute("data-align"), "start");
  assert.equal(
    content.querySelector("[data-placement]")?.getAttribute("data-placement"),
    "bottom:start",
  );
  assert.ok(arrow instanceof HTMLElement);
  assert.equal(arrow.getAttribute("part"), "arrow");
  assert.equal(document.activeElement, content.querySelector("button"));

  await handle.click(trigger);
  await settlePopover();
  assert.equal(handle.root().getAttribute("data-state"), "closed");
  assert.equal(popoverContent(handle.root()), null);
  assert.equal(document.activeElement, trigger);
  handle.unmount();
});

test("controlled open state emits requests without mutating before parent acceptance", async () => {
  const handle = mountPopover({ open: false });
  const trigger = handle.getByRole("button", { name: "Filters" });

  await handle.click(trigger);
  await settlePopover();
  assert.deepEqual(handle.wrapper.emitted("update:open")?.at(-1), [true]);
  assert.equal(popoverContent(handle.root()), null);

  await handle.wrapper.setProps({ open: true });
  await settlePopover();
  assert.ok(popoverContent(handle.root()));

  const content = popoverContent(handle.root());
  assert.ok(content);
  dispatchEscape(content);
  await settlePopover();
  assert.deepEqual(handle.wrapper.emitted("update:open")?.at(-1), [false]);
  assert.ok(popoverContent(handle.root()));

  await handle.wrapper.setProps({ open: false });
  await settlePopover();
  assert.equal(popoverContent(handle.root()), null);
  handle.unmount();
});

test("Escape and outside pointer dismissal are preventable and otherwise close", async () => {
  const prevented = mountPopover(
    { defaultOpen: true },
    {},
    {
      onEscapeKeyDown(event: { preventDefault: () => void }) {
        event.preventDefault();
      },
    },
  );
  await settlePopover();
  const preventedContent = popoverContent(prevented.root());
  assert.ok(preventedContent);
  dispatchEscape(preventedContent);
  await settlePopover();
  assert.ok(popoverContent(prevented.root()));
  prevented.unmount();

  const handle = mountPopover({ defaultOpen: true });
  await settlePopover();
  const content = popoverContent(handle.root());
  assert.ok(content);
  dispatchEscape(content);
  await settlePopover();
  assert.equal(popoverContent(handle.root()), null);
  handle.unmount();

  const outside = document.createElement("button");
  outside.textContent = "Outside";
  document.body.append(outside);
  const outsideHandle = mountPopover({ defaultOpen: true });
  await settlePopover();
  dispatchPointerDown(outside);
  await settlePopover();
  assert.equal(popoverContent(outsideHandle.root()), null);
  outsideHandle.unmount();
  outside.remove();
});

test("modal content traps focus, inerts outside content, and restores trigger focus", async () => {
  const outside = document.createElement("button");
  outside.textContent = "Outside";
  document.body.append(outside);
  const handle = mountPopover({ modal: true });
  const trigger = handle.getByRole("button", { name: "Filters" });

  trigger.focus();
  await handle.click(trigger);
  await settlePopover();
  const content = popoverContent(handle.root());
  assert.ok(content);
  const first = content.querySelector("button");
  assert.ok(first instanceof HTMLButtonElement);
  const guards = handle.root().querySelectorAll('[data-vize-ui="popover-focus-guard"]');

  assert.equal(content.getAttribute("aria-modal"), "true");
  assert.equal(content.getAttribute("data-modal"), "true");
  assert.equal(outside.hasAttribute("inert"), true);
  assert.equal(document.documentElement.getAttribute("data-vize-scroll-locked"), "");
  assert.equal(guards.length, 2);
  assert.equal(document.activeElement, first);

  outside.focus();
  assert.equal(document.activeElement, first);

  dispatchPointerDown(outside);
  await settlePopover();
  assert.equal(popoverContent(handle.root()), null);
  assert.equal(outside.hasAttribute("inert"), false);
  assert.equal(document.documentElement.getAttribute("data-vize-scroll-locked"), null);
  assert.equal(document.activeElement, trigger);
  handle.unmount();
  outside.remove();
});

test("disabled root and trigger block activation and request closing on disable", async () => {
  const rootDisabled = mountPopover({ disabled: true });
  const rootTrigger = rootDisabled.getByRole("button", { name: "Filters" }) as HTMLButtonElement;
  assert.equal(rootTrigger.disabled, true);
  await handleClick(rootDisabled, rootTrigger);
  assert.equal(rootDisabled.wrapper.emitted("update:open"), undefined);
  assert.equal(popoverContent(rootDisabled.root()), null);
  rootDisabled.unmount();

  const triggerDisabled = mountPopover({}, { disabled: true });
  const localTrigger = triggerDisabled.getByRole("button", {
    name: "Filters",
  }) as HTMLButtonElement;
  assert.equal(localTrigger.disabled, true);
  await handleClick(triggerDisabled, localTrigger);
  assert.equal(triggerDisabled.wrapper.emitted("update:open"), undefined);
  triggerDisabled.unmount();

  const handle = mountPopover({ defaultOpen: true });
  await settlePopover();
  await handle.wrapper.setProps({ disabled: true });
  await settlePopover();
  assert.equal(handle.root().getAttribute("data-disabled"), "true");
  assert.equal(handle.root().getAttribute("data-state"), "closed");
  assert.deepEqual(handle.wrapper.emitted("update:open")?.at(-1), [false]);
  handle.unmount();
});

test("force-mounted closed content remains hidden without document controllers", async () => {
  const handle = mountPopover({}, {}, { forceMount: true });
  await settlePopover();
  const trigger = handle.getByRole("button", { name: "Filters" });
  const contentHost = handle.root().querySelector('[data-vize-ui="popover-content-host"]');
  const content = popoverContent(handle.root());

  assert.ok(contentHost instanceof HTMLElement);
  assert.ok(content);
  assert.equal(trigger.getAttribute("aria-expanded"), "false");
  assert.equal(contentHost.hidden, true);
  assert.equal(content.hidden, true);
  assert.equal(content.getAttribute("data-state"), "closed");
  assert.equal(document.documentElement.getAttribute("data-vize-scroll-locked"), null);
  assert.equal(handle.root().querySelectorAll('[data-vize-ui="popover-focus-guard"]').length, 0);
  handle.unmount();
});

test("exposes state methods and focus helpers", async () => {
  const handle = mountPopover({ defaultOpen: false });
  const root = handle.exposes<PopoverRootExpose>();

  assert.equal(root.open, false);
  assert.equal(root.state, "closed");
  assert.equal(root.contentId, "filters-content");
  assert.equal(root.openPopover(), true);
  await settlePopover();
  assert.equal(root.open, true);
  const content = popoverContent(handle.root());
  assert.ok(content);
  assert.equal(document.activeElement, content.querySelector("button"));
  assert.equal(root.toggle(), true);
  await settlePopover();
  assert.equal(root.open, false);
  handle.unmount();
});

async function handleClick(
  handle: ReturnType<typeof mountPopover>,
  trigger: HTMLButtonElement,
): Promise<void> {
  await handle.click(trigger);
  await settlePopover();
}
