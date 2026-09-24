import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import ConfirmProvider from "./confirm-provider.vue";
import { useConfirm } from "./confirm.ts";
import type { ConfirmApi, ConfirmSlotProps } from "./confirm.ts";
import { mountInteraction } from "../../../testing/mount.ts";

interface Invoice {
  readonly number: string;
}

async function settle(): Promise<void> {
  await nextTick();
  await nextTick();
}

function mountConfirm(
  props: Record<string, unknown> = {},
  content?: (props: ConfirmSlotProps) => unknown,
): { readonly handle: ReturnType<typeof mountInteraction>; readonly api: ConfirmApi<Invoice> } {
  let api: ConfirmApi<Invoice> | null = null;
  const Consumer = defineComponent({
    name: "ConfirmConsumer",
    setup() {
      api = useConfirm<Invoice>();
      return () => h("button", { type: "button", class: "opener" }, "Delete invoice");
    },
  });
  const slots: Record<string, unknown> = { default: () => h(Consumer) };
  if (content) slots.content = content;
  const handle = mountInteraction(ConfirmProvider, {
    props: { portalDisabled: true, id: "confirm", ...props },
    slots,
  });
  if (api === null) assert.fail("useConfirm must resolve inside the provider");
  return { api, handle };
}

function dialog(root: HTMLElement): HTMLElement | null {
  return root.querySelector('[role="alertdialog"]');
}

function button(root: HTMLElement, action: string): HTMLButtonElement {
  const found = root.querySelector(`[data-confirm-action="${action}"]`);
  assert.ok(found instanceof HTMLButtonElement, `missing ${action} action`);
  return found;
}

test("confirm() opens a labelled alertdialog and resolves true from the accepting action", async () => {
  const { api, handle } = mountConfirm();
  const root = handle.root();
  const opener = root.querySelector<HTMLButtonElement>(".opener");
  assert.ok(opener);
  opener.focus();
  assert.equal(dialog(root), null);
  assert.equal(root.getAttribute("data-state"), "idle");

  const result = api.confirm({
    title: "Delete invoice?",
    description: "This cannot be undone.",
    confirmLabel: "Delete",
    destructive: true,
    data: { number: "INV-7" },
  });
  await settle();

  const content = dialog(root);
  assert.ok(content);
  assert.equal(root.getAttribute("data-state"), "pending");
  assert.equal(root.getAttribute("data-pending"), "1");
  assert.equal(content.getAttribute("aria-modal"), "true");
  assert.equal(content.getAttribute("aria-labelledby"), "confirm-title");
  assert.equal(content.getAttribute("aria-describedby"), "confirm-description");
  assert.match(content.textContent ?? "", /Delete invoice\?/);
  const accept = button(root, "confirm");
  assert.equal(accept.textContent?.trim(), "Delete");
  assert.equal(accept.getAttribute("data-destructive"), "true");
  assert.ok(document.activeElement === button(root, "cancel"), "least destructive action focused");

  await handle.click(accept);
  assert.equal(await result, true);
  await settle();
  assert.equal(dialog(root), null);
  assert.equal(root.getAttribute("data-state"), "idle");
  assert.ok(document.activeElement === opener, "focus returns to the opener");
  assert.equal(handle.wrapper.emitted("settle")?.[0]?.[1], "confirm");
  handle.unmount();
});

test("cancel and Escape resolve false, and missing descriptions omit aria-describedby", async () => {
  const { api, handle } = mountConfirm({ cancelLabel: "Keep", confirmLabel: "Yes" });
  const root = handle.root();

  const cancelled = api.confirm({ title: "Archive?" });
  await settle();
  const content = dialog(root);
  assert.ok(content);
  assert.equal(
    content.getAttribute("aria-describedby") ?? "",
    "",
    "no dangling description reference",
  );
  assert.equal(button(root, "cancel").textContent?.trim(), "Keep");
  assert.equal(button(root, "confirm").textContent?.trim(), "Yes");
  await handle.click(button(root, "cancel"));
  assert.equal(await cancelled, false);

  const escaped = api.confirm({ title: "Archive again?" });
  await settle();
  const escapedContent = dialog(root);
  assert.ok(escapedContent);
  escapedContent.dispatchEvent(
    new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "Escape" }),
  );
  assert.equal(await escaped, false);
  handle.unmount();
});

