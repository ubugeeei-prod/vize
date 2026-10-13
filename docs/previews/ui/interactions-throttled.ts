/** Browser laws for the source-owned throttled release-title preview. */
import assert from "node:assert/strict";
import type { Page } from "playwright";

export async function exerciseThrottledExample(page: Page): Promise<string[]> {
  const draft = page.getByRole("textbox", { name: "Draft release title", exact: true });
  const preview = page.locator('output[aria-label="Release title preview"]');
  const status = page.getByRole("status", { name: "Preview update status", exact: true });
  const show = page.getByRole("button", { name: "Update preview now", exact: true });
  const cancel = page.getByRole("button", { name: "Cancel waiting update", exact: true });
  assert.equal(await draft.inputValue(), "Vize release notes");
  assert.equal(await preview.textContent(), "Vize release notes");
  assert.equal(await status.textContent(), "No update is waiting.");
  assert.equal(await show.isDisabled(), true);
  assert.equal(await cancel.isDisabled(), true);
  await draft.fill("Leading draft");
  assert.equal(await preview.textContent(), "Leading draft", "first change leads immediately");
  assert.equal(await status.textContent(), "No update is waiting.");
  await draft.fill("Intermediate draft");
  await draft.fill("Newest trailing draft");
  assert.equal(await preview.textContent(), "Leading draft");
  assert.equal(await status.textContent(), "Newest draft is waiting.");
  await page.clock.runFor(799);
  assert.equal(await preview.textContent(), "Leading draft", "cooldown has not ended");
  await page.clock.runFor(1);
  assert.equal(await preview.textContent(), "Newest trailing draft");
  assert.equal(await status.textContent(), "No update is waiting.");
  await draft.fill("Chained trailing draft");
  assert.equal(
    await preview.textContent(),
    "Newest trailing draft",
    "trailing delivery opens another window",
  );
  assert.equal(await status.textContent(), "Newest draft is waiting.");
  await page.clock.runFor(799);
  assert.equal(await preview.textContent(), "Newest trailing draft");
  await page.clock.runFor(1);
  assert.equal(await preview.textContent(), "Chained trailing draft");
  const exactDraft = "  Vize release draft  ";
  await draft.fill(exactDraft);
  assert.equal(await status.textContent(), "Newest draft is waiting.");
  await show.focus();
  await page.keyboard.press("Enter");
  assert.equal(await preview.textContent(), exactDraft, "flush retains the complete draft");
  assert.equal(await draft.inputValue(), exactDraft);
  assert.equal(await status.textContent(), "No update is waiting.");
  assert.equal(await show.isDisabled(), true);
  assert.equal(await cancel.isDisabled(), true);
  await page.getByText("Newest draft shown now.", { exact: true }).waitFor();
  await page.clock.runFor(1600);
  assert.equal(await preview.textContent(), exactDraft, "flushed work does not replay");
  await draft.fill("Accepted preview");
  assert.equal(await preview.textContent(), "Accepted preview", "flush closed the previous window");
  const retainedDraft = "  Keep editing this draft  ";
  await draft.fill(retainedDraft);
  assert.equal(await status.textContent(), "Newest draft is waiting.");
  await cancel.focus();
  await page.keyboard.press("Space");
  assert.equal(await preview.textContent(), "Accepted preview");
  assert.equal(await draft.inputValue(), retainedDraft, "cancel keeps the draft");
  assert.equal(await status.textContent(), "No update is waiting.");
  await page.getByText("Waiting preview update canceled; draft kept.", { exact: true }).waitFor();
  await page.clock.runFor(1600);
  assert.equal(
    await preview.textContent(),
    "Accepted preview",
    "canceled work is not delivered later",
  );
  assert.equal(await draft.inputValue(), retainedDraft);
  await draft.focus();
  await page.keyboard.press("End");
  await page.keyboard.type("!");
  assert.equal(await draft.inputValue(), `${retainedDraft}!`);
  assert.equal(
    await preview.textContent(),
    `${retainedDraft}!`,
    "an edit after cancel leads again",
  );
  await page.keyboard.type("?");
  assert.equal(await draft.inputValue(), `${retainedDraft}!?`);
  assert.equal(await preview.textContent(), `${retainedDraft}!`);
  assert.equal(await status.textContent(), "Newest draft is waiting.");
  assert.equal(await show.isEnabled(), true);
  assert.equal(await cancel.isEnabled(), true);
  return [
    "initial SSR-derived title and idle native controls",
    "first change applies on the leading edge",
    "newest trailing draft applies at the 800 ms boundary",
    "trailing delivery opens a consecutive cooldown window",
    "Enter flush retains exact draft bytes and closes the timer window",
    "Space cancel retains the draft and last preview without late delivery",
    "native typing after cancel starts a fresh leading edge and pending update",
  ];
}
