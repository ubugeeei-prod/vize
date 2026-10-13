/** Verify native connectivity and actual color preference effects. */
import assert from "node:assert/strict";
import type { Page } from "playwright";

async function activate(page: Page, label: string): Promise<void> {
  await page.getByRole("button", { name: label, exact: true }).focus();
  await page.keyboard.press("Enter");
}

async function expectOutput(page: Page, text: string): Promise<void> {
  await page.locator("output").filter({ hasText: text }).waitFor();
  assert.equal(await page.locator("output").textContent(), text);
}

export async function exercisePreferenceBrowserExample(
  page: Page,
  name: string,
): Promise<string[]> {
  if (name === "network") {
    const draft = page.getByRole("textbox", { name: "Local guide draft", exact: true });
    const eventValue = (label: string) =>
      page.locator("dt").filter({ hasText: label }).locator("+ dd");
    await expectOutput(page, "Browser status: online");
    assert.equal(await page.evaluate(() => navigator.onLine), true);
    assert.equal(await eventValue("Last online event").textContent(), "not observed");
    assert.equal(await eventValue("Last offline event").textContent(), "not observed");
    await draft.pressSequentially("Draft remains local");
    try {
      await page.context().setOffline(true);
      await expectOutput(page, "Browser status: offline");
      assert.equal(await page.evaluate(() => navigator.onLine), false);
      await page
        .getByText("The browser reports offline. Your local draft remains editable.", {
          exact: true,
        })
        .waitFor();
      const offline = await eventValue("Last offline event").textContent();
      assert.ok(offline && offline !== "not observed");
      assert.ok(Number.isFinite(Date.parse(offline)));
      await draft.focus();
      await page.keyboard.press("End");
      await page.keyboard.type(" while offline");
      assert.equal(await draft.inputValue(), "Draft remains local while offline");
      await activate(page, "Review local draft");
      await page.getByText("Local review count: 1.", { exact: true }).waitFor();
      await page.clock.runFor(25);
      await page.context().setOffline(false);
      await expectOutput(page, "Browser status: online");
      assert.equal(await page.evaluate(() => navigator.onLine), true);
      const online = await eventValue("Last online event").textContent();
      assert.ok(online && Number.isFinite(Date.parse(online)));
      assert.ok(Date.parse(online) > Date.parse(offline));
      assert.equal(await eventValue("Last offline event").textContent(), offline);
      assert.equal(await draft.inputValue(), "Draft remains local while offline");
    } finally {
      await page.context().setOffline(false);
    }
    return [
      "initial native online state retains unobserved event timestamps",
      "actual browser offline mode delivers the native flag and timestamp",
      "native draft typing and Enter review remain usable without connectivity",
      "actual reconnection records a later online event and retains the offline timestamp",
      "reconnection preserves the complete local draft",
    ];
  }

  assert.equal(name, "color-mode", "every preference browser example has an interaction law");
  const preference = page.getByRole("combobox", { name: "Guide color preference", exact: true });
  const choose = async (key: "a" | "d" | "l" | "c", value: string): Promise<void> => {
    await preference.focus();
    await page.keyboard.press(key);
    await page.keyboard.press("Tab");
    assert.equal(await preference.inputValue(), value);
  };
  const attribute = () => page.locator("html").getAttribute("data-guide-mode");
  const background = () =>
    page
      .getByRole("region", { name: "Color preference sample" })
      .evaluate((element) => getComputedStyle(element).backgroundColor);
  await page.emulateMedia({ colorScheme: "light" });
  await activate(page, "Follow system preference");
  await expectOutput(page, "Chosen: auto · System: light · Applied: light");
  assert.equal(await attribute(), "light");
  await choose("d", "dark");
  await expectOutput(page, "Chosen: dark · System: light · Applied: dark");
  assert.equal(await attribute(), "dark");
  assert.equal(await background(), "rgb(24, 40, 51)");
  assert.equal(await page.evaluate(() => localStorage.getItem("vize:docs:color-mode")), "dark");
  await page.emulateMedia({ colorScheme: "dark" });
  await expectOutput(page, "Chosen: dark · System: dark · Applied: dark");
  await choose("l", "light");
  await expectOutput(page, "Chosen: light · System: dark · Applied: light");
  assert.equal(await attribute(), "light");
  assert.equal(await background(), "rgb(255, 255, 255)");
  await choose("a", "auto");
  await expectOutput(page, "Chosen: auto · System: dark · Applied: dark");
  await page.emulateMedia({ colorScheme: "light" });
  await expectOutput(page, "Chosen: auto · System: light · Applied: light");
  await choose("c", "contrast");
  await expectOutput(page, "Chosen: contrast · System: light · Applied: contrast");
  assert.equal(await attribute(), "contrast");
  assert.equal(await background(), "rgb(0, 0, 0)");
  const response = await page.reload({ waitUntil: "networkidle" });
  assert.equal(response?.status(), 200);
  await page.locator('html[data-preview-ready="color-mode"]').waitFor();
  assert.equal(await page.locator("html").getAttribute("data-preview-hydration-retained"), "true");
  await expectOutput(page, "Chosen: contrast · System: light · Applied: contrast");
  assert.equal(await preference.inputValue(), "contrast");
  await page.getByText("Preference read at mount: contrast.", { exact: true }).waitFor();
  await activate(page, "Follow system preference");
  await expectOutput(page, "Chosen: auto · System: light · Applied: light");
  assert.equal(await page.evaluate(() => localStorage.getItem("vize:docs:color-mode")), "auto");
  await page.emulateMedia({ colorScheme: null });
  return [
    "native keyboard selection applies the actual dark attribute and source-owned CSS",
    "manual Light overrides an actual dark system media preference",
    "Automatic follows native prefers-color-scheme changes without remounting",
    "configured Contrast applies its actual custom attribute and black sample card",
    "reload retains strict initial SSR identity and then reads the actual saved contrast preference",
    "native Follow system preference restores auto and writes the real dedicated storage key",
  ];
}
