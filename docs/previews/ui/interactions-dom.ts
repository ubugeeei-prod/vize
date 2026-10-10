/** Exercise the actual scoped DOM listeners in the complete public-import SFCs. */
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

export async function exerciseDomExample(page: Page, name: string): Promise<string[]> {
  if (name === "focus") {
    const badge = page.getByRole("textbox", { name: "Badge name" });
    const contact = page.getByRole("textbox", { name: "Workshop contact" });
    await expectOutput(page, "Current: name focused no · profile focus no");
    await activate(page, "Focus badge name");
    assert.equal(await badge.evaluate((element) => element === document.activeElement), true);
    await expectOutput(page, "Current: name focused yes · profile focus yes");
    await page.keyboard.press("Escape");
    assert.equal(await badge.evaluate((element) => element === document.activeElement), false);
    await expectOutput(page, "Current: name focused no · profile focus no");
    await activate(page, "Focus badge name");
    await page.keyboard.press("Tab");
    assert.equal(await contact.evaluate((element) => element === document.activeElement), true);
    await expectOutput(page, "Current: name focused no · profile focus yes");
    await activate(page, "Blur badge name");
    await expectOutput(page, "Current: name focused no · profile focus no");
    await badge.focus();
    await expectOutput(page, "Current: name focused yes · profile focus yes");
    await page.keyboard.press("Tab");
    await page.keyboard.press("Tab");
    assert.equal(
      await page
        .getByRole("button", { name: "Stop focus tracking" })
        .evaluate((element) => element === document.activeElement),
      true,
    );
    await page.keyboard.press("Tab");
    await expectOutput(page, "Current: name focused no · profile focus no");
    await activate(page, "Stop focus tracking");
    await expectOutput(page, "Last observed: name focused no · profile focus yes");
    await badge.fill("Ada");
    await page.keyboard.press("Tab");
    await expectOutput(page, "Last observed: name focused no · profile focus yes");
    assert.equal(await page.getByRole("button", { name: "Focus badge name" }).isDisabled(), true);
    assert.equal(await page.getByRole("button", { name: "Blur badge name" }).isDisabled(), true);
    assert.equal(
      await page.getByRole("button", { name: "Stop focus tracking" }).isDisabled(),
      true,
    );
    return [
      "writable focus ref focuses the actual name element",
      "Escape assigns false and blurs the actual focused element",
      "Tab between descendants changes individual focus and retains group focus",
      "keyboard movement outside clears the group flag",
      "stop freezes explicitly labelled last-observed flags and disables controls",
      "stopped observation leaves native profile inputs editable",
    ];
  }

  if (name === "on-click-outside") {
    const region = () => page.getByRole("region", { name: "Guide preferences", exact: true });
    await activate(page, "Open guide preferences");
    assert.equal(
      await page
        .getByRole("button", { name: "Close guide preferences" })
        .getAttribute("aria-expanded"),
      "true",
    );
    await page.getByRole("textbox", { name: "Review note" }).fill("Label every control.");
    await activate(page, "Apply local preference");
    await expectOutput(page, "Outside dismissals: 0 · Local note: Label every control.");
    assert.equal(await region().count(), 1);
    await page.getByRole("button", { name: "Keep preferences open" }).click();
    assert.equal(await region().count(), 1);
    await activate(page, "Close guide preferences");
    assert.equal(await region().count(), 0);
    await expectOutput(page, "Outside dismissals: 0 · Local note: Label every control.");
    await activate(page, "Open guide preferences");
    assert.equal(await region().count(), 1);
    const inside = await page.getByRole("button", { name: "Apply local preference" }).boundingBox();
    const outside = await page
      .getByRole("button", { name: "Continue reading outside" })
      .boundingBox();
    assert.ok(inside && outside);
    await page.mouse.move(inside.x + inside.width / 2, inside.y + inside.height / 2);
    await page.mouse.down();
    await page.mouse.move(outside.x + outside.width / 2, outside.y + outside.height / 2);
    await page.mouse.up();
    assert.equal(await region().count(), 1, "drag starting inside does not dismiss the region");
    await page.getByRole("button", { name: "Continue reading outside" }).click();
    await expectOutput(page, "Outside dismissals: 1 · Dismissed by an outside click.");
    assert.equal(await region().count(), 0);
    await activate(page, "Open guide preferences");
    await page.getByRole("textbox", { name: "Review note" }).focus();
    await page.keyboard.press("Escape");
    await expectOutput(page, "Outside dismissals: 1 · Closed with Escape inside the preferences.");
    assert.equal(await region().count(), 0);
    assert.equal(
      await page
        .getByRole("button", { name: "Open guide preferences" })
        .evaluate((element) => element === document.activeElement),
      true,
    );
    await page.keyboard.press("Enter");
    await activate(page, "Continue reading outside");
    await expectOutput(page, "Outside dismissals: 2 · Dismissed by an outside click.");
    assert.equal(await region().count(), 0);
    await activate(page, "Open guide preferences");
    await page.getByRole("textbox", { name: "Review note" }).fill("Keep the source aligned.");
    await activate(page, "Apply local preference");
    return [
      "keyboard trigger and inside activation preserve the region and exact local note",
      "ignored outside element preserves the region",
      "ignored trigger closes and reopens without counting an outside dismissal",
      "actual pointer drag beginning inside does not dismiss",
      "outside pointer click dismisses and increments only its own counter",
      "Escape closes and restores actual trigger focus without an outside dismissal",
      "recreated v-if target observes a native keyboard outside click",
    ];
  }

  if (name === "on-key-stroke") {
    const primary = page.getByRole("button", { name: "Primary review control", exact: true });
    const secondary = page.getByRole("button", { name: "Secondary review control", exact: true });
    const help = page.getByText(
      "Review one component at a time. Name its props, label its controls, and test empty data.",
      { exact: true },
    );
    await expectOutput(page, "Tip 1 of 3: Name the props");
    await secondary.focus();
    await page.keyboard.press("j");
    await expectOutput(page, "Tip 1 of 3: Name the props");
    await primary.focus();
    await page.keyboard.down("j");
    await page.keyboard.down("j");
    await page.keyboard.up("j");
    await expectOutput(page, "Tip 2 of 3: Label the controls");
    await page.keyboard.press("j");
    await page.keyboard.press("j");
    await expectOutput(page, "Tip 3 of 3: Handle the empty state");
    await page.keyboard.press("k");
    await expectOutput(page, "Tip 2 of 3: Label the controls");
    await page.keyboard.press("?");
    await help.waitFor();
    assert.equal(await help.count(), 1);
    await page.keyboard.press("?");
    await help.waitFor({ state: "detached" });
    assert.equal(await help.count(), 0);
    await page.getByRole("combobox", { name: "Shortcut target" }).focus();
    await page.keyboard.press("s");
    await page.keyboard.press("Tab");
    await primary.focus();
    await page.keyboard.press("k");
    await expectOutput(page, "Tip 2 of 3: Label the controls");
    await secondary.focus();
    await page.keyboard.press("k");
    await expectOutput(page, "Tip 1 of 3: Name the props");
    await activate(page, "Stop review shortcuts");
    await secondary.focus();
    await page.keyboard.press("j");
    await expectOutput(page, "Tip 1 of 3: Name the props");
    await activate(page, "Next review tip");
    await expectOutput(page, "Tip 2 of 3: Label the controls");
    assert.equal(
      await page.getByRole("button", { name: "Stop review shortcuts" }).isDisabled(),
      true,
    );
    await page
      .getByText("Shortcuts stopped. The native Previous/Next buttons still work.", { exact: true })
      .waitFor();
    return [
      "only the selected actual element receives J/K shortcuts",
      "repeated keydown is deduplicated and independent presses respect bounds",
      "question-mark key toggles actual help content",
      "keyboard target selection detaches old and attaches new element listener",
      "stop removes shortcuts while native navigation remains usable",
    ];
  }

  assert.equal(name, "event-listener", "every scoped DOM example has an interaction law");
  const first = page.getByRole("textbox", { name: "First workshop draft", exact: true });
  const second = page.getByRole("textbox", { name: "Second workshop draft", exact: true });
  const selector = page.getByRole("combobox", { name: "Observed draft" });
  await expectOutput(page, "Listening to first draft: yes · Observed input events: 0");
  await first.fill("Compiler draft revised");
  await expectOutput(page, "Listening to first draft: yes · Observed input events: 1");
  await page.getByText("Last observed text: Compiler draft revised", { exact: true }).waitFor();
  await second.fill("Unobserved editor revision");
  await expectOutput(page, "Listening to first draft: yes · Observed input events: 1");
  await selector.focus();
  await page.keyboard.press("s");
  await page.keyboard.press("Tab");
  await expectOutput(page, "Listening to second draft: yes · Observed input events: 1");
  await first.fill("Detached first draft");
  await expectOutput(page, "Listening to second draft: yes · Observed input events: 1");
  await second.fill("Observed editor revision");
  await expectOutput(page, "Listening to second draft: yes · Observed input events: 2");
  await page.getByText("Last observed text: Observed editor revision", { exact: true }).waitFor();
  await activate(page, "Pause input observation");
  await expectOutput(page, "Listening to second draft: no · Observed input events: 2");
  await second.fill("Edited while paused");
  await selector.focus();
  await page.keyboard.press("f");
  await page.keyboard.press("Tab");
  await first.fill("Current draft while paused");
  await expectOutput(page, "Listening to first draft: no · Observed input events: 2");
  await page.getByText("Last observed text: Observed editor revision", { exact: true }).waitFor();
  await activate(page, "Resume input observation");
  await expectOutput(page, "Listening to first draft: yes · Observed input events: 2");
  await second.fill("Still detached second draft");
  await expectOutput(page, "Listening to first draft: yes · Observed input events: 2");
  await first.fill("Resumed compiler revision");
  await expectOutput(page, "Listening to first draft: yes · Observed input events: 3");
  await page.getByText("Last observed text: Resumed compiler revision", { exact: true }).waitFor();
  assert.equal(await second.inputValue(), "Still detached second draft");
  return [
    "mounted actual input target attaches without observing untouched drafts",
    "native input event records exact text and excludes the other editable draft",
    "keyboard retarget removes the previous listener",
    "pause removes observation while fields and target selection remain writable",
    "resume follows the current target and ignores the old target",
  ];
}