test("requests queue in FIFO order and cancelAll settles every pending request", async () => {
  const { api, handle } = mountConfirm();
  const root = handle.root();
  const first = api.confirm({ title: "First?" });
  const second = api.confirm({ title: "Second?" });
  const third = api.confirm({ title: "Third?" });
  await settle();
  assert.equal(api.pending.value, 3);
  assert.match(dialog(root)?.textContent ?? "", /First\?/);

  await handle.click(button(root, "confirm"));
  assert.equal(await first, true);
  await settle();
  assert.equal(api.pending.value, 2);
  assert.match(dialog(root)?.textContent ?? "", /Second\?/);

  api.cancelAll();
  assert.equal(await second, false);
  assert.equal(await third, false);
  await settle();
  assert.equal(dialog(root), null);
  handle.unmount();
});

test("choose() resolves the typed action value or null", async () => {
  const { api, handle } = mountConfirm();
  const root = handle.root();
  const choice = api.choose({
    title: "Unsaved changes",
    actions: [
      { value: "save", label: "Save" },
      { value: "discard", label: "Discard", destructive: true },
    ],
  });
  await settle();
  const content = root.querySelector('[data-vize-ui="alert-dialog-content"]');
  assert.equal(content?.getAttribute("data-confirm-kind"), "choose");
  assert.equal(content?.getAttribute("data-destructive"), "true");
  await handle.click(button(root, "discard"));
  const value: "discard" | "save" | null = await choice;
  assert.equal(value, "discard");

  const dismissed = api.choose({ title: "Again", actions: [{ value: "save", label: "Save" }] });
  await settle();
  await handle.click(button(root, "cancel"));
  assert.equal(await dismissed, null);
  handle.unmount();
});

test("invalid options throw stable diagnostics", async () => {
  const { api, handle } = mountConfirm();
  await assert.rejects(api.confirm({ title: " " }), /VIZE_UI_CONFIRM_OPTION/);
  await assert.rejects(
    api.choose({
      title: "Pick",
      actions: [
        { value: "a", label: "A" },
        { value: "a", label: "Again" },
      ],
    }),
    /VIZE_UI_CONFIRM_OPTION/,
  );
  assert.equal(api.pending.value, 0);
  handle.unmount();
});

test("the content slot replaces the body and settles through resolve and cancel", async () => {
  const seen: unknown[] = [];
  const { api, handle } = mountConfirm({}, ({ request, resolve, cancel }) => {
    seen.push(request.data);
    return [
      h("h2", { id: "custom-title" }, request.title),
      h(
        "button",
        { type: "button", class: "custom-yes", onClick: () => resolve("confirm") },
        "Yes",
      ),
      h("button", { type: "button", class: "custom-no", onClick: cancel }, "No"),
      h("button", { type: "button", class: "custom-bad", onClick: () => resolve("bogus") }, "?"),
    ];
  });
  const root = handle.root();
  const accepted = api.confirm({ title: "Custom", data: { number: "INV-9" } });
  await settle();
  assert.deepEqual(seen.at(-1), { number: "INV-9" });
  root.querySelector<HTMLButtonElement>(".custom-bad")?.click();
  await settle();
  assert.ok(dialog(root), "unknown values do not settle");
  root.querySelector<HTMLButtonElement>(".custom-yes")?.click();
  assert.equal(await accepted, true);

  const rejected = api.confirm({ title: "Custom 2" });
  await settle();
  root.querySelector<HTMLButtonElement>(".custom-no")?.click();
  assert.equal(await rejected, false);
  handle.unmount();
});

test("unmounting the provider resolves pending and later requests as not confirmed", async () => {
  const { api, handle } = mountConfirm();
  const pending = api.confirm({ title: "Leave?" });
  await settle();
  handle.unmount();
  assert.equal(await pending, false);
  assert.equal(await api.confirm({ title: "After unmount" }), false);
});

test("the provider teleports the dialog to the body by default", async () => {
  const { api, handle } = mountConfirm({ portalDisabled: false });
  const request = api.confirm({ title: "Portalled?" });
  await settle();
  const content = document.body.querySelector('[role="alertdialog"]');
  assert.ok(content instanceof HTMLElement);
  assert.equal(handle.root().contains(content), false);
  const accept = document.body.querySelector('[data-confirm-action="confirm"]');
  assert.ok(accept instanceof HTMLButtonElement);
  accept.click();
  assert.equal(await request, true);
  handle.unmount();
});

test("useConfirm outside a provider throws a stable diagnostic", () => {
  const Orphan = defineComponent({
    name: "ConfirmOrphan",
    setup() {
      useConfirm();
      return () => h("div");
    },
  });
  assert.throws(() => mountInteraction(Orphan), /VIZE_UI_CONFIRM_PROVIDER_MISSING/);
});
