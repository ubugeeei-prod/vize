import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";

import type { DialogRootExpose } from "./dialog.ts";
import DialogClose from "./dialog-close.vue";
import DialogContent from "./dialog-content.vue";
import DialogDescription from "./dialog-description.vue";
import DialogOverlay from "./dialog-overlay.vue";
import DialogPortal from "./dialog-portal.vue";
import DialogRoot from "./dialog-root.vue";
import DialogTitle from "./dialog-title.vue";
import DialogTrigger from "./dialog-trigger.vue";
import { mountInteraction } from "../../../testing/mount.ts";

async function settleDialog(): Promise<void> {
  await nextTick();
}

function dialogContent(): HTMLElement {
  const content = document.body.querySelector('[data-vize-ui="dialog-content"]');
  assert.ok(content instanceof HTMLElement);
  return content;
}

function dispatchPointerDown(target: Element): void {
  const ViewPointer = target.ownerDocument.defaultView?.PointerEvent;
  const event = ViewPointer
    ? new ViewPointer("pointerdown", { bubbles: true, cancelable: true, composed: true })
    : new MouseEvent("pointerdown", { bubbles: true, cancelable: true });
  target.dispatchEvent(event);
}

function dispatchEscape(target: Element): void {
  target.dispatchEvent(
    new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "Escape" }),
  );
}

function mountDialogFixture() {
  return mountInteraction(DialogRoot, {
    props: { id: "settings" },
    slots: {
      default: () => [
        h(DialogTrigger, null, () => "Open settings"),
        h(DialogPortal, null, () => [
          h(DialogOverlay),
          h(DialogContent, null, () => [
            h(DialogTitle, null, () => "Settings"),
            h(DialogDescription, null, () => "Change account settings."),
            h("button", { type: "button" }, "First"),
            h(DialogClose, null, () => "Close"),
          ]),
        ]),
      ],
    },
  });
}

test("opens uncontrolled content from the trigger and wires modal ARIA", async () => {
  const handle = mountDialogFixture();
  const trigger = handle.getByRole("button", { name: "Open settings" });
  trigger.focus();

  await handle.click(trigger);
  await settleDialog();
  const content = dialogContent();
  const close = document.body.querySelector('[data-vize-ui="dialog-close"]');
  const title = document.body.querySelector('[data-vize-ui="dialog-title"]');
  const description = document.body.querySelector('[data-vize-ui="dialog-description"]');
  const first = [...content.querySelectorAll("button")][0];

  assert.equal(handle.root().getAttribute("data-state"), "open");
  assert.equal(trigger.getAttribute("aria-expanded"), "true");
  assert.equal(trigger.getAttribute("aria-controls"), "settings-content");
  assert.equal(content.id, "settings-content");
  assert.equal(content.getAttribute("role"), "dialog");
  assert.equal(content.getAttribute("aria-modal"), "true");
  assert.equal(content.getAttribute("aria-labelledby"), "settings-title");
  assert.equal(content.getAttribute("aria-describedby"), "settings-description");
  assert.equal(title?.id, "settings-title");
  assert.equal(description?.id, "settings-description");
  assert.equal(document.activeElement, first);

  assert.ok(close instanceof HTMLButtonElement);
  await handle.click(close);
  await settleDialog();
  assert.equal(handle.root().getAttribute("data-state"), "closed");
  assert.equal(document.body.querySelector('[data-vize-ui="dialog-content"]'), null);
  assert.equal(document.activeElement, trigger);
  handle.unmount();
});

test("controlled open state emits requests without mutating before parent acceptance", async () => {
  const handle = mountInteraction(DialogRoot, {
    props: { open: false },
    slots: {
      default: () => [
        h(DialogTrigger, null, () => "Open"),
        h(DialogPortal, { disabled: true }, () =>
          h(DialogContent, null, () => h(DialogClose, null, () => "Close")),
        ),
      ],
    },
  });
  const trigger = handle.getByRole("button", { name: "Open" });

  await handle.click(trigger);
  await settleDialog();
  assert.deepEqual(handle.wrapper.emitted("update:open")?.at(-1), [true]);
  assert.equal(document.body.querySelector('[data-vize-ui="dialog-content"]'), null);

  await handle.wrapper.setProps({ open: true });
  await settleDialog();
  const content = dialogContent();
  dispatchEscape(content);
  await settleDialog();
  assert.deepEqual(handle.wrapper.emitted("update:open")?.at(-1), [false]);
  assert.ok(document.body.querySelector('[data-vize-ui="dialog-content"]'));

  await handle.wrapper.setProps({ open: false });
  await settleDialog();
  assert.equal(document.body.querySelector('[data-vize-ui="dialog-content"]'), null);
  handle.unmount();
});

