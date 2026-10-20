import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const root = new URL("../../", import.meta.url);
const read = (name) => readFileSync(new URL(name, root), "utf8");
const hash = (source) => createHash("sha256").update(source).digest("hex");
const archive = "tests/_fixtures/differential/config/vue-namespace/source-custody/";

void test("the original archives remain pinned while active package cases follow the fluent API", () => {
  const config = read(archive + "original-vite-plus-config.test.ts.fixture");
  assert.equal(hash(config), "82570d354aa7cfb9300bca05be49e7bd451694beea019b635cac1ccb7c8056a4");
  const currentConfig = read("npm/builder/vite/src/vite-plus/config.test.ts");
  for (const title of [
    "defineConfig composes async Vite+ and native config without mutating either input",
    "tool ownership and conflict defaults can be disabled independently",
    "defineConfig installs its compiler and preserves caller plugins",
    "generated tasks preserve existing scripts and tasks, and reject ambiguous renames",
    "config extends reports the drop-in migration instead of composing another config",
    "legacy vize config keys report an explicit Vue migration",
  ])
    assert.ok(currentConfig.includes(`test("${title}"`), title);
  const packed = read(archive + "original-vite-plus-fast-path.test.ts.fixture");
  assert.equal(hash(packed), "2e50572dce23541ce29d219e6a8eefe46c440d80acae92f0ab6c1f0664082b6b");
  const addedImport =
    'import { checkCanonicalVueConsumer } from "./support/vite-vue-packed-consumer.ts";\n';
  const addedCall =
    "    checkCanonicalVueConsumer(\n" +
    "      directory,\n" +
    '      path.join(path.dirname(require.resolve("typescript/package.json")), "bin/tsc"),\n' +
    '      path.dirname(require.resolve("vite/package.json")),\n' +
    "    );\n";
  const current = read("tests/tooling/vite-plus-fast-path.test.ts");
  assert.equal(current.split(addedImport).length, 2);
  assert.equal(current.split(addedCall).length, 2);
  const migratedPacked = packed.replaceAll("vize: {", "vue: {");
  assert.equal(current.replace(addedImport, "").replace(addedCall, ""), migratedPacked);

  const dialect = read(archive + "original-vue-version-index.test.ts.fixture");
  assert.equal(hash(dialect), "6e97eb6a6af314856b7cd16395544c34a839eb7eac9bd651d110fafe5580aeaf");
  const addedDialect =
    "  assert.equal(\n" +
    "    isLegacyVueCompatibilityMode({ vueVersion: 0.1 }),\n" +
    "    true,\n" +
    '    "vueVersion: 0.10 should enable legacy Vue compatibility mode",\n' +
    "  );\n\n";
  // #9730 added the host plugin-vue removal case.
  const addedHostImport = "  removeHostVueSfcCompilers,\n";
  const addedHostCase =
    "{\n" +
    "  // `@void/vue`'s `voidVue()` bundles @vitejs/plugin-vue. Both compiling the\n" +
    "  // same SFC made plugin-vue read Vize's style virtual IDs from disk (ENOENT).\n" +
    '  const hostVue = { name: "vite:vue", transform() {} };\n' +
    '  const vizeShim = { name: "vite:vue" };\n' +
    '  const pages = { name: "void-vue:pages", transform() {} };\n' +
    "  const plugins = [hostVue, vizeShim, pages];\n\n" +
    "  assert.equal(removeHostVueSfcCompilers([...plugins], { exclude: /legacy/ }), 0);\n" +
    "  assert.equal(removeHostVueSfcCompilers(plugins, {}), 1);\n" +
    '  assert.deepEqual(plugins, [vizeShim, pages], "only the compiling plugin-vue is removed");\n' +
    "}\n\n";
  const liveDialect = read("npm/builder/vite/src/plugin/index.test.ts");
  for (const added of [addedDialect, addedHostImport, addedHostCase])
    assert.equal(liveDialect.split(added).length, 2);
  assert.equal(
    liveDialect.replace(addedDialect, "").replace(addedHostImport, "").replace(addedHostCase, ""),
    dialect,
  );
});

void test("the shared Vue configuration guide documents the canonical entry points", () => {
  for (const source of [
    read("docs/content/guide/vue-configuration.md"),
    read("docs/content/ja/guide/vue-configuration.md"),
  ]) {
    assert.match(source, /`basePath`.*`files`.*`ignores`.*`linter`/s);
    assert.match(source, /defineConfig\(vpConfig\)\.withVue\(vueConfig\)/);
  }
});
