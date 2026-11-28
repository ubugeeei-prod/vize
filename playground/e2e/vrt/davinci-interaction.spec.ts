import { test, expect } from "@playwright/test";
import type { Page } from "@playwright/test";

async function paneWidths(page: Page) {
  return page
    .locator(".dv-workspace [data-pane]")
    .evaluateAll((panes) => panes.map((pane) => pane.getBoundingClientRect().width));
}

test.beforeEach(async ({ page }) => {
  await page.goto("/?tab=davinci");
  await expect(page.locator('[data-layer="l3"] .davinci-folio')).toBeVisible({ timeout: 30_000 });
});

test("all IR layers share a source trace in DSL and raw notation", async ({ page }) => {
  await expect(page.locator(".dv-layer")).toHaveCount(4);
  const source = page.locator(".dv-trace-source");
  await expect(source).toContainText('v-model="draft"');
  for (const layer of ["l1", "l2", "l3"]) {
    await expect(
      page.locator(`[data-layer="${layer}"] .davinci-line.linked`).first(),
    ).toBeVisible();
  }
  await expect(page.locator('[data-layer="l2"] .davinci-folio')).toContainText("module @disegno");
  await expect(page.locator(".dv-dump-notice")).toContainText("not a stable interface");
  await expect(page.locator(".dv-trace-path")).toHaveCount(3);
  await page.getByRole("button", { name: "Raw", exact: true }).click();
  await expect(page.locator('[data-layer="l2"] .davinci-folio')).toContainText("[l2-dump-v2]");
  await expect(source).toContainText('v-model="draft"');
  await page.getByRole("button", { name: "IR DSL", exact: true }).click();
  const interpolation = page
    .locator('[data-layer="l2"] .davinci-line')
    .filter({ hasText: 'js("todo.text"' });
  await interpolation.click();
  await page.getByRole("heading", { name: "Davinci IR Observatory" }).hover();
  await expect(source).toContainText("{{ todo.text }}");
  await expect(
    page
      .locator('[data-layer="l3"] .davinci-line.linked')
      .filter({ hasText: "l3.set-text" })
      .first(),
  ).toBeVisible();
  await page.getByLabel("L3 IR dump").selectOption("l3-values/values");
  await expect(source).toContainText("{{ todo.text }}");
  await expect(page.locator('[data-layer="l3"] .davinci-line.linked').first()).toContainText(
    "todo.text",
  );
  await page.getByRole("button", { name: "Expand L2 IR" }).click();
  await expect(page.getByRole("navigation", { name: "Compiler stages" })).toBeVisible();
  await expect(source).toContainText("{{ todo.text }}");
  await page.getByRole("button", { name: "All layers", exact: true }).click();
  await expect(page.locator(".dv-layer")).toHaveCount(4);
});

test("source cursor maps Unicode correctly and edits invalidate the old trace", async ({
  page,
}) => {
  const code =
    '<script setup>const message = "日本語🌸"</script>\n<template><p>日本語🌸</p><p>{{ message }}</p></template>';
  await page.locator(".davinci-source .monaco-container").evaluate((element, value) => {
    const editor = (element as HTMLElement & { __monacoEditor: { setValue(value: string): void } })
      .__monacoEditor;
    editor.setValue(value);
  }, code);
  await expect(page.locator('[data-layer="l2"] .davinci-folio')).toContainText('js("message"');
  await page.locator(".davinci-source .monaco-container").evaluate(
    (element, offset) => {
      const editor = (
        element as HTMLElement & {
          __monacoEditor: {
            getModel(): { getPositionAt(offset: number): { lineNumber: number; column: number } };
            setPosition(position: { lineNumber: number; column: number }): void;
          };
        }
      ).__monacoEditor;
      editor.setPosition(editor.getModel().getPositionAt(offset));
    },
    code.indexOf("{{ message }}") + 4,
  );
  await expect(page.locator(".dv-trace-source")).toHaveText("{{ message }}");
  await expect(
    page
      .locator('[data-layer="l3"] .davinci-line.linked')
      .filter({ hasText: "l3.set-text" })
      .first(),
  ).toBeVisible();
  await page.getByRole("button", { name: "Clear trace" }).click();
  await expect(page.locator(".dv-trace-source")).toHaveCount(0);
  await expect(page.locator(".davinci-line.linked")).toHaveCount(0);
});

