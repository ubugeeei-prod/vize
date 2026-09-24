import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import PhoneField from "./phone-field.vue";
import PhoneFieldCountrySelect from "./phone-field-country-select.vue";
import PhoneFieldInput from "./phone-field-input.vue";
import {
  definePhoneCountries,
  formatNationalNumber,
  parsePhoneNumber,
  toE164,
} from "./phone-field-country.ts";
import type { PhoneFieldExpose, PhoneFieldSlotState } from "./phone-field-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

const countries = definePhoneCountries([
  { code: "JP", name: "Japan", dialCode: "81", pattern: "99-9999-9999", trunkPrefix: "0" },
  { code: "US", name: "United States", dialCode: "1", pattern: "(999) 999-9999" },
  { code: "GB", name: "United Kingdom", dialCode: "44", pattern: "9999 999999", trunkPrefix: "0" },
  { code: "CA", name: "Canada", dialCode: "1", pattern: "(999) 999-9999" },
]);
type Code = (typeof countries)[number]["code"];
const [japan, usa, , canada] = countries;

function mountPhone(props: Record<string, unknown> = {}) {
  return mountInteraction(PhoneField, {
    props: { countries, ariaLabel: "Phone", ...props },
    record: ["update:modelValue", "update:country", "complete"],
    slots: {
      default: (state: PhoneFieldSlotState<Code>) => [
        h(PhoneFieldCountrySelect),
        h(PhoneFieldInput),
        h("output", `${state.country.code}|${state.e164}|${state.international}|${state.state}`),
      ],
    },
  });
}

function parts(root: HTMLElement) {
  const input = root.querySelector('[data-vize-ui="phone-field-input"]');
  const select = root.querySelector("select");
  assert.ok(input instanceof HTMLInputElement && select instanceof HTMLSelectElement);
  return { input, select, output: () => root.querySelector("output")?.textContent };
}

async function type(input: HTMLInputElement, text: string): Promise<void> {
  input.focus();
  input.value = text;
  input.setSelectionRange(text.length, text.length);
  input.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "insertText" }));
  await nextTick();
  await nextTick();
}

test("parses national and international text with trunk prefixes and dial-code matching", () => {
  assert.ok(japan && usa && canada);
  assert.deepEqual(parsePhoneNumber("090-1234-5678", countries, japan), {
    country: japan,
    nationalNumber: "9012345678",
  });
  assert.equal(parsePhoneNumber("+44 020 7946 0958", countries, japan)?.country.code, "GB");
  assert.equal(
    parsePhoneNumber("0044 20 7946 0958", countries, japan)?.nationalNumber,
    "2079460958",
  );
  assert.equal(
    parsePhoneNumber("+1 416 555 0100", countries, canada)?.country.code,
    "CA",
    "ties prefer the current country",
  );
  assert.equal(parsePhoneNumber("+1 416 555 0100", countries, japan)?.country.code, "US");
  assert.equal(parsePhoneNumber("+999 1", countries, japan), undefined);
  assert.equal(
    parsePhoneNumber("０９０", countries, japan)?.nationalNumber,
    "90",
    "full-width digits fold",
  );
  assert.equal(formatNationalNumber("9012345678", japan), "90-1234-5678");
  assert.equal(formatNationalNumber("21255", usa), "(212) 55");
  assert.equal(toE164("", japan), "");
  assert.equal(toE164("9012345678", japan), "+819012345678");
  assert.throws(
    () => definePhoneCountries([{ code: "XX", name: "Bad", dialCode: "+1" }]),
    /VIZE_UI_PHONE_FIELD_DIAL_CODE/,
  );
});

test("renders a tel input, a NativeSelect country picker, and a hidden E.164 value", () => {
  const handle = mountPhone({ defaultValue: "+819012345678", name: "phone", id: "phone" });
  const root = handle.root();
  const { input, select, output } = parts(root);

  assert.equal(root.getAttribute("data-vize-ui"), "phone-field");
  assert.equal(root.getAttribute("data-country"), "JP");
  assert.equal(input.id, "phone");
  assert.equal(input.type, "tel");
  assert.equal(input.getAttribute("inputmode"), "tel");
  assert.equal(input.getAttribute("autocomplete"), "tel-national");
  assert.equal(input.value, "90-1234-5678");
  assert.equal(select.getAttribute("data-vize-ui"), "phone-field-country-select");
  assert.equal(select.getAttribute("aria-label"), "Country");
  assert.equal(select.getAttribute("aria-controls"), "phone");
  assert.equal(select.value, "JP");
  assert.deepEqual(
    [...select.options].map((option) => option.textContent),
    ["Japan (+81)", "United States (+1)", "United Kingdom (+44)", "Canada (+1)"],
  );
  assert.equal(
    root.querySelector<HTMLInputElement>('input[type="hidden"]')?.value,
    "+819012345678",
  );
  assert.equal(output(), "JP|+819012345678|+81 90-1234-5678|complete");
  handle.unmount();
});

