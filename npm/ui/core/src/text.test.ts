import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import type { TextExpose, TextSlotState } from "./text.ts";
import Text from "./text.vue";
import { mountInteraction } from "./testing/mount.ts";

test("renders neutral body text by default without adding semantics or styling", async () => {
  const handle = mountInteraction(Text, {
    slots: { default: "Readable copy" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "SPAN");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "text");
  assert.equal(root.getAttribute("data-size"), "md");
  assert.equal(root.getAttribute("data-weight"), "regular");
  assert.equal(root.getAttribute("data-tone"), "neutral");
  assert.equal(root.getAttribute("data-truncate"), "false");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.textContent, "Readable copy");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("mirrors typography tokens and truncation intent on a custom host", () => {
  const handle = mountInteraction(Text, {
    props: {
      as: "p",
      size: "xl",
      tone: "accent",
      truncate: true,
      weight: "semibold",
    },
    slots: { default: "Launch notes" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "P");
  assert.equal(root.getAttribute("data-size"), "xl");
  assert.equal(root.getAttribute("data-weight"), "semibold");
  assert.equal(root.getAttribute("data-tone"), "accent");
  assert.equal(root.getAttribute("data-truncate"), "true");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.textContent, "Launch notes");
  handle.unmount();
});

test("keeps ARIA and focus policy consumer owned through attrs", async () => {
  const handle = mountInteraction(Text, {
    attrs: {
      "aria-live": "polite",
      role: "status",
      tabindex: "0",
    },
    props: {
      tone: "success",
    },
    slots: { default: "Saved" },
  });
  const root = handle.getByRole("status");

  assert.equal(root.getAttribute("aria-live"), "polite");
  assert.equal(root.getAttribute("tabindex"), "0");
  assert.equal(root.getAttribute("data-vize-ui"), "text");
  assert.equal(root.getAttribute("data-tone"), "success");
  assert.equal(root.textContent, "Saved");
  assert.equal(await handle.tab(), root);
  handle.unmount();
});

test("passes slot state and exposes live text state", async () => {
  const handle = mountInteraction(Text, {
    props: {
      size: "lg",
      tone: "warning",
      truncate: true,
      weight: "medium",
    },
    slots: {
      default: (state: TextSlotState) =>
        `${state.size}:${state.weight}:${state.tone}:${state.truncate}`,
    },
  });
  const exposed = handle.exposes<TextExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.size, "lg");
  assert.equal(exposed.weight, "medium");
  assert.equal(exposed.tone, "warning");
  assert.equal(exposed.truncate, true);
  assert.equal(handle.root().textContent, "lg:medium:warning:true");

  await handle.wrapper.setProps({
    size: "xs",
    tone: "danger",
    truncate: false,
    weight: "bold",
  });
  assert.equal(exposed.size, "xs");
  assert.equal(exposed.weight, "bold");
  assert.equal(exposed.tone, "danger");
  assert.equal(exposed.truncate, false);
  assert.equal(handle.root().getAttribute("data-size"), "xs");
  assert.equal(handle.root().getAttribute("data-weight"), "bold");
  assert.equal(handle.root().getAttribute("data-tone"), "danger");
  assert.equal(handle.root().getAttribute("data-truncate"), "false");
  assert.equal(handle.root().textContent, "xs:bold:danger:false");
  handle.unmount();
});