test("target-specific missing layers are explicit, and DOM pass tools remain available", async ({
  page,
}) => {
  await page.getByRole("button", { name: "DOM", exact: true }).click();
  await expect(page.locator('[data-layer="l3"]')).toContainText("No L3 capture");
  await page.getByLabel("L2 IR dump").selectOption("l2/hoist-static");
  await page.getByRole("button", { name: "Expand L2 IR" }).click();
  await page.getByRole("button", { name: /^Diff vs/ }).click();
  await expect(page.locator(".davinci-diff")).toBeVisible();
  await page.getByRole("button", { name: /^Remarks/ }).click();
  await expect(page.locator(".davinci-remarks")).toBeVisible();
  await page.getByRole("button", { name: "All layers", exact: true }).click();
  await page.getByRole("button", { name: "Inspect the SSR pipeline" }).click();
  await expect(page.locator('[data-layer="l3"] .davinci-folio')).toBeVisible();
  const bounds = await page.locator(".dv-layer").evaluateAll((elements) =>
    elements.map((element) => {
      const rect = element.getBoundingClientRect();
      return { left: rect.left, right: rect.right, top: rect.top, bottom: rect.bottom };
    }),
  );
  expect(bounds.every((rect) => rect.left >= 0 && rect.right <= 1280 && rect.bottom <= 900)).toBe(
    true,
  );
  expect(new Set(bounds.map((rect) => rect.top)).size).toBe(1);
});

test("pane dividers resize every layer, keep traces aligned, and restore widths", async ({
  page,
}) => {
  const original = await paneWidths(page);
  const divider = page.getByRole("separator", { name: "Resize Surface panes" });
  const bounds = (await divider.boundingBox())!;
  await page.mouse.move(bounds.x + bounds.width / 2, bounds.y + 60);
  await page.mouse.down();
  await page.mouse.move(bounds.x + bounds.width / 2 + 240, bounds.y + 60, { steps: 12 });
  await page.mouse.up();
  await expect.poll(async () => (await paneWidths(page))[0]).toBeCloseTo(original[0] + 240, 0);
  const enlarged = await paneWidths(page);
  expect(enlarged.every((width, index) => width >= (index === 0 ? 180 : 160) - 1)).toBe(true);
  expect(enlarged.reduce((sum, width) => sum + width, 0)).toBeCloseTo(
    original.reduce((sum, width) => sum + width, 0),
    0,
  );
  await expect
    .poll(async () =>
      page
        .locator(".dv-link-l1 circle")
        .first()
        .evaluate((circle) => {
          const source = document.querySelector('[data-pane="source"]')!.getBoundingClientRect();
          const workspace = document.querySelector(".dv-workspace")!.getBoundingClientRect();
          return Math.abs(Number(circle.getAttribute("cx")) - (source.right - workspace.left - 16));
        }),
    )
    .toBeLessThan(1);
  await divider.dblclick({ position: { x: 5, y: 60 } });
  await expect.poll(async () => (await paneWidths(page))[0]).toBeCloseTo(original[0], 0);

  for (const [index, name] of ["Disegno", "Impeto", "Emit"].entries()) {
    const separator = page.getByRole("separator", { name: `Resize ${name} panes` });
    const before = await paneWidths(page);
    await separator.focus();
    await separator.press("Shift+ArrowRight");
    await expect
      .poll(async () => (await paneWidths(page))[index + 1])
      .toBeCloseTo(before[index + 1] + 50, 0);
    await separator.press("Enter");
  }
  await divider.press("Home");
  await expect.poll(async () => (await paneWidths(page))[0]).toBeCloseTo(180, 0);
  await divider.press("End");
  await expect.poll(async () => (await paneWidths(page)).slice(1)).toEqual([160, 160, 160, 160]);
  await divider.press("Enter");

  await page.getByRole("button", { name: "Expand L2 IR" }).click();
  const focusDivider = page.getByRole("separator", { name: "Resize Source and stage panes" });
  const beforeFocus = await paneWidths(page);
  await focusDivider.press("Shift+ArrowRight");
  await expect.poll(async () => (await paneWidths(page))[0]).toBeCloseTo(beforeFocus[0] + 50, 0);
  await page.getByRole("button", { name: "All layers", exact: true }).click();
  await expect.poll(async () => (await paneWidths(page))[0]).toBeCloseTo(original[0], 0);
  await page.setViewportSize({ width: 1600, height: 1000 });
  await expect
    .poll(async () => (await paneWidths(page)).reduce((sum, width) => sum + width, 0))
    .toBeGreaterThan(1500);
});

