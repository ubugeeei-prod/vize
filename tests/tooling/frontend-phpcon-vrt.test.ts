import assert from "node:assert/strict";
import test from "node:test";

import {
  MOBILE_VIEWPORT,
  NEWS_ROUTE_MAX_DIFF_RATIO,
  PREVIEW_MOBILE_MAX_DIFF_PIXELS,
  STRICT_ROUTE_MAX_DIFF_RATIO,
  frontendPhpconVisualRoutes,
  maxDiffPixelsForFrontendPhpconMode,
} from "../app/vrt/frontend-phpcon-routes.ts";

test("frontend-phpcon preview mobile visual budget is mode-scoped", () => {
  const homeMobile = route("home-mobile");
  const mobileMenu = route("mobile-menu");

  assert.equal(STRICT_ROUTE_MAX_DIFF_RATIO, 0.004);
  assert.equal(NEWS_ROUTE_MAX_DIFF_RATIO, 0.009);
  assert.equal(PREVIEW_MOBILE_MAX_DIFF_PIXELS, 43_887);
  assert.deepEqual(homeMobile.viewport, MOBILE_VIEWPORT);
  assert.equal(homeMobile.maxDiffRatio, STRICT_ROUTE_MAX_DIFF_RATIO);
  assert.equal(maxDiffPixelsForFrontendPhpconMode(homeMobile, "preview"), 43_887);
  assert.equal(maxDiffPixelsForFrontendPhpconMode(homeMobile, "dev"), undefined);
  assert.deepEqual(mobileMenu.viewport, MOBILE_VIEWPORT);
  assert.equal(mobileMenu.maxDiffRatio, STRICT_ROUTE_MAX_DIFF_RATIO);
  assert.equal(maxDiffPixelsForFrontendPhpconMode(mobileMenu, "preview"), 43_887);
  assert.equal(maxDiffPixelsForFrontendPhpconMode(mobileMenu, "dev"), undefined);
});

test("frontend-phpcon timetable route budgets preview text antialiasing", () => {
  // Preview builds render identical timetable content with sub-pixel text
  // antialiasing drift (~0.0027 measured), above the helper default of 0.002.
  assert.equal(route("timetable").maxDiffRatio, STRICT_ROUTE_MAX_DIFF_RATIO);
});

test("frontend-phpcon news routes have article-specific visual tolerance", () => {
  // Preview builds render identical article copy with sub-pixel antialiasing
  // drift over thousands of text rows (~0.0084 measured on english-news), so
  // the shared news budget stays above the strict route budget.
  assert.ok(NEWS_ROUTE_MAX_DIFF_RATIO > STRICT_ROUTE_MAX_DIFF_RATIO);
  assert.equal(route("news").maxDiffRatio, NEWS_ROUTE_MAX_DIFF_RATIO);
  assert.equal(route("english-news").maxDiffRatio, NEWS_ROUTE_MAX_DIFF_RATIO);
});

function route(name: string) {
  const found = frontendPhpconVisualRoutes.find((route) => route.name === name);
  assert.ok(found, `${name} route should exist`);
  return found;
}
