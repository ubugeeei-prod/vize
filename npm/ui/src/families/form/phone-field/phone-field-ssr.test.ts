import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import PhoneField from "./phone-field.vue";
import PhoneFieldCountrySelect from "./phone-field-country-select.vue";
import PhoneFieldInput from "./phone-field-input.vue";
import { definePhoneCountries } from "./phone-field-country.ts";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

const countries = definePhoneCountries([
  { code: "JP", name: "Japan", dialCode: "81", pattern: "99-9999-9999", trunkPrefix: "0" },
  { code: "US", name: "United States", dialCode: "1", pattern: "(999) 999-9999" },
]);

test("renders byte-identical phone markup and hydrates without mismatches", async () => {
  const { html, host, dispose } = await renderAndHydrate(
    defineComponent({
      name: "PhoneFieldSsrProbe",
      setup: () => () =>
        h(
          PhoneField,
          { countries, defaultValue: "+12125550100", name: "phone", ariaLabel: "Phone" },
          { default: () => [h(PhoneFieldCountrySelect), h(PhoneFieldInput)] },
        ),
    }),
  );
  try {
    assert.match(html, /data-country="US"/);
    assert.match(html, /type="hidden" name="phone" value="\+12125550100"/);
    assert.match(html, /type="tel"[^>]*value="\(212\) 555-0100"/);
    assert.equal(host.querySelector("select")?.value, "US");
  } finally {
    dispose();
  }
});
