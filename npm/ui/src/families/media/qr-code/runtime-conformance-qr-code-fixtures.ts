import assert from "node:assert/strict";

import { h } from "vue";

import QrCode from "./qr-code.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const qrCodeRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "qr-code",
    sourceFile: "families/media/qr-code/qr-code.vue",
    render: () => h(QrCode, { value: "https://vizejs.dev", errorCorrection: "Q", label: "Vize" }),
    assertServerMarkup(html) {
      assert.match(html, /^<svg/);
      assert.match(html, /data-vize-ui="qr-code"/);
      assert.match(html, /role="img"/);
      assert.match(html, /aria-label="Vize"/);
      assert.match(html, /data-state="ready"/);
      assert.match(html, /data-error-correction="Q"/);
      assert.match(html, /data-vize-ui="qr-code-modules"/);
    },
    assertHydratedDom(host) {
      const svg = host.querySelector('[data-vize-ui="qr-code"]');
      assert.ok(svg instanceof SVGSVGElement);
      assert.equal(svg.getAttribute("data-state"), "ready");
      assert.ok((svg.querySelector('[part="modules"]')?.getAttribute("d") ?? "").length > 0);
    },
  },
];
