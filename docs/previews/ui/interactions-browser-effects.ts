/** Verify observable browser effects against the same complete public-import SFCs. */
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

async function reloadExample(page: Page, name: string): Promise<void> {
  const response = await page.reload({ waitUntil: "networkidle" });
  assert.equal(response?.status(), 200, `${name}: reload serves the actual preview`);
  await page.waitForSelector(`html[data-preview-ready="${name}"]`);
  assert.equal(
    await page.locator("html").getAttribute("data-preview-hydration-retained"),
    "true",
    `${name}: saved browser state still hydrates from the server fallback`,
  );
}

export async function exerciseBrowserExample(page: Page, name: string): Promise<string[]> {
  if (name === "use-storage") {
    const section = page.getByRole("combobox", { name: "Guide section" });
    await page.getByText("Browser storage is available.", { exact: true }).waitFor();
    await activate(page, "Reset reading progress");
    await expectOutput(page, "Resume at: Getting started");
    await section.focus();
    await page.keyboard.press("c");
    await page.keyboard.press("Tab");
    assert.equal(await section.inputValue(), "components");
    await expectOutput(page, "Resume at: Components");
    assert.equal(
      await page.evaluate(() => localStorage.getItem("vize:docs:reading-progress")),
      "components",
      "the real origin's localStorage receives the selected value",
    );
    await reloadExample(page, name);
    await expectOutput(page, "Resume at: Components");
    assert.equal(await section.inputValue(), "components");
    await activate(page, "Reset reading progress");
    await expectOutput(page, "Resume at: Getting started");
    assert.equal(
      await page.evaluate(() => localStorage.getItem("vize:docs:reading-progress")),
      null,
      "remove restores the default without writing it back",
    );
    await reloadExample(page, name);
    await expectOutput(page, "Resume at: Getting started");
    assert.equal(await section.inputValue(), "getting-started");
    assert.equal(
      await page.evaluate(() => localStorage.getItem("vize:docs:reading-progress")),
      null,
    );
    await section.focus();
    await page.keyboard.press("p");
    await page.keyboard.press("Tab");
    await expectOutput(page, "Resume at: Publishing");
    return [
      "keyboard selection writes components to the real origin's localStorage",
      "page reload hydrates the SSR fallback and restores Components",
      "keyboard reset removes the stored key and restores Getting started",
      "reload after reset keeps the default and leaves the key absent",
      "continued selection updates visible reading progress to Publishing",
    ];
  }

  if (name === "media-query") {
    const viewport = page.viewportSize();
    assert.ok(viewport, "media-query exercise needs a controllable browser viewport");
    const breakpoint = page.getByRole("combobox", { name: "Roomy layout starts at" });
    const chooseBreakpoint = async (prefix: "6" | "8"): Promise<void> => {
      await breakpoint.focus();
      await page.keyboard.press(prefix);
      await page.keyboard.press("Tab");
      assert.equal(await breakpoint.inputValue(), prefix === "6" ? "600" : "800");
    };
    const guides = page.getByRole("list", { name: "Recommended guides" });
    const columns = () =>
      guides.evaluate((element) => getComputedStyle(element).gridTemplateColumns.split(" ").length);
    try {
      await chooseBreakpoint("6");
      await page.setViewportSize({ width: 760, height: viewport.height });
      await expectOutput(page, "Layout: Roomy");
      assert.equal(await page.evaluate(() => matchMedia("(min-width: 600px)").matches), true);
      assert.equal(await columns(), 2);
      await page.setViewportSize({ width: 520, height: viewport.height });
      await expectOutput(page, "Layout: Compact");
      assert.equal(await page.evaluate(() => matchMedia("(min-width: 600px)").matches), false);
      assert.equal(await columns(), 1);
      await page.setViewportSize({ width: 760, height: viewport.height });
      await expectOutput(page, "Layout: Roomy");
      await chooseBreakpoint("8");
      await expectOutput(page, "Layout: Compact");
      assert.equal(await page.evaluate(() => matchMedia("(min-width: 800px)").matches), false);
      assert.equal(await columns(), 1);
      await chooseBreakpoint("6");
      await expectOutput(page, "Layout: Roomy");
      assert.equal(await columns(), 2);
      await chooseBreakpoint("8");
    } finally {
      await page.setViewportSize(viewport);
    }
    await expectOutput(page, viewport.width >= 800 ? "Layout: Roomy" : "Layout: Compact");
    return [
      "real matchMedia changes at 760px/520px switch between two/one grid columns",
      "keyboard query change to 800px at 760px switches to Compact",
      "changing the query back to 600px restores Roomy without remounting",
      "capture viewport is restored with the live 800px query",
    ];
  }

  assert.equal(name, "active-element", "every browser-effects example has an interaction law");
  const displayName = page.getByRole("textbox", { name: "Display name", exact: true });
  const email = page.getByRole("textbox", { name: "Email address", exact: true });
  const note = page.getByRole("textbox", { name: "Workshop note", exact: true });
  await displayName.focus();
  await expectOutput(page, "Focused field: Display name");
  await page.getByText("Use the name you want on your workshop badge.", { exact: true }).waitFor();
  await page.keyboard.press("Tab");
  await expectOutput(page, "Focused field: Email address");
  assert.equal(await email.evaluate((element) => document.activeElement === element), true);
  await page
    .getByText("Use an address where you can receive workshop details.", { exact: true })
    .waitFor();
  await page.keyboard.press("Tab");
  await expectOutput(page, "Focused field: Workshop note");
  assert.equal(await note.evaluate((element) => document.activeElement === element), true);
  await page.getByText("Write a note for the workshop organizer.", { exact: true }).waitFor();
  await activate(page, "Focus display name");
  await expectOutput(page, "Focused field: Display name");
  assert.equal(await displayName.evaluate((element) => document.activeElement === element), true);
  await activate(page, "Clear focus");
  await expectOutput(page, "Focused field: Outside the profile fields");
  await page.getByText("Focus a profile field to see its guidance.", { exact: true }).waitFor();
  assert.equal(await page.evaluate(() => document.activeElement === document.body), true);
  await note.focus();
  await expectOutput(page, "Focused field: Workshop note");
  return [
    "real document focus on Display name selects its contextual guidance",
    "Tab moves actual focus to Email address and Workshop note with matching guidance",
    "keyboard activation moves actual focus back to Display name",
    "keyboard Clear focus blurs to document.body and restores the outside-field guidance",
  ];
}
