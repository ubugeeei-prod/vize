import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import {
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogDescription,
  AlertDialogOverlay,
  AlertDialogPortal,
  AlertDialogRoot,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "./alert-dialog.ts";
import AlertDialogContent from "./alert-dialog-content.vue";
import type { AlertDialogContentExpose, AlertDialogSlotState } from "./alert-dialog-types.ts";
import { mountInteraction } from "./testing/mount.ts";

async function settleAlertDialog(): Promise<void> {
  await nextTick();
}

function alertDialogContent(): HTMLElement {
  const content = document.body.querySelector('[role="alertdialog"]');
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

function mountAlertDialogFixture(
  contentProps: Partial<InstanceType<typeof AlertDialogContent>["$props"]> = {},
) {
  return mountInteraction(AlertDialogRoot, {
    props: { id: "delete-project" },
    slots: {
      default: () => [
        h(AlertDialogTrigger, null, () => "Delete project"),
        h(AlertDialogPortal, null, () => [
          h(AlertDialogOverlay),
          h(AlertDialogContent, contentProps, {
            default: (state: AlertDialogSlotState) => [
              h(AlertDialogTitle, null, () => "Delete project?"),
              h(AlertDialogDescription, null, () => `Permanent action: ${state.state}`),
              h(AlertDialogCancel, null, () => "Cancel"),
              h(AlertDialogAction, null, () => "Delete"),
            ],
          }),
        ]),
      ],
    },
  });
}

test("opens as a labelled modal alertdialog with explicit close actions", async () => {
  const handle = mountAlertDialogFixture();
  const trigger = handle.getByRole("button", { name: "Delete project" });
  trigger.focus();

  await handle.click(trigger);
  await settleAlertDialog();
  const content = alertDialogContent();
  const shell = document.body.querySelector('[data-vize-ui="alert-dialog-content"]');
  const overlay = document.body.querySelector('[data-vize-ui="dialog-overlay"]');
  const cancel = document.body.querySelector('[data-vize-ui="dialog-close"]');

  assert.ok(shell instanceof HTMLElement);
  assert.ok(cancel instanceof HTMLButtonElement);
  assert.equal(shell.getAttribute("data-state"), "open");
  assert.equal(shell.getAttribute("data-modal"), "true");
  assert.equal(content.getAttribute("data-vize-ui"), "dialog-content");
  assert.equal(content.getAttribute("role"), "alertdialog");
  assert.equal(content.getAttribute("aria-modal"), "true");
  assert.equal(content.getAttribute("aria-labelledby"), "delete-project-title");
  assert.equal(content.getAttribute("aria-describedby"), "delete-project-description");
  assert.match(content.textContent ?? "", /Permanent action: open/);
  assert.ok(overlay instanceof HTMLElement);

  dispatchPointerDown(overlay);
  await settleAlertDialog();
  assert.ok(document.body.querySelector('[role="alertdialog"]'));

  await handle.click(cancel);
  await settleAlertDialog();
  assert.equal(document.body.querySelector('[role="alertdialog"]'), null);
  assert.equal(document.activeElement, trigger);
  handle.unmount();
});

test("can opt into outside pointer dismissal", async () => {
  const events: string[] = [];
  const handle = mountAlertDialogFixture({
    closeOnPointerDownOutside: true,
    onDismiss: () => events.push("dismiss"),
    onPointerDownOutside: () => events.push("pointer"),
  });
  const trigger = handle.getByRole("button", { name: "Delete project" });

  await handle.click(trigger);
  await settleAlertDialog();
  const overlay = document.body.querySelector('[data-vize-ui="dialog-overlay"]');
  assert.ok(overlay instanceof HTMLElement);

  dispatchPointerDown(overlay);
  await settleAlertDialog();
  assert.deepEqual(events, ["pointer", "dismiss"]);
  assert.equal(document.body.querySelector('[role="alertdialog"]'), null);
  handle.unmount();
});

test("exposes Dialog content focus helpers through AlertDialogContent", async () => {
  let exposed: AlertDialogContentExpose | null = null;
  const Probe = {
    setup: () => () =>
      h(AlertDialogRoot, { defaultOpen: true }, () =>
        h(AlertDialogPortal, { disabled: true }, () =>
          h(
            AlertDialogContent,
            {
              ref(value) {
                exposed = value as AlertDialogContentExpose | null;
              },
            },
            () => h("button", { type: "button" }, "Confirm"),
          ),
        ),
      ),
  };
  const handle = mountInteraction(Probe);
  await settleAlertDialog();

  assert.ok(exposed);
  assert.equal(exposed.open, true);
  assert.equal(exposed.modal, true);
  assert.equal(exposed.state, "open");
  assert.ok(exposed.element instanceof HTMLDivElement);
  assert.equal(exposed.focusFirst()?.textContent, "Confirm");
  handle.unmount();
});
