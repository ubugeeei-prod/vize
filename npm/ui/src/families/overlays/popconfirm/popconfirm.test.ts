import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type { PopconfirmRootExpose } from "./popconfirm.ts";
import PopconfirmCancel from "./popconfirm-cancel.vue";
import PopconfirmConfirm from "./popconfirm-confirm.vue";
import PopconfirmContent from "./popconfirm-content.vue";
import PopconfirmRoot from "./popconfirm-root.vue";
import PopconfirmTrigger from "./popconfirm-trigger.vue";
import { mountInteraction } from "../../../testing/mount.ts";

async function settle(): Promise<void> {
  for (let index = 0; index < 4; index++) await nextTick();
}

function deferred(): {
  readonly promise: Promise<void>;
  readonly resolve: () => void;
  readonly reject: (reason: unknown) => void;
} {
  let resolve: () => void = () => undefined;
  let reject: (reason: unknown) => void = () => undefined;
  const promise = new Promise<void>((onResolve, onReject) => {
    resolve = onResolve;
    reject = onReject;
  });
  return { promise, reject, resolve };
}

function alertDialog(): HTMLElement | null {
  const element = document.querySelector('[data-vize-ui="popover-content"]');
  assert.ok(element === null || element instanceof HTMLElement);
  return element;
}

function mountPopconfirm(
  rootProps: Record<string, unknown> = {},
  contentProps: Record<string, unknown> = {},
) {
  return mountInteraction(PopconfirmRoot, {
    props: { id: "delete", ...rootProps },
    slots: {
      default: () => [
        h(PopconfirmTrigger, null, () => "Delete"),
        h(
          PopconfirmContent,
          {
            portalDisabled: true,
            title: "Delete this file?",
            description: "This cannot be undone.",
            ...contentProps,
          },
          () => [
            h(PopconfirmCancel, null, () => "Keep"),
            h(PopconfirmConfirm, null, ({ pending }: { readonly pending: boolean }) =>
              pending ? "Deleting" : "Delete file",
            ),
          ],
        ),
      ],
    },
  });
}

function button(name: string): HTMLButtonElement {
  const match = [...document.querySelectorAll("button")].find(
    (element) => element.textContent?.trim() === name,
  );
  assert.ok(match instanceof HTMLButtonElement, `button ${name}`);
  return match;
}

test("opens an alertdialog labelled by title and description with cancel focused", async () => {
  const handle = mountPopconfirm();
  const trigger = button("Delete");

  assert.equal(handle.root().getAttribute("data-vize-ui"), "popconfirm-root");
  assert.equal(trigger.getAttribute("data-vize-ui"), "popconfirm-trigger");
  assert.equal(trigger.getAttribute("aria-expanded"), "false");
  trigger.focus();
  await handle.click(trigger);
  await settle();
  const dialog = alertDialog();
  assert.ok(dialog);
  assert.equal(dialog.getAttribute("role"), "alertdialog");
  assert.equal(dialog.getAttribute("aria-labelledby"), "delete-title");
  assert.equal(dialog.getAttribute("aria-describedby"), "delete-description");
  assert.equal(document.getElementById("delete-title")?.textContent?.trim(), "Delete this file?");
  assert.ok(document.activeElement === button("Keep"), "cancel receives initial focus");

  handle.unmount();
});

test("initialFocus can target confirm or leave focus to the popover", async () => {
  const handle = mountPopconfirm({ defaultOpen: true }, { initialFocus: "confirm" });
  await settle();
  assert.ok(document.activeElement === button("Delete file"));
  handle.unmount();
});

test("synchronous confirm closes and emits confirmed", async () => {
  let calls = 0;
  const handle = mountPopconfirm({ defaultOpen: true, onConfirm: () => void calls++ });
  await settle();

  await handle.click(button("Delete file"));
  await settle();
  assert.equal(calls, 1);
  assert.equal(alertDialog(), null);
  assert.equal(handle.wrapper.emitted("confirmed")?.length, 1);
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[false]]);

  handle.unmount();
});

test("async confirm stays open and busy until the promise resolves", async () => {
  const work = deferred();
  const handle = mountPopconfirm({ defaultOpen: true, onConfirm: () => work.promise });
  await settle();

  await handle.click(button("Delete file"));
  await settle();
  const confirm = button("Deleting");
  const content = document.querySelector('[data-vize-ui="popconfirm-content"]');
  assert.equal(confirm.disabled, true);
  assert.equal(confirm.getAttribute("aria-busy"), "true");
  assert.equal(button("Keep").disabled, true);
  assert.equal(content?.getAttribute("aria-busy"), "true");
  assert.equal(handle.root().getAttribute("data-popconfirm-state"), "pending");

  const escape = new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true });
  confirm.dispatchEvent(escape);
  await settle();
  assert.ok(alertDialog(), "Escape cannot cancel a pending confirmation");

  work.resolve();
  await settle();
  assert.equal(alertDialog(), null);
  assert.equal(handle.wrapper.emitted("confirmed")?.length, 1);
  assert.equal(handle.wrapper.emitted("cancel"), undefined);

  handle.unmount();
});

