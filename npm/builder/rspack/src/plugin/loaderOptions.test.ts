import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";
import { applyNativeCssMode, isVizeMainLoader } from "./loaderOptions.ts";

const loader = "@vizejs/rspack-plugin/loader";

void test("shares CSS mode across nested static rule forms without changing compilation options", () => {
  const compilerOptions = { experimentalSelfComponent: true };
  const first = { loader, options: { ssr: true, vapor: false, compilerOptions } };
  const second = { loader, options: { ssr: false, vapor: true } };
  const rules = [
    { oneOf: [{ rules: [first, { use: [second, "other-loader"] }] }] },
    { use: loader },
    { use: { loader } },
    { loader: "other-loader", options: { css: { native: false } } },
  ];
  applyNativeCssMode(rules, true);
  assert.deepEqual(first.options, {
    ssr: true,
    vapor: false,
    compilerOptions,
    css: { native: true },
  });
  assert.equal(first.options.compilerOptions, compilerOptions);
  assert.deepEqual(second.options, { ssr: false, vapor: true, css: { native: true } });
  assert.deepEqual(rules[1].use, { loader, options: { css: { native: true } } });
  assert.deepEqual(rules[2].use, { loader, options: { css: { native: true } } });
  assert.deepEqual(rules[3].options, { css: { native: false } });
});

void test("automatic rules reject conflicting CSS modes and accept matching modes", () => {
  const rules = [{ loader, options: { css: { native: false } } }];
  assert.throws(() => applyNativeCssMode(rules, true), /Loader css.native conflicts/);
  assert.equal(rules[0].options.css.native, false);
  assert.doesNotThrow(() => applyNativeCssMode(rules, false));
});

void test("manual rules preserve per-loader CSS modes and default only unspecified modes", () => {
  for (const native of [true, false]) {
    const explicitNative = { loader, options: { ssr: true, css: { native: true } } };
    const explicitJs = { loader, options: { vapor: true, css: { native: false } } };
    const inherited = { loader, options: { ssr: false } };
    const nativeOptions = explicitNative.options;
    const jsOptions = explicitJs.options;
    const rules = [{ rules: [{ oneOf: [explicitNative, { use: [explicitJs] }, inherited] }] }];
    applyNativeCssMode(rules, native, false);
    assert.equal(explicitNative.options, nativeOptions);
    assert.equal(explicitJs.options, jsOptions);
    assert.deepEqual(inherited.options, { ssr: false, css: { native } });
  }
});

void test("recognizes package exports and the resolved package loader", () => {
  assert.equal(isVizeMainLoader(loader), true);
  assert.equal(isVizeMainLoader(createRequire(import.meta.url).resolve(loader)), true);
  assert.equal(
    isVizeMainLoader("C:\\node_modules\\@vizejs\\rspack-plugin\\dist\\loader\\index.mjs"),
    true,
  );
  assert.equal(isVizeMainLoader("@vizejs/rspack-plugin/jsx-loader"), false);
  assert.equal(isVizeMainLoader("unrelated-loader"), false);
});

void test("preserves explicit CSS modes in JSON string options for manual rules", () => {
  for (const native of [true, false]) {
    const options = JSON.stringify({ ssr: true, hotReload: false, css: { native } });
    const entry = { loader, options };
    applyNativeCssMode([{ use: [entry] }], !native, false);
    assert.equal(entry.options, options);
    assert.throws(() => applyNativeCssMode([{ use: [entry] }], !native), /conflicts/);
  }
});

void test("adds CSS defaults while preserving Rspack's JSON and query-string option semantics", () => {
  const json = { loader, options: '{"ssr":true,"hotReload":false}' };
  const query = { loader, options: "root=src%2Fcomponents&include=src&include=shared" };
  applyNativeCssMode([{ oneOf: [json, query] }], true, false);
  assert.deepEqual(json.options, { ssr: true, hotReload: false, css: { native: true } });
  assert.deepEqual(query.options, {
    root: "src/components",
    include: ["src", "shared"],
    css: { native: true },
  });
});

void test("does not silently discard malformed JSON loader options", () => {
  assert.throws(
    () => applyNativeCssMode([{ loader, options: '{"ssr":}' }], true, false),
    SyntaxError,
  );
});
