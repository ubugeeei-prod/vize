import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import type { Page } from "playwright";
import { assertUsageProps } from "../../src/usage-code.test-helpers.ts";

export async function checkCopiedProps(page: Page, repository: string, output: string) {
  const fixture = JSON.parse(
    await readFile(
      path.join(repository, "tests/_fixtures/differential/musea/props-usage.json"),
      "utf8",
    ),
  ) as { values: Record<string, unknown> };
  const label = fixture.values.label;
  assert.ok(typeof label === "string");
  await page.getByRole("textbox", { name: "label", exact: true }).fill(label);
  for (const [name, value] of Object.entries(fixture.values)) {
    if (name === "label") continue;
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
  const json = page.locator(".props-json-code code");
  await page.waitForFunction(
    (expected) => document.querySelector(".props-json-code code")?.textContent === expected,
    JSON.stringify(fixture.values, null, 2),
  );
  assert.deepEqual(JSON.parse((await json.textContent())!), fixture.values);
  await page.locator(".props-copy-btn").click();
  const copied = await page.evaluate(() => navigator.clipboard.readText());
  assert.equal(await page.locator(".props-usage-code code").textContent(), copied);
  const template = copied.match(/<template>\n([\s\S]*)\n<\/template>$/)?.[1];
  assert.ok(template, "Copied usage includes a complete Vue template");
  const received = await assertUsageProps(template, fixture.values, "MuseaButton");
  await page
    .locator(".props-usage-code")
    .screenshot({ path: path.join(output, "typed-props.png") });
  return { panel: "typed-props", copied, received, expected: fixture.values };
}
