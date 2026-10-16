/** Exercise native scrolling and the real rendered slice, not a simulated viewport. */
import assert from "node:assert/strict";
import type { Page } from "playwright";

export async function exerciseVirtualListExample(page: Page): Promise<string[]> {
  const region = page.getByRole("region", { name: "Guide catalogue" });
  const rows = page.getByRole("list", { name: "Matching guides" }).getByRole("listitem");
  const firstRow = rows.first();
  await page.waitForFunction(() => {
    const row = document.querySelector('[role="listitem"]');
    return row?.getAttribute("aria-posinset") === "1";
  });
  assert.ok((await rows.count()) < 20, "1,000 records do not create 1,000 DOM rows");
  assert.equal(await firstRow.getAttribute("aria-setsize"), "1000");
  assert.equal(await firstRow.evaluate((row) => row.getBoundingClientRect().height), 40);

  await region.focus();
  await page.keyboard.press("End");
  await page.getByText("Guide 1000", { exact: true }).waitFor();
  assert.equal(await region.evaluate((element) => element === document.activeElement), true);
  assert.equal(await rows.last().getAttribute("aria-posinset"), "1000");
  assert.equal(await page.getByText("Guide 0001", { exact: true }).count(), 0);
  assert.ok((await rows.count()) < 20);
  await page.keyboard.press("Home");
  await page.getByText("Guide 0001", { exact: true }).waitFor();
  await page.keyboard.press("PageDown");
  await page.waitForFunction(
    () => Number(document.querySelector('[role="listitem"]')?.getAttribute("aria-posinset")) > 1,
  );

  const search = page.getByRole("searchbox", { name: "Find a guide" });
  await search.fill("099");
  await page.getByText("11 matching guides", { exact: true }).waitFor();
  await page.getByText("Guide 0099", { exact: true }).waitFor();
  assert.equal(await search.evaluate((element) => element === document.activeElement), true);
  assert.equal(await firstRow.getAttribute("aria-posinset"), "1");
  assert.equal(await firstRow.getAttribute("aria-setsize"), "11");
  await search.fill("missing");
  await page.getByText("No guides match. Try a different search.").waitFor();
  assert.equal(await rows.count(), 0);
  assert.equal(await page.getByRole("button", { name: "Last result" }).isDisabled(), true);
  assert.equal(await search.evaluate((element) => element === document.activeElement), true);

  await search.fill("");
  await page.getByRole("button", { name: "Last result" }).focus();
  await page.keyboard.press("Enter");
  await page.getByText("Guide 1000", { exact: true }).waitFor();
  await page.getByRole("button", { name: "First result" }).focus();
  await page.keyboard.press("Space");
  await page.getByText("Guide 0001", { exact: true }).waitFor();
  await region.focus();
  await page.keyboard.press("End");
  await page.getByText("Guide 1000", { exact: true }).waitFor();
  return [
    "native Home, End and PageDown change a bounded DOM slice while retaining list focus",
    "40px rows expose source positions and total size to assistive technology",
    "filter resets the scroll position and retains input focus",
    "empty results disable jump controls and recover without losing input focus",
    "Enter and Space activate last/first result controls",
  ];
}
