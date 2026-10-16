/** Native editing, box geometry and width observation in the local draft editor. */
import assert from "node:assert/strict";
import type { Page } from "playwright";

async function activate(page: Page, label: string): Promise<void> {
  await page.getByRole("button", { name: label, exact: true }).focus();
  await page.keyboard.press("Enter");
}

export async function exerciseTextareaAutosizeExample(page: Page): Promise<string[]> {
  const field = page.getByRole("textbox", { name: "Release notes", exact: true });
  const measure = () =>
    field.evaluate((element: HTMLTextAreaElement) => ({
      height: element.getBoundingClientRect().height,
      width: element.getBoundingClientRect().width,
      clientHeight: element.clientHeight,
      scrollHeight: element.scrollHeight,
      scrollTop: element.scrollTop,
      maxHeight: Number.parseFloat(getComputedStyle(element).maxHeight),
      start: element.selectionStart,
      end: element.selectionEnd,
      value: element.value,
      focused: element === document.activeElement,
    }));
  const settle = async () => {
    const handle = await field.elementHandle();
    assert.ok(handle);
    try {
      await handle.waitForElementState("stable");
    } finally {
      await handle.dispose();
    }
  };
  const empty = await measure();
  assert.equal(await field.getAttribute("rows"), "2");
  await field.focus();
  await page.keyboard.insertText(
    "Compiler fixes preserve keyboard navigation, accessible form controls and reliable server rendering.\nVerify the release.",
  );
  await settle();
  const typed = await measure();
  assert.ok(typed.height > empty.height, "native multiline input grows the editor");
  assert.ok(typed.scrollHeight <= typed.clientHeight + 1, "padding and borders do not clip text");
  assert.equal(typed.focused, true);
  await page.keyboard.press("Home");
  await page.keyboard.press("Shift+ArrowRight");
  const selected = await measure();
  assert.ok(selected.end > selected.start);

  await activate(page, "Compact editor");
  await settle();
  const compact = await measure();
  assert.ok(compact.width < typed.width, "the real editor width changes");
  assert.ok(compact.height >= typed.height, "width observation remeasures wrapping");
  assert.equal(compact.value, typed.value);
  assert.deepEqual([compact.start, compact.end], [selected.start, selected.end]);
  assert.equal(
    await page.getByRole("button", { name: "Compact editor" }).getAttribute("aria-pressed"),
    "true",
  );
  await activate(page, "Compact editor");
  await settle();
  assert.equal((await measure()).height, typed.height, "widening restores the measured size");

  await activate(page, "Clear draft");
  await settle();
  assert.equal((await measure()).height, empty.height, "programmatic clear restores the row floor");
  assert.equal(await field.inputValue(), "");
  assert.equal(
    await page
      .getByRole("button", { name: "Clear draft" })
      .evaluate((button) => button === document.activeElement),
    true,
  );
  await activate(page, "Insert release template");
  await settle();
  const long = await measure();
  assert.equal(long.height, long.maxHeight, "ordinary CSS maximum caps the editor");
  assert.ok(long.scrollHeight > long.clientHeight, "long drafts remain scrollable");
  await field.focus();
  await page.keyboard.press("ControlOrMeta+End");
  const lastLine = await measure();
  assert.equal(lastLine.start, lastLine.value.length);
  assert.equal(lastLine.end, lastLine.value.length);
  assert.ok(lastLine.scrollTop + lastLine.clientHeight >= lastLine.scrollHeight - 1);
  return [
    "native multiline editing grows without clipped padding or lost focus",
    "real compact-width observation preserves the draft and native selection, then widening shrinks",
    "programmatic clear restores declared rows while its keyboard control retains focus",
    "CSS maximum height permits native keyboard navigation to the last line of the inserted draft",
  ];
}

export async function verifyTextareaAutosizeUnmount(page: Page): Promise<string[]> {
  await page.locator("[data-hydration-root]").evaluate((root) => {
    const app = (root as HTMLElement & { __vue_app__?: { unmount(): void } }).__vue_app__;
    if (!app) throw new Error("Expected the real hydrated Vue application");
    app.unmount();
  });
  const viewport = page.viewportSize();
  assert.ok(viewport);
  await page.setViewportSize({ width: viewport.width + 10, height: viewport.height });
  await page.clock.runFor(200);
  assert.equal(await page.locator("[data-hydration-root]").textContent(), "");
  await page.setViewportSize(viewport);
  return ["actual Vue unmount removes the editor before a further real viewport resize"];
}
