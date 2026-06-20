import { test } from "node:test";
import { normalizeOptions } from "./unplugin.ts";

void test("defaults from an empty options object", (t) => {
  const options = normalizeOptions({ isProduction: false });

  t.assert.equal(options.ssr, false);
  t.assert.equal(options.vapor, false);
  t.assert.equal(options.customRenderer, false);
  t.assert.equal(options.templateSyntax, "standard");
  t.assert.equal(options.runtimeModuleName, "vue");
  t.assert.equal(options.runtimeGlobalName, "Vue");
  t.assert.equal(options.vueVersion, 3);
  t.assert.equal(options.hostCompiler, false);
  t.assert.equal(options.mode, "module");
  t.assert.equal(options.debug, false);
  t.assert.equal(options.root, process.cwd());
  t.assert.deepStrictEqual(options.compatibility, {});
});

void test("normalizeOptions can be called with no arguments", (t) => {
  const options = normalizeOptions();

  t.assert.equal(options.vueVersion, 3);
  t.assert.equal(options.mode, "module");
  t.assert.equal(options.templateSyntax, "standard");
  t.assert.equal(options.hostCompiler, false);
});

void test("legacy vue versions activate the host compiler", (t) => {
  t.assert.equal(normalizeOptions({ vueVersion: 0.11, isProduction: true }).hostCompiler, true);
  t.assert.equal(normalizeOptions({ vueVersion: 1, isProduction: true }).hostCompiler, true);
  t.assert.equal(normalizeOptions({ vueVersion: 2, isProduction: true }).hostCompiler, true);
  t.assert.equal(normalizeOptions({ vueVersion: "legacy", isProduction: true }).hostCompiler, true);
});

void test("vue 3 keeps the host compiler off", (t) => {
  t.assert.equal(normalizeOptions({ vueVersion: 3, isProduction: true }).hostCompiler, false);
});

void test("explicit compatibility.hostCompiler overrides the inferred value", (t) => {
  // Legacy version would infer true, but an explicit false wins.
  t.assert.equal(
    normalizeOptions({
      vueVersion: 2,
      compatibility: { hostCompiler: false },
      isProduction: true,
    }).hostCompiler,
    false,
  );

  // Vue 3 would infer false, but an explicit true wins.
  t.assert.equal(
    normalizeOptions({
      vueVersion: 3,
      compatibility: { hostCompiler: true },
      isProduction: true,
    }).hostCompiler,
    true,
  );
});

void test("vueVersion falls back to compatibility.vueVersion", (t) => {
  const options = normalizeOptions({ compatibility: { vueVersion: 2 }, isProduction: true });

  t.assert.equal(options.vueVersion, 2);
  // The legacy version inferred from compatibility also activates the host compiler.
  t.assert.equal(options.hostCompiler, true);
});

void test("top-level vueVersion wins over compatibility.vueVersion", (t) => {
  const options = normalizeOptions({
    vueVersion: 3,
    compatibility: { vueVersion: 2 },
    isProduction: true,
  });

  t.assert.equal(options.vueVersion, 3);
  t.assert.equal(options.hostCompiler, false);
});

void test("scriptSetupInStandalone switches mode to function", (t) => {
  const options = normalizeOptions({
    compatibility: { scriptSetupInStandalone: true },
    isProduction: true,
  });

  t.assert.equal(options.mode, "function");
});

void test("explicit mode wins over scriptSetupInStandalone", (t) => {
  const options = normalizeOptions({
    mode: "module",
    compatibility: { scriptSetupInStandalone: true },
    isProduction: true,
  });

  t.assert.equal(options.mode, "module");
});

void test("sourceMap defaults to the inverse of isProduction", (t) => {
  t.assert.equal(normalizeOptions({ isProduction: true }).sourceMap, false);
  t.assert.equal(normalizeOptions({ isProduction: false }).sourceMap, true);
});

void test("explicit sourceMap is respected regardless of isProduction", (t) => {
  t.assert.equal(normalizeOptions({ isProduction: true, sourceMap: true }).sourceMap, true);
  t.assert.equal(normalizeOptions({ isProduction: false, sourceMap: false }).sourceMap, false);
});

void test("templateSyntax passes through non-standard values", (t) => {
  t.assert.equal(
    normalizeOptions({ templateSyntax: "strict", isProduction: true }).templateSyntax,
    "strict",
  );
  t.assert.equal(
    normalizeOptions({ templateSyntax: "quirks", isProduction: true }).templateSyntax,
    "quirks",
  );
});

void test("runtime module and global names can be overridden", (t) => {
  const options = normalizeOptions({
    runtimeModuleName: "@vue/runtime-core",
    runtimeGlobalName: "VueGlobal",
    isProduction: true,
  });

  t.assert.equal(options.runtimeModuleName, "@vue/runtime-core");
  t.assert.equal(options.runtimeGlobalName, "VueGlobal");
});

void test("the compatibility object is passed through by reference", (t) => {
  const compatibility = { scriptSetupInStandalone: false };
  const options = normalizeOptions({ compatibility, isProduction: true });

  t.assert.equal(options.compatibility, compatibility);
});

void test("explicit root overrides process.cwd()", (t) => {
  const options = normalizeOptions({ root: "/custom/root", isProduction: true });

  t.assert.equal(options.root, "/custom/root");
});
