/** Assert countdown and relative date behavior in the complete source-owned SFCs. */
import assert from "node:assert/strict";
import type { Page } from "playwright";

async function expectOutput(page: Page, text: string): Promise<void> {
  await page.locator("output").filter({ hasText: text }).waitFor();
  assert.equal((await page.locator("output").textContent())?.trim(), text);
}

export async function exerciseTimeFormatExample(page: Page, name: string): Promise<string[]> {
  if (name === "use-countdown") {
    await expectOutput(page, "No code sent yet.");
    await page.getByRole("button", { name: "Send code", exact: true }).focus();
    await page.keyboard.press("Enter");
    await expectOutput(page, "Code sent. Resend available in 30 s.");
    const resend = page.getByRole("button", { name: "Resend code", exact: true });
    assert.equal(await resend.isDisabled(), true);
    await page.clock.runFor(1000);
    await expectOutput(page, "Code sent. Resend available in 29 s.");
    await page.clock.runFor(29_000);
    await expectOutput(page, "Code sent 1 times. You can request another one.");
    assert.equal(await resend.isDisabled(), false);
    return ["cooldown disables the button", "ticks every second", "completion re-enables resend"];
  }
  if (name === "use-relative-time-format") {
    const days = page.getByRole("spinbutton", { name: "Days from today" });
    await expectOutput(page, "The meetup is in 3 days.");
    for (const [value, text] of [
      ["1", "tomorrow"],
      ["0", "today"],
      ["-1", "yesterday"],
      ["3", "in 3 days"],
    ]) {
      await days.fill(value);
      await expectOutput(page, `The meetup is ${text}.`);
    }
    const locale = page.getByRole("combobox", { name: "Reader locale" });
    await locale.selectOption({ label: "German (Germany)" });
    await expectOutput(page, "The meetup is in 3 Tagen.");
    await locale.selectOption({ label: "Japanese (Japan)" });
    await expectOutput(page, "The meetup is 3 日後.");
    return ["nearby days use words", "locale changes the phrase"];
  }
  throw new Error(`No time formatting law for ${name}`);
}
