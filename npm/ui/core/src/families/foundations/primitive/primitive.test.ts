import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import PrimitiveElement from "./primitive-element.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("renders the requested element with slotted content", () => {
  const handle = mountInteraction(PrimitiveElement, {
    props: { as: "section" },
    slots: { default: "Content" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "SECTION");
  assert.equal(root.getAttribute("data-vize-ui"), "primitive");
  assert.equal(root.textContent, "Content");
  handle.unmount();
});

test("defaults to a div and exposes the rendered element", () => {
  const handle = mountInteraction(PrimitiveElement, { slots: { default: "Content" } });

  assert.equal(handle.root().tagName, "DIV");
  const exposed = handle.exposes<{ element: HTMLElement | null }>();
  assert.ok(exposed.element === handle.root(), "the exposed element must be the rendered node");
  handle.unmount();
});

test("forwards every named slot to a component target", () => {
  const Card = defineComponent({
    setup:
      (_, { slots }) =>
      () =>
        h("article", [h("header", slots.title?.()), h("main", slots.default?.())]),
  });
  const handle = mountInteraction(PrimitiveElement, {
    props: { as: Card },
    slots: { title: "Heading", default: "Body" },
  });

  assert.equal(handle.root().querySelector("header")?.textContent, "Heading");
  assert.equal(handle.root().querySelector("main")?.textContent, "Body");
  handle.unmount();
});
