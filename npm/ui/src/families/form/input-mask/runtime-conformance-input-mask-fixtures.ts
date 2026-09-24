import assert from "node:assert/strict";

import { h } from "vue";

import MaskedInput from "./masked-input.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const inputMaskRuntimeFixture: RuntimeFixture = {
  name: "masked-input",
  sourceFile: "families/form/input-mask/masked-input.vue",
  render: () =>
    h(MaskedInput, {
      mask: "(999) 999-9999",
      id: "phone",
      name: "phone",
      ariaLabel: "Phone",
      defaultValue: "5551234567",
    }),
  assertServerMarkup(html) {
    assert.match(html, /^<input id="phone"/);
    assert.match(html, /value="\(555\) 123-4567"/);
    assert.match(html, /data-vize-ui="masked-input"/);
  },
  assertHydratedDom(host) {
    const input = host.querySelector("input");
    assert.ok(input instanceof HTMLInputElement);
    assert.equal(input.value, "(555) 123-4567");
    assert.equal(input.getAttribute("inputmode"), "numeric");
  },
};
