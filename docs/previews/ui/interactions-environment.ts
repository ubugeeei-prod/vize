/** Check actual viewport geometry and native media queries without replacing browser hosts. */
import assert from "node:assert/strict";
import type { Page } from "playwright";

async function outputContains(page: Page, text: string): Promise<void> {
  await page.locator("output").filter({ hasText: text }).waitFor();
}

async function chooseSecond(page: Page, label: string): Promise<void> {
  await page.getByRole("combobox", { name: label, exact: true }).focus();
  await page.keyboard.press("s");
  await page.keyboard.press("Tab");
}

export async function exerciseEnvironmentExample(page: Page, name: string): Promise<string[]> {
  const originalViewport = page.viewportSize();
  assert.ok(originalViewport, "the viewport examples run in a real bounded browser viewport");
  try {
    if (name === "breakpoints") {
      const draft = page.getByRole("textbox", { name: "Review draft", exact: true });
      await draft.fill("  Keep this review across widths  ");
      const readings = page.locator(".breakpoint-readings");
      const verify = async (width: number): Promise<void> => {
        await page.setViewportSize({ width, height: originalViewport.height });
        const actual = await page.evaluate(() => ({
          width: innerWidth,
          compact: matchMedia("(min-width: 480px)").matches,
          expanded: matchMedia("(min-width: 768px)").matches,
          above: matchMedia("(min-width: 768.02px)").matches,
          below: matchMedia("(max-width: 767.98px)").matches,
          atMost: matchMedia("(max-width: 768px)").matches,
        }));
        assert.equal(actual.width, width, "the real viewport changed rather than an injected host");
        assert.equal(actual.compact, width >= 480);
        assert.equal(actual.expanded, width >= 768);
        assert.equal(actual.above, width > 768);
        assert.equal(actual.below, width < 768);
        assert.equal(actual.atMost, width <= 768);
        const yesNo = (value: boolean): string => (value ? "yes" : "no");
        for (const [label, expected] of [
          ["At least 480 px", actual.compact],
          ["At least 768 px", actual.expanded],
          ["Above 768 px", actual.above],
          ["Below 768 px", actual.below],
          ["At most 768 px", actual.atMost],
          ["480 to below 768 px", actual.compact && actual.below],
        ] as const) {
          await readings
            .locator("div")
            .filter({ has: page.getByText(label, { exact: true }) })
            .locator("dd")
            .filter({ hasText: new RegExp(`^${yesNo(expected)}$`) })
            .waitFor();
        }
        const matched =
          [actual.compact ? "compact" : "", actual.expanded ? "expanded" : ""]
            .filter(Boolean)
            .join(", ") || "none";
        const active = actual.expanded ? "expanded" : actual.compact ? "compact" : "none";
        await outputContains(page, `Matched: ${matched} · Active: ${active}`);
        assert.equal(await draft.inputValue(), "  Keep this review across widths  ");
        assert.equal(
          await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
          true,
        );
      };
      for (const width of [479, 480, 481, 767, 768, 769]) await verify(width);
      await outputContains(page, "Detail: full");
      await chooseSecond(page, "Guide detail");
      await outputContains(page, "Detail: summary");
      await verify(800);
      await outputContains(page, "Detail: summary");
      const preference = page.getByRole("combobox", { name: "Guide detail", exact: true });
      await preference.focus();
      await page.keyboard.press("f");
      await page.keyboard.press("Tab");
      await verify(360);
      await outputContains(page, "Detail: full");
      assert.equal(
        await page
          .getByText("Compare the source locations and generated render calls before publishing.", {
            exact: true,
          })
          .count(),
        1,
      );
      return [
        "native matchMedia results agree with real 479/480/481 and 767/768/769 pixel viewports",
        "inclusive and strict comparisons disagree correctly at the exact 768px boundary",
        "the middle range includes 480 and excludes 768 without overlapping the expanded tier",
        "current lists ascending matched names and active selects the largest actual breakpoint",
        "native Summary and Full detail choices persist independently of viewport query updates",
        "exact draft whitespace survives resizing and all verified widths remain free of overflow",
      ];
    }

    assert.equal(name, "window-size", "every environment example has a real interaction law");
    const draft = page.getByRole("textbox", { name: "Viewport review draft", exact: true });
    await draft.fill("  Preserve the viewport review  ");
    const measurements = page.getByRole("checkbox", {
      name: "Show layout measurements",
      exact: true,
    });
    await measurements.focus();
    await page.keyboard.press("Space");
    const verify = async (width: number, height: number, columns: number): Promise<void> => {
      await page.setViewportSize({ width, height });
      const actual = await page.evaluate(() => ({ width: innerWidth, height: innerHeight }));
      assert.deepEqual(actual, { width, height });
      await outputContains(
        page,
        `Viewport: ${actual.width} × ${actual.height} CSS pixels · Window support: available`,
      );
      await page
        .getByText(`Layout: ${columns} ${columns === 1 ? "column" : "columns"} · Choice:`, {
          exact: false,
        })
        .waitFor();
      assert.equal(await measurements.isChecked(), true);
      assert.equal(await draft.inputValue(), "  Preserve the viewport review  ");
      const cardCount = await page
        .locator(".viewport-guides")
        .evaluate((element) => getComputedStyle(element).gridTemplateColumns.split(" ").length);
      assert.equal(cardCount, columns, "native grid geometry matches the chosen layout");
      assert.equal(
        await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
        true,
      );
    };
    await verify(639, 420, 1);
    await verify(640, 600, 2);
    await verify(641, 540, 2);
    await chooseSecond(page, "Guide layout");
    await verify(800, 600, 1);
    await verify(360, 420, 1);
    return [
      "readonly dimensions and support agree with the actual iframe window at every resized viewport",
      "automatic layout uses one column at 639px and two at the inclusive 640px boundary",
      "native rendered grid columns agree with the measured width and user layout choice",
      "the native Single column choice persists at both wide and narrow viewports",
      "exact draft whitespace and measurement visibility survive independent width and height changes",
    ];
  } finally {
    await page.setViewportSize(originalViewport);
    const actual = await page.evaluate(() => ({ width: innerWidth, height: innerHeight }));
    if (name === "window-size")
      await outputContains(
        page,
        `Viewport: ${actual.width} × ${actual.height} CSS pixels · Window support: available`,
      );
    else
      await outputContains(
        page,
        `Active: ${actual.width >= 768 ? "expanded" : actual.width >= 480 ? "compact" : "none"}`,
      );
  }
}