test("IR dumps scroll horizontally without wrapping by default", async ({ page }) => {
  await expect(page.getByRole("button", { name: "Wrap", exact: true })).toHaveAttribute(
    "aria-pressed",
    "false",
  );
  const dump = page.locator('[data-layer="l2"] .davinci-folio');
  await expect(dump.locator(".davinci-code").first()).toHaveCSS("white-space", "pre");
  const overflow = await dump.evaluate((element) => {
    element.scrollLeft = 120;
    return { scroll: element.scrollLeft, width: element.clientWidth, content: element.scrollWidth };
  });
  expect(overflow.content).toBeGreaterThan(overflow.width);
  expect(overflow.scroll).toBeGreaterThan(0);
  await page.getByRole("button", { name: "Wrap", exact: true }).click();
  await expect(dump.locator(".davinci-code").first()).toHaveCSS("white-space", "pre-wrap");
});

test("L1 hovers individual analysis units on a single line and preserves a pinned unit", async ({
  page,
}) => {
  const code =
    '<script setup>const title = "日本語🌸"; const first = 1; const last = 2;</script>\n<template><p id="a" :title="title">{{ first }}<b>{{ last }}</b></p></template>';
  await page.locator(".davinci-source .monaco-container").evaluate((element, value) => {
    (
      element as HTMLElement & { __monacoEditor: { setValue(value: string): void } }
    ).__monacoEditor.setValue(value);
  }, code);
  const surface = page.locator('[data-layer="l1"] .davinci-folio');
  const attribute = surface.getByRole("button", { name: ':title="title"', exact: true });
  await expect(attribute).toBeVisible();
  await page.getByRole("button", { name: "Clear trace" }).click();
  await surface.locator(".davinci-ln").hover();
  await expect(page.locator(".dv-trace-source")).toHaveCount(0);
  await attribute.hover();
  await expect(page.locator(".dv-trace-source")).toHaveText(':title="title"');
  await expect(page.locator('[data-layer="l2"] .davinci-line.linked').first()).toContainText(
    "ui.bind",
  );
  const last = surface.getByRole("button", { name: "{{ last }}", exact: true });
  await last.hover();
  await expect(page.locator(".dv-trace-source")).toHaveText("{{ last }}");
  await last.click();
  await page.getByRole("heading", { name: "Davinci IR Observatory" }).hover();
  await expect(page.locator(".dv-trace-source")).toHaveText("{{ last }}");
  await page.getByRole("button", { name: "Expand L1 IR" }).click();
  await expect(page.locator(".dv-surface-unit.linked")).toHaveText("{{ last }}");
});

test("manual L3 scrolling is not undone by hover previews", async ({ page }) => {
  const dump = page.locator('[data-layer="l3"] .davinci-folio');
  const target = await dump.evaluate((element) => {
    element.scrollTop = element.scrollHeight - element.clientHeight;
    return element.scrollTop;
  });
  expect(target).toBeGreaterThan(0);
  const position = await dump.evaluate((element) => {
    const bounds = element.getBoundingClientRect();
    const rows = [...element.querySelectorAll(".davinci-line.linkable")];
    const row = rows
      .map((row) => row.getBoundingClientRect())
      .find((row) => row.top > bounds.top + 20 && row.bottom < bounds.bottom - 20)!;
    return { x: bounds.left + 100, y: row.top + row.height / 2 };
  });
  await page.mouse.move(position.x, position.y);
  // Sample several frames: a hover/scroll feedback loop can oscillate back to its starting point.
  const positions = await dump.evaluate(async (element) => {
    const readings: number[] = [];
    for (let i = 0; i < 12; i++) {
      await new Promise(requestAnimationFrame);
      readings.push(element.scrollTop);
    }
    return readings;
  });
  expect(positions.every((scroll) => Math.abs(scroll - target) < 1)).toBe(true);
  await expect(page.getByLabel("Example").locator("option")).toHaveCount(12);
  const workspace = (await page.locator(".dv-workspace").boundingBox())!;
  const inspector = (await page.locator(".dv-trace-inspector").boundingBox())!;
  expect(inspector.y + inspector.height).toBeLessThanOrEqual(workspace.y);
  expect(workspace.height).toBeGreaterThan(600);
});
