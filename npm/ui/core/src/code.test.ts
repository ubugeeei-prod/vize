import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import type { CodeExpose, CodeSlotState } from "./code.ts";
import Code from "./code.vue";
import { mountInteraction } from "./testing/mount.ts";

test("renders native code by default without adding semantics or styling", async () => {
  const handle = mountInteraction(Code, {
    slots: { default: "const value = 1;" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "CODE");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "code");
  assert.equal(root.getAttribute("data-size"), "md");
  assert.equal(root.getAttribute("data-variant"), "inline");
  assert.equal(root.getAttribute("data-tone"), "neutral");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.textContent, "const value = 1;");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("mirrors code presentation hooks on a custom host", () => {
  const handle = mountInteraction(Code, {
    props: {
      as: "pre",
      size: "lg",
      tone: "accent",
      variant: "block",
    },
    slots: { default: "vp test run src/code.test.ts" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "PRE");
  assert.equal(root.getAttribute("data-size"), "lg");
  assert.equal(root.getAttribute("data-variant"), "block");
  assert.equal(root.getAttribute("data-tone"), "accent");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.textContent, "vp test run src/code.test.ts");
  handle.unmount();
});

test("keeps custom semantics and focus policy consumer owned through attrs", async () => {
  const handle = mountInteraction(Code, {
    attrs: {
      "aria-label": "Package subpath",
      role: "term",
      tabindex: "0",
    },
    props: {
      as: "span",
      tone: "success",
      variant: "snippet",
    },
    slots: { default: "@vizejs/ui/code" },
  });
  const root = handle.getByRole("term", { name: "Package subpath" });

  assert.equal(root.tagName, "SPAN");
  assert.equal(root.getAttribute("tabindex"), "0");
  assert.equal(root.getAttribute("data-vize-ui"), "code");
  assert.equal(root.getAttribute("data-variant"), "snippet");
  assert.equal(root.getAttribute("data-tone"), "success");
  assert.equal(root.textContent, "@vizejs/ui/code");
  assert.equal(await handle.tab(), root);
  handle.unmount();
});

test("passes slot state and exposes live code state", async () => {
  const handle = mountInteraction(Code, {
    props: {
      size: "sm",
      tone: "warning",
      variant: "inline",
    },
    slots: {
      default: (state: CodeSlotState) => `${state.size}:${state.variant}:${state.tone}`,
    },
  });
  const exposed = handle.exposes<CodeExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.size, "sm");
  assert.equal(exposed.variant, "inline");
  assert.equal(exposed.tone, "warning");
  assert.equal(handle.root().textContent, "sm:inline:warning");

  await handle.wrapper.setProps({
    size: "lg",
    tone: "danger",
    variant: "block",
  });
  assert.equal(exposed.size, "lg");
  assert.equal(exposed.variant, "block");
  assert.equal(exposed.tone, "danger");
  assert.equal(handle.root().getAttribute("data-size"), "lg");
  assert.equal(handle.root().getAttribute("data-variant"), "block");
  assert.equal(handle.root().getAttribute("data-tone"), "danger");
  assert.equal(handle.root().textContent, "lg:block:danger");
  handle.unmount();
});
