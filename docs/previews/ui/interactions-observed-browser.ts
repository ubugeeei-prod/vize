/** Exercise actual browser clipboard permissions, scrolling, and native observers. */
import assert from "node:assert/strict";
import { setTimeout as deliveryOpportunity } from "node:timers/promises";
import type { FrameLocator, Locator, Page } from "playwright";

/** The real rendered documentation iframe must also retain clipboard access. */
export async function exerciseEmbeddedClipboard(page: Page, frame: FrameLocator): Promise<void> {
  await frame.getByText("Clipboard read permission: granted · Last failure: none").waitFor();
  const command = "  vp install @vizejs/composable vue  ";
  await frame.getByRole("textbox", { name: "Install command", exact: true }).fill(command);
  await frame.getByRole("button", { name: "Copy install command", exact: true }).focus();
  await page.keyboard.press("Enter");
  await frame.locator("output").filter({ hasText: "Copied using clipboard." }).waitFor();
  assert.equal(await page.evaluate(() => navigator.clipboard.readText()), command);
  assert.equal(await frame.locator("code").textContent(), command);
  await page.context().clearPermissions();
  const origin = new URL(page.url()).origin;
  await page.context().grantPermissions(["clipboard-write"], { origin });
  await frame.getByRole("button", { name: "Read clipboard text", exact: true }).focus();
  await page.keyboard.press("Enter");
  await frame
    .locator("output")
    .filter({ hasText: "Read did not complete: permission-denied." })
    .waitFor();
  assert.equal(await frame.locator("code").textContent(), command);
  await page.context().grantPermissions(["clipboard-read", "clipboard-write"], { origin });
  await page.keyboard.press("Enter");
  await frame.locator("output").filter({ hasText: "Read the clipboard text." }).waitFor();
}

async function activate(page: Page, label: string): Promise<void> {
  await page.getByRole("button", { name: label, exact: true }).focus();
  await page.keyboard.press("Enter");
}

async function toggle(page: Page, label: string): Promise<void> {
  await page.getByRole("checkbox", { name: label, exact: true }).focus();
  await page.keyboard.press("Space");
}

async function outputContains(page: Page, text: string): Promise<void> {
  await page.locator("output").filter({ hasText: text }).waitFor();
}

async function observePanel(page: Page, panel: Locator, name: string): Promise<number> {
  const contentWidth = await panel.evaluate((element) => {
    const style = getComputedStyle(element);
    return Math.round(
      element.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight),
    );
  });
  await outputContains(page, `Last observed: ${name} · Content box: ${contentWidth} ×`);
  return contentWidth;
}

async function nativeScrollCompleted(completion: Promise<void>, action: string): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    const deadline = setTimeout(() => reject(new Error(`${action} must finish scrolling`)), 10000);
    completion.then(
      () => {
        clearTimeout(deadline);
        resolve();
      },
      (error) => {
        clearTimeout(deadline);
        reject(error);
      },
    );
  });
}

