/** Assert observable effects of the complete source-owned composable examples. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import type { Browser, Page } from "playwright";
import { previewComposableExamples } from "./build-config.ts";

async function activate(page: Page, label: string): Promise<void> {
  await page.getByRole("button", { name: label, exact: true }).focus();
  await page.keyboard.press("Enter");
}

async function exercise(page: Page, name: string): Promise<string[]> {
  if (name === "use-toggle") {
    assert.equal(await page.getByText("Free delivery in 2–3 business days.").count(), 0);
    await activate(page, "Show delivery details");
    assert.equal(await page.locator("output").textContent(), "Details are visible.");
    assert.equal(
      await page
        .getByRole("button", { name: "Hide delivery details" })
        .getAttribute("aria-expanded"),
      "true",
    );
    await page.keyboard.press("Space");
    assert.equal(await page.getByText("Free delivery in 2–3 business days.").count(), 0);
    await activate(page, "Show delivery details");
    return [
      "Enter opens disclosure",
      "Space closes disclosure",
      "visible state and aria-expanded agree",
    ];
  }
  if (name === "use-counter") {
    assert.equal(await page.getByRole("button", { name: "Remove seat" }).isDisabled(), true);
    for (let seat = 2; seat <= 5; seat += 1) await activate(page, "Add seat");
    assert.equal(await page.locator("output").textContent(), "5 seats");
    assert.equal(await page.getByRole("button", { name: "Add seat" }).isDisabled(), true);
    await page.getByText("You reached the five-seat limit.").waitFor();
    await activate(page, "Reset seats");
    assert.equal(await page.locator("output").textContent(), "1 seat");
    await activate(page, "Add seat");
    return [
      "lower and upper bounds disable controls",
      "keyboard changes quantity",
      "reset restores initial reservation",
    ];
  }
  if (name === "use-debounced") {
    const query = page.getByRole("searchbox", { name: "Find a guide" });
    const results = page.getByRole("list", { name: "Matching guides" }).getByRole("listitem");
    await query.fill("component");
    assert.equal(await results.count(), 4, "pending search keeps settled results");
    assert.equal(await page.locator("output").textContent(), "Waiting for typing to stop…");
    await page.clock.runFor(500);
    assert.deepEqual(await results.allTextContents(), ["Component styling"]);
    await query.fill("keyboard");
    await activate(page, "Search now");
    assert.deepEqual(await results.allTextContents(), ["Keyboard accessibility"]);
    await query.fill("missing");
    await activate(page, "Cancel pending search");
    await page.clock.runFor(500);
    assert.deepEqual(await results.allTextContents(), ["Keyboard accessibility"]);
    await query.fill("no matching guide");
    await page.clock.runFor(500);
    assert.equal(await results.count(), 0);
    await page.getByText("No guides match this search.").waitFor();
    return [
      "pending state retains settled results",
      "500ms trailing update",
      "keyboard flush and cancel",
      "empty result state",
    ];
  }
  if (name === "use-field") {
    const field = page.getByRole("textbox", { name: "Display name" });
    await field.fill("Al");
    assert.equal(await page.locator('p[role="status"]').textContent(), "Enter a display name.");
    await page.keyboard.press("Tab");
    await page.getByText("Use at least three characters.", { exact: true }).waitFor();
    assert.equal(await field.getAttribute("aria-invalid"), "true");
    assert.equal(await page.locator("output").textContent(), "Changed: yes · Visited: yes");
    await field.fill("Ada");
    await page.keyboard.press("Tab");
    await page.getByText("This name is ready to use.").waitFor();
    assert.equal(await field.getAttribute("aria-invalid"), "false");
    await activate(page, "Reset name");
    assert.equal(await field.inputValue(), "");
    assert.equal(await page.locator("output").textContent(), "Changed: no · Visited: no");
    await field.fill("Al");
    await page.keyboard.press("Tab");
    await page.getByText("Use at least three characters.", { exact: true }).waitFor();
    return [
      "validation waits for keyboard blur",
      "error and aria-invalid",
      "valid state",
      "reset clears interaction state",
    ];
  }
  if (name === "use-history") {
    const title = page.getByRole("textbox", { name: "Release title" });
    assert.equal(await page.getByRole("button", { name: "Undo", exact: true }).isDisabled(), true);
    await title.fill("A new release");
    await activate(page, "Apply publication title");
    assert.equal(await page.locator("output").textContent(), "2 undo steps · 0 redo steps");
    await activate(page, "Undo");
    assert.equal(await title.inputValue(), "A new release");
    await activate(page, "Redo");
    assert.equal(await title.inputValue(), "Vize release notes");
    await activate(page, "Undo");
    await title.fill("Another draft");
    assert.equal(await page.getByRole("button", { name: "Redo", exact: true }).isDisabled(), true);
    await activate(page, "Clear history");
    assert.equal(await title.inputValue(), "Another draft");
    assert.equal(await page.locator("output").textContent(), "0 undo steps · 0 redo steps");
    await activate(page, "Apply publication title");
    return [
      "keyboard undo and redo restore exact values",
      "batch creates one undo step",
      "new edit drops redo",
      "clear preserves live value",
    ];
  }
  assert.equal(name, "use-offset-pagination", "every registered example has an interaction law");
  const items = page.getByRole("list", { name: "Guides on this page" }).getByRole("listitem");
  assert.equal(await page.getByRole("button", { name: "Previous page" }).isDisabled(), true);
  assert.deepEqual(await items.allTextContents(), ["Setup", "Components", "Composables"]);
  await activate(page, "Next page");
  await activate(page, "Next page");
  assert.deepEqual(await items.allTextContents(), ["Deployment"]);
  assert.equal(await page.getByRole("button", { name: "Next page" }).isDisabled(), true);
  await page.getByRole("combobox", { name: "Guides per page" }).focus();
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Enter");
  assert.equal(await page.locator("output").textContent(), "Page 2 of 2");
  assert.deepEqual(await items.allTextContents(), ["Testing", "Deployment"]);
  await activate(page, "Previous page");
  assert.deepEqual(await items.allTextContents(), [
    "Setup",
    "Components",
    "Composables",
    "Styling",
    "Accessibility",
  ]);
  return [
    "keyboard pagination",
    "first and last boundaries",
    "page-size change clamps page and slices actual data",
  ];
}

/** Runs unchanged against the built application or an explicitly supplied deployed URL. */
export async function captureComposablePreviews(
  browser: Browser,
  baseUrl: string,
  outputRoot: string,
): Promise<void> {
  mkdirSync(outputRoot, { recursive: true });
  const receipts = [];
  for (const width of [720, 360]) {
    const page = await browser.newPage({
      viewport: { width, height: 540 },
      reducedMotion: "reduce",
    });
    const diagnostics: string[] = [];
    page.on("pageerror", (error) => diagnostics.push(error.message));
    page.on("console", (message) => {
      if (message.type() === "error" || message.type() === "warning")
        diagnostics.push(message.text());
    });
    try {
      await page.clock.install({ time: 0 });
      await page.clock.pauseAt(1000);
      for (const example of previewComposableExamples) {
        const response = await page.goto(`${baseUrl}index.html?composable=${example.name}`, {
          waitUntil: "networkidle",
        });
        assert.equal(response?.status(), 200, example.name);
        await page.waitForSelector(`html[data-preview-ready="${example.name}"]`);
        assert.equal(
          await page.locator("html").getAttribute("data-preview-source-sha256"),
          example.sourceSha256,
        );
        assert.equal(
          await page.locator("html").getAttribute("data-preview-hydration-retained"),
          "true",
          `${example.name}: hydration preserves complete SSR markup and element identities`,
        );
        const screenshots = [];
        for (const state of ["initial", "interacted"] as const) {
          const interactions = state === "interacted" ? await exercise(page, example.name) : [];
          assert.equal(
            await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
            true,
            `${example.name}: ${width}px overflow`,
          );
          assert.deepEqual(diagnostics, [], `${example.name}: browser diagnostics`);
          const file = `${example.name}-${width}-${state}.png`;
          await page.screenshot({ path: path.join(outputRoot, file), fullPage: true });
          screenshots.push({
            state,
            file,
            sha256: createHash("sha256")
              .update(readFileSync(path.join(outputRoot, file)))
              .digest("hex"),
            interactions,
          });
        }
        receipts.push({
          name: example.name,
          sourceSha256: example.sourceSha256,
          width,
          screenshots,
          browserDiagnostics: [...diagnostics],
        });
      }
    } finally {
      await page.close();
    }
  }
  writeFileSync(
    path.join(outputRoot, "browser-evidence.json"),
    `${JSON.stringify({ schema: "vize.composable-docs-browser-evidence", browserVersion: browser.version(), baseUrl, examples: receipts }, null, 2)}\n`,
  );
  process.stdout.write(
    `Verified six composable examples in initial/interacted desktop/mobile states\n`,
  );
}