test("rejected confirm stays open, exposes the error, and emits error", async () => {
  const work = deferred();
  const handle = mountPopconfirm({ defaultOpen: true, onConfirm: () => work.promise });
  await settle();

  await handle.click(button("Delete file"));
  work.reject(new Error("offline"));
  await settle();
  assert.ok(alertDialog());
  assert.equal(button("Delete file").disabled, false);
  const errors = handle.wrapper.emitted("error");
  assert.equal(errors?.length, 1);
  assert.ok(errors?.[0]?.[0] instanceof Error);
  assert.equal(
    document.querySelector('[data-vize-ui="popconfirm-content"]')?.getAttribute("data-error"),
    "true",
  );

  const thrower = mountPopconfirm({
    defaultOpen: true,
    id: "thrower",
    onConfirm: () => {
      throw new Error("sync failure");
    },
  });
  await settle();
  handle.unmount();
  await thrower.click(button("Delete file"));
  await settle();
  assert.equal(thrower.wrapper.emitted("error")?.length, 1);
  assert.ok(alertDialog());
  thrower.unmount();
});

test("cancel button, Escape, and outside pointer-down cancel with a reason", async () => {
  const handle = mountPopconfirm({ defaultOpen: true });
  await settle();
  await handle.click(button("Keep"));
  await settle();
  assert.equal(alertDialog(), null);
  assert.equal(handle.wrapper.emitted("cancel")?.[0]?.[0], "cancel-button");
  handle.unmount();

  const escaped = mountPopconfirm({ defaultOpen: true });
  await settle();
  const dialog = alertDialog();
  assert.ok(dialog);
  dialog.dispatchEvent(
    new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }),
  );
  await settle();
  assert.equal(alertDialog(), null);
  assert.equal(escaped.wrapper.emitted("cancel")?.[0]?.[0], "dismiss");
  escaped.unmount();

  const outside = document.createElement("button");
  document.body.append(outside);
  const clicked = mountPopconfirm({ defaultOpen: true });
  await settle();
  outside.dispatchEvent(
    new PointerEvent("pointerdown", { bubbles: true, cancelable: true, pointerType: "mouse" }),
  );
  await settle();
  assert.equal(alertDialog(), null);
  assert.equal(clicked.wrapper.emitted("cancel")?.[0]?.[0], "dismiss");
  clicked.unmount();
  outside.remove();
});

test("controlled open and root expose drive the confirmation", async () => {
  let root: PopconfirmRootExpose | null = null;
  const Probe = defineComponent({
    name: "PopconfirmExposeProbe",
    setup: () => () =>
      h(
        PopconfirmRoot,
        {
          id: "probe",
          ref: (value) => {
            root = value as PopconfirmRootExpose | null;
          },
        },
        () => [
          h(PopconfirmTrigger, null, () => "Archive"),
          h(PopconfirmContent, { portalDisabled: true, title: "Archive?" }, () => [
            h(PopconfirmConfirm, null, () => "Yes"),
          ]),
        ],
      ),
  });
  const handle = mountInteraction(Probe);
  if (root === null) assert.fail("PopconfirmRoot must expose its API");
  const exposed: PopconfirmRootExpose = root;

  assert.equal(exposed.state, "idle");
  assert.equal(exposed.titleId, "probe-title");
  assert.equal(await exposed.confirm(), false, "confirm is refused while closed");
  assert.equal(exposed.setOpen(true), true);
  await settle();
  assert.equal(alertDialog()?.getAttribute("aria-describedby"), null);
  assert.equal(exposed.cancel(), true);
  await settle();
  assert.equal(exposed.open, false);
  assert.equal(exposed.setOpen(true), true);
  await settle();
  assert.equal(await exposed.confirm(), true);
  assert.equal(exposed.open, false);
  handle.unmount();

  const controlled = mountPopconfirm({ open: false });
  await controlled.click(button("Delete"));
  await settle();
  assert.equal(alertDialog(), null);
  assert.deepEqual(controlled.wrapper.emitted("update:open"), [[true]]);
  await controlled.wrapper.setProps({ open: true });
  await settle();
  assert.ok(alertDialog());
  controlled.unmount();
});

test("disabled roots keep the trigger inert", async () => {
  const handle = mountPopconfirm({ disabled: true });
  const trigger = button("Delete");
  await handle.click(trigger);
  await settle();
  assert.equal(alertDialog(), null);
  handle.unmount();
});

test("parts require a Popconfirm root", () => {
  assert.throws(() => mountInteraction(PopconfirmConfirm), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(PopconfirmCancel), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(PopconfirmContent), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(PopconfirmTrigger), /VIZE_UI_CONTEXT_MISSING/);
});
