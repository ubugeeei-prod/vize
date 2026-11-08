import assert from "node:assert/strict";
import { test } from "vitest";

import { firstChild, mountComponent } from "../testing/index.js";
import { defineQrCodeProvider, QrCode } from "./index.js";

const provider = defineQrCodeProvider((_value, options) => {
  assert.equal(options.errorCorrection, "high");
  return [
    [true, false, true],
    [false, true, false],
    [true, false, true],
  ];
});

test("renders a provider matrix with configurable cells and quiet zone", () => {
  const mounted = mountComponent(QrCode, {
    value: "https://example.test",
    provider,
    errorCorrection: "high",
    quietZone: 1,
    cellWidth: 1,
    dark: "#",
    light: ".",
    fg: "black",
    bg: "white",
  });
  const root = firstChild(mounted);

  assert.equal(root.props["aria-label"], "QR code for https://example.test");
  assert.deepEqual(
    root.children.map((row) => row.props.text),
    [".....", ".#.#.", "..#..", ".#.#.", "....."],
  );
  assert.ok(root.children.every((row) => row.props.fg === "black" && row.props.bg === "white"));
  mounted.unmount();
});

test("supports inverted QR cells", () => {
  const mounted = mountComponent(QrCode, {
    value: "value",
    provider: () => [[true]],
    quietZone: 0,
    cellWidth: 2,
    dark: "X",
    light: "_",
    invert: true,
  });
  assert.deepEqual(
    firstChild(mounted).children.map((row) => row.props.text),
    ["__"],
  );
  mounted.unmount();
});

test("rejects malformed provider matrices", () => {
  assert.throws(
    () => mountComponent(QrCode, { value: "bad", provider: () => [[true, false]] }),
    /FRESCO_QR_MATRIX_SQUARE/u,
  );
  assert.throws(
    () =>
      mountComponent(QrCode, {
        value: "bad",
        provider: () => [[1]] as unknown as readonly (readonly boolean[])[],
      }),
    /FRESCO_QR_MATRIX_CELL/u,
  );
});
