import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test, after } from "node:test";
import { fileURLToPath } from "node:url";
import { ESLint, Linter, SourceCode } from "eslint";

const require = createRequire(import.meta.url);
const native = require("@vizejs/native");
const original = native.lintPatinaSfc;
const calls = [];
const packets = [];
const profiles = [];
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
native.lintPatinaSfc = (source, options) => {
  const start = process.hrtime.bigint();
  const result = original(source, options);
  const elapsed = process.hrtime.bigint() - start;
  calls.push({
    sourceSha256: hash(source),
    filename: options.filename,
    enabledRules: options.enabledRules,
    typeAware: options.typeAware,
    nanoseconds: String(elapsed),
    result,
  });
  return result;
};
const { default: plugin, ruleConfigs, createVizeConfig } = await import("../dist/index.mjs");
const { default: bridge } = await import("oxlint-plugin-vize/eslint");
const packageJson = JSON.parse(fs.readFileSync(new URL("../package.json", import.meta.url)));
const linter = new Linter();
const config = (rules) => [
  {
    ...plugin.configs.essential,
    settings: { vize: { helpLevel: "none", preset: "incremental" } },
    rules,
  },
];
function verify(source, rules, filename = "Unsaved.vue") {
  const result = linter.verify(source, config(rules), { filename });
  packets.push({ source, rules, filename, result });
  return result;
}
function parsedSource(source) {
  const parsed = plugin.configs.essential.languageOptions.parser.parseForESLint(source, {
    ...plugin.configs.essential.languageOptions.parserOptions,
    range: true,
    loc: true,
    tokens: true,
    comment: true,
  });
  return new SourceCode({ ...parsed, text: source, parserServices: parsed.services });
}
const alt = { "vize/a11y/alt-text": "error" };
const source = '<template>😀<img src="x"></template>\r\n';

void test("the standalone public plugin shares rules and complete flat presets", () => {
  assert.equal(plugin.meta.name, "eslint-plugin-vize");
  assert.equal(plugin.meta.version, packageJson.version);
  assert.equal(plugin.rules, bridge.rules);
  for (const [name, rules] of Object.entries(ruleConfigs)) {
    assert.deepEqual(plugin.configs[name].rules, rules);
    assert.equal(plugin.configs[name].plugins.vize, plugin);
    assert.deepEqual(plugin.configs[name].files, ["**/*.vue"]);
  }
  assert.deepEqual(createVizeConfig().rules, ruleConfigs.recommended);
  for (const rule of Object.values(plugin.rules)) {
    assert.equal(typeof rule.create, "function");
    assert.ok(rule.meta.docs.description);
    assert.ok(Array.isArray(rule.meta.schema));
  }
});

void test("actual ESLint reports unsaved Unicode/CRLF source and native UTF-16 fixes", () => {
  const count = calls.length;
  const messages = verify(source, alt);
  assert.equal(calls.length - count, 1);
  assert.equal(messages.length, 1);
  assert.deepEqual(
    [messages[0].ruleId, messages[0].severity, messages[0].line, messages[0].column],
    ["vize/a11y/alt-text", 2, 1, 13],
  );
  assert.deepEqual(verify(source.replace('src="x"', 'src="x" alt="Logo"'), alt), []);
  const before = '<template>😀<div  id="x"></div></template>\r\n';
  const fixed = linter.verifyAndFix(before, config({ "vize/vue/no-multi-spaces": "error" }), {
    filename: "Fix.vue",
  });
  packets.push({ source: before, fixed });
  assert.equal(fixed.fixed, true);
  assert.equal(fixed.output, before.replace("div  id", "div id"));
  assert.deepEqual(fixed.messages, []);
});

void test("scriptless, JS, TS, TSX and mixed script blocks retain the complete source", () => {
  for (const script of [
    "",
    '<script>export default { name: "Example" };</script>',
    '<script setup lang="ts">const n: number = 1;</script>',
    '<script setup lang="tsx">const node = <span />;</script>',
    '<script>export default {};</script><script setup lang="ts">const n: number = 1;</script>',
  ]) {
    const value = `${source}${script}\r\n<style scoped>img { color: red; }</style>`;
    const count = calls.length;
    const messages = verify(value, alt);
    assert.equal(messages.length, 1);
    assert.equal(messages[0].ruleId, "vize/a11y/alt-text");
    assert.equal(calls.length - count, 1);
    assert.equal(calls.at(-1).sourceSha256, hash(value));
  }
});

