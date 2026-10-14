/** Assert collection, formatting, time and app-state behavior in the complete source-owned SFCs. */
import assert from "node:assert/strict";
import type { Page } from "playwright";

const visible = (text: string | null) =>
  (text ?? "")
    .replace(/[  ]/g, " ")
    .replace(/\s+/g, " ")
    .trim();

async function output(page: Page): Promise<string> {
  return visible(await page.locator("output").first().textContent());
}

async function expectOutput(page: Page, text: string): Promise<void> {
  await page
    .waitForFunction(
      (expected) =>
        (document.querySelector("output")?.textContent ?? "")
          .replace(/[  ]/g, " ")
          .replace(/\s+/g, " ")
          .trim() === expected,
      text,
    )
    .catch(async () => assert.equal(await output(page), text));
}

async function press(page: Page, name: string): Promise<void> {
  const button = page.getByRole("button", { name, exact: true });
  await button.focus();
  await page.keyboard.press("Enter");
}

async function choose(page: Page, label: string, option: string): Promise<void> {
  await page.getByRole("combobox", { name: label }).selectOption({ label: option });
}

async function listItems(page: Page, name: string): Promise<string[]> {
  return (await page.getByRole("list", { name }).getByRole("listitem").allTextContents()).map(
    visible,
  );
}

