/** Exercise actual document focus, pointer delays, held actions, and native typing. */
import assert from "node:assert/strict";
import type { Locator, Page } from "playwright";

async function activate(page: Page, label: string): Promise<void> {
  await page.getByRole("button", { name: label, exact: true }).focus();
  await page.keyboard.press("Enter");
}

async function outputContains(page: Page, text: string): Promise<void> {
  await page.locator("output").filter({ hasText: text }).waitFor();
}

async function pointerPosition(target: Locator): Promise<{ x: number; y: number }> {
  const box = await target.boundingBox();
  assert.ok(box);
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

export async function exerciseScopedInputExample(page: Page, name: string): Promise<string[]> {
  if (name === "element-bounding") {
    const panel = page.getByRole("region", { name: "Measured guide panel", exact: true });
    const toggle = async (label: string): Promise<void> => {
      await page.getByRole("checkbox", { name: label, exact: true }).focus();
      await page.keyboard.press("Space");
    };
    const rectangle = async (): Promise<string> =>
      panel.evaluate((element) => {
        const { left, top, width, height } = element.getBoundingClientRect();
        return `rectangle: left ${Math.round(left)}, top ${Math.round(top)}, width ${Math.round(width)}, height ${Math.round(height)} CSS pixels`;
      });
    await outputContains(page, await rectangle());
    await toggle("Wide guide panel");
    await outputContains(page, await rectangle());
    assert.equal(await panel.evaluate((element) => element.getBoundingClientRect().width), 230);
    const prior = await page.locator("output").textContent();
    const left = await panel.evaluate((element) => element.getBoundingClientRect().left);
    await toggle("Shift guide panel");
    assert.equal(
      await panel.evaluate((element) => element.getBoundingClientRect().left),
      left + 40,
    );
    assert.equal(await page.locator("output").textContent(), prior);
    await activate(page, "Measure current rectangle");
    await outputContains(page, await rectangle());
    await toggle("Show guide panel");
    assert.equal(await panel.count(), 0);
    await outputContains(page, "rectangle: left 0, top 0, width 0, height 0 CSS pixels");
    await toggle("Show guide panel");
    await outputContains(page, await rectangle());
    await activate(page, "Stop rectangle observation");
    const stoppedOutput = await page.locator("output").textContent();
    await toggle("Wide guide panel");
    assert.equal(await panel.evaluate((element) => element.getBoundingClientRect().width), 160);
    await page.clock.runFor(200);
    assert.equal(await page.locator("output").textContent(), stoppedOutput);
    assert.equal(
      await page.getByRole("button", { name: "Measure current rectangle" }).isDisabled(),
      true,
    );
    return [
      "recorded native border-box geometry agrees with actual viewport-relative bounds",
      "native checkbox resizing delivers the new actual width automatically",
      "a real 40px transform retains the previous measurement until public update refreshes it",
      "target unmount resets every geometry value and remount observes the recreated actual panel",
      "stop retains the explicitly last-recorded rectangle while native resizing remains usable",
    ];
  }

  if (name === "element-hover") {
    const first = page.getByRole("region", { name: "Compiler guide card", exact: true });
    const second = page.getByRole("region", { name: "Keyboard guide card", exact: true });
    const advice = page.getByText(
      "Review the selected card's behavior, diagnostics, and native controls.",
      { exact: true },
    );
    const outside = await pointerPosition(
      page.getByRole("button", { name: "Show persistent guidance" }),
    );
    const firstPosition = await pointerPosition(first);
    await page.mouse.move(firstPosition.x, firstPosition.y);
    await page.clock.runFor(299);
    await outputContains(page, "Delayed pointer hover: no · Persistent guidance: no");
    assert.equal(await advice.count(), 0);
    await page.clock.runFor(1);
    await outputContains(page, "Delayed pointer hover: yes · Persistent guidance: no");
    assert.equal(await advice.count(), 1);
    await page.mouse.move(outside.x, outside.y);
    await page.clock.runFor(199);
    assert.equal(await advice.count(), 1);
    await page.mouse.move(firstPosition.x, firstPosition.y);
    await page.clock.runFor(201);
    assert.equal(await advice.count(), 1, "return cancels delayed departure");
    await page.mouse.move(outside.x, outside.y);
    await page.clock.runFor(200);
    await outputContains(page, "Delayed pointer hover: no · Persistent guidance: no");
    assert.equal(await advice.count(), 0);
    await page.mouse.move(firstPosition.x, firstPosition.y);
    await page.clock.runFor(100);
    await page.getByRole("combobox", { name: "Guidance target" }).focus();
    await page.keyboard.press("k");
    await page.keyboard.press("Tab");
    await page.clock.runFor(300);
    await outputContains(page, "Delayed pointer hover: no · Persistent guidance: no");
    assert.equal(await advice.count(), 0, "retarget cancels pending old-target entry");
    const secondPosition = await pointerPosition(second);
    await page.mouse.move(secondPosition.x, secondPosition.y);
    await page.clock.runFor(300);
    await outputContains(page, "Delayed pointer hover: yes · Persistent guidance: no");
    await page.mouse.move(outside.x, outside.y);
    await page.clock.runFor(200);
    await activate(page, "Show persistent guidance");
    await outputContains(page, "Delayed pointer hover: no · Persistent guidance: yes");
    assert.equal(await advice.count(), 1);
    assert.equal(
      await page
        .getByRole("button", { name: "Hide persistent guidance" })
        .getAttribute("aria-pressed"),
      "true",
    );
    await page.keyboard.press("Space");
    assert.equal(await advice.count(), 0);
    await activate(page, "Show persistent guidance");
    return [
      "actual pointer entry delivers only at the 300ms boundary",
      "actual departure waits 200ms and returning cancels its pending timer",
      "keyboard retarget cancels old-target entry and resets the delayed flag",
      "the new real section receives pointer guidance",
      "native Enter and Space independently expose the advice and synchronize aria-pressed",
    ];
  }

  if (name === "on-long-press") {
    const reveal = page.getByRole("button", { name: "Reveal review note", exact: true });
    const advice = page.getByText(
      "Check the deployed keyboard path before calling a fix delivered.",
      { exact: true },
    );
    await activate(page, "Reveal review note");
    assert.equal(await advice.count(), 1);
    await outputContains(page, "Long-press deliveries: 0 · Last pointer: none");
    await activate(page, "Hide review note");
    const position = await pointerPosition(reveal);
    await page.mouse.move(position.x, position.y);
    await page.mouse.down();
    await page.clock.runFor(499);
    assert.equal(await advice.count(), 0);
    await page.mouse.up();
    await outputContains(page, "Long-press deliveries: 0 · Last pointer: none");
    await page
      .getByText("Last pointer release: long press no · 499 ms · 0 pixels.", { exact: true })
      .waitFor();
    assert.equal(await advice.count(), 1, "native short click still reveals the note");
    await activate(page, "Hide review note");
    await page.mouse.down();
    await page.clock.runFor(499);
    assert.equal(await advice.count(), 0);
    await page.clock.runFor(1);
    await outputContains(page, "Long-press deliveries: 1 · Last pointer: mouse");
    assert.equal(await advice.count(), 1, "held action reveals before release");
    await page.clock.runFor(1000);
    await outputContains(page, "Long-press deliveries: 1 · Last pointer: mouse");
    await page.mouse.up();
    await page
      .getByText("Last pointer release: long press yes · 1500 ms · 0 pixels.", { exact: true })
      .waitFor();
    await activate(page, "Hide review note");
    await page.mouse.down();
    await page.mouse.move(position.x + 20, position.y);
    await page.clock.runFor(500);
    await outputContains(page, "Long-press deliveries: 1 · Last pointer: mouse");
    assert.equal(await advice.count(), 0);
    await page.mouse.up();
    await page
      .getByText("Last pointer release: long press no · 500 ms · 20 pixels.", { exact: true })
      .waitFor();
    await activate(page, "Stop long-press observation");
    await activate(page, "Hide review note");
    await page.mouse.move(position.x, position.y);
    await page.mouse.down();
    await page.clock.runFor(700);
    await outputContains(page, "Long-press deliveries: 1 · Last pointer: mouse");
    assert.equal(await advice.count(), 0);
    await page.mouse.up();
    assert.equal(await advice.count(), 1, "stopped held observation leaves native click usable");
    return [
      "native Enter reveals the same note without pretending to be a pointer long press",
      "499ms early release cancels timed delivery while its native click remains usable",
      "real pointer hold reveals at 500ms before release and delivers once despite continued holding",
      "native release reports actual elapsed time and initiating pointer type",
      "20px real pointer movement cancels the held action and reports release distance",
      "stop cancels held actions while independent native reveal/hide controls remain usable",
    ];
  }

  assert.equal(name, "on-start-typing", "every scoped input example has an interaction law");
  const reading = page.getByRole("button", { name: "Reading control", exact: true });
  const search = page.getByRole("searchbox", { name: "Search local guides", exact: true });
  const note = page.getByRole("textbox", { name: "Workshop note", exact: true });
  const guides = page.getByRole("list", { name: "Matching local guides" }).getByRole("listitem");
  await reading.focus();
  await page.keyboard.press("Space");
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("Control+a");
  await outputContains(page, "Typing shortcut deliveries: 0 · Active: yes");
  assert.equal(await reading.evaluate((element) => element === document.activeElement), true);
  await page.keyboard.press("c");
  await outputContains(page, "Typing shortcut deliveries: 1 · Active: yes");
  assert.equal(await search.evaluate((element) => element === document.activeElement), true);
  await search.fill("");
  await search.pressSequentially("linter");
  assert.deepEqual(await guides.allTextContents(), ["Linter guide"]);
  await outputContains(page, "Typing shortcut deliveries: 1 · Active: yes");
  await note.fill("");
  await note.pressSequentially("Native draft remains editable");
  assert.equal(await note.evaluate((element) => element === document.activeElement), true);
  await outputContains(page, "Typing shortcut deliveries: 1 · Active: yes");
  await activate(page, "Stop typing shortcut");
  await reading.focus();
  await page.keyboard.press("x");
  assert.equal(await reading.evaluate((element) => element === document.activeElement), true);
  await outputContains(page, "Typing shortcut deliveries: 1 · Active: no");
  await search.fill("keyboard");
  assert.deepEqual(await guides.allTextContents(), ["Keyboard guide"]);
  return [
    "native Space, navigation, and Control-letter shortcut preserve reading focus without delivering",
    "a printable native key redirects actual focus to the search input once",
    "native search typing filters actual local data without re-triggering the shortcut",
    "native note editing retains its focus and exact text without redirecting to search",
    "stop permanently removes the redirect while native fields and filtering remain usable",
  ];
}
