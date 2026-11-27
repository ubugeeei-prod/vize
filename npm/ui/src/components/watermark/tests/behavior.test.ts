import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h } from "vue";

import { escapeXml, watermarkImage } from "../watermark-image.ts";
import Watermark from "../watermark.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("repeats text in an inert overlay", () => {
  const handle = mountInteraction(Watermark, {
    props: { text: "DRAFT" },
    slots: { default: () => "Quarterly plan" },
  });
  const overlay = handle.root().querySelector("[data-part='overlay']");

  assert.ok(overlay instanceof HTMLElement);
  assert.equal(overlay.getAttribute("aria-hidden"), "true");
  assert.equal(overlay.style.pointerEvents, "none");
  assert.equal(handle.root().style.position, "relative");
  assert.match(overlay.getAttribute("data-image") ?? "", /data:image\/svg\+xml/);
  assert.match(overlay.getAttribute("data-image") ?? "", /DRAFT/);
  assert.match(overlay.getAttribute("style") ?? "", /pointer-events:\s*none/);
  assert.match(overlay.getAttribute("style") ?? "", /background-repeat:\s*repeat/);
  handle.unmount();
});

test("keeps slotted controls interactive", async () => {
  const handle = mountInteraction(Watermark, {
    props: { text: "DRAFT" },
    slots: { default: () => h("button", { type: "button" }, "Publish") },
    record: ["click"],
  });
  const button = handle.getByRole("button", { name: "Publish" });

  await handle.click(button);
  assert.equal(button.tagName, "BUTTON");
  handle.unmount();
});

test("escapes markup before building the image", () => {
  assert.equal(escapeXml(`A&B<C>"'`), "A&amp;B&lt;C&gt;&quot;&apos;");
  const image = watermarkImage({ text: "A&B<C>", gap: 160, rotate: -24 });
  assert.match(image, new RegExp(encodeURIComponent("A&amp;B&lt;C&gt;")));
  assert.equal(image.includes("<script"), false);
});
