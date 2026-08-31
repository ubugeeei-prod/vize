import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type { TooltipRootExpose } from "./tooltip.ts";
import TooltipContent from "./tooltip-content.vue";
import TooltipRoot from "./tooltip-root.vue";
import TooltipTrigger from "./tooltip-trigger.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function settleTooltip(): Promise<void> {
  await nextTick();
  await wait(0);
  await nextTick();
}

function tooltipContent(scope: ParentNode = document.body): HTMLElement | null {
  const content = scope.querySelector('[data-vize-ui="tooltip-content"]');
  assert.ok(content == null || content instanceof HTMLElement);
  return content;
}

function pointer(target: Element, type: string, init: Partial<PointerEventInit> = {}): Event {
  const ViewPointer = target.ownerDocument.defaultView?.PointerEvent;
  const event = ViewPointer
    ? new ViewPointer(type, {
        bubbles: false,
        cancelable: true,
        composed: true,
        pointerType: "mouse",
        ...init,
      })
    : new MouseEvent(type, { bubbles: false, cancelable: true, ...init });
  target.dispatchEvent(event);
  return event;
}

function focus(target: Element, type: "blur" | "focus"): FocusEvent {
  const event = new FocusEvent(type, { bubbles: false, cancelable: true });
  target.dispatchEvent(event);
  return event;
}

function escape(target: Element): KeyboardEvent {
  const event = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key: "Escape",
  });
  target.dispatchEvent(event);
  return event;
}

function mountTooltip(
  rootProps: Record<string, unknown> = {},
  triggerProps: Record<string, unknown> = {},
  contentProps: Record<string, unknown> = {},
) {
  return mountInteraction(TooltipRoot, {
    props: { delayDuration: 0, id: "help", ...rootProps },
    slots: {
      default: () => [
        h(TooltipTrigger, { ariaLabel: "More info", ...triggerProps }, () => "More info"),
        h(TooltipContent, { portalDisabled: true, ...contentProps }, ({ placement }) =>
          h("span", { "data-placement": placement }, "Helpful copy"),
        ),
      ],
    },
  });
}

test("opens from hover and focus with tooltip ARIA and data hooks", async () => {
  const handle = mountTooltip({}, {}, { placement: "bottom" });
  const trigger = handle.getByRole("button", { name: "More info" }) as HTMLButtonElement;

  assert.equal(trigger.id, "help-trigger");
  assert.equal(trigger.getAttribute("aria-describedby"), null);
  assert.equal(tooltipContent(handle.root()), null);

  pointer(trigger, "pointerenter");
  await settleTooltip();
  const content = tooltipContent(handle.root());
  assert.ok(content);
  assert.equal(handle.root().getAttribute("data-state"), "open");
  assert.equal(trigger.getAttribute("aria-describedby"), "help-content");
  assert.equal(trigger.getAttribute("data-state"), "open");
  assert.equal(content.id, "help-content");
  assert.equal(content.getAttribute("role"), "tooltip");
  assert.equal(content.getAttribute("part"), "content");
  assert.equal(content.getAttribute("data-state"), "open");
  assert.equal(content.querySelector("[data-placement]")?.getAttribute("data-placement"), "bottom");

  pointer(trigger, "pointerleave");
  await settleTooltip();
  assert.equal(trigger.getAttribute("aria-describedby"), null);
  assert.equal(tooltipContent(handle.root()), null);

  focus(trigger, "focus");
  await settleTooltip();
  assert.ok(tooltipContent(handle.root()));
  focus(trigger, "blur");
  await settleTooltip();
  assert.equal(tooltipContent(handle.root()), null);
  handle.unmount();
});

test("honors open delay and skips it after a recent close", async () => {
  const handle = mountTooltip({ delayDuration: 25, skipDelayDuration: 80 });
  const trigger = handle.getByRole("button", { name: "More info" });

  pointer(trigger, "pointerenter");
  await settleTooltip();
  assert.equal(tooltipContent(handle.root()), null);

  await wait(35);
  await settleTooltip();
  assert.ok(tooltipContent(handle.root()));

  pointer(trigger, "pointerleave");
  await settleTooltip();
  assert.equal(tooltipContent(handle.root()), null);

  pointer(trigger, "pointerenter");
  await settleTooltip();
  assert.ok(tooltipContent(handle.root()));
  handle.unmount();
});

test("cancels pending opens and closes from trigger pointer or Escape", async () => {
  const pending = mountTooltip({ delayDuration: 25, skipDelayDuration: 0 });
  const pendingTrigger = pending.getByRole("button", { name: "More info" });

  pointer(pendingTrigger, "pointerenter");
  await settleTooltip();
  pointer(pendingTrigger, "pointerleave");
  await wait(35);
  await settleTooltip();
  assert.equal(tooltipContent(pending.root()), null);
  pending.unmount();

  const pointerClose = mountTooltip({ defaultOpen: true });
  await settleTooltip();
  const pointerTrigger = pointerClose.getByRole("button", { name: "More info" });
  assert.ok(tooltipContent(pointerClose.root()));
  pointer(pointerTrigger, "pointerdown");
  await settleTooltip();
  assert.equal(tooltipContent(pointerClose.root()), null);
  pointerClose.unmount();

  const escapeClose = mountTooltip({ defaultOpen: true });
  await settleTooltip();
  const escapeTrigger = escapeClose.getByRole("button", { name: "More info" });
  assert.ok(tooltipContent(escapeClose.root()));
  const escapeEvent = escape(escapeTrigger);
  await settleTooltip();
  assert.equal(escapeEvent.defaultPrevented, true);
  assert.equal(tooltipContent(escapeClose.root()), null);
  escapeClose.unmount();
});

