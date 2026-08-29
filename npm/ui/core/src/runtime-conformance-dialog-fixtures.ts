import assert from "node:assert/strict";

import { h } from "vue";

import {
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
  DialogTrigger,
} from "./dialog.ts";
import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";

function dialog(children: () => unknown): ReturnType<typeof h> {
  return h(DialogRoot, { defaultOpen: true, id: "runtime-dialog" }, children);
}

function assertRoot(host: HTMLElement): void {
  const root = host.querySelector('[data-vize-ui="dialog-root"]');
  assert.ok(root instanceof HTMLElement);
  assert.equal(root.getAttribute("data-state"), "open");
}

export const dialogRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "dialog-root",
    sourceFile: "dialog-root.vue",
    render: () => dialog(() => "Dialog"),
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-dialog"/);
      assert.match(html, /data-vize-ui="dialog-root"/);
      assert.match(html, /data-state="open"/);
    },
    assertHydratedDom: assertRoot,
  },
  {
    name: "dialog-trigger",
    sourceFile: "dialog-trigger.vue",
    render: () => dialog(() => h(DialogTrigger, null, () => "Open dialog")),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="dialog-trigger"/);
      assert.match(html, /aria-controls="runtime-dialog-content"/);
      assert.match(html, /aria-expanded="true"/);
    },
    assertHydratedDom(host) {
      const trigger = host.querySelector('[data-vize-ui="dialog-trigger"]');
      assert.ok(trigger instanceof HTMLButtonElement);
      assert.equal(trigger.getAttribute("aria-haspopup"), "dialog");
    },
  },
  {
    name: "dialog-portal",
    sourceFile: "dialog-portal.vue",
    render: () => dialog(() => h(DialogPortal, { disabled: true }, () => "Layer")),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="dialog-portal"/);
      assert.match(html, /data-vize-ui="portal-host"/);
      assert.match(html, /Layer/);
    },
    assertHydratedDom(host) {
      assert.ok(host.querySelector('[data-vize-ui="dialog-portal"]') instanceof HTMLElement);
      assert.ok(host.querySelector('[data-vize-ui="portal"]') instanceof HTMLElement);
    },
  },
  {
    name: "dialog-overlay",
    sourceFile: "dialog-overlay.vue",
    render: () => dialog(() => h(DialogOverlay, null, () => "Backdrop")),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="dialog-overlay"/);
      assert.match(html, /aria-hidden="true"/);
    },
    assertHydratedDom(host) {
      const overlay = host.querySelector('[data-vize-ui="dialog-overlay"]');
      assert.ok(overlay instanceof HTMLElement);
      assert.equal(overlay.getAttribute("aria-hidden"), "true");
    },
  },
  {
    name: "dialog-content",
    sourceFile: "dialog-content.vue",
    render: () => dialog(() => h(DialogContent, { lockScroll: false }, () => "Content")),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="dialog-content"/);
      assert.match(html, /role="dialog"/);
      assert.match(html, /aria-modal="true"/);
    },
    assertHydratedDom(host) {
      const content = host.querySelector('[data-vize-ui="dialog-content"]');
      assert.ok(content instanceof HTMLElement);
      assert.equal(content.getAttribute("role"), "dialog");
    },
  },
  {
    name: "dialog-title",
    sourceFile: "dialog-title.vue",
    render: () => dialog(() => h(DialogTitle, null, () => "Title")),
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-dialog-title"/);
      assert.match(html, /data-vize-ui="dialog-title"/);
      assert.match(html, /Title/);
    },
    assertHydratedDom(host) {
      assert.ok(host.querySelector('[data-vize-ui="dialog-title"]') instanceof HTMLHeadingElement);
    },
  },
  {
    name: "dialog-description",
    sourceFile: "dialog-description.vue",
    render: () => dialog(() => h(DialogDescription, null, () => "Description")),
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-dialog-description"/);
      assert.match(html, /data-vize-ui="dialog-description"/);
    },
    assertHydratedDom(host) {
      assert.ok(host.querySelector('[data-vize-ui="dialog-description"]') instanceof HTMLElement);
    },
  },
  {
    name: "dialog-close",
    sourceFile: "dialog-close.vue",
    render: () => dialog(() => h(DialogClose, null, () => "Close")),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="dialog-close"/);
      assert.match(html, /type="button"/);
    },
    assertHydratedDom(host) {
      assert.ok(host.querySelector('[data-vize-ui="dialog-close"]') instanceof HTMLButtonElement);
    },
  },
];
