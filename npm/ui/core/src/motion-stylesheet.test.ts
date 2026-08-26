import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
// Paths are resolved from the package cwd: the runner virtualizes import.meta.url.
import path from "node:path";

import { test } from "vite-plus/test";

import { motionDelays, motionDurations, motionEasings } from "./motion.ts";

const stylesheet = await readFile(path.resolve("dist/style.css"), "utf8");

/** Value of one custom property in the packaged stylesheet. */
function shippedToken(name: string): string {
  const pattern = new RegExp(`${name.replaceAll(/[-]/g, "\\-")}:([^;}]+)[;}]`);
  const match = pattern.exec(stylesheet);
  assert.ok(match?.[1], `dist/style.css must define ${name}`);
  return match[1];
}

/** CSS time literal in milliseconds, tolerant of minifier unit rewrites. */
function timeToMs(value: string): number {
  const trimmed = value.trim();
  if (trimmed.endsWith("ms")) return Number.parseFloat(trimmed);
  assert.ok(trimmed.endsWith("s"), `${value} must be a CSS time`);
  return Number.parseFloat(trimmed) * 1_000;
}

/** Normalized comparison form, tolerant of minifier whitespace and zero trims. */
function normalizeCss(value: string): string {
  return value.replaceAll(/\s+/g, "").replaceAll("0.", ".");
}

test("ships layered zero-specificity motion tokens matching the mirrors", () => {
  assert.match(stylesheet, /@layer vize\.ui\{/);
  assert.match(stylesheet, /:where\(:root,:host\)\{--vize-ui-motion-duration-instant:/);

  for (const [token, value] of Object.entries(motionDurations)) {
    assert.equal(timeToMs(shippedToken(`--vize-ui-motion-duration-${token}`)), timeToMs(value));
  }
  for (const [token, value] of Object.entries(motionDelays)) {
    assert.equal(timeToMs(shippedToken(`--vize-ui-motion-delay-${token}`)), timeToMs(value));
  }
  for (const [token, value] of Object.entries(motionEasings)) {
    assert.equal(normalizeCss(shippedToken(`--vize-ui-motion-ease-${token}`)), normalizeCss(value));
  }

  // Recipe hooks resolve through the base scales so one override retunes a phase.
  assert.equal(
    shippedToken("--vize-ui-motion-enter-easing"),
    "var(--vize-ui-motion-ease-decelerate)",
  );
  assert.equal(
    shippedToken("--vize-ui-motion-exit-duration"),
    "var(--vize-ui-motion-duration-fast)",
  );
});

test("pairs enter and exit recipes with presence and transition hooks", () => {
  for (const recipe of ["fade", "scale", "slide"] as const) {
    assert.match(
      stylesheet,
      new RegExp(
        `:where\\(\\[data-vize-motion~=${recipe}\\]\\):where\\(\\[data-vize-presence=entering\\],` +
          `\\[data-vize-transition=entering\\]\\)\\{animation-name:vize-ui-motion-${recipe}-in\\}`,
      ),
    );
    assert.match(
      stylesheet,
      new RegExp(
        `:where\\(\\[data-vize-motion~=${recipe}\\]\\):where\\(\\[data-vize-presence=exiting\\],` +
          `\\[data-vize-transition=exiting\\]\\)\\{animation-name:vize-ui-motion-${recipe}-out\\}`,
      ),
    );
    assert.match(stylesheet, new RegExp(`@keyframes vize-ui-motion-${recipe}-in\\{`));
    assert.match(stylesheet, new RegExp(`@keyframes vize-ui-motion-${recipe}-out\\{`));
  }

  // Shared enter/exit timing reads the recipe hooks, never literal durations.
  assert.match(
    stylesheet,
    /\[data-vize-transition=entering\]\)\{animation-duration:var\(--vize-ui-motion-enter-duration\);animation-timing-function:var\(--vize-ui-motion-enter-easing\);animation-fill-mode:both\}/,
  );
  assert.match(
    stylesheet,
    /\[data-vize-transition=exiting\]\)\{animation-duration:var\(--vize-ui-motion-exit-duration\);animation-timing-function:var\(--vize-ui-motion-exit-easing\);animation-fill-mode:both\}/,
  );
});

test("ships move and emphasis recipes with token-driven timing", () => {
  assert.match(
    stylesheet,
    /:where\(\[data-vize-motion~=move\]\)\{transition-property:translate,transform,inset-block-start,inset-block-end,inset-inline-start,inset-inline-end;transition-duration:var\(--vize-ui-motion-move-duration\);transition-timing-function:var\(--vize-ui-motion-move-easing\)\}/,
  );
  assert.match(
    stylesheet,
    /:where\(\[data-vize-motion~=pulse\]\)\{animation:vize-ui-motion-pulse var\(--vize-ui-motion-emphasis-duration\) var\(--vize-ui-motion-emphasis-easing\)\}/,
  );
  assert.match(stylesheet, /@keyframes vize-ui-motion-shake\{/);
  // Document-level view transitions inherit the shared tokens.
  assert.match(
    stylesheet,
    /::view-transition-old\(root\)\{animation-duration:var\(--vize-ui-motion-duration-base\)/,
  );
});

test("ships the starting-style and scroll-driven recipes verbatim", () => {
  assert.match(
    stylesheet,
    /:where\(\[data-vize-motion~=enter\]\)\{transition-property:opacity,translate,scale;transition-duration:var\(--vize-ui-motion-enter-duration\)/,
  );
  assert.match(
    stylesheet,
    /@starting-style\{:where\(\[data-vize-motion~=enter\]\)\{opacity:0;translate:0 var\(--vize-ui-motion-slide-distance\)\}\}/,
  );
  // Both at-features are newer than the floor and must pass through unlowered.
  // The authored `entry 0% entry 100%` range minifies to its `entry` shorthand.
  assert.match(
    stylesheet,
    /:where\(\[data-vize-motion~=reveal\]\)\{animation:vize-ui-motion-fade-in var\(--vize-ui-motion-ease-standard\) both;animation-timeline:view\(\);animation-range:entry\}/,
  );
});

test("zeroes packaged motion under reduced motion", () => {
  const start = stylesheet.indexOf("@media (prefers-reduced-motion:reduce)");
  assert.ok(start >= 0, "the reduced-motion policy block must ship");
  const block = stylesheet.slice(start, stylesheet.indexOf("@media (forced-colors:active)"));

  for (const token of Object.keys(motionDurations)) {
    assert.match(block, new RegExp(`--vize-ui-motion-duration-${token}:0s`));
  }
  for (const token of Object.keys(motionDelays)) {
    assert.match(block, new RegExp(`--vize-ui-motion-delay-${token}:0s`));
  }
  assert.match(
    block,
    /:where\(\[data-vize-motion\]\)\{transition-duration:0s;animation-duration:0s\}/,
  );
  // Timeline-driven animations ignore zeroed durations, so reveal stands down.
  assert.match(block, /:where\(\[data-vize-motion~=reveal\]\)\{animation:none\}/);
});

test("stands down under forced colors", () => {
  const start = stylesheet.indexOf("@media (forced-colors:active)");
  assert.ok(start >= 0, "the forced-colors policy block must ship");
  assert.match(
    stylesheet.slice(start),
    /:where\(\[data-vize-motion\]\)\{transition-property:none;animation-name:none\}/,
  );
});
