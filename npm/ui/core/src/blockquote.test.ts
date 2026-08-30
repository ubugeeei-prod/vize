import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import type { BlockquoteExpose, BlockquoteSlotState } from "./blockquote.ts";
import Blockquote from "./blockquote.vue";
import { mountInteraction } from "./testing/mount.ts";

test("renders native blockquote by default without adding semantics or styling", async () => {
  const handle = mountInteraction(Blockquote, {
    slots: { default: "Design is how it works." },
  });
  const root = handle.root();

  assert.equal(root.tagName, "BLOCKQUOTE");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "blockquote");
  assert.equal(root.getAttribute("data-size"), "md");
  assert.equal(root.getAttribute("data-tone"), "neutral");
  assert.equal(root.getAttribute("cite"), null);
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.textContent, "Design is how it works.");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("mirrors quote hooks and native citation on the root", () => {
  const handle = mountInteraction(Blockquote, {
    props: {
      cite: "https://example.com/interview",
      size: "lg",
      tone: "accent",
    },
    slots: { default: "Make the important thing easy to quote." },
  });
  const root = handle.root();

  assert.equal(root.tagName, "BLOCKQUOTE");
  assert.equal(root.getAttribute("cite"), "https://example.com/interview");
  assert.equal(root.getAttribute("data-size"), "lg");
  assert.equal(root.getAttribute("data-tone"), "accent");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.textContent, "Make the important thing easy to quote.");
  handle.unmount();
});

test("keeps custom semantics and focus policy consumer owned through attrs", async () => {
  const handle = mountInteraction(Blockquote, {
    attrs: {
      "aria-label": "Release quote",
      role: "group",
      tabindex: "0",
    },
    props: {
      as: "figure",
      tone: "success",
    },
    slots: { default: "The migration completed cleanly." },
  });
  const root = handle.getByRole("group", { name: "Release quote" });

  assert.equal(root.tagName, "FIGURE");
  assert.equal(root.getAttribute("tabindex"), "0");
  assert.equal(root.getAttribute("data-vize-ui"), "blockquote");
  assert.equal(root.getAttribute("data-tone"), "success");
  assert.equal(root.textContent, "The migration completed cleanly.");
  assert.equal(await handle.tab(), root);
  handle.unmount();
});

test("passes slot state and exposes live blockquote state", async () => {
  const handle = mountInteraction(Blockquote, {
    props: {
      cite: "https://example.com/alpha",
      size: "sm",
      tone: "warning",
    },
    slots: {
      default: (state: BlockquoteSlotState) => `${state.size}:${state.tone}:${state.cite}`,
    },
  });
  const exposed = handle.exposes<BlockquoteExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.size, "sm");
  assert.equal(exposed.tone, "warning");
  assert.equal(exposed.cite, "https://example.com/alpha");
  assert.equal(handle.root().textContent, "sm:warning:https://example.com/alpha");

  await handle.wrapper.setProps({
    cite: "https://example.com/beta",
    size: "lg",
    tone: "danger",
  });
  assert.equal(exposed.size, "lg");
  assert.equal(exposed.tone, "danger");
  assert.equal(exposed.cite, "https://example.com/beta");
  assert.equal(handle.root().getAttribute("cite"), "https://example.com/beta");
  assert.equal(handle.root().getAttribute("data-size"), "lg");
  assert.equal(handle.root().getAttribute("data-tone"), "danger");
  assert.equal(handle.root().textContent, "lg:danger:https://example.com/beta");
  handle.unmount();
});
