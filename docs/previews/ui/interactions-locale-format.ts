/** Assert locale-aware formatting in the complete source-owned SFCs. */
import assert from "node:assert/strict";
import type { Page } from "playwright";

/** ICU uses no-break spaces in some locales; compare their visible text. */
function visible(text: string | null): string {
  return (text ?? "").replace(/[  ]/g, " ").trim();
}

async function expectOutput(page: Page, text: string): Promise<void> {
  const output = page.locator("output");
  await output.filter({ hasText: text.split(" ")[0] }).waitFor();
  assert.equal(visible(await output.textContent()), text);
}

async function chooseLocale(page: Page, label: string): Promise<void> {
  const select = page.getByRole("combobox", { name: "Reader locale" });
  await select.focus();
  await select.selectOption({ label });
}

export async function exerciseLocaleFormatExample(page: Page, name: string): Promise<string[]> {
  if (name === "use-number-format") {
    await expectOutput(page, "€1,234.50");
    await chooseLocale(page, "German (Germany)");
    await expectOutput(page, "1.234,50 €");
    await page.getByRole("spinbutton", { name: "Order total in euros" }).fill("99");
    await expectOutput(page, "99,00 €");
    await chooseLocale(page, "Japanese (Japan)");
    await expectOutput(page, "€99.00");
    return ["locale changes separators and symbol", "amount updates the formatted total"];
  }
  if (name === "use-list-format") {
    await expectOutput(page, "Ada, Grace, and Linus");
    await chooseLocale(page, "German (Germany)");
    await expectOutput(page, "Ada, Grace und Linus");
    await chooseLocale(page, "Japanese (Japan)");
    await expectOutput(page, "Ada、Grace、Linus");
    await chooseLocale(page, "English (United States)");
    await page.getByRole("button", { name: "Add speaker" }).click();
    await expectOutput(page, "Ada, Grace, Linus, and Margaret");
    const remove = page.getByRole("button", { name: "Remove speaker" });
    for (let index = 0; index < 4; index += 1) await remove.click();
    await expectOutput(page, "No speakers yet.");
    assert.equal(await remove.isDisabled(), true);
    return ["locale changes the conjunction", "adding and removing names updates the sentence"];
  }
  if (name === "use-plural-rules") {
    const count = page.getByRole("spinbutton", { name: "Unread messages" });
    await expectOutput(page, "No unread messages");
    await count.fill("1");
    await expectOutput(page, "1 unread message");
    assert.equal(await page.locator("code").textContent(), "one");
    await count.fill("2");
    await expectOutput(page, "2 unread messages");
    assert.equal(await page.locator("code").textContent(), "other");
    return ["exact zero message", "singular and plural categories"];
  }
  throw new Error(`No locale formatting law for ${name}`);
}
