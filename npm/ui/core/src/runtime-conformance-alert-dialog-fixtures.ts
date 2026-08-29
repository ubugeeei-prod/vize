import assert from "node:assert/strict";

import { h } from "vue";

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
import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";

export const alertDialogRuntimeFixture: RuntimeFixture = {
  name: "alert-dialog",
  sourceFile: "alert-dialog-content.vue",
  render: () =>
    h(AlertDialogRoot, { defaultOpen: true, id: "confirm-delete" }, () => [
      h(AlertDialogTrigger, null, () => "Delete"),
      h(AlertDialogPortal, null, () => [
        h(AlertDialogOverlay),
        h(AlertDialogContent, null, () => [
          h(AlertDialogTitle, null, () => "Delete item?"),
          h(AlertDialogDescription, null, () => "This action cannot be undone."),
          h(AlertDialogAction, null, () => "Delete"),
        ]),
      ]),
    ]),
  assertServerMarkup(html) {
    assert.match(html, /data-vize-ui="alert-dialog-content"/);
    assert.match(html, /data-vize-ui="dialog-content"/);
    assert.match(html, /role="alertdialog"/);
    assert.match(html, /aria-labelledby="confirm-delete-title"/);
    assert.match(html, /aria-describedby="confirm-delete-description"/);
  },
  assertHydratedDom(host) {
    const shell = host.querySelector('[data-vize-ui="alert-dialog-content"]');
    const content = host.querySelector('[role="alertdialog"]');
    assert.ok(shell instanceof HTMLElement);
    assert.ok(content instanceof HTMLElement);
    assert.equal(content.getAttribute("aria-modal"), "true");
    assert.equal(content.getAttribute("aria-labelledby"), "confirm-delete-title");
    assert.equal(content.textContent?.includes("Delete item?"), true);
  },
};
