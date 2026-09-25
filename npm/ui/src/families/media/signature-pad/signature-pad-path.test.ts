import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  SignaturePadError,
  isSignatureEmpty,
  parseSignature,
  serializeSignature,
  signatureToDataUrl,
  signatureToSvg,
  simulatePressure,
  strokeOutlinePath,
} from "./signature-pad-path.ts";
import type { SignaturePoint, SignatureStroke } from "./signature-pad-types.ts";

function stroke(...points: readonly (readonly [number, number, number?])[]): SignatureStroke {
  return {
    points: points.map(([x, y, pressure = 0.5], index) => ({ x, y, pressure, time: index * 10 })),
  };
}

test("detects empty signatures", () => {
  assert.equal(isSignatureEmpty([]), true);
  assert.equal(isSignatureEmpty([{ points: [] }]), true);
  assert.equal(isSignatureEmpty([stroke([1, 1])]), false);
});

test("simulates pressure from velocity with easing", () => {
  assert.equal(simulatePressure(undefined, 0, 0, 0), 0.5);
  const previous: SignaturePoint = { x: 0, y: 0, pressure: 0.5, time: 0 };
  const slow = simulatePressure(previous, 1, 0, 100);
  const fast = simulatePressure(previous, 100, 0, 10);
  assert.ok(slow > previous.pressure, "slow movement thickens");
  assert.ok(fast < previous.pressure, "fast movement thins");
  assert.ok(fast >= 0 && slow <= 1);
  assert.equal(simulatePressure(previous, 0, 0, 0), 0.5 * 0.7 + 0.3);
});

test("renders single samples as dots and empty strokes as nothing", () => {
  assert.equal(strokeOutlinePath({ points: [] }), "");
  assert.equal(
    strokeOutlinePath(stroke([10, 10])),
    "M 8.95 10 a 1.05 1.05 0 1 0 2.1 0 a 1.05 1.05 0 1 0 -2.1 0 Z",
  );
  assert.equal(strokeOutlinePath(stroke([10, 10]), { size: 0 }), "");
  assert.equal(
    strokeOutlinePath(stroke([5, 5], [5, 5.001])),
    strokeOutlinePath(stroke([5, 5])),
    "near-duplicate samples collapse",
  );
});

test("outlines strokes with pressure-dependent width, smoothing, and round caps", () => {
  const straight = strokeOutlinePath(stroke([0, 0, 1], [10, 0, 1], [20, 0, 1]), {
    size: 4,
    thinning: 0,
    smoothing: 0,
  });
  assert.equal(
    straight,
    "M 0 2 Q 10 2 10 2 L 20 2 A 2 2 0 0 0 20 -2 Q 10 -2 10 -2 L 0 -2 A 2 2 0 0 0 0 2 Z",
  );
  const smoothed = strokeOutlinePath(stroke([0, 0, 1], [10, 0, 1], [20, 0, 1]), {
    size: 4,
    thinning: 0,
    smoothing: 1,
  });
  assert.match(smoothed, /Q 10 2 15 2/);
  const thin = strokeOutlinePath(stroke([0, 0, 0], [10, 0, 0]), { size: 4, thinning: 1 });
  assert.match(thin, /^M 0 0\.01 /, "zero pressure keeps a hairline");
  const partial = strokeOutlinePath(stroke([0, 0, 0.5], [0, 10, 0.5]), { size: 4, thinning: 0.5 });
  assert.match(partial, /^M -1\.5 0 /, "vertical strokes offset along x");
});

test("serializes standalone SVG with escaped attributes and optional background", () => {
  const svg = signatureToSvg([stroke([1, 1]), { points: [] }], {
    width: 100,
    height: 50,
    color: 'red"><script>',
    background: "#fff",
  });
  assert.match(
    svg,
    /^<svg xmlns="http:\/\/www\.w3\.org\/2000\/svg" viewBox="0 0 100 50" width="100" height="50">/,
  );
  assert.match(svg, /<rect width="100%" height="100%" fill="#fff"\/>/);
  assert.equal(svg.match(/<path /g)?.length, 1);
  assert.match(svg, /fill="red&quot;&gt;&lt;script&gt;"/);
  assert.doesNotMatch(signatureToSvg([], { width: 10, height: 10 }), /<rect|<path/);
});