export async function exerciseObservedBrowserExample(page: Page, name: string): Promise<string[]> {
  if (name === "use-clipboard") {
    await page.getByText("Clipboard read permission: granted · Last failure: none").waitFor();
    const command = page.getByRole("textbox", { name: "Install command", exact: true });
    const first = "  vp install @vizejs/composable  ";
    await command.fill(first);
    await activate(page, "Copy install command");
    await outputContains(page, "Copied using clipboard. Recently copied: yes.");
    assert.equal(await page.evaluate(() => navigator.clipboard.readText()), first);
    assert.equal(await page.locator("code").textContent(), first);
    await page.clock.runFor(1000);
    const second = "vp install @vizejs/composable vue";
    await command.fill(second);
    await activate(page, "Copy install command");
    await page.waitForFunction(
      (text) => document.querySelector("code")?.textContent === text,
      second,
    );
    await outputContains(page, "Copied using clipboard. Recently copied: yes.");
    await page.clock.runFor(500);
    await outputContains(page, "Recently copied: yes.");
    await page.clock.runFor(1000);
    await outputContains(page, "Recently copied: no.");
    const readText = "Review the deployed guide.";
    await page.evaluate((text) => navigator.clipboard.writeText(text), readText);
    await activate(page, "Read clipboard text");
    await outputContains(page, "Read the clipboard text. Recently copied: no.");
    assert.equal(await page.locator("code").textContent(), readText);
    const origin = new URL(page.url()).origin;
    await page.context().clearPermissions();
    await page.context().grantPermissions(["clipboard-write"], { origin });
    await activate(page, "Read clipboard text");
    await outputContains(page, "Read did not complete: permission-denied.");
    assert.equal(await page.locator("code").textContent(), readText);
    try {
      await page
        .getByText("Clipboard read permission: denied · Last failure: permission-denied")
        .waitFor();
    } catch (error) {
      const displayed = await page.evaluate(() => ({
        paragraphs: [...document.querySelectorAll("p")].map((element) => element.textContent),
        output: document.querySelector("output")?.textContent,
        text: document.querySelector("code")?.textContent,
        viewport: { width: innerWidth, height: innerHeight },
        hydration: document.documentElement.dataset.previewHydrationRetained,
        now: Date.now(),
      }));
      throw new Error(
        `Subscribed clipboard permission did not update: ${JSON.stringify(displayed)}`,
        {
          cause: error,
        },
      );
    }
    // Query only after observing the subscribed state. Chromium shares its native
    // PermissionStatus listener and query can update it without a change event.
    assert.equal(
      await page.evaluate(
        async () =>
          (await navigator.permissions.query({ name: "clipboard-read" as PermissionName })).state,
      ),
      "denied",
      "actual browser read permission is denied",
    );
    await page.context().grantPermissions(["clipboard-read", "clipboard-write"], { origin });
    await activate(page, "Read clipboard text");
    await outputContains(page, "Read the clipboard text. Recently copied: no.");
    await page.getByText("Clipboard read permission: granted · Last failure: none").waitFor();
    return [
      "native keyboard copy writes exact editable text including leading and trailing spaces to the real clipboard",
      "replacement copy restarts its 1500ms feedback deadline",
      "real clipboard read updates text without restarting expired copy feedback",
      "actual denied browser permission reports the failure and retains the last successful text",
      "restored browser permission permits a new read and clears the last failure",
    ];
  }

  if (name === "scroll") {
    const preview = page.getByRole("textbox", { name: "Preparation guide preview", exact: true });
    assert.equal(await preview.getAttribute("readonly"), "");
    assert.equal(
      await page.getByRole("button", { name: "Previous guide section" }).isDisabled(),
      true,
    );
    const outsideOffset = await page.evaluate(() => scrollY);
    await activate(page, "Next guide section");
    await outputContains(page, "Vertical offset: 120 pixels · At beginning: no");
    assert.equal(await preview.evaluate((element) => element.scrollTop), 120);
    const scrollSteps = [];
    for (let step = 0; step < 25; step += 1) {
      if (await page.getByRole("button", { name: "Next guide section" }).isDisabled()) break;
      const before = await preview.evaluate((element) => element.scrollTop);
      const maximum = await preview.evaluate(
        (element) => element.scrollHeight - element.clientHeight,
      );
      const expected = Math.min(before + 120, maximum);
      await activate(page, "Next guide section");
      await outputContains(page, `Vertical offset: ${Math.round(expected)} pixels ·`);
      const observed = await preview.evaluate((element) => element.scrollTop);
      scrollSteps.push({ before, expected, observed });
      assert.equal(
        observed,
        expected,
        "each native activation reaches its geometry-derived offset",
      );
    }
    assert.equal(
      await page.getByRole("button", { name: "Next guide section" }).isDisabled(),
      true,
      `bounded native scrolling must reach the bottom: ${JSON.stringify(scrollSteps)}`,
    );
    await outputContains(page, "At end: yes");
    assert.equal(
      await preview.evaluate(
        (element) => element.scrollTop + element.clientHeight >= element.scrollHeight - 1,
      ),
      true,
    );
    await activate(page, "Return to the beginning");
    await outputContains(page, "Vertical offset: 0 pixels · At beginning: yes");
    assert.equal(await preview.evaluate((element) => element.scrollTop), 0);
    await page.clock.runFor(250);
    await page.getByText("Scrolling activity: idle", { exact: true }).waitFor();
    await preview.focus();
    const scrollEnd = preview.evaluate(
      (element) =>
        new Promise<void>((resolve) => {
          element.addEventListener("scrollend", () => resolve(), { once: true });
        }),
    );
    await page.keyboard.press("PageDown");
    await page.clock.runFor(250);
    await nativeScrollCompleted(scrollEnd, "Native PageDown");
    await page.waitForFunction(() => (document.querySelector("textarea")?.scrollTop ?? 0) > 0);
    await outputContains(page, "At beginning: no");
    await page.clock.runFor(250);
    try {
      await page.getByText("Scrolling activity: idle", { exact: true }).waitFor();
    } catch (error) {
      const observed = await preview.evaluate((element) => ({
        scrollTop: element.scrollTop,
        scrollHeight: element.scrollHeight,
        clientHeight: element.clientHeight,
        output: document.querySelector("output")?.textContent,
        paragraphs: [...document.querySelectorAll("p")].map((item) => item.textContent),
        now: Date.now(),
      }));
      throw new Error(`Native PageDown did not settle: ${JSON.stringify(observed)}`, {
        cause: error,
      });
    }
    const nativeOffset = await preview.evaluate((element) => Math.round(element.scrollTop));
    await outputContains(page, `Vertical offset: ${nativeOffset} pixels · At beginning: no`);
    assert.equal(await page.evaluate(() => scrollY), outsideOffset);
    await activate(page, "Return to the beginning");
    await activate(page, "Next guide section");
    return [
      "writable y scrolls the actual bounded textarea by 120 pixels",
      "actual lower edge clamps scrolling and disables the Next control",
      "scrollTo resets the actual offset and restores the beginning boundary",
      "native PageDown updates the same observed offset and activity settles",
      "element navigation preserves the surrounding document offset",
    ];
  }

  if (name === "resize-observer") {
    const first = page.getByRole("region", { name: "First guide panel", exact: true });
    const second = page.getByRole("region", { name: "Second guide panel", exact: true });
    const firstWidth = await observePanel(page, first, "First guide panel");
    await toggle(page, "Wide first panel");
    const narrowWidth = await observePanel(page, first, "First guide panel");
    assert.ok(narrowWidth < firstWidth);
    const unobservedOutput = await page.locator("output").textContent();
    await toggle(page, "Wide second panel");
    await deliveryOpportunity(100);
    assert.equal(await page.locator("output").textContent(), unobservedOutput);
    await page.getByRole("combobox", { name: "Observed guide panel" }).focus();
    await page.keyboard.press("s");
    await page.keyboard.press("Tab");
    const secondWidth = await observePanel(page, second, "Second guide panel");
    assert.ok(secondWidth > narrowWidth);
    const detachedOutput = await page.locator("output").textContent();
    await toggle(page, "Wide first panel");
    await deliveryOpportunity(100);
    assert.equal(await page.locator("output").textContent(), detachedOutput);
    await toggle(page, "Wide second panel");
    assert.ok((await observePanel(page, second, "Second guide panel")) < secondWidth);
    await activate(page, "Stop size observation");
    const stoppedOutput = await page.locator("output").textContent();
    await toggle(page, "Wide second panel");
    await deliveryOpportunity(100);
    assert.equal(await page.locator("output").textContent(), stoppedOutput);
    assert.equal(
      await page.getByRole("button", { name: "Stop size observation" }).isDisabled(),
      true,
    );
    assert.equal(await page.getByRole("checkbox", { name: "Wide second panel" }).isChecked(), true);
    return [
      "native content-box delivery agrees with actual rendered padding-excluded dimensions",
      "keyboard checkbox resizing changes the selected panel's real measured width",
      "unselected panel resizing leaves the measurement and delivery count unchanged",
      "keyboard target selection disconnects the previous panel and observes the new panel",
      "stop retains the last delivered measurement while native panel resizing remains usable",
    ];
  }

  assert.equal(
    name,
    "intersection-observer",
    "every observed browser example has an interaction law",
  );
  const viewport = page.getByRole("region", { name: "Scrollable guide cards", exact: true });
  await outputContains(page, "Last observed featured visibility: below 60% · Visible ratio: 0%");
  await activate(page, "Show featured guide");
  await outputContains(page, "Last observed featured visibility: at least 60%");
  assert.ok((await viewport.evaluate((element) => element.scrollTop)) > 0);
  assert.equal(await page.locator("article.visible").count(), 1);
  await activate(page, "Pause visibility observation");
  const pausedOutput = await page.locator("output").textContent();
  await activate(page, "Return to preparation");
  assert.equal(await viewport.evaluate((element) => element.scrollTop), 0);
  await deliveryOpportunity(100);
  assert.equal(await page.locator("output").textContent(), pausedOutput);
  await page.getByText("Observation: paused.", { exact: true }).waitFor();
  await activate(page, "Resume visibility observation");
  await outputContains(page, "Last observed featured visibility: below 60% · Visible ratio: 0%");
  assert.equal(await page.locator("article.visible").count(), 0);
  await activate(page, "Show featured guide");
  await outputContains(page, "Last observed featured visibility: at least 60%");
  await activate(page, "Stop visibility observation");
  const stoppedOutput = await page.locator("output").textContent();
  await activate(page, "Return to preparation");
  await deliveryOpportunity(100);
  assert.equal(await page.locator("output").textContent(), stoppedOutput);
  for (const label of [
    "Pause visibility observation",
    "Resume visibility observation",
    "Stop visibility observation",
  ])
    assert.equal(await page.getByRole("button", { name: label }).isDisabled(), true);
  return [
    "real ancestor-root geometry crosses the native 60 percent intersection threshold and updates the highlight",
    "pause retains the explicitly last-observed readout while real viewport scrolling continues",
    "resume observes the current hidden position and clears the highlight",
    "stop permanently disables observer controls while the native scrolling buttons remain usable",
  ];
}