export async function exerciseDataStateExample(page: Page, name: string): Promise<string[]> {
  switch (name) {
    case "use-array":
      await expectOutput(page, "Still to pack: Charger, Umbrella");
      await page.getByRole("checkbox", { name: "Charger" }).check();
      await page.getByRole("checkbox", { name: "Umbrella" }).check();
      await expectOutput(page, "Everything is packed.");
      await page.getByRole("checkbox", { name: "Passport" }).uncheck();
      await expectOutput(page, "Still to pack: Passport");
      return ["filter follows nested changes", "every reports completion"];
    case "use-set":
      await expectOutput(page, "Following: Vue");
      await press(page, "Vite");
      await press(page, "TypeScript");
      await press(page, "Vue");
      await expectOutput(page, "Following: Vite, TypeScript");
      assert.equal(
        await page.getByRole("button", { name: "Vite", exact: true }).getAttribute("aria-pressed"),
        "true",
      );
      await press(page, "Clear topics");
      await expectOutput(page, "No topics selected.");
      return ["toggle adds and removes", "pressed state follows the set"];
    case "use-map":
      await press(page, "Add Sticker");
      await press(page, "Add Sticker");
      await press(page, "Add Mug");
      assert.deepEqual(await listItems(page, "Cart"), ["Sticker × 2", "Mug × 1"]);
      await expectOutput(page, "2 products, 3 items");
      await press(page, "Empty cart");
      await expectOutput(page, "0 products, 0 items");
      return ["update accumulates per key", "clear empties the map"];
    case "use-queue":
      await press(page, "Print next");
      await expectOutput(page, "Printed Invoice.");
      await press(page, "Add print job");
      assert.deepEqual(await listItems(page, "Print queue"), ["Ticket", "Report 1"]);
      await press(page, "Print next");
      await press(page, "Print next");
      await expectOutput(page, "Printed Report 1.");
      assert.equal(await page.getByRole("button", { name: "Print next" }).isDisabled(), true);
      return ["first in, first out", "empty queue disables printing"];
    case "use-stack":
      assert.equal(await page.getByRole("button", { name: "Back" }).isDisabled(), true);
      await press(page, "Open Docs");
      await press(page, "Open Guide");
      await expectOutput(page, "Current page: Guide · History: Home › Docs › Guide");
      await press(page, "Back");
      await expectOutput(page, "Current page: Docs · History: Home › Docs");
      return ["push opens a page", "pop returns to the previous page"];
    case "use-sorted":
      assert.deepEqual(await listItems(page, "Leaderboard"), ["95", "72", "64"]);
      await press(page, "Add score");
      assert.deepEqual(await listItems(page, "Leaderboard"), ["95", "80", "72", "64"]);
      await expectOutput(page, "Entered order: 72, 95, 64, 80");
      return ["sorted view", "source order unchanged"];
    case "use-sorted-locale":
      await expectOutput(page, "Anna, Ärla, Örjan, Oskar, Zoe");
      await choose(page, "Sort for", "Swedish");
      await expectOutput(page, "Anna, Oskar, Zoe, Ärla, Örjan");
      return ["German collation", "Swedish collation"];
    case "use-memoize":
      await press(page, "Get quote");
      await press(page, "Get quote");
      await expectOutput(page, "Shipping: €12 · Calculated 1 times");
      await choose(page, "Shipping zone", "Asia");
      await press(page, "Get quote");
      await expectOutput(page, "Shipping: €18 · Calculated 2 times");
      await press(page, "Forget quotes");
      await choose(page, "Shipping zone", "Europe");
      await press(page, "Get quote");
      await expectOutput(page, "Shipping: €12 · Calculated 3 times");
      return ["repeated arguments hit the cache", "clear forgets results"];
    case "use-id-generator": {
      await press(page, "Add guest");
      await press(page, "Add guest");
      const ids = await page
        .locator("input")
        .evaluateAll((inputs) => inputs.map((input) => input.id));
      assert.equal(new Set(ids).size, 3);
      await page.getByLabel("Guest 3 name").fill("Grace");
      assert.equal(await page.locator(`input[id="${ids[2]}"]`).inputValue(), "Grace");
      await expectOutput(page, "3 guest fields with unique ids");
      return ["each field has a unique id", "labels target their own input"];
    }
    case "use-date-format":
      await expectOutput(page, "Starts 2026-10-24 01:00");
      await choose(page, "Time zone", "Tokyo");
      await expectOutput(page, "Starts 2026-10-24 10:00");
      await choose(page, "Time zone", "UTC");
      await choose(page, "Format", "ddd D MMM, h:mm A");
      await expectOutput(page, "Starts Sat 24 Oct, 1:00 AM");
      return ["time zone changes the hour", "pattern changes the format"];
    case "use-date-time-format":
      await expectOutput(page, "October 24, 2026");
      await choose(page, "Reader locale", "German (Germany)");
      await expectOutput(page, "24. Oktober 2026");
      await choose(page, "Reader locale", "Japanese (Japan)");
      await expectOutput(page, "2026年10月24日");
      return ["locale changes the date format"];
    case "use-display-names":
      await expectOutput(page, "Japanese");
      await choose(page, "Reader locale", "German");
      await expectOutput(page, "Japanisch");
      await choose(page, "Reader locale", "Japanese");
      await expectOutput(page, "日本語");
      return ["language names follow the reader locale"];
    case "use-segmenter":
      await expectOutput(page, "10 characters · 2 words");
      await page.getByRole("textbox", { name: "Status update" }).fill("Deploying now 🎉");
      await expectOutput(page, "15 characters · 2 words");
      return ["emoji count as one character", "counts follow edits"];
    case "use-now": {
      const pageClock = async () =>
        `${new Date(await page.evaluate(() => Date.now())).toISOString().slice(11, 19)} UTC`;
      await page.clock.runFor(2000);
      const ticking = await pageClock();
      await expectOutput(page, ticking);
      await press(page, "Pause clock");
      await page.clock.runFor(2000);
      await expectOutput(page, ticking);
      await press(page, "Resume clock");
      await page.clock.runFor(1000);
      await expectOutput(page, await pageClock());
      return ["clock follows the current time", "pause stops updates"];
    }
    case "use-time-ago":
      await expectOutput(page, "Not saved yet.");
      await press(page, "Save draft");
      await expectOutput(page, "Saved now");
      await page.clock.runFor(60_000);
      await expectOutput(page, "Saved 1 minute ago");
      return ["relative text after saving", "text refreshes over time"];
    case "use-supported": {
      const expected = await page.evaluate(() => [
        "share" in navigator,
        "clipboard" in navigator,
        "startViewTransition" in document,
      ]);
      const items = await listItems(page, "Browser features");
      assert.deepEqual(
        items.map((item) => item.endsWith(": available")),
        expected,
      );
      return ["checks match the browser"];
    }
    case "use-mounted": {
      const width = await page.evaluate(() => window.innerWidth);
      await expectOutput(page, `Viewport width: ${width}px`);
      return ["browser-only value after mount"];
    }
    case "use-confirm-dialog":
      await press(page, "Delete draft");
      await page.getByRole("alertdialog", { name: "Delete the draft?" }).waitFor();
      await press(page, "Keep draft");
      await expectOutput(page, "Kept the draft.");
      await press(page, "Delete draft");
      await press(page, "Delete");
      await expectOutput(page, "Deleted the draft.");
      assert.equal(await page.getByRole("alertdialog").count(), 0);
      return ["cancel resolves as canceled", "confirm resolves as confirmed"];
    case "use-async-queue":
      assert.deepEqual(await listItems(page, "Steps"), [
        "Pick a photo: pending",
        "Resize it: pending",
        "Upload it: pending",
      ]);
      await press(page, "Run steps");
      await expectOutput(page, "Uploaded photo.jpg resized to 1200 px");
      assert.deepEqual(await listItems(page, "Steps"), [
        "Pick a photo: fulfilled",
        "Resize it: fulfilled",
        "Upload it: fulfilled",
      ]);
      return ["steps run in order", "results chain between steps"];
    case "use-v-model":
      await expectOutput(page, "Hello, Ada!");
      await page.getByRole("textbox", { name: "Nickname" }).fill("Grace");
      await expectOutput(page, "Hello, Grace!");
      return ["passive model keeps a local value"];
    case "use-form-field-props": {
      const input = page.getByRole("textbox", { name: "Newsletter email" });
      await input.fill("ada@");
      await press(page, "Subscribe");
      await page.getByText("Enter a complete email address.").waitFor();
      assert.equal(await input.getAttribute("aria-invalid"), "true");
      const errorId = await input.getAttribute("aria-errormessage");
      assert.ok(errorId);
      assert.equal(
        await page.locator(`[id="${errorId}"]`).textContent(),
        "Enter a complete email address.",
      );
      await input.fill("ada@example.test");
      await page.waitForFunction(
        () => !document.querySelector("input")?.hasAttribute("aria-invalid"),
      );
      return ["label names the input", "error message is referenced while invalid"];
    }
    case "use-route-query":
      await expectOutput(page, "Query: (none)");
      await page.getByRole("tab", { name: "pricing" }).click();
      await expectOutput(page, "Query: tab=pricing");
      assert.equal(
        await page.getByRole("tab", { name: "pricing" }).getAttribute("aria-selected"),
        "true",
      );
      await page.getByRole("tab", { name: "overview" }).click();
      await expectOutput(page, "Query: (none)");
      return ["tab writes the query", "default value is omitted"];
    case "use-url-hash":
      await expectOutput(page, "No section in the URL.");
      await press(page, "Go to install");
      await expectOutput(page, "Hash: #install");
      assert.equal(await page.evaluate(() => location.hash), "#install");
      await page.evaluate(() => {
        location.hash = "usage";
      });
      await expectOutput(page, "Hash: #usage");
      return ["writes the hash", "follows external hash changes"];
    case "use-url-search-params": {
      await page.getByRole("searchbox", { name: "Search" }).fill("vapor");
      await press(page, "Next page");
      await expectOutput(page, "Searching “vapor”, page 2");
      const search = new URLSearchParams(await page.evaluate(() => location.search));
      assert.equal(search.get("q"), "vapor");
      assert.equal(search.get("page"), "2");
      assert.ok(search.get("composable"), "other parameters are kept");
      await press(page, "Previous page");
      await page.waitForFunction(() => !new URLSearchParams(location.search).has("page"));
      return ["inputs write typed parameters", "defaults are omitted"];
    }
    case "use-cookie":
      await press(page, "Use dark theme");
      await expectOutput(page, "Saved theme: dark");
      assert.match(await page.evaluate(() => document.cookie), /docs-example-theme=dark/);
      await press(page, "Forget preference");
      await expectOutput(page, "No saved theme.");
      assert.doesNotMatch(await page.evaluate(() => document.cookie), /docs-example-theme=/);
      return ["writes the cookie", "remove deletes it"];
    case "use-indexed-db": {
      const notes = page.getByRole("textbox", { name: "Notes" });
      await page.waitForFunction(() => !document.querySelector("textarea")?.disabled);
      await notes.fill("Remember the slides");
      await expectOutput(page, "19 characters saved in this browser.");
      await page.waitForTimeout(100);
      await page.reload({ waitUntil: "networkidle" });
      await page.waitForFunction(
        () => document.querySelector("textarea")?.value === "Remember the slides",
      );
      await page.getByRole("textbox", { name: "Notes" }).fill("");
      return ["value persists across reload"];
    }
    default:
      throw new Error(`No data and state law for ${name}`);
  }
}
