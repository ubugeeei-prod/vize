import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  AlertDialogAction,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogOverlay,
  AlertDialogPortal,
  AlertDialogRoot,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "./alert-dialog.ts";

const AlertDialogSsrProbe = defineComponent({
  name: "AlertDialogSsrProbe",
  setup: () => () =>
    h(AlertDialogRoot, { defaultOpen: true, id: "discard-draft" }, () => [
      h(AlertDialogTrigger, null, () => "Discard"),
      h(AlertDialogPortal, null, () => [
        h(AlertDialogOverlay),
        h(AlertDialogContent, null, () => [
          h(AlertDialogTitle, null, () => "Discard draft?"),
          h(AlertDialogDescription, null, () => "This cannot be undone."),
          h(AlertDialogAction, null, () => "Discard"),
        ]),
      ]),
    ]),
});

test("renders deterministic alertdialog markup on the server", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(AlertDialogSsrProbe)),
    renderToString(createSSRApp(AlertDialogSsrProbe)),
  ]);

  assert.equal(left, right);
  assert.match(left, /data-vize-ui="dialog-root"/);
  assert.match(left, /data-vize-ui="alert-dialog-content"/);
  assert.match(left, /data-vize-ui="dialog-content"/);
  assert.match(left, /role="alertdialog"/);
  assert.match(left, /aria-modal="true"/);
  assert.match(left, /id="discard-draft-content"/);
  assert.match(left, /aria-labelledby="discard-draft-title"/);
  assert.match(left, /aria-describedby="discard-draft-description"/);
  assert.doesNotMatch(left, /role="dialog"|data-vize-scroll-locked|pointerdown|focusin|function/);
});
