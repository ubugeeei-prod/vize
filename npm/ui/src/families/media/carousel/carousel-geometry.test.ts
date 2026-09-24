import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  nearestSlideIndex,
  resolveSlideIndex,
  scrollEdges,
  scrollPositionForSlide,
  slideIndexAfterDrag,
  slideStartOffset,
} from "./carousel-geometry.ts";
import type { CarouselRect } from "./carousel-geometry.ts";

function box(left: number, width: number, top = 0, height = 100): CarouselRect {
  return { top, right: left + width, bottom: top + height, left };
}

const ltr = { orientation: "horizontal", dir: "ltr" } as const;
const rtl = { orientation: "horizontal", dir: "rtl" } as const;
const vertical = { orientation: "vertical", dir: "ltr" } as const;

test("resolves slide indexes by clamping, wrapping, and rounding", () => {
  assert.equal(resolveSlideIndex(2, 5, false), 2);
  assert.equal(resolveSlideIndex(-1, 5, false), 0);
  assert.equal(resolveSlideIndex(9, 5, false), 4);
  assert.equal(resolveSlideIndex(-1, 5, true), 4);
  assert.equal(resolveSlideIndex(5, 5, true), 0);
  assert.equal(resolveSlideIndex(-6, 5, true), 4);
  assert.equal(resolveSlideIndex(1.6, 5, false), 2);
  assert.equal(resolveSlideIndex(Number.NaN, 5, false), 0);
  assert.equal(resolveSlideIndex(Number.POSITIVE_INFINITY, 5, true), 0);
  assert.equal(resolveSlideIndex(3, 0, false), 0);
});

test("measures slide start offsets for ltr, rtl, and vertical axes", () => {
  const viewport = box(100, 300, 50, 200);
  assert.equal(slideStartOffset(viewport, box(250, 100), ltr), 150);
  assert.equal(slideStartOffset(viewport, box(-50, 100), ltr), -150);
  assert.equal(slideStartOffset(viewport, box(200, 100), rtl), -100);
  assert.equal(slideStartOffset(viewport, box(0, 100, 350, 100), vertical), 300);
  assert.equal(
    scrollPositionForSlide(
      { position: 40, clientSize: 300, scrollSize: 900 },
      viewport,
      box(250, 100),
      ltr,
    ),
    190,
  );
  assert.equal(
    scrollPositionForSlide(
      { position: -40, clientSize: 300, scrollSize: 900 },
      viewport,
      box(200, 100),
      rtl,
    ),
    -140,
  );
});

test("selects the slide nearest the viewport start and skips unmeasured slides", () => {
  const viewport = box(0, 100);
  assert.equal(nearestSlideIndex(viewport, [box(-80, 100), box(20, 100), box(120, 100)], ltr), 1);
  assert.equal(nearestSlideIndex(viewport, [box(-40, 100), box(60, 100)], ltr), 0);
  assert.equal(nearestSlideIndex(viewport, [null, box(300, 100)], ltr), 1);
  assert.equal(nearestSlideIndex(viewport, [null, null], ltr), -1);
  assert.equal(nearestSlideIndex(viewport, [], ltr), -1);
  assert.equal(nearestSlideIndex(viewport, [box(0, 100), box(-100, 100)], rtl), 0);
});

test("reports scroll edges, including unknown edges without overflow", () => {
  assert.deepEqual(scrollEdges({ position: 0, clientSize: 100, scrollSize: 100 }), {
    atStart: null,
    atEnd: null,
  });
  assert.deepEqual(scrollEdges({ position: 0, clientSize: 100, scrollSize: 400 }), {
    atStart: true,
    atEnd: false,
  });
  assert.deepEqual(scrollEdges({ position: 150, clientSize: 100, scrollSize: 400 }), {
    atStart: false,
    atEnd: false,
  });
  assert.deepEqual(scrollEdges({ position: 299.5, clientSize: 100, scrollSize: 400 }), {
    atStart: false,
    atEnd: true,
  });
  assert.deepEqual(scrollEdges({ position: -300, clientSize: 100, scrollSize: 400 }), {
    atStart: false,
    atEnd: true,
  });
});

test("pages short drags past the threshold and otherwise keeps the nearest slide", () => {
  const base = { count: 4, loop: false, threshold: 40 } as const;
  assert.equal(slideIndexAfterDrag({ ...base, startIndex: 1, nearestIndex: 1, delta: 60 }), 2);
  assert.equal(slideIndexAfterDrag({ ...base, startIndex: 1, nearestIndex: 1, delta: -60 }), 0);
  assert.equal(slideIndexAfterDrag({ ...base, startIndex: 1, nearestIndex: 1, delta: 20 }), 1);
  assert.equal(slideIndexAfterDrag({ ...base, startIndex: 1, nearestIndex: 3, delta: 220 }), 3);
  assert.equal(slideIndexAfterDrag({ ...base, startIndex: 3, nearestIndex: 3, delta: 60 }), 3);
  assert.equal(
    slideIndexAfterDrag({ ...base, loop: true, startIndex: 3, nearestIndex: 3, delta: 60 }),
    0,
  );
  assert.equal(slideIndexAfterDrag({ ...base, startIndex: 2, nearestIndex: -1, delta: 0 }), 2);
});