test("controlled open state emits requests before parent acceptance", async () => {
  const handle = mountTooltip({ open: false });
  const trigger = handle.getByRole("button", { name: "More info" });

  pointer(trigger, "pointerenter");
  await settleTooltip();
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[true]]);
  assert.equal(tooltipContent(handle.root()), null);

  await handle.wrapper.setProps({ open: true });
  await settleTooltip();
  assert.ok(tooltipContent(handle.root()));

  pointer(trigger, "pointerleave");
  await settleTooltip();
  assert.deepEqual(handle.wrapper.emitted("update:open")?.at(-1), [false]);
  assert.ok(tooltipContent(handle.root()));

  await handle.wrapper.setProps({ open: false });
  await settleTooltip();
  assert.equal(tooltipContent(handle.root()), null);
  handle.unmount();
});

test("Escape dismissal is preventable and otherwise closes the tooltip", async () => {
  const prevented = mountTooltip(
    { defaultOpen: true },
    {},
    {
      onEscapeKeyDown(event: { preventDefault: () => void }) {
        event.preventDefault();
      },
    },
  );
  await settleTooltip();
  const preventedContent = tooltipContent(prevented.root());
  assert.ok(preventedContent);
  escape(preventedContent);
  await settleTooltip();
  assert.ok(tooltipContent(prevented.root()));
  prevented.unmount();

  const handle = mountTooltip({ defaultOpen: true });
  await settleTooltip();
  const content = tooltipContent(handle.root());
  assert.ok(content);
  escape(content);
  await settleTooltip();
  assert.equal(handle.root().getAttribute("data-state"), "closed");
  assert.equal(tooltipContent(handle.root()), null);
  handle.unmount();
});

test("disabled root and trigger block activation and closing is requested on disable", async () => {
  const rootDisabled = mountTooltip({ disabled: true });
  const rootTrigger = rootDisabled.getByRole("button", { name: "More info" }) as HTMLButtonElement;
  assert.equal(rootTrigger.disabled, true);
  pointer(rootTrigger, "pointerenter");
  focus(rootTrigger, "focus");
  await settleTooltip();
  assert.equal(rootDisabled.wrapper.emitted("update:open"), undefined);
  assert.equal(tooltipContent(rootDisabled.root()), null);
  rootDisabled.unmount();

  const triggerDisabled = mountTooltip({}, { disabled: true });
  const localTrigger = triggerDisabled.getByRole("button", {
    name: "More info",
  }) as HTMLButtonElement;
  assert.equal(localTrigger.disabled, true);
  pointer(localTrigger, "pointerenter");
  await settleTooltip();
  assert.equal(triggerDisabled.wrapper.emitted("update:open"), undefined);
  triggerDisabled.unmount();

  const handle = mountTooltip({ defaultOpen: true });
  await settleTooltip();
  await handle.wrapper.setProps({ disabled: true });
  await settleTooltip();
  assert.equal(handle.root().getAttribute("data-disabled"), "true");
  assert.equal(handle.root().getAttribute("data-state"), "closed");
  assert.deepEqual(handle.wrapper.emitted("update:open")?.at(-1), [false]);
  handle.unmount();
});

test("force-mounted closed content remains hidden without trigger description", () => {
  const handle = mountTooltip({}, {}, { forceMount: true });
  const trigger = handle.getByRole("button", { name: "More info" });
  const contentHost = handle.root().querySelector('[data-vize-ui="tooltip-content-host"]');
  const content = tooltipContent(handle.root());

  assert.ok(contentHost instanceof HTMLElement);
  assert.ok(content);
  assert.equal(trigger.getAttribute("aria-describedby"), null);
  assert.equal(contentHost.hidden, true);
  assert.equal(content.hidden, true);
  assert.equal(content.getAttribute("data-state"), "closed");
  handle.unmount();
});

test("exposes state methods and requires a matching root provider", async () => {
  const handle = mountTooltip({ delayDuration: -1, skipDelayDuration: -1 });
  const root = handle.exposes<TooltipRootExpose>();

  assert.equal(root.delayDuration, 0);
  assert.equal(root.skipDelayDuration, 0);
  assert.equal(root.open, false);
  assert.equal(root.state, "closed");
  assert.equal(root.scheduleOpen(), true);
  await settleTooltip();
  assert.equal(root.open, true);
  assert.equal(root.close(), true);
  await settleTooltip();
  assert.equal(root.openTooltip(), true);
  await settleTooltip();
  assert.equal(root.setOpen(false), true);
  assert.match(root.contentId, /^help-content$/);
  handle.unmount();

  assert.throws(() => mountInteraction(TooltipTrigger), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(TooltipContent), /VIZE_UI_CONTEXT_MISSING/);
});