test("exports SVG data URLs without a canvas and rejects raster export without one", async () => {
  const url = await signatureToDataUrl([stroke([1, 1])], {
    width: 10,
    height: 10,
    type: "image/svg+xml",
  });
  assert.match(url, /^data:image\/svg\+xml;charset=utf-8,%3Csvg/);

  const previous = globalThis.Path2D;
  Reflect.deleteProperty(globalThis, "Path2D");
  try {
    await assert.rejects(
      signatureToDataUrl([stroke([1, 1])], { width: 10, height: 10 }),
      (error: unknown) =>
        error instanceof SignaturePadError &&
        error.code === "VIZE_UI_SIGNATURE_PAD_CANVAS_UNAVAILABLE",
    );
  } finally {
    if (previous !== undefined) globalThis.Path2D = previous;
  }
});

test("draws raster exports through Path2D on a scaled canvas", async () => {
  const calls: string[] = [];
  const context = {
    fillStyle: "",
    scale: (x: number, y: number) => calls.push(`scale ${x} ${y}`),
    fillRect: (...args: number[]) => calls.push(`fillRect ${args.join(" ")} ${context.fillStyle}`),
    fill: (path: unknown) =>
      calls.push(
        `fill ${path instanceof FakePath2D ? path.d.slice(0, 4) : "?"} ${context.fillStyle}`,
      ),
  };
  class FakePath2D {
    constructor(readonly d: string) {}
  }
  const previousPath = globalThis.Path2D;
  const createElement = document.createElement.bind(document);
  globalThis.Path2D = FakePath2D as unknown as typeof Path2D;
  document.createElement = ((tag: string) => {
    if (tag !== "canvas") return createElement(tag);
    const canvas = createElement("canvas");
    Object.defineProperty(canvas, "getContext", { value: () => context });
    Object.defineProperty(canvas, "toDataURL", {
      value: (type: string, quality?: number) =>
        `data:${type};w=${canvas.width};h=${canvas.height};q=${String(quality)}`,
    });
    return canvas;
  }) as typeof document.createElement;
  try {
    const url = await signatureToDataUrl([stroke([1, 1])], {
      width: 100,
      height: 50,
      scale: 2,
      type: "image/jpeg",
      quality: 0.8,
      background: "white",
      color: "navy",
    });
    assert.equal(url, "data:image/jpeg;w=200;h=100;q=0.8");
    assert.deepEqual(calls, ["scale 2 2", "fillRect 0 0 100 50 white", "fill M -0 navy"]);
  } finally {
    document.createElement = createElement as typeof document.createElement;
    if (previousPath === undefined) Reflect.deleteProperty(globalThis, "Path2D");
    else globalThis.Path2D = previousPath;
  }
});

test("round-trips compact JSON and serializes SVG for forms", () => {
  const value = [
    {
      points: [
        { x: 1.23456, y: 2.34567, pressure: 0.45678, time: 12.6 },
        { x: 3, y: 4, pressure: 1, time: 20 },
      ],
    },
  ];
  const json = serializeSignature(value, "json", { width: 10, height: 10 });
  assert.equal(
    json,
    '[{"points":[{"x":1.23,"y":2.35,"pressure":0.457,"time":13},{"x":3,"y":4,"pressure":1,"time":20}]}]',
  );
  assert.deepEqual(parseSignature(json), JSON.parse(json));
  assert.ok(Object.isFrozen(parseSignature(json)));
  assert.equal(serializeSignature([], "json", { width: 10, height: 10 }), "");
  assert.match(serializeSignature(value, "svg", { width: 10, height: 10 }), /^<svg /);
  assert.deepEqual(parseSignature("  "), []);
  assert.equal(
    parseSignature('[{"points":[{"x":1,"y":1,"pressure":3,"time":0}]}]')[0]?.points[0]?.pressure,
    1,
  );
  for (const invalid of [
    "{",
    "{}",
    '[{"points":1}]',
    '[{"points":[{"x":"1","y":1,"pressure":1,"time":0}]}]',
    "[1]",
  ]) {
    assert.throws(
      () => parseSignature(invalid),
      (error: unknown) =>
        error instanceof SignaturePadError &&
        error.code === "VIZE_UI_SIGNATURE_PAD_INVALID_VALUE" &&
        error.message.startsWith("VIZE_UI_SIGNATURE_PAD_INVALID_VALUE: "),
    );
  }
});
