import assert from "node:assert/strict";
import { test } from "vitest";

import { firstChild, mountComponent } from "../testing/index.js";
import { LineNumber } from "./LineNumber.js";

test("renders a compact line-number gutter by default", () => {
  const mounted = mountComponent(LineNumber, { value: 7 });
  const line = firstChild(mounted);

  assert.equal(line.type, "text");
  assert.equal(line.props.text, "7 │ ");
  assert.equal(line.props.fg, "gray");
  assert.equal(line.props["aria-label"], "Line 7");
  mounted.unmount();
});

test("aligns numbers and exposes active-line styling", () => {
  const mounted = mountComponent(LineNumber, {
    value: 12.9,
    width: 4,
    separator: "→",
    padding: 0,
    active: true,
    activeFg: "black",
    activeBg: "yellow",
  });
  const line = firstChild(mounted);

  assert.equal(line.props.text, "  12→");
  assert.equal(line.props.fg, "black");
  assert.equal(line.props.bg, "yellow");
  assert.equal(line.props.bold, true);
  mounted.unmount();
});

test("normalizes non-finite values and negative layout inputs", () => {
  const mounted = mountComponent(LineNumber, {
    value: Number.NaN,
    width: -10,
    padding: -4,
  });

  assert.equal(firstChild(mounted).props.text, "0│");
  assert.equal(firstChild(mounted).props["aria-label"], "Line 0");
  mounted.unmount();
});