test("typing formats by the country pattern, strips the trunk prefix, and emits completion", async () => {
  const handle = mountPhone();
  const { input, output } = parts(handle.root());

  await type(input, "0");
  assert.equal(input.value, "", "a lone trunk prefix is dropped");
  await type(input, "9012");
  assert.equal(input.value, "90-12");
  assert.equal(output(), "JP|+819012|+81 90-12|incomplete");
  await type(input, "90-12345678");
  assert.equal(input.value, "90-1234-5678");
  assert.deepEqual(handle.recorded().at(-1), {
    event: "complete",
    payload: ["+819012345678", japan],
  });
  handle.unmount();
});

test("choosing a country keeps the digits and re-targets the dial code", async () => {
  const handle = mountPhone({ defaultValue: "2125550100", defaultCountry: "US" });
  const { select, input, output } = parts(handle.root());

  assert.equal(input.value, "(212) 555-0100");
  select.value = "GB";
  select.dispatchEvent(new Event("change", { bubbles: true }));
  await nextTick();
  assert.equal(output()?.split("|")[0], "GB");
  assert.equal(output()?.split("|")[1], "+442125550100");
  assert.equal(input.value, "2125 550100");
  assert.ok(
    handle
      .recorded()
      .some((entry) => entry.event === "update:country" && entry.payload[0] === "GB"),
  );
  handle.unmount();
});

test("international input and paste switch the country automatically", async () => {
  const handle = mountPhone();
  const { input, select, output } = parts(handle.root());

  await type(input, "+44 20 7946 0958");
  assert.equal(output(), "GB|+442079460958|+44 2079 460958|complete");
  assert.equal(select.value, "GB");
  const paste = new Event("paste", { bubbles: true, cancelable: true });
  Object.defineProperty(paste, "clipboardData", { value: { getData: () => "+1 (212) 555-0100" } });
  input.dispatchEvent(paste);
  await nextTick();
  assert.equal(paste.defaultPrevented, true);
  assert.equal(output()?.split("|")[0], "US");
  assert.equal(input.value, "(212) 555-0100");
  handle.unmount();
});

test("controlled values from another country win, and the API sets, switches, and clears", async () => {
  const handle = mountPhone({ modelValue: "+447946095800", country: "JP" });
  const { output } = parts(handle.root());
  const api = handle.exposes<PhoneFieldExpose<Code>>();

  assert.equal(output()?.split("|")[0], "GB", "the stored number's country wins");
  assert.equal(api.setValue("+1 212 555 0100"), true);
  assert.deepEqual(
    handle.recorded().find((entry) => entry.event === "update:modelValue"),
    {
      event: "update:modelValue",
      payload: ["+12125550100"],
    },
  );
  assert.equal(api.setValue("+999"), false);
  handle.unmount();

  const free = mountPhone();
  const freeApi = free.exposes<PhoneFieldExpose<Code>>();
  freeApi.setValue("09012345678");
  assert.equal(freeApi.e164, "+819012345678");
  assert.equal(freeApi.setCountry("US"), true);
  assert.equal(freeApi.e164, "+19012345678");
  freeApi.clear();
  assert.equal(freeApi.state, "empty");
  free.unmount();
});

test("form reset restores defaults and disabled fields ignore input", async () => {
  const Probe = defineComponent({
    setup: () => () =>
      h("form", [
        h(
          PhoneField,
          { countries, name: "tel", defaultValue: "+819012345678", ariaLabel: "Phone" },
          { default: () => h(PhoneFieldInput) },
        ),
      ]),
  });
  const handle = mountInteraction(Probe);
  const form = handle.root();
  assert.ok(form instanceof HTMLFormElement);
  const input = form.querySelector<HTMLInputElement>('[data-vize-ui="phone-field-input"]');
  assert.ok(input);
  await type(input, "80");
  assert.equal(new FormData(form).get("tel"), "+8180");
  form.reset();
  await nextTick();
  await nextTick();
  assert.equal(new FormData(form).get("tel"), "+819012345678");
  handle.unmount();

  const disabled = mountPhone({ disabled: true, name: "tel" });
  const disabledParts = parts(disabled.root());
  assert.equal(disabledParts.input.disabled, true);
  assert.equal(disabledParts.select.disabled, true);
  assert.equal(disabled.exposes<PhoneFieldExpose<Code>>().setValue("+81 90"), false);
  disabled.unmount();
});

test("rejects empty country lists and parts outside a PhoneField", () => {
  const warn = console.warn;
  console.warn = () => undefined;
  try {
    assert.throws(
      () => mountInteraction(PhoneField, { props: { countries: [] } }),
      /VIZE_UI_PHONE_FIELD_COUNTRIES/,
    );
    assert.throws(() => mountInteraction(PhoneFieldInput), /VIZE_UI_CONTEXT_MISSING: PhoneField/);
    assert.throws(
      () => mountInteraction(PhoneFieldCountrySelect),
      /VIZE_UI_CONTEXT_MISSING: PhoneField/,
    );
  } finally {
    console.warn = warn;
  }
});
