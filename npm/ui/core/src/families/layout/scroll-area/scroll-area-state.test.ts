import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";

import { test } from "vite-plus/test";

import {
  normalizeScrollAreaAriaToken,
  normalizeScrollAreaLength,
  resolveScrollAreaAria,
  resolveScrollAreaLayout,
  resolveScrollAreaOverflow,
} from "./scroll-area-runtime.ts";

const scrollAreaCss = await readFile(
  path.resolve("src/families/layout/scroll-area/scroll-area.css"),
  "utf8",
);

test("resolves native overflow, sizing, and CSS hook state without DOM reads", () => {
  assert.equal(normalizeScrollAreaAriaToken(undefined), undefined);
  assert.equal(normalizeScrollAreaAriaToken("   "), undefined);
  assert.equal(normalizeScrollAreaAriaToken(" title   help "), "title help");
  assert.equal(normalizeScrollAreaLength(240, "auto"), "240px");
  assert.equal(normalizeScrollAreaLength(Number.NaN, "auto"), "auto");
  assert.deepEqual(resolveScrollAreaOverflow("vertical"), {
    overflowX: "hidden",
    overflowY: "auto",
  });
  assert.deepEqual(resolveScrollAreaOverflow("horizontal"), {
    overflowX: "auto",
    overflowY: "hidden",
  });
  assert.deepEqual(resolveScrollAreaOverflow("both"), {
    overflowX: "auto",
    overflowY: "auto",
  });
  assert.deepEqual(resolveScrollAreaAria({ ariaDescribedby: " help ", ariaLabel: " Updates " }), {
    ariaDescribedby: "help",
    ariaLabel: "Updates",
    ariaLabelledby: undefined,
  });
  assert.deepEqual(
    resolveScrollAreaLayout({
      blockSize: 320,
      dir: "rtl",
      focusable: true,
      inlineSize: "min(100%, 32rem)",
      maxBlockSize: "70vh",
      orientation: "both",
      overscrollBehavior: "contain",
      scrollBehavior: "smooth",
      scrollbarGutter: "stable both-edges",
      scrollbarWidth: "thin",
    }),
    {
      blockSize: "320px",
      dir: "rtl",
      focusable: true,
      inlineSize: "min(100%, 32rem)",
      maxBlockSize: "70vh",
      maxInlineSize: "none",
      orientation: "both",
      overflowX: "auto",
      overflowY: "auto",
      overscrollBehavior: "contain",
      scrollBehavior: "smooth",
      scrollbarGutter: "stable both-edges",
      scrollbarWidth: "thin",
      state: "scrollable",
      style: {
        "--vize-ui-scroll-area-block-size": "320px",
        "--vize-ui-scroll-area-inline-size": "min(100%, 32rem)",
        "--vize-ui-scroll-area-max-block-size": "70vh",
        "--vize-ui-scroll-area-max-inline-size": "none",
        "--vize-ui-scroll-area-overscroll-behavior": "contain",
        "--vize-ui-scroll-area-overflow-x": "auto",
        "--vize-ui-scroll-area-overflow-y": "auto",
        "--vize-ui-scroll-area-scroll-behavior": "smooth",
        "--vize-ui-scroll-area-scrollbar-gutter": "stable both-edges",
        "--vize-ui-scroll-area-scrollbar-width": "thin",
      },
    },
  );
});

test("ships zero-specificity native CSS hooks for motion and high contrast preferences", () => {
  assert.match(scrollAreaCss, /@layer vize\.ui/);
  assert.match(scrollAreaCss, /:where\(\[data-vize-ui="scroll-area"\]\)/);
  assert.match(scrollAreaCss, /:where\(\[data-vize-ui="scroll-area-viewport"\]\)/);
  assert.match(scrollAreaCss, /max-block-size: var\(--vize-ui-scroll-area-max-block-size, none\);/);
  assert.match(
    scrollAreaCss,
    /@media \(prefers-reduced-motion: reduce\) \{[\s\S]*scroll-behavior: auto;/,
  );
  assert.match(
    scrollAreaCss,
    /@media \(forced-colors: active\) \{[\s\S]*scrollbar-color: auto;[\s\S]*scrollbar-width: auto;/,
  );
});
