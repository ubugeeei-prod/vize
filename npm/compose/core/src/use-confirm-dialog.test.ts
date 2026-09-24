import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { useConfirmDialog } from "./use-confirm-dialog.ts";

void test("reveal resolves with the confirm payload", async () => {
  const dialog = useConfirmDialog<{ name: string }, "keep" | "delete">();
  const events: string[] = [];
  dialog.onReveal((data) => events.push(`reveal:${data.name}`));
  dialog.onConfirm((choice) => events.push(`confirm:${choice}`));

  assert.equal(dialog.state.value, "idle");
  const pending = dialog.reveal({ name: "a.txt" });
  assert.equal(dialog.reveal({ name: "ignored" }), pending);
  assert.equal(dialog.isRevealed.value, true);
  assert.equal(dialog.confirm("delete"), true);
  assert.deepEqual(await pending, { isCanceled: false, data: "delete" });
  assert.equal(dialog.state.value, "idle");
  assert.deepEqual(events, ["reveal:a.txt", "confirm:delete"]);
});

void test("cancel resolves as cancelled; commands while idle are ignored", async () => {
  const dialog = useConfirmDialog();
  assert.equal(dialog.confirm(), false);
  assert.equal(dialog.cancel(), false);
  let cancelled = 0;
  dialog.onCancel(() => (cancelled += 1));
  const pending = dialog.reveal();
  dialog.cancel();
  assert.deepEqual(await pending, { isCanceled: true, data: undefined });
  assert.equal(cancelled, 1);
});

void test("listeners registered in a scope stop with it", () => {
  const dialog = useConfirmDialog();
  let reveals = 0;
  const scope = effectScope();
  scope.run(() => dialog.onReveal(() => (reveals += 1)));
  scope.stop();
  void dialog.reveal();
  assert.equal(reveals, 0);
});