test("keeps force-mounted closed layers hidden without document side effects", async () => {
  const handle = mountInteraction(DialogRoot, {
    props: { id: "details" },
    slots: {
      default: () =>
        h(DialogPortal, { disabled: true, forceMount: true }, () => [
          h(DialogOverlay, { forceMount: true }),
          h(DialogContent, { forceMount: true }, () => h(DialogTitle, null, () => "Details")),
        ]),
    },
  });
  await settleDialog();

  const portal = handle.root().querySelector('[data-vize-ui="dialog-portal"]');
  const overlay = handle.root().querySelector('[data-vize-ui="dialog-overlay"]');
  const contentHost = handle.root().querySelector('[data-vize-ui="dialog-content-host"]');
  const content = handle.root().querySelector('[data-vize-ui="dialog-content"]');

  assert.ok(portal instanceof HTMLElement);
  assert.ok(overlay instanceof HTMLElement);
  assert.ok(contentHost instanceof HTMLElement);
  assert.ok(content instanceof HTMLElement);
  assert.equal(portal.hasAttribute("hidden"), false);
  assert.equal(overlay.hasAttribute("hidden"), true);
  assert.equal(contentHost.hasAttribute("hidden"), true);
  assert.equal(content.getAttribute("data-state"), "closed");
  assert.equal(document.documentElement.getAttribute("data-vize-scroll-locked"), null);
  assert.equal(handle.root().querySelectorAll('[data-vize-ui="dialog-focus-guard"]').length, 0);
  handle.unmount();
});

test("non-modal content opts out of isolation while preserving dialog semantics", async () => {
  const outside = document.createElement("button");
  outside.textContent = "Outside";
  document.body.append(outside);
  const handle = mountInteraction(DialogRoot, {
    props: { defaultOpen: true, id: "inspect", modal: false },
    slots: {
      default: () =>
        h(DialogPortal, { disabled: true }, () =>
          h(DialogContent, { closeOnFocusOutside: false, closeOnPointerDownOutside: false }, () => [
            h(DialogTitle, null, () => "Inspect"),
            h("button", { type: "button" }, "Focusable"),
          ]),
        ),
    },
  });
  await settleDialog();
  const content = handle.root().querySelector('[data-vize-ui="dialog-content"]');

  assert.ok(content instanceof HTMLElement);
  assert.equal(content.getAttribute("aria-modal"), null);
  assert.equal(content.getAttribute("data-modal"), "false");
  assert.equal(outside.hasAttribute("inert"), false);
  assert.equal(document.documentElement.getAttribute("data-vize-scroll-locked"), null);
  assert.equal(handle.root().querySelectorAll('[data-vize-ui="dialog-focus-guard"]').length, 0);

  outside.focus();
  await settleDialog();
  assert.equal(document.activeElement, outside);
  assert.equal(content.isConnected, true);
  handle.unmount();
  outside.remove();
});

