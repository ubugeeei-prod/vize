/** Browser laws for actual asynchronous producers and lifecycle-owned timers. */
import assert from "node:assert/strict";
import type { Page } from "playwright";

async function activate(page: Page, name: string): Promise<void> {
  await page.getByRole("button", { name, exact: true }).focus();
  await page.keyboard.press("Enter");
}

export async function exerciseAsyncExample(page: Page, name: string): Promise<string[]> {
  if (name === "use-interval") {
    assert.equal(await page.locator("output").textContent(), "0 seconds of review time");
    await activate(page, "Start review timer");
    await page.clock.runFor(2000);
    assert.equal(await page.locator("output").textContent(), "2 seconds of review time");
    await activate(page, "Pause review timer");
    await page.clock.runFor(2000);
    assert.equal(await page.locator("output").textContent(), "2 seconds of review time");
    await activate(page, "Reset review time");
    assert.equal(await page.locator("output").textContent(), "0 seconds of review time");
    await activate(page, "Start review timer");
    await page.clock.runFor(1000);
    assert.equal(await page.locator("output").textContent(), "1 second of review time");
    await activate(page, "Reset review time");
    assert.equal(await page.getByRole("button", { name: "Pause review timer" }).isEnabled(), true);
    await page.clock.runFor(1000);
    assert.equal(await page.locator("output").textContent(), "1 second of review time");
    return [
      "keyboard start/pause",
      "real interval ticks",
      "paused timer does not tick",
      "reset keeps active state",
      "resume continues ticks",
    ];
  }
  if (name === "use-timeout") {
    await activate(page, "Schedule reminder");
    await page.clock.runFor(500);
    await activate(page, "Cancel reminder");
    await page.clock.runFor(1000);
    assert.equal(await page.locator("output").textContent(), "Reminder canceled. Delivered: 0");
    assert.equal(await page.getByRole("button", { name: "Cancel reminder" }).isDisabled(), true);
    await activate(page, "Schedule reminder");
    await page.clock.runFor(500);
    await activate(page, "Schedule reminder");
    await page.clock.runFor(500);
    assert.equal(
      await page.locator("output").textContent(),
      "Reminder scheduled for one second from now. Delivered: 0",
    );
    await page.clock.runFor(500);
    assert.equal(await page.locator("output").textContent(), "Reminder delivered. Delivered: 1");
    await page.clock.runFor(2000);
    assert.equal(await page.locator("output").textContent(), "Reminder delivered. Delivered: 1");
    return [
      "keyboard schedule/cancel",
      "cancel prevents callback",
      "restart replaces deadline",
      "callback runs once",
    ];
  }
  assert.equal(name, "use-async-state", "every async example must have an observable law");
  await activate(page, "Try missing profile");
  assert.equal(await page.locator("output").textContent(), "Loading profile…");
  await page.clock.runFor(500);
  assert.equal(
    await page.locator("output").textContent(),
    "This profile could not be loaded. Try another profile.",
  );
  await activate(page, "Load Ada");
  await page.clock.runFor(500);
  assert.equal(await page.locator("output").textContent(), "Loaded Ada Lovelace");
  await activate(page, "Load Ada");
  assert.equal(
    await page.getByRole("region", { name: "Loaded profile" }).count(),
    0,
    "reset hides old data while loading",
  );
  await activate(page, "Load Grace");
  await page.clock.runFor(100);
  assert.equal(await page.locator("output").textContent(), "Loaded Grace Hopper");
  await page.clock.runFor(400);
  assert.equal(
    await page.locator("output").textContent(),
    "Loaded Grace Hopper",
    "older result is ignored",
  );
  return [
    "loading state",
    "actual rejected producer shows error",
    "successful retry clears error",
    "reset clears old data",
    "newest request ignores stale completion",
  ];
}
