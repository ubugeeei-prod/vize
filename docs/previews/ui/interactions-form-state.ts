/** Assert real form and list-state transitions in the complete source-owned SFCs. */
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

export async function exerciseFormStateExample(page: Page, name: string): Promise<string[]> {
  if (name === "use-form") {
    const attendee = page.getByRole("textbox", { name: "Attendee name", exact: true });
    const email = page.getByRole("textbox", { name: "Contact email", exact: true });
    const nameError = page.locator('p[id$="-name-error"]');
    const emailError = page.locator('p[id$="-email-error"]');
    const url = page.url();
    await attendee.fill("Al");
    await email.fill("incomplete");
    await page.keyboard.press("Tab");
    assert.equal(await nameError.textContent(), "");
    assert.equal(await emailError.textContent(), "");
    await activate(page, "Preview registration");
    await nameError.filter({ hasText: "Use at least three characters." }).waitFor();
    await emailError.filter({ hasText: "Enter an email address with a domain." }).waitFor();
    assert.equal(await attendee.getAttribute("aria-invalid"), "true");
    assert.equal(await email.getAttribute("aria-invalid"), "true");
    await expectOutput(page, "Changed: yes · Submit attempts: 1");
    assert.equal(await page.getByText("No registration preview yet.", { exact: true }).count(), 1);
    assert.equal(page.url(), url, "handleSubmit prevents native form navigation");
    await attendee.fill("Ada");
    await email.fill("ada@example.test");
    await nameError.filter({ hasText: /^$/ }).waitFor({ state: "attached" });
    await emailError.filter({ hasText: /^$/ }).waitFor({ state: "attached" });
    assert.equal(await attendee.getAttribute("aria-invalid"), "false");
    assert.equal(await email.getAttribute("aria-invalid"), "false");
    await activate(page, "Preview registration");
    await page.getByText("Last preview: Ada · ada@example.test", { exact: true }).waitFor();
    await expectOutput(page, "Changed: yes · Submit attempts: 2");
    await activate(page, "Reset registration");
    await expectOutput(page, "Changed: no · Submit attempts: 0");
    assert.equal(await attendee.inputValue(), "");
    assert.equal(await email.inputValue(), "");
    assert.equal(await nameError.textContent(), "");
    assert.equal(await emailError.textContent(), "");
    await page.getByText("No registration preview yet.", { exact: true }).waitFor();
    await attendee.fill("Ada");
    await email.fill("ada@example.test");
    await activate(page, "Preview registration");
    await page.getByText("Last preview: Ada · ada@example.test", { exact: true }).waitFor();
    return [
      "field errors wait until the first keyboard submit attempt",
      "invalid submit exposes both errors and aria-invalid without creating a preview",
      "field edits after submission clear recorded errors through live revalidation",
      "valid keyboard submission creates the exact local registration preview",
      "reset clears values, errors, submit count, and local preview",
    ];
  }

  if (name === "use-stepper") {
    await expectOutput(page, "Step 1 of 3: Attendee");
    assert.equal(await page.getByRole("button", { name: "Previous step" }).isDisabled(), true);
    const steps = page.getByRole("list", { name: "Registration steps" });
    assert.equal(
      await steps.getByRole("button", { name: "Session", exact: true }).isDisabled(),
      true,
    );
    assert.equal(
      await steps
        .getByRole("button", { name: "Attendee", exact: true })
        .getAttribute("aria-current"),
      "step",
    );
    await page.getByRole("textbox", { name: "Workshop attendee" }).fill("Grace");
    await activate(page, "Next step");
    await expectOutput(page, "Step 2 of 3: Session");
    await page.getByRole("combobox", { name: "Workshop session" }).focus();
    await page.keyboard.press("t");
    await page.keyboard.press("Tab");
    await activate(page, "Next step");
    await expectOutput(page, "Step 3 of 3: Review");
    await page.getByText("Attendee: Grace", { exact: true }).waitFor();
    await page.getByText("Session: Type checking clinic", { exact: true }).waitFor();
    assert.equal(await page.getByRole("button", { name: "Next step" }).isDisabled(), true);
    assert.equal(
      await steps.getByRole("button", { name: "Review", exact: true }).getAttribute("aria-current"),
      "step",
    );
    assert.equal(
      await steps
        .getByRole("button", { name: "Attendee", exact: true })
        .getAttribute("aria-current"),
      null,
    );
    await steps.getByRole("button", { name: "Attendee", exact: true }).focus();
    await page.keyboard.press("Enter");
    await expectOutput(page, "Step 1 of 3: Attendee");
    assert.equal(
      await page.getByRole("textbox", { name: "Workshop attendee" }).inputValue(),
      "Grace",
    );
    await page.getByRole("textbox", { name: "Workshop attendee" }).fill("Grace Hopper");
    await activate(page, "Next step");
    assert.equal(
      await page.getByRole("combobox", { name: "Workshop session" }).inputValue(),
      "Type checking clinic",
    );
    await activate(page, "Previous step");
    await expectOutput(page, "Step 1 of 3: Attendee");
    await activate(page, "Next step");
    await activate(page, "Next step");
    await expectOutput(page, "Step 3 of 3: Review");
    await page.getByText("Attendee: Grace Hopper", { exact: true }).waitFor();
    return [
      "first-step previous boundary and later-step disabled states",
      "keyboard Next reveals real session selection and exact review values",
      "last-step Next boundary and aria-current track the active screen",
      "keyboard earlier-step jump preserves attendee and session values",
      "Previous and Next retain edits through conditional screens and update the review",
    ];
  }

  if (name === "use-selection") {
    const compiler = page.getByRole("checkbox", { name: "Compiler clinic", exact: true });
    const types = page.getByRole("checkbox", { name: "Type checking clinic", exact: true });
    const editor = page.getByRole("checkbox", { name: "Editor tools clinic", exact: true });
    const full = page.getByRole("checkbox", { name: "SSR workshop (full)", exact: true });
    const selected = () =>
      page.getByRole("list", { name: "Selected sessions" }).getByRole("listitem");
    assert.equal(await full.isDisabled(), true);
    await editor.focus();
    await page.keyboard.press("Space");
    await expectOutput(page, "1 of 2 places used");
    assert.equal(await editor.isChecked(), true);
    await compiler.focus();
    await page.keyboard.press("Space");
    await expectOutput(page, "2 of 2 places used");
    assert.equal(await types.isDisabled(), true);
    assert.deepEqual(await selected().allTextContents(), [
      "Compiler clinic",
      "Editor tools clinic",
    ]);
    await activate(page, "Revise compiler title");
    assert.equal(
      await page
        .getByRole("checkbox", { name: "Compiler clinic (updated)", exact: true })
        .isChecked(),
      true,
    );
    assert.equal(await editor.isChecked(), true);
    assert.deepEqual(await selected().allTextContents(), [
      "Compiler clinic (updated)",
      "Editor tools clinic",
    ]);
    await activate(page, "Clear session choices");
    await expectOutput(page, "0 of 2 places used");
    await page.getByText("No sessions selected.", { exact: true }).waitFor();
    assert.equal(await editor.isChecked(), false);
    assert.equal(await types.isDisabled(), false);
    await activate(page, "Select available sessions");
    await expectOutput(page, "2 of 2 places used");
    assert.deepEqual(await selected().allTextContents(), [
      "Compiler clinic (updated)",
      "Type checking clinic",
    ]);
    assert.equal(await full.isChecked(), false);
    assert.equal(await editor.isDisabled(), true);
    return [
      "Space toggles actual native checkboxes and chosen-session output",
      "two-place limit disables further choices and the full workshop stays disabled",
      "selected items follow catalogue order rather than click order",
      "immutable catalogue replacement retains selected ids and updates visible titles",
      "keyboard Clear empties choices and re-enables eligible checkboxes",
      "Select available sessions observes the composable's cap and skips the full workshop",
    ];
  }

  assert.equal(name, "use-cycle-list", "every form/state example has an interaction law");
  const tip = page.getByRole("combobox", { name: "Review tip" });
  await expectOutput(page, "Tip 1 of 3: Name the props");
  await activate(page, "Previous tip");
  await expectOutput(page, "Tip 3 of 3: Handle the empty state");
  assert.equal(await tip.inputValue(), "2");
  await page
    .getByText("Explain what users can do when a list has no items.", { exact: true })
    .waitFor();
  await activate(page, "Next tip");
  await expectOutput(page, "Tip 1 of 3: Name the props");
  await tip.focus();
  await page.keyboard.press("l");
  await page.keyboard.press("Tab");
  await expectOutput(page, "Tip 2 of 3: Label the controls");
  await page
    .getByText("Give each form control a label that explains its purpose.", { exact: true })
    .waitFor();
  await activate(page, "Next tip");
  await expectOutput(page, "Tip 3 of 3: Handle the empty state");
  await activate(page, "Previous tip");
  await expectOutput(page, "Tip 2 of 3: Label the controls");
  return [
    "keyboard Previous wraps from the first tip to the last with exact content",
    "keyboard Next wraps from the last tip to the first",
    "keyboard direct selection updates writable index, title, and explanation together",
    "optional-argument navigation continues from the selected tip without receiving the native event",
  ];
}
