import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { usagePropsAttributes } from "../gallery/utils/usageCode.ts";
import { assertUsageProps } from "./usage-code.test-helpers.ts";
import { initialPaletteValues, restorePaletteState } from "../gallery/composables/paletteState";

const fixture = JSON.parse(
  await readFile(
    new URL("../../../../tests/_fixtures/differential/musea/props-usage.json", import.meta.url),
    "utf8",
  ),
) as { values: Record<string, unknown> };
const namesFixture = JSON.parse(
  await readFile(
    new URL(
      "../../../../tests/_fixtures/differential/musea/props-usage-names.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as {
  values: Record<string, unknown>;
  props: Record<string, unknown>;
  attrs: Record<string, unknown>;
};

export function registerUsageCodeContracts() {
  void test("own prototype names combined with ordinary props do not create inherited attributes", async () => {
    await assertUsageProps(
      `<Probe${usagePropsAttributes({ label: "Regular prop", ...namesFixture.values })} />`,
      { label: "Regular prop", ...namesFixture.props },
      "Probe",
      namesFixture.attrs,
    );
  });
  void test("palette defaults and saved custom names retain own prototype keys", () => {
    const ownValues = Object.fromEntries([["__proto__", namesFixture.values.__proto__]]);
    const control = {
      name: "__proto__",
      control: "object",
      default_value: ownValues.__proto__,
      required: false,
      options: [],
    };
    const initial = initialPaletteValues([control]);
    assert.deepEqual(initial, ownValues);
    assert.equal(Object.getPrototypeOf(initial), Object.prototype);
    const restored = restorePaletteState([], {
      version: 1,
      values: ownValues,
      customProps: [{ name: "__proto__", control: "object", default_value: null }],
      deletedPaletteProps: [],
    });
    assert.deepEqual(restored.values, ownValues);
    assert.equal(Object.getPrototypeOf(restored.values), Object.prototype);
    assert.equal(restored.customProps[0].name, "__proto__");
  });
  void test("custom prop names remain values rather than Vue template syntax", async () => {
    await assertUsageProps(
      `<Probe${usagePropsAttributes(namesFixture.values)} />`,
      namesFixture.props,
      "Probe",
      namesFixture.attrs,
    );
  });
  void test("copied usage templates preserve edited JSON prop values and attribute characters", async () => {
    const attributes = usagePropsAttributes(fixture.values);
    await assertUsageProps(`<Probe${attributes} />`, fixture.values);
  });

  void test("ordinary usage attributes keep existing output and omit undefined", async () => {
    const values = { label: "Button", count: 4, disabled: true, expanded: false };
    const attributes = usagePropsAttributes({ ...values, omitted: undefined });
    assert.equal(attributes, ' label="Button" :count="4" disabled :expanded="false"');
    await assertUsageProps(`<Probe${attributes} />`, values);
  });

  void test("numeric usage bindings preserve zero sign and non-finite values", async () => {
    const values = { negativeZero: -0, config: { zero: -0 }, infinite: Infinity, invalid: NaN };
    await assertUsageProps(`<Probe${usagePropsAttributes(values)} />`, values);
  });

  void test("text usage attributes preserve authored CR, LF and tab characters", async () => {
    const values = { label: "First\r\n\tSecond", empty: "" };
    await assertUsageProps(`<Probe${usagePropsAttributes(values)} />`, values);
  });
}