void test("selected rule options, severity, off, parse errors and reused ASTs are authoritative", () => {
  const value =
    '<template><my-component /></template><script setup>import MyComponent from "./MyComponent.vue";</script>';
  const casing = (option) => ({ "vize/vue/component-name-in-template-casing": ["warn", option] });
  const code = parsedSource(value);
  for (const [option, expected] of [
    ["PascalCase", 1],
    ["kebab-case", 0],
    ["PascalCase", 1],
  ]) {
    const result = linter.verify(code, config(casing(option)), { filename: "Casing.vue" });
    packets.push({ source: value, option, result });
    assert.equal(result.length, expected);
    if (expected) assert.equal(result[0].severity, 1);
  }
  const count = calls.length;
  assert.deepEqual(verify(source, { "vize/a11y/alt-text": "off" }), []);
  assert.equal(calls.length, count);
  assert.throws(() => verify(value, casing("unknown")), /invalid|allowed|match/iu);
  const refused = verify("<script setup>const = ;</script>", alt);
  assert.ok(refused.some((diagnostic) => diagnostic.fatal));
  const ignored = linter.verify(source, config(alt), { filename: "Unsupported.html" });
  assert.ok(ignored.some((diagnostic) => /configuration/i.test(diagnostic.message)));
  assert.equal(calls.length, count);
});

