import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import type { Page } from "playwright";
import { assertUsageProps } from "../../src/usage-code.test-helpers";

export async function checkCopiedPropNames(
  page: Page,
  repository: string,
  baseValues: Record<string, unknown>,
  observations: unknown[],
) {
  const fixture = JSON.parse(
    await readFile(
      path.join(repository, "tests/_fixtures/differential/musea/props-usage-names.json"),
      "utf8",
    ),
  ) as {
    values: Record<string, unknown>;
    props: Record<string, unknown>;
    attrs: Record<string, unknown>;
  };
  for (const [name, value] of Object.entries(fixture.values)) {
    await page.getByRole("button", { name: "Add Prop", exact: true }).click();
    await page.getByRole("textbox", { name: "Prop name", exact: true }).fill(name);
    await page
      .getByRole("combobox", { name: "Prop control type", exact: true })
      .selectOption(typeof value === "string" ? "text" : "object");
    await page
      .getByRole("textbox", { name: "Default value", exact: true })
      .fill(typeof value === "string" ? value : JSON.stringify(value));
    await page.getByRole("button", { name: "Add", exact: true }).click();
  }
  const values = { ...baseValues, ...fixture.values };
  await page.waitForFunction(
    (expected) => document.querySelector(".props-json-code code")?.textContent === expected,
    JSON.stringify(values, null, 2),
  );
  assert.deepEqual(
    JSON.parse((await page.locator(".props-json-code code").textContent())!),
    values,
  );
  await page.locator(".props-copy-btn").click();
  const copied = await page.evaluate(() => navigator.clipboard.readText());
  assert.equal(await page.locator(".props-usage-code code").textContent(), copied);
  const template = copied.match(/<template>\n([\s\S]*)\n<\/template>$/)?.[1];
  assert.ok(template);
  const expected = { ...baseValues, ...fixture.props };
  const observation: Record<string, unknown> = {
    panel: "custom-prop-names",
    fixture,
    values,
    copied,
    expected,
  };
  observations.push(observation);
  observation.received = await assertUsageProps(template, expected, "MuseaButton", fixture.attrs);
  await page.locator(".props-save").click();
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.reload();
  await page.locator(".art-item").first().click();
  await page.locator(".tab-btn").filter({ hasText: "Props" }).click();
  await page.waitForFunction(
    (expected) => document.querySelector(".props-json-code code")?.textContent === expected,
    JSON.stringify(values, null, 2),
  );
  const reloaded = JSON.parse((await page.locator(".props-json-code code").textContent())!);
  observation.reloaded = reloaded;
  assert.deepEqual(reloaded, values);
  await page.locator(".props-copy-btn").click();
  const reloadedCopy = await page.evaluate(() => navigator.clipboard.readText());
  observation.reloadedCopy = reloadedCopy;
  assert.equal(reloadedCopy, copied);
}
