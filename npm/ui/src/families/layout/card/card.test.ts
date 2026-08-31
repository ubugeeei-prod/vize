import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import type { CardExpose, CardSlotState } from "./card.ts";
import Card from "./card.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("renders a neutral section card by default without styling or focus policy", async () => {
  const handle = mountInteraction(Card, {
    slots: { default: "Account overview" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "SECTION");
  assert.equal(root.getAttribute("class"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "card");
  assert.equal(root.getAttribute("data-variant"), "card");
  assert.equal(root.getAttribute("data-density"), "comfortable");
  assert.equal(root.getAttribute("data-tone"), "neutral");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.textContent, "Account overview");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("mirrors strict surface tokens without adding semantics", () => {
  const panel = mountInteraction(Card, {
    props: {
      as: "article",
      density: "spacious",
      tone: "accent",
      variant: "panel",
    },
    slots: { default: "Plan details" },
  });
  const panelRoot = panel.root();

  assert.equal(panelRoot.tagName, "ARTICLE");
  assert.equal(panelRoot.getAttribute("data-variant"), "panel");
  assert.equal(panelRoot.getAttribute("data-density"), "spacious");
  assert.equal(panelRoot.getAttribute("data-tone"), "accent");
  assert.equal(panelRoot.getAttribute("role"), null);
  assert.equal(panelRoot.getAttribute("tabindex"), null);
  assert.equal(panelRoot.getAttribute("aria-hidden"), null);
  assert.equal(panelRoot.textContent, "Plan details");
  panel.unmount();

  const surface = mountInteraction(Card, {
    props: {
      as: "aside",
      density: "compact",
      tone: "danger",
      variant: "surface",
    },
    slots: { default: "Storage limit" },
  });
  const surfaceRoot = surface.root();

  assert.equal(surfaceRoot.tagName, "ASIDE");
  assert.equal(surfaceRoot.getAttribute("data-variant"), "surface");
  assert.equal(surfaceRoot.getAttribute("data-density"), "compact");
  assert.equal(surfaceRoot.getAttribute("data-tone"), "danger");
  assert.equal(surfaceRoot.getAttribute("role"), null);
  assert.equal(surfaceRoot.getAttribute("tabindex"), null);
  assert.equal(surfaceRoot.getAttribute("aria-live"), null);
  assert.equal(surfaceRoot.textContent, "Storage limit");
  surface.unmount();
});

test("keeps semantics and focus policy consumer owned through attrs", async () => {
  const handle = mountInteraction(Card, {
    attrs: {
      "aria-describedby": "billing-help",
      "aria-label": "Billing summary",
      "aria-live": "polite",
      role: "region",
      tabindex: "0",
    },
    props: {
      tone: "info",
      variant: "panel",
    },
    slots: {
      default: '<p id="billing-help">Usage updates after payment events.</p>',
    },
  });
  const root = handle.getByRole("region", { name: "Billing summary" });

  assert.equal(root.getAttribute("tabindex"), "0");
  assert.equal(root.getAttribute("aria-live"), "polite");
  assert.equal(root.getAttribute("aria-describedby"), "billing-help");
  assert.equal(root.getAttribute("data-variant"), "panel");
  assert.equal(root.getAttribute("data-density"), "comfortable");
  assert.equal(root.getAttribute("data-tone"), "info");
  assert.equal(await handle.tab(), root);
  handle.unmount();
});

test("passes slot state and exposes live card state", async () => {
  const handle = mountInteraction(Card, {
    props: {
      density: "compact",
      tone: "warning",
      variant: "surface",
    },
    slots: {
      default: (state: CardSlotState) => `${state.variant}:${state.density}:${state.tone}`,
    },
  });
  const exposed = handle.exposes<CardExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.variant, "surface");
  assert.equal(exposed.density, "compact");
  assert.equal(exposed.tone, "warning");
  assert.equal(handle.root().textContent, "surface:compact:warning");

  await handle.wrapper.setProps({
    density: "spacious",
    tone: "success",
    variant: "panel",
  });
  assert.equal(exposed.variant, "panel");
  assert.equal(exposed.density, "spacious");
  assert.equal(exposed.tone, "success");
  assert.equal(handle.root().getAttribute("data-variant"), "panel");
  assert.equal(handle.root().getAttribute("data-density"), "spacious");
  assert.equal(handle.root().getAttribute("data-tone"), "success");
  assert.equal(handle.root().textContent, "panel:spacious:success");
  handle.unmount();
});
