/** Native input and geometry laws for four existing public composables. */
import assert from "node:assert/strict";
import type { Locator, Page } from "playwright";

async function activate(page: Page, name: string): Promise<void> {
  await page.getByRole("button", { name, exact: true }).focus();
  await page.keyboard.press("Enter");
}

async function toggle(page: Page, name: string): Promise<void> {
  await page.getByRole("checkbox", { name, exact: true }).focus();
  await page.keyboard.press("Space");
}

async function rectangle(locator: Locator) {
  const box = await locator.boundingBox();
  assert.ok(box, "native target must have real visible geometry");
  return { ...box, right: box.x + box.width, bottom: box.y + box.height };
}

async function output(page: Page, expected: string | RegExp): Promise<void> {
  await page.locator("output").filter({ hasText: expected }).waitFor();
}

export async function exercisePointerSelectionExample(page: Page, name: string): Promise<string[]> {
  if (name === "pointer") {
    const pad = page.getByRole("region", { name: "Pointer guide pad" });
    const box = await rectangle(pad);
    const first = { x: box.x + 24, y: box.y + 32 };
    await page.mouse.move(first.x, first.y);
    await output(
      page,
      `Latest pointer: mouse · Inside: yes · Client: ${Math.round(first.x)}, ${Math.round(first.y)}`,
    );
    assert.equal(await page.locator(".pointer-pressure").textContent(), "0");
    await page.mouse.down();
    assert.ok(
      Number(await page.locator(".pointer-pressure").textContent()) > 0,
      "actual pressed mouse PointerEvent has pressure",
    );
    await page.mouse.up();
    await page.locator(".pointer-pressure").filter({ hasText: /^0$/ }).waitFor();
    await page.mouse.move(box.x + 24, box.bottom + 8);
    await output(page, "Inside: no");
    const retained = await page.locator("output").textContent();
    await toggle(page, "Show pointer guide pad");
    assert.equal(await pad.count(), 0);
    await page.mouse.move(8, 8);
    assert.equal(
      await page.locator("output").textContent(),
      retained,
      "removed target cannot receive later native pointer input",
    );
    await toggle(page, "Show pointer guide pad");
    const replacement = await rectangle(pad);
    const next = { x: replacement.x + 42, y: replacement.y + 45 };
    await page.mouse.move(next.x, next.y);
    await output(
      page,
      `Latest pointer: mouse · Inside: yes · Client: ${Math.round(next.x)}, ${Math.round(next.y)}`,
    );
    return [
      "real mouse PointerEvent coordinates agree with the actual iframe client position",
      "native pointerdown pressure becomes positive and pointerup restores zero",
      "pointerleave changes inside while retaining the last snapshot",
      "keyboard hiding removes the actual observed target",
      "later native input cannot update the removed target snapshot",
      "a replacement target resumes native observation without replacing the provider",
    ];
  }

  if (name === "mouse-in-element") {
    const pad = page.getByRole("region", { name: "Coordinate pad" });
    const first = await rectangle(pad);
    const point = { x: Math.round(first.x + 30), y: Math.round(first.y + 35) };
    await page.mouse.move(point.x, point.y);
    await page
      .locator(".mouse-client")
      .filter({ hasText: `Client: ${point.x}, ${point.y} CSS pixels` })
      .waitFor();
    const relative = { x: Math.round(point.x - first.x), y: Math.round(point.y - first.y) };
    await output(page, `Relative: ${relative.x}, ${relative.y} · Outside: no · Observing`);
    await page
      .locator(".mouse-size")
      .filter({
        hasText: `Recorded pad: ${Math.round(first.width)} × ${Math.round(first.height)} CSS pixels`,
      })
      .waitFor();
    await toggle(page, "Shift coordinate pad");
    const shifted = await rectangle(pad);
    assert.ok(
      Math.abs(shifted.x - first.x - 24) < 1,
      "actual transform shifts the native element rectangle",
    );
    const moved = { x: point.x + 1, y: point.y + 1 };
    await page.mouse.move(moved.x, moved.y);
    await page
      .locator(".mouse-client")
      .filter({ hasText: `Client: ${moved.x}, ${moved.y} CSS pixels` })
      .waitFor();
    const inside = { x: Math.round(moved.x - shifted.x), y: Math.round(moved.y - shifted.y) };
    await output(page, `Relative: ${inside.x}, ${inside.y} · Outside: no · Observing`);
    await page.mouse.move(Math.round(shifted.right) + 8, moved.y);
    await output(page, `Relative: ${inside.x}, ${inside.y} · Outside: yes · Observing`);
    await activate(page, "Stop relative readings");
    await output(page, `Relative: ${inside.x}, ${inside.y} · Outside: yes · Stopped`);
    const frozen = await page.locator("output").textContent();
    const frozenSize = await page.locator(".mouse-size").textContent();
    const next = await rectangle(pad);
    const live = { x: Math.round(next.x + 55), y: Math.round(next.y + 60) };
    await page.mouse.move(live.x, live.y);
    await page
      .locator(".mouse-client")
      .filter({
        hasText: `Client: ${live.x}, ${live.y} CSS pixels`,
      })
      .waitFor();
    assert.equal(await page.locator("output").textContent(), frozen);
    assert.equal(await page.locator(".mouse-size").textContent(), frozenSize);
    assert.equal(
      await page.getByRole("button", { name: "Stop relative readings" }).isDisabled(),
      true,
    );
    return [
      "native client coordinates map to the measured element-relative location",
      "recorded width and height match the actual native border rectangle",
      "a later native move re-reads transformed geometry rather than an injected position",
      "handleOutside false retains the last inside coordinates while outside changes",
      "stop freezes relative state and geometry and disables its idempotent control",
      "the real underlying client mouse subscription remains live after relative observation stops",
    ];
  }

  if (name === "draggable") {
    const board = page.getByRole("region", { name: "Bounded note board" });
    const note = page.getByRole("article", { name: "Draggable review note" });
    const draft = page.getByRole("textbox", { name: "Note text", exact: true });
    await draft.fill("  Preserve this local review  ");
    await activate(page, "Reset note position");
    const drag = async (x: number, y: number) => {
      const handle = await rectangle(page.getByRole("button", { name: "Drag review note" }));
      await page.mouse.move(handle.x + handle.width / 2, handle.y + handle.height / 2);
      await page.mouse.down();
      await output(page, "Dragging: yes");
      await page.mouse.move(x, y, { steps: 4 });
      await page.mouse.up();
      await output(page, "Dragging: no");
    };
    const bounds = await rectangle(board);
    const viewport = page.viewportSize();
    assert.ok(
      viewport && bounds.bottom - 3 < viewport.height,
      "actual board edge is reachable by native pointer input",
    );
    await drag(bounds.right - 3, bounds.bottom - 3);
    const bounded = await rectangle(note);
    assert.ok(
      Math.abs(bounded.right - bounds.right) < 1 && Math.abs(bounded.bottom - bounds.bottom) < 1,
      "native dragging subtracts the whole note size at both bounds",
    );
    await output(page, "Starts: 1 · Ends: 1");
    const axis = page.getByRole("combobox", { name: "Drag axis", exact: true });
    await axis.focus();
    await page.keyboard.press("x");
    await page.keyboard.press("Tab");
    const current = await rectangle(board);
    await drag(current.x + 4, current.y + 20);
    const horizontal = await rectangle(note);
    assert.ok(
      Math.abs(horizontal.y - bounded.y) < 1 && horizontal.x < bounded.x,
      "reactive x axis keeps the existing y during native moves",
    );
    await output(page, "Starts: 2 · Ends: 2");
    await toggle(page, "Lock pointer dragging");
    const locked = await rectangle(note);
    const handle = await rectangle(page.getByRole("button", { name: "Drag review note" }));
    await page.mouse.move(handle.x + 20, handle.y + 12);
    await page.mouse.down();
    await page.mouse.move(handle.x + 55, handle.y + 30);
    await page.mouse.up();
    assert.deepEqual(
      await rectangle(note),
      locked,
      "disabled pointer gesture cannot move the note",
    );
    await output(page, "Starts: 2 · Ends: 2");
    await activate(page, "Move note right");
    assert.ok(
      (await rectangle(note)).x > locked.x,
      "keyboard placement writes independently of pointer disabled state",
    );
    await activate(page, "Stop drag listeners");
    const stopped = await rectangle(note);
    await toggle(page, "Lock pointer dragging");
    const stoppedHandle = await rectangle(page.getByRole("button", { name: "Drag review note" }));
    await page.mouse.move(stoppedHandle.x + 20, stoppedHandle.y + 12);
    await page.mouse.down();
    await page.mouse.move(stoppedHandle.x + 60, stoppedHandle.y + 35);
    await page.mouse.up();
    assert.deepEqual(
      await rectangle(note),
      stopped,
      "removed drag listeners cannot process later native gestures",
    );
    await activate(page, "Reset note position");
    const reset = await rectangle(note);
    const resetBounds = await rectangle(board);
    assert.ok(
      Math.abs(reset.x - resetBounds.x - 12) < 1 && Math.abs(reset.y - resetBounds.y - 12) < 1,
    );
    assert.equal(await draft.inputValue(), "  Preserve this local review  ");
    assert.equal(
      await note
        .locator("span")
        .evaluate((element) => element.scrollHeight <= element.clientHeight),
      true,
      "the native note shows the complete bounded review text without vertical clipping",
    );
    assert.equal(
      await page.getByRole("button", { name: "Stop drag listeners" }).isDisabled(),
      true,
    );
    return [
      "a real handle starts a native pointer drag and owner-document moves continue beyond it",
      "right and bottom bounds subtract the whole native note rectangle",
      "start and end callbacks count completed real pointer gestures",
      "reactive X only keeps y while actual pointer movement changes x",
      "the native Lock control prevents a new pointer drag without changing its counters",
      "keyboard placement explicitly clamps writable coordinates independently of pointer disabled state",
      "stop removes native drag listeners while explicit keyboard reset still works",
      "the exact local draft whitespace survives bounds, axis, disabled state and listener disposal",
    ];
  }

  assert.equal(
    name,
    "text-selection",
    "each pointer/selection example owns a real native input law",
  );
  const excerpt = page.getByRole("textbox", { name: "Editable guide excerpt", exact: true });
  const selection = await page.evaluateHandle(() => document.getSelection());
  try {
    await excerpt.focus();
    await page.keyboard.press("ControlOrMeta+A");
    await output(page, "Selected text: Inspect this Vue guide.");
    await page
      .locator(".selection-counts")
      .filter({ hasText: "Ranges: 1 · Rectangles: 1" })
      .waitFor();
    const verifyGeometry = async () => {
      const rect = await page.evaluate(() => {
        const range = document.getSelection()?.getRangeAt(0);
        assertRange(range);
        const box = range.getBoundingClientRect();
        return { width: Math.round(box.width), height: Math.round(box.height) };
        function assertRange(value: Range | undefined): asserts value is Range {
          if (!value) throw new Error("native selection must retain a real range");
        }
      });
      await page
        .locator(".selection-geometry")
        .filter({ hasText: `First rectangle: ${rect.width} × ${rect.height} CSS pixels` })
        .waitFor();
    };
    await verifyGeometry();
    await page.keyboard.press("ArrowLeft");
    for (let i = 0; i < 5; i += 1) await page.keyboard.press("Shift+ArrowRight");
    await output(page, "Selected text: Inspe");
    assert.equal(
      await page.evaluate((original) => document.getSelection() === original, selection),
      true,
      "the real browser mutates the same Selection object",
    );
    await verifyGeometry();
    await activate(page, "Clear text selection");
    await output(page, "Selected text: none");
    await page
      .locator(".selection-counts")
      .filter({ hasText: "Ranges: 0 · Rectangles: 0" })
      .waitFor();
    await excerpt.focus();
    await page.keyboard.press("ControlOrMeta+A");
    await page.keyboard.press("ArrowLeft");
    await page.locator(".selection-shape").filter({ hasText: "Collapsed caret" }).waitFor();
    await output(page, "Selected text: none");
    await page
      .locator(".selection-counts")
      .filter({ hasText: "Ranges: 1 · Rectangles: 1" })
      .waitFor();
    await page.keyboard.press("ControlOrMeta+A");
    await page.keyboard.insertText("Local compiler note");
    assert.equal(await excerpt.textContent(), "Local compiler note");
    await page.keyboard.press("ControlOrMeta+A");
    await page.keyboard.press("ArrowLeft");
    for (let i = 0; i < 5; i += 1) await page.keyboard.press("Shift+ArrowRight");
    await output(page, "Selected text: Local");
    await verifyGeometry();
    return [
      "native keyboard selection reports the complete editable passage and a real range",
      "selected rectangle dimensions match the actual native Range geometry",
      "selectionchange refreshes a shorter selection while the same platform Selection object is mutated in place",
      "the native Clear operation removes all ranges and selected text",
      "empty selected text is distinguished from a live collapsed caret range",
      "native local editing and a new keyboard selection refresh the readonly derived views without persistence",
    ];
  } finally {
    await selection.dispose();
  }
}
