/** Assert page resource, network, worker and scheduling behavior in the complete source-owned SFCs. */
import assert from "node:assert/strict";
import type { Page } from "playwright";

const normalize = (text: string | null) => (text ?? "").replace(/\s+/g, " ").trim();

async function expectOutput(page: Page, expected: string | RegExp): Promise<string> {
  await page
    .waitForFunction(
      ({ source, flags, text }) => {
        const actual = (document.querySelector("output")?.textContent ?? "")
          .replace(/\s+/g, " ")
          .trim();
        return source === null ? actual === text : new RegExp(source, flags).test(actual);
      },
      typeof expected === "string"
        ? { source: null, flags: "", text: expected }
        : { source: expected.source, flags: expected.flags, text: "" },
      { timeout: 10_000 },
    )
    .catch(() => undefined);
  const actual = normalize(await page.locator("output").first().textContent());
  if (typeof expected === "string") assert.equal(actual, expected);
  else assert.match(actual, expected);
  return actual;
}

async function press(page: Page, name: string): Promise<void> {
  const button = page.getByRole("button", { name, exact: true });
  await button.focus();
  await page.keyboard.press("Enter");
}

export async function exerciseBrowserPageExample(page: Page, name: string): Promise<string[]> {
  switch (name) {
    case "use-css-var": {
      await press(page, "Blue accent");
      await expectOutput(page, "#3b82f6");
      const value = await page
        .locator("article")
        .evaluate((element) => getComputedStyle(element).getPropertyValue("--accent").trim());
      assert.equal(value, "#3b82f6");
      return ["writes the variable on the element"];
    }
    case "use-document-title":
      await press(page, "Receive message");
      await press(page, "Receive message");
      await page.waitForFunction(() => document.title === "(2) Inbox");
      await press(page, "Mark all read");
      await page.waitForFunction(() => document.title === "Inbox");
      return ["title follows the unread count"];
    case "use-favicon": {
      await press(page, "Build failed");
      await page.waitForFunction(() =>
        decodeURIComponent(
          document.querySelector<HTMLLinkElement>('link[rel="icon"]')?.href ?? "",
        ).includes("#dc2626"),
      );
      await press(page, "Build passed");
      await page.waitForFunction(() =>
        decodeURIComponent(
          document.querySelector<HTMLLinkElement>('link[rel="icon"]')?.href ?? "",
        ).includes("#16a34a"),
      );
      return ["icon link follows the status"];
    }
    case "use-style-tag": {
      const outline = () =>
        page.locator(".example-note").evaluate((element) => getComputedStyle(element).outlineStyle);
      assert.equal(await outline(), "none");
      await press(page, "Add highlight styles");
      await expectOutput(page, "Style tag added.");
      assert.equal(await outline(), "solid");
      await press(page, "Remove highlight styles");
      await expectOutput(page, "No style tag.");
      assert.equal(await outline(), "none");
      return ["load inserts the rules", "unload removes them"];
    }
    case "use-script-tag":
      await expectOutput(page, "Script status: idle");
      await press(page, "Load widget");
      await expectOutput(page, "Script status: loaded");
      assert.equal(
        await page.evaluate(
          () =>
            (window as unknown as { exampleWidget?: { version: string } }).exampleWidget?.version,
        ),
        "2.1",
      );
      await press(page, "Remove widget");
      await expectOutput(page, "Script status: idle");
      return ["loads on demand", "unload removes the tag"];
    case "use-object-url": {
      await page.getByRole("textbox", { name: "Notes" }).fill("Slides ready");
      await expectOutput(page, "12 bytes ready to download");
      const href = await page
        .getByRole("link", { name: "Download notes.txt" })
        .getAttribute("href");
      assert.match(href ?? "", /^blob:/);
      assert.equal(
        await page.evaluate(async (url) => (await fetch(url)).text(), href ?? ""),
        "Slides ready",
      );
      return ["link serves the current text"];
    }
    case "use-image":
      await expectOutput(page, "Loaded 64 × 64");
      await page.getByRole("combobox", { name: "Avatar source" }).selectOption("Broken");
      await expectOutput(page, "Could not load the avatar; showing initials instead.");
      return ["loaded state with size", "error fallback"];
    case "use-fetch":
      await expectOutput(page, "Not checked yet.");
      await press(page, "Check for updates");
      await expectOutput(page, "Latest version: 0.449.0");
      return ["fetches on demand"];
    case "use-event-source":
      await press(page, "Connect");
      await expectOutput(page, "Status: open");
      await page.clock.runFor(3000);
      await expectOutput(page, "Status: open · Build finished");
      await press(page, "Disconnect");
      await expectOutput(page, "Status: closed · Build finished");
      return ["opens on demand", "messages update data", "close ends the stream"];
    case "use-websocket":
      assert.equal(await page.getByRole("button", { name: "Say hello" }).isDisabled(), true);
      await press(page, "Connect");
      await expectOutput(page, "Status: open");
      await press(page, "Say hello");
      await expectOutput(page, "Status: open · Echo: Hello!");
      await press(page, "Disconnect");
      await expectOutput(page, /^Status: closed/);
      return ["send needs an open socket", "messages update data", "close ends the socket"];
    case "use-broadcast-channel":
      await press(page, "Save in this tab");
      await expectOutput(page, "Other tab received: Draft saved at revision 3");
      return ["another channel receives the message"];
    case "use-worker-fn":
      await press(page, "Count primes up to 200,000");
      await expectOutput(page, "17984 primes found in a worker");
      return ["worker returns the result"];
    case "use-compression-stream":
      await press(page, "Compress and restore");
      await expectOutput(page, /^800 bytes → \d+ bytes gzip · restored exactly$/);
      return ["gzip round trip restores the text"];
    case "use-raf-fn":
      await press(page, "Start");
      await page.clock.runFor(1000);
      await expectOutput(page, /^(4\d|5\d)%$/);
      await press(page, "Pause");
      const paused = normalize(await page.locator("output").textContent());
      await page.clock.runFor(500);
      await expectOutput(page, paused);
      await press(page, "Start");
      await page.clock.runFor(2000);
      await expectOutput(page, "100%");
      return ["frames advance with time", "pause stops frames"];
    case "use-request-idle-callback":
      await press(page, "Record a click");
      await press(page, "Record a click");
      await press(page, "Record a click");
      await expectOutput(page, /^3 waiting · 0 sent · flush scheduled$/);
      await page.clock.runFor(2000);
      await expectOutput(page, "0 waiting · 3 sent");
      return ["events wait for idle time", "one flush sends them together"];
    case "use-scheduler-post-task":
      await press(page, "Queue three tasks");
      await page.waitForFunction(() => document.querySelectorAll("li").length === 3);
      assert.deepEqual(
        (
          await page
            .getByRole("list", { name: "Run order" })
            .getByRole("listitem")
            .allTextContents()
        ).map(normalize),
        ["respond to the click", "update the list", "prefetch next page"],
      );
      return ["higher priority runs first"];
    case "use-web-locks":
      await press(page, "Start export");
      await expectOutput(page, "Export running in this tab. Locks held: export");
      await press(page, "Try another export");
      await expectOutput(page, "Another export is already running. Locks held: export");
      await press(page, "Finish export");
      await expectOutput(page, "Export finished. Locks held: none");
      await press(page, "Try another export");
      await expectOutput(page, /^Started another export\./);
      return ["exclusive lock", "ifAvailable does not wait"];
    case "use-view-transition":
      await press(page, "Show as list");
      await expectOutput(page, "Layout: list");
      assert.equal(
        await page
          .getByRole("list", { name: "Talks" })
          .evaluate((element) => getComputedStyle(element).display),
        "block",
      );
      await press(page, "Show as grid");
      await expectOutput(page, "Layout: grid");
      return ["update applies with or without animation"];
    case "use-performance-observer": {
      await press(page, "Send a request");
      await press(page, "Send a request");
      await expectOutput(page, "2 requests recorded");
      const recorded = await page
        .getByRole("list", { name: "Recorded requests" })
        .getByRole("listitem")
        .allTextContents();
      assert.deepEqual(
        recorded.map((item) => item.split(" in ")[0]),
        ["?request=1", "?request=2"],
      );
      return ["observer receives resource entries"];
    }
    case "use-web-vitals": {
      const supported = await page.evaluate(() => typeof PerformanceObserver === "function");
      if (!supported) await expectOutput(page, "This browser does not report Web Vitals.");
      else await expectOutput(page, /^(Waiting for the first metric…|\d+ metrics reported)$/);
      return ["reports metrics or waits for them"];
    }
    case "use-storage-estimate":
      await expectOutput(page, /^Using [\d.]+ MB of [\d.]+ MB \([\d.]+%\)$/);
      await press(page, "Refresh estimate");
      await expectOutput(page, /^Using [\d.]+ MB of [\d.]+ MB/);
      return ["reads usage and quota"];
    case "use-visual-viewport": {
      const [width, height] = await page.evaluate(() => [
        Math.round(visualViewport?.width ?? 0),
        Math.round(visualViewport?.height ?? 0),
      ]);
      await expectOutput(page, `Visible area ${width} × ${height} at 1× zoom · keyboard closed`);
      return ["matches the visual viewport"];
    }
    case "use-user-activation":
      await expectOutput(page, "Waiting for the first interaction.");
      await page.getByRole("button", { name: "Interact with the page" }).click();
      await page.clock.runFor(1000);
      await expectOutput(page, "The page may now play sound.");
      return ["first interaction activates the page"];
    case "use-navigation-api":
      if (!(await page.evaluate(() => "navigation" in window))) {
        await expectOutput(page, "The Navigation API is not available.");
        return ["reports missing support"];
      }
      await press(page, "Go to step 2");
      await expectOutput(page, "Current entry: step-2");
      await press(page, "Go to step 3");
      await expectOutput(page, "Current entry: step-3");
      await press(page, "Back");
      await expectOutput(page, "Current entry: step-2");
      return ["navigate adds entries", "back traverses them"];
    case "use-drop-zone": {
      const zone = page.getByRole("region", { name: "Image drop area" });
      const transfer = await page.evaluateHandle(() => {
        const data = new DataTransfer();
        data.items.add(new File(["<svg/>"], "logo.svg", { type: "image/svg+xml" }));
        data.items.add(new File(["png"], "photo.png", { type: "image/png" }));
        return data;
      });
      await zone.dispatchEvent("dragenter", { dataTransfer: transfer });
      await zone.dispatchEvent("dragover", { dataTransfer: transfer });
      await page.getByText("Release to add the images").waitFor();
      await zone.dispatchEvent("drop", { dataTransfer: transfer });
      await expectOutput(page, "Added: logo.svg, photo.png");
      return ["highlights while dragging", "drop lists accepted files"];
    }
    case "use-file-dialog": {
      const chooser = page.waitForEvent("filechooser");
      await page.getByRole("button", { name: "Choose a CSV file" }).click();
      await (
        await chooser
      ).setFiles({
        name: "attendees.csv",
        mimeType: "text/csv",
        buffer: Buffer.from("name\nAda\n"),
      });
      await expectOutput(page, "Selected attendees.csv (9 bytes)");
      await press(page, "Clear");
      await expectOutput(page, "No file selected.");
      return ["opens the native picker", "reset clears the selection"];
    }
    case "use-media-controls":
      await expectOutput(page, "0.0 / 1.0 s");
      await press(page, "Jump to 0.5 s");
      await expectOutput(page, "0.5 / 1.0 s");
      await press(page, "Speed 1×");
      assert.equal(
        await page.locator("audio").evaluate((audio: HTMLAudioElement) => audio.playbackRate),
        2,
      );
      await press(page, "Mute");
      assert.equal(
        await page.locator("audio").evaluate((audio: HTMLAudioElement) => audio.muted),
        true,
      );
      return ["reads duration", "seek, speed and mute write the element"];
    default:
      throw new Error(`No browser page law for ${name}`);
  }
}
