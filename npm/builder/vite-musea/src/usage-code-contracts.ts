import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { usagePropsAttributes } from "../gallery/utils/usageCode.ts";
import { assertUsageProps } from "./usage-code.test-helpers.ts";

const fixture = JSON.parse(
  await readFile(
    new URL("../../../../tests/_fixtures/differential/musea/props-usage.json", import.meta.url),
    "utf8",
  ),
) as { values: Record<string, unknown> };

export function registerUsageCodeContracts() {
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
