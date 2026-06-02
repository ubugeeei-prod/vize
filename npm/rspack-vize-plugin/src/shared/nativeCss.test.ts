import { describe, test } from "node:test";
import assert from "node:assert/strict";
import { getLegacyNativeCssState, getRspackMajor, resolveNativeCss } from "./nativeCss.ts";

void describe("getLegacyNativeCssState", () => {
  void test("returns 'unavailable' when experiments is missing", () => {
    assert.equal(getLegacyNativeCssState({}), "unavailable");
    assert.equal(getLegacyNativeCssState(undefined), "unavailable");
  });

  void test("returns 'unavailable' when experiments has no css key (Rspack 2.x)", () => {
    assert.equal(getLegacyNativeCssState({ experiments: {} }), "unavailable");
  });

  void test("reflects an explicit experiments.css value", () => {
    assert.equal(getLegacyNativeCssState({ experiments: { css: true } }), "enabled");
    assert.equal(getLegacyNativeCssState({ experiments: { css: false } }), "disabled");
  });
});

void describe("getRspackMajor", () => {
  void test("parses the major version from a version string", () => {
    assert.equal(getRspackMajor("2.0.3"), 2);
    assert.equal(getRspackMajor("1.4.0"), 1);
    assert.equal(getRspackMajor("10.1.0"), 10);
  });

  void test("returns null for non-string or unparseable input", () => {
    assert.equal(getRspackMajor(undefined), null);
    assert.equal(getRspackMajor(null), null);
    assert.equal(getRspackMajor("next"), null);
  });
});

void describe("resolveNativeCss", () => {
  void test("explicit css.native always wins over everything", () => {
    // explicit false beats Rspack 2.x native default
    assert.equal(resolveNativeCss(false, {}, "2.0.3"), false);
    // explicit false beats experiments.css: true
    assert.equal(resolveNativeCss(false, { experiments: { css: true } }, "1.4.0"), false);
    // explicit true beats experiments.css: false
    assert.equal(resolveNativeCss(true, { experiments: { css: false } }, "1.4.0"), true);
  });

  void test("honors explicit experiments.css when css.native is unset (Rspack 1.x)", () => {
    assert.equal(resolveNativeCss(undefined, { experiments: { css: true } }, "1.4.0"), true);
    assert.equal(resolveNativeCss(undefined, { experiments: { css: false } }, "1.4.0"), false);
  });

  void test("defaults to native on Rspack 2.x+ when no explicit signal is given", () => {
    assert.equal(resolveNativeCss(undefined, {}, "2.0.3"), true);
    assert.equal(resolveNativeCss(undefined, { experiments: {} }, "2.0.3"), true);
    assert.equal(resolveNativeCss(undefined, {}, "3.0.0"), true);
  });

  void test("defaults to non-native on Rspack 1.x when experiments.css is omitted", () => {
    assert.equal(resolveNativeCss(undefined, {}, "1.4.0"), false);
    assert.equal(resolveNativeCss(undefined, { experiments: {} }, "1.4.0"), false);
  });

  void test("defaults to non-native when the Rspack version is unknown", () => {
    assert.equal(resolveNativeCss(undefined, {}, undefined), false);
    assert.equal(resolveNativeCss(undefined, {}), false);
  });
});
