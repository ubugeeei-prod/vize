import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import type { HeadingExpose, HeadingSlotState } from "./heading.ts";
import Heading from "./heading.vue";
import { mountInteraction } from "./testing/mount.ts";

test("renders a semantic h2 by default without adding focus or styling", async () => {
  const handle = mountInteraction(Heading, {
    slots: { default: "Release notes" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "H2");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "heading");
  assert.equal(root.getAttribute("data-level"), "2");
  assert.equal(root.getAttribute("data-size"), "md");
  assert.equal(root.getAttribute("data-weight"), "semibold");
  assert.equal(root.getAttribute("data-tone"), "neutral");
  assert.equal(root.getAttribute("data-truncate"), "false");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-level"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.textContent, "Release notes");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("derives the native heading host from level when as is omitted", async () => {
  const handle = mountInteraction(Heading, {
    props: {
      level: 1,
      size: "2xl",
      tone: "accent",
      truncate: true,
      weight: "bold",
    },
    slots: { default: "Overview" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "H1");
  assert.equal(root.getAttribute("data-level"), "1");
  assert.equal(root.getAttribute("data-size"), "2xl");
  assert.equal(root.getAttribute("data-weight"), "bold");
  assert.equal(root.getAttribute("data-tone"), "accent");
  assert.equal(root.getAttribute("data-truncate"), "true");
  assert.equal(root.textContent, "Overview");

  await handle.wrapper.setProps({ level: 3 });
  assert.equal(handle.root().tagName, "H3");
  assert.equal(handle.root().getAttribute("data-level"), "3");
  handle.unmount();
});

test("keeps custom host semantics and focus policy consumer owned", async () => {
  const handle = mountInteraction(Heading, {
    attrs: {
      "aria-level": "4",
      role: "heading",
      tabindex: "0",
    },
    props: {
      as: "div",
      level: 4,
      tone: "success",
    },
    slots: { default: "Saved views" },
  });
  const root = handle.getByRole("heading");

  assert.equal(root.tagName, "DIV");
  assert.equal(root.getAttribute("aria-level"), "4");
  assert.equal(root.getAttribute("tabindex"), "0");
  assert.equal(root.getAttribute("data-vize-ui"), "heading");
  assert.equal(root.getAttribute("data-level"), "4");
  assert.equal(root.getAttribute("data-tone"), "success");
  assert.equal(root.textContent, "Saved views");
  assert.equal(await handle.tab(), root);
  handle.unmount();
});

test("passes slot state and exposes live heading state", async () => {
  const handle = mountInteraction(Heading, {
    props: {
      level: 5,
      size: "sm",
      tone: "warning",
      truncate: true,
      weight: "medium",
    },
    slots: {
      default: (state: HeadingSlotState) =>
        `${state.level}:${state.size}:${state.weight}:${state.tone}:${state.truncate}`,
    },
  });
  const exposed = handle.exposes<HeadingExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.level, 5);
  assert.equal(exposed.size, "sm");
  assert.equal(exposed.weight, "medium");
  assert.equal(exposed.tone, "warning");
  assert.equal(exposed.truncate, true);
  assert.equal(handle.root().textContent, "5:sm:medium:warning:true");

  await handle.wrapper.setProps({
    level: 6,
    size: "xs",
    tone: "danger",
    truncate: false,
    weight: "regular",
  });
  assert.equal(exposed.level, 6);
  assert.equal(exposed.size, "xs");
  assert.equal(exposed.weight, "regular");
  assert.equal(exposed.tone, "danger");
  assert.equal(exposed.truncate, false);
  assert.equal(handle.root().tagName, "H6");
  assert.equal(handle.root().getAttribute("data-level"), "6");
  assert.equal(handle.root().getAttribute("data-size"), "xs");
  assert.equal(handle.root().getAttribute("data-weight"), "regular");
  assert.equal(handle.root().getAttribute("data-tone"), "danger");
  assert.equal(handle.root().getAttribute("data-truncate"), "false");
  assert.equal(handle.root().textContent, "6:xs:regular:danger:false");
  handle.unmount();
});
