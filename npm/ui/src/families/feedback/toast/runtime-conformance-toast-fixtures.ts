import assert from "node:assert/strict";

import { defineComponent, h } from "vue";

import {
  ToastAction,
  ToastClose,
  ToastDescription,
  ToastProvider,
  ToastRoot,
  ToastTitle,
  ToastViewport,
  useToast,
} from "./toast.ts";
import type { ToastViewportSlotState } from "./toast.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const Seed = defineComponent({
  name: "RuntimeToastSeed",
  setup() {
    useToast().toast({
      title: "Runtime toast",
      description: "Rendered on the server",
      action: { label: "Open", altText: "Open from the sidebar" },
    });
    return () => null;
  },
});

function toaster(): ReturnType<typeof h> {
  return h(ToastProvider, null, () => [
    h(Seed),
    h(ToastViewport, null, {
      default: ({ toast }: ToastViewportSlotState) =>
        h(ToastRoot, { toast }, () => [
          h(ToastTitle),
          h(ToastDescription),
          h(ToastAction, { altText: "Open from the sidebar" }),
          h(ToastClose),
        ]),
    }),
  ]);
}

function assertPart(host: HTMLElement, part: string, tag: string): void {
  const element = host.querySelector(`[data-vize-ui="${part}"]`);
  assert.ok(element instanceof HTMLElement, `${part} must hydrate`);
  assert.equal(element.tagName, tag);
}

export const toastRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "toast-provider",
    sourceFile: "families/feedback/toast/toast-provider.vue",
    render: toaster,
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="toast-provider"/);
    },
    assertHydratedDom: (host) => assertPart(host, "toast-provider", "DIV"),
  },
  {
    name: "toast-viewport",
    sourceFile: "families/feedback/toast/toast-viewport.vue",
    render: toaster,
    assertServerMarkup(html) {
      assert.match(html, /<section[^>]*aria-label="Notifications \(F8\)"/);
      assert.match(html, /data-vize-ui="live-region"/);
    },
    assertHydratedDom: (host) => assertPart(host, "toast-viewport", "SECTION"),
  },
  {
    name: "toast-root",
    sourceFile: "families/feedback/toast/toast-root.vue",
    render: toaster,
    assertServerMarkup(html) {
      assert.match(html, /<li[^>]*role="status"[^>]*aria-live="off"/);
    },
    assertHydratedDom(host) {
      assertPart(host, "toast", "LI");
      assert.equal(host.querySelector('[data-vize-ui="toast"]')?.getAttribute("role"), "status");
    },
  },
  {
    name: "toast-title",
    sourceFile: "families/feedback/toast/toast-title.vue",
    render: toaster,
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="toast-title"[\s\S]{0,80}Runtime toast/);
    },
    assertHydratedDom: (host) => assertPart(host, "toast-title", "DIV"),
  },
  {
    name: "toast-description",
    sourceFile: "families/feedback/toast/toast-description.vue",
    render: toaster,
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="toast-description"[\s\S]{0,80}Rendered on the server/);
    },
    assertHydratedDom: (host) => assertPart(host, "toast-description", "DIV"),
  },
  {
    name: "toast-action",
    sourceFile: "families/feedback/toast/toast-action.vue",
    render: toaster,
    assertServerMarkup(html) {
      assert.match(html, /data-alt-text="Open from the sidebar"/);
    },
    assertHydratedDom: (host) => assertPart(host, "toast-action", "BUTTON"),
  },
  {
    name: "toast-close",
    sourceFile: "families/feedback/toast/toast-close.vue",
    render: toaster,
    assertServerMarkup(html) {
      assert.match(html, /aria-label="Dismiss notification"/);
    },
    assertHydratedDom: (host) => assertPart(host, "toast-close", "BUTTON"),
  },
];