void test("real multi-file ESLint runs keep file-local participation and edits separate", async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-eslint-files-"));
  try {
    fs.writeFileSync(path.join(directory, "One.vue"), source);
    fs.writeFileSync(path.join(directory, "Two.vue"), '<template><img alt="OK"></template>\n');
    const eslint = new ESLint({
      cwd: directory,
      overrideConfigFile: true,
      overrideConfig: config(alt),
      cache: true,
      cacheLocation: path.join(directory, ".eslintcache"),
    });
    const count = calls.length;
    const results = await eslint.lintFiles(["One.vue", "Two.vue"]);
    packets.push({ multipleFiles: results });
    assert.equal(calls.length - count, 2);
    assert.deepEqual(
      results.map((result) => result.errorCount),
      [1, 0],
    );
    const cachedCount = calls.length;
    const cached = await eslint.lintFiles(["One.vue", "Two.vue"]);
    assert.deepEqual(
      cached.map((result) => result.messages),
      results.map((result) => result.messages),
    );
    assert.equal(
      calls.length,
      cachedCount,
      "unchanged cached files must not call native lint again",
    );
    const selectedConfig = await eslint.calculateConfigForFile("One.vue");
    assert.equal(selectedConfig.plugins.vize.meta.name, "eslint-plugin-vize");
    assert.doesNotThrow(() => JSON.stringify(selectedConfig));
    const unsaved = await eslint.lintText('<template><img alt="Unsaved"></template>\n', {
      filePath: "One.vue",
    });
    assert.equal(unsaved[0].errorCount, 0);
    assert.equal(fs.readFileSync(path.join(directory, "One.vue"), "utf8"), source);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

void test("explicit type-aware selection uses the actual runtime and exposes missing providers", () => {
  const value =
    '<script setup lang="ts">async function load(): Promise<number> { return 1; }\nload();</script>\n';
  const rules = { "vize/type/no-floating-promises": "error" };
  const selected = config(rules);
  const runtime = path.join(
    path.dirname(
      require.resolve(`@typescript/typescript-${process.platform}-${process.arch}/package.json`),
    ),
    "lib",
    process.platform === "win32" ? "tsc.exe" : "tsc",
  );
  selected[0].settings.vize.corsaPath = runtime;
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-eslint-types-"));
  const typedLinter = new Linter({ cwd: directory });
  const filename = path.join(directory, "Typed.vue");
  const diskSource = value.replace("\nload();", "\nawait load();");
  const tsconfig = { compilerOptions: { strict: true, target: "ESNext", skipLibCheck: true } };
  try {
    fs.writeFileSync(filename, diskSource);
    fs.writeFileSync(path.join(directory, "tsconfig.json"), JSON.stringify(tsconfig));
    fs.symlinkSync(
      fileURLToPath(new URL("../../../node_modules/", import.meta.url)),
      path.join(directory, "node_modules"),
      "dir",
    );
    const count = calls.length;
    const result = typedLinter.verify(value, selected, { filename });
    packets.push({
      source: value,
      diskSource,
      tsconfig,
      runtime,
      runtimeSha256: hash(fs.readFileSync(runtime)),
      result,
    });
    assert.equal(calls.length - count, 1);
    assert.equal(calls.at(-1).typeAware, true);
    assert.ok(result.some((message) => message.ruleId === "vize/type/no-floating-promises"));
    assert.ok(!result.some((message) => /runtime|executable|failed/i.test(message.message)));
    assert.deepEqual(typedLinter.verify(diskSource, selected, { filename }), []);
    selected[0].settings.vize.corsaPath = path.join(directory, "missing-typescript-runtime");
    const reusable = parsedSource(value);
    const missingCount = calls.length;
    const missing = typedLinter.verify(reusable, selected, { filename });
    packets.push({ source: value, missingRuntime: missing });
    assert.equal(calls.length - missingCount, 1);
    assert.equal(missing.length, 1);
    assert.ok(
      missing.some((message) => /Configured Corsa executable does not exist/.test(message.message)),
    );
    selected[0].rules["vize/type/strict-boolean-expressions"] = "error";
    const twoCount = calls.length;
    const twoRules = typedLinter.verify(reusable, selected, { filename });
    packets.push({ source: value, missingRuntimeWithTwoSelectedRules: twoRules });
    assert.equal(calls.length - twoCount, 1);
    assert.equal(twoRules.length, 1);
    assert.equal(twoRules[0].message, missing[0].message);
    const another = path.join(directory, "Another.vue");
    fs.writeFileSync(another, diskSource);
    const anotherMessages = typedLinter.verify(reusable, selected, { filename: another });
    packets.push({ source: value, missingRuntimeOtherFile: anotherMessages });
    assert.equal(anotherMessages.length, 1);
    assert.match(anotherMessages[0].message, /Configured Corsa executable does not exist/);
    const off = config({
      "vize/type/no-floating-promises": "off",
      "vize/type/strict-boolean-expressions": "off",
    });
    off[0].settings.vize.corsaPath = selected[0].settings.vize.corsaPath;
    const offCount = calls.length;
    assert.deepEqual(typedLinter.verify(reusable, off, { filename }), []);
    assert.equal(calls.length, offCount);
    const unrelated = config(alt);
    unrelated[0].settings.vize.corsaPath = selected[0].settings.vize.corsaPath;
    const unrelatedSource = "<template><img></template>" + value;
    const unrelatedMessages = typedLinter.verify(unrelatedSource, unrelated, { filename });
    packets.push({ source: unrelatedSource, zeroTypeRules: unrelatedMessages });
    assert.deepEqual(
      unrelatedMessages.map((message) => message.ruleId),
      ["vize/a11y/alt-text"],
    );
    selected[0].settings.vize.corsaPath = runtime;
    const repaired = typedLinter.verify(reusable, selected, { filename });
    packets.push({ source: value, repairedRuntime: repaired });
    assert.deepEqual(
      repaired.map((message) => message.ruleId),
      ["vize/type/no-floating-promises"],
    );
    assert.equal(fs.readFileSync(filename, "utf8"), diskSource);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

void test("real native call counts stay one per pass as enabled rules grow", () => {
  const available = Object.keys(ruleConfigs.all);
  for (const size of new Set([1, 8, 64, available.length])) {
    const rules = Object.fromEntries(available.slice(0, size).map((name) => [name, "error"]));
    const start = calls.length;
    const rounds = 12;
    const durations = [];
    for (let round = 0; round < rounds; round++) {
      const before = process.hrtime.bigint();
      const messages = linter.verify(source, config(rules), { filename: "Profile.vue" });
      durations.push(String(process.hrtime.bigint() - before));
      assert.ok(!messages.some((message) => message.fatal));
    }
    assert.equal(calls.length - start, rounds);
    profiles.push({
      enabledRuleCount: Math.min(size, available.length),
      rounds,
      actualNativeCalls: calls.length - start,
      wholeCallNanoseconds: durations,
      nativeCallNanoseconds: calls.slice(start).map((call) => call.nanoseconds),
      scope:
        "Node whole ESLint call and real NAPI call observations; no Rust phase or cross-tool speed claim",
    });
  }
});

after(() => {
  native.lintPatinaSfc = original;
  const images = Object.values(require.cache)
    .filter((entry) => entry?.filename.endsWith(".node") && entry.exports === native)
    .map((entry) => ({
      path: fs.realpathSync(entry.filename),
      sha256: hash(fs.readFileSync(entry.filename)),
    }));
  assert.equal(images.length, 1, "one actual native provider image is required; no fallback");
  assert.ok(
    images[0].path.startsWith(
      fs.realpathSync(path.dirname(require.resolve("@vizejs/native"))) + path.sep,
    ),
  );
  assert.match(native.getNativeBuildToken(), /^[0-9a-f]{32}$/);
  const output = process.env.VIZE_ESLINT_EVIDENCE;
  const evidence = {
    eslintVersion: Linter.version,
    packageVersion: packageJson.version,
    providers: Object.fromEntries(
      ["eslint", "vue-eslint-parser", "@typescript-eslint/parser"].map((name) => {
        const entry = fs.realpathSync(require.resolve(name));
        const packagePath = fs.realpathSync(require.resolve(`${name}/package.json`));
        const metadata = JSON.parse(fs.readFileSync(packagePath, "utf8"));
        return [
          name,
          {
            entry,
            entrySha256: hash(fs.readFileSync(entry)),
            version: metadata.version,
            packagePath,
            packageSha256: hash(fs.readFileSync(packagePath)),
          },
        ];
      }),
    ),
    nativeImages: images,
    nativeBuildToken: native.getNativeBuildToken(),
    nativeMetadata: native.getPatinaRules(),
    packets,
    calls,
    profiles,
  };
  if (output) {
    fs.mkdirSync(output, { recursive: true });
    fs.writeFileSync(path.join(output, "consumer.json"), JSON.stringify(evidence, null, 2) + "\n");
  }
  console.log(JSON.stringify({ eslint: Linter.version, images, profiles }));
});