test("modal content traps focus, inerts outside content, and unlocks on backdrop dismissal", async () => {
  const outside = document.createElement("button");
  outside.textContent = "Outside";
  document.body.append(outside);
  const handle = mountDialogFixture();
  const trigger = handle.getByRole("button", { name: "Open settings" });

  trigger.focus();
  await handle.click(trigger);
  await settleDialog();
  const content = dialogContent();
  const overlay = document.body.querySelector('[data-vize-ui="dialog-overlay"]');
  const first = [...content.querySelectorAll("button")][0]!;
  const last = [...content.querySelectorAll("button")].at(-1)!;
  const guards = document.body.querySelectorAll('[data-vize-ui="dialog-focus-guard"]');

  assert.equal(outside.hasAttribute("inert"), true);
  assert.equal(overlay?.hasAttribute("inert"), false);
  assert.equal(document.documentElement.getAttribute("data-vize-scroll-locked"), "");
  assert.equal(guards.length, 2);
  assert.equal(guards[0]?.getAttribute("tabindex"), "0");

  last.focus();
  const tab = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "Tab" });
  last.dispatchEvent(tab);
  assert.equal(tab.defaultPrevented, true);
  assert.equal(document.activeElement, first);

  assert.ok(overlay instanceof HTMLElement);
  dispatchPointerDown(overlay);
  await settleDialog();
  assert.equal(document.body.querySelector('[data-vize-ui="dialog-content"]'), null);
  assert.equal(outside.hasAttribute("inert"), false);
  assert.equal(document.documentElement.getAttribute("data-vize-scroll-locked"), null);
  assert.equal(document.activeElement, trigger);
  handle.unmount();
  outside.remove();
});

test("routes Escape dismissal only to the top nested dialog", async () => {
  const handle = mountInteraction(DialogRoot, {
    props: { defaultOpen: true, id: "outer" },
    slots: {
      default: () =>
        h(DialogPortal, { disabled: true }, () =>
          h(DialogContent, { inertOutside: false, lockScroll: false, trapFocus: false }, () => [
            h(DialogTitle, null, () => "Outer"),
            h(DialogRoot, { defaultOpen: true, id: "inner" }, () =>
              h(DialogPortal, { disabled: true }, () =>
                h(DialogContent, { inertOutside: false, lockScroll: false, trapFocus: false }, () =>
                  h(DialogTitle, null, () => "Inner"),
                ),
              ),
            ),
          ]),
        ),
    },
  });
  await settleDialog();

  const outerContent = handle.root().querySelector("#outer-content");
  const innerContent = handle.root().querySelector("#inner-content");
  assert.ok(outerContent instanceof HTMLElement);
  assert.ok(innerContent instanceof HTMLElement);

  dispatchEscape(innerContent);
  await settleDialog();
  assert.ok(handle.root().querySelector("#outer-content") instanceof HTMLElement);
  assert.equal(handle.root().querySelector("#inner-content"), null);
  assert.equal(handle.root().getAttribute("data-state"), "open");
  handle.unmount();
});

test("exposes state methods and slot state for command-style composition", async () => {
  const seen: string[] = [];
  const handle = mountInteraction(DialogRoot, {
    slots: {
      default: (state) => {
        seen.push(`${state.state}:${state.open}:${state.modal}`);
        return h("output", state.state);
      },
    },
  });
  const exposed = handle.exposes<DialogRootExpose>();

  assert.equal(exposed.open, false);
  assert.equal(exposed.state, "closed");
  assert.match(exposed.contentId, /^vize-v-\d+-dialog-content$/);
  assert.equal(exposed.openDialog(), true);
  await nextTick();
  assert.equal(exposed.open, true);
  assert.equal(exposed.toggle(), true);
  await nextTick();
  assert.equal(exposed.open, false);
  assert.ok(seen.includes("closed:false:true"));
  assert.ok(seen.includes("open:true:true"));
  handle.unmount();
});

test("preventable dismissal events can retain an open dialog", async () => {
  const prevented = ref(false);
  const PreventProbe = defineComponent({
    setup: () => () =>
      h(DialogRoot, { defaultOpen: true }, () =>
        h(DialogPortal, { disabled: true }, () =>
          h(
            DialogContent,
            {
              onEscapeKeyDown(event) {
                event.preventDefault();
                prevented.value = true;
              },
            },
            () => h(DialogClose, null, () => "Close"),
          ),
        ),
      ),
  });
  const handle = mountInteraction(PreventProbe);
  await settleDialog();
  const content = dialogContent();

  dispatchEscape(content);
  await settleDialog();
  assert.equal(prevented.value, true);
  assert.ok(document.body.querySelector('[data-vize-ui="dialog-content"]'));
  handle.unmount();
});
