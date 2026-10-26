/**
 * Smoke-test the publishable @vizejs/wasm package directory.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

const REQUIRED_FILES = [
  "index.js",
  "index.d.ts",
  "lint-format.d.ts",
  "vize_vitrine.js",
  "vize_vitrine.d.ts",
  "vize_vitrine_bg.wasm",
];

const EXPECTED_EXPORTS = [
  "Compiler",
  "compile",
  "compileCss",
  "compileJsx",
  "compileSfc",
  "compileVapor",
  "default",
  "formatSfc",
  "init",
  "isInitialized",
  "lintSfc",
  "parseSfc",
  "parseTemplate",
];

function requireArg(argv) {
  const packageDir = argv[0];
  if (!packageDir) {
    throw new Error("Usage: smoke-wasm-package.mjs <npm/wasm>");
  }
  return path.resolve(packageDir);
}

function readPackageJson(packageDir) {
  return JSON.parse(fs.readFileSync(path.join(packageDir, "package.json"), "utf8"));
}

function assertManifest(packageJson) {
  assert.equal(packageJson.name, "@vizejs/wasm");
  assert.equal(packageJson.type, "module");
  assert.equal(packageJson.main, "./index.js");
  assert.equal(packageJson.types, "./index.d.ts");
  assert.equal(packageJson.exports?.["."]?.import, "./index.js");
  assert.equal(packageJson.exports?.["."]?.types, "./index.d.ts");
  assert.equal(packageJson.exports?.["./vize_vitrine.js"]?.import, "./vize_vitrine.js");
  assert.equal(packageJson.exports?.["./vize_vitrine.js"]?.types, "./vize_vitrine.d.ts");
  assert.equal(packageJson.exports?.["./vize_vitrine_bg.wasm"], "./vize_vitrine_bg.wasm");

  for (const file of REQUIRED_FILES) {
    assert.ok(packageJson.files?.includes(file), `package files must include ${file}`);
  }
}

function assertFilesExist(packageDir) {
  for (const file of REQUIRED_FILES) {
    assert.ok(fs.existsSync(path.join(packageDir, file)), `${file} is missing`);
  }
}

function assertCompilerOptions(entry) {
  const moduleResult = entry.compile("<div>{{ message }}</div>", {
    mode: "module",
    runtimeModuleName: "@acme/vue-runtime",
    sourceMap: true,
    filename: "src/Component.vue",
  });
  assert.match(moduleResult.preamble, /@acme\/vue-runtime/);
  assert.equal(moduleResult.map?.version, 3);
  assert.deepEqual(moduleResult.map?.sources, ["src/Component.vue"]);

  const functionResult = entry.compile("<div></div>", {
    mode: "function",
    runtimeGlobalName: "AcmeVue",
  });
  assert.match(`${functionResult.preamble}\n${functionResult.code}`, /AcmeVue/);

  const legacyQuirks = entry.compile("<div /><span></span>", { vueParserQuirks: true });
  assert.equal(legacyQuirks.ast.children[0].isSelfClosing, true);
  assert.throws(() => entry.compile("<div /><span></span>", { templateSyntax: "strict" }));
  const parsedQuirks = entry.parseTemplate("<div />", { vueParserQuirks: true });
  assert.equal(parsedQuirks.children[0].isSelfClosing, true);
  assert.throws(() =>
    entry.parseTemplate("<div />", {
      templateSyntax: "standard",
      vueParserQuirks: true,
    }),
  );

  const typedSfc = '<script setup lang="ts">const count: number = 1</script>';
  const preserved = entry.compileSfc(typedSfc, { scriptExt: "preserve" });
  const downcompiled = entry.compileSfc(typedSfc, { scriptExt: "downcompile" });
  assert.match(preserved.script.code, /count:\s*number/);
  assert.doesNotMatch(downcompiled.script.code, /count:\s*number/);

  const sfcSource = "<template><div></div></template>";
  const moduleSfc = entry.compileSfc(sfcSource, {
    runtimeModuleName: "@acme/vue-runtime",
    sourceMap: true,
  });
  assert.match(moduleSfc.script.code, /@acme\/vue-runtime/);
  assert.deepEqual(moduleSfc.template.map?.sources, ["anonymous.vue"]);
  const standaloneSfc = entry.compileSfc(sfcSource, {
    mode: "function",
    runtimeModuleName: "@acme/vue-runtime",
    runtimeGlobalName: "AcmeVue",
  });
  assert.match(standaloneSfc.script.code, /AcmeVue/);
  assert.doesNotMatch(standaloneSfc.script.code, /@acme\/vue-runtime/);
}

async function assertEntryPoint(packageDir) {
  const entry = await import(
    `${pathToFileURL(path.join(packageDir, "index.js")).href}?smoke=${Date.now()}`
  );
  assert.deepEqual(Object.keys(entry).sort(), EXPECTED_EXPORTS);

  assert.equal(entry.default, entry.init);
  assert.equal(entry.isInitialized(), false);
  assert.throws(() => entry.compile("<div />"), /Call `await init\(\)` first/);
  assert.throws(() => entry.lintSfc("<template />"), /Call `await init\(\)` first/);
  assert.throws(() => entry.formatSfc("<template />"), /Call `await init\(\)` first/);

  await assert.rejects(() => entry.init(new Uint8Array()), /.+/);
  assert.equal(entry.isInitialized(), false);

  const failedUrl = new URL("https://vize.invalid/vize_vitrine_bg.wasm");
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (input, init) => {
    const inputUrl =
      typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
    if (inputUrl === failedUrl.href) {
      throw new Error("intentional WASM fetch failure");
    }
    return originalFetch(input, init);
  };
  try {
    await assert.rejects(() => entry.init(failedUrl), /intentional WASM fetch failure/);
  } finally {
    globalThis.fetch = originalFetch;
  }
  assert.equal(entry.isInitialized(), false);

  const wasm = fs.readFileSync(path.join(packageDir, "vize_vitrine_bg.wasm"));
  await entry.init(wasm);
  assert.equal(entry.isInitialized(), true);
  assertCompilerOptions(entry);

  const source = '<template>\n  <div  id="x"></div>\n</template>\n';
  const formattedSource = '<template>\n  <div id="x"></div>\n</template>\n';
  const lintResult = entry.lintSfc(source, {
    filename: "Smoke.vue",
    locale: "en",
    enabledRules: ["vue/no-multi-spaces"],
  });
  assert.equal(lintResult.filename, "Smoke.vue");
  assert.equal(lintResult.errorCount, 0);
  assert.equal(lintResult.warningCount, 1);
  assert.equal(lintResult.diagnostics.length, 1);
  assert.equal(lintResult.diagnostics[0].rule, "vue/no-multi-spaces");
  assert.equal(lintResult.diagnostics[0].severity, "warning");
  assert.equal(lintResult.diagnostics[0].location.start.offset, 17);
  assert.equal(lintResult.diagnostics[0].help, undefined);

  const formatResult = entry.formatSfc(source, { printWidth: 80 });
  assert.equal(formatResult.code, formattedSource);
  assert.equal(formatResult.changed, true);
}

export async function smokeWasmPackage(packageDir) {
  const packageJson = readPackageJson(packageDir);
  assertManifest(packageJson);
  assertFilesExist(packageDir);
  await assertEntryPoint(packageDir);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    await smokeWasmPackage(requireArg(process.argv.slice(2)));
    console.log("@vizejs/wasm package smoke passed");
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
