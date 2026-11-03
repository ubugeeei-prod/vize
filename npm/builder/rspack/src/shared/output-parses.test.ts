// #11583: output assembly reads the native `moduleShape` instead of parsing the
// emitted module again. A child process mocks `oxc-parser` before the output
// helpers load and counts parses over real native compiles.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";

const url = (path: string) => new URL(path, import.meta.url).href;

const probe = `
import { mock } from "node:test";
const real = await import("oxc-parser");
let parses = 0;
mock.module("oxc-parser", {
  namedExports: { ...real, parseSync: (...args) => (parses++, real.parseSync(...args)) },
});
const { compileFile } = await import(${JSON.stringify(url("./compiler.ts"))});
const { generateOutputWithMap } = await import(${JSON.stringify(url("./output.ts"))});
const cases = {
  setup: '<script setup>const n = "日本 🐱"</script><template><p :class="$style.a">{{ n }}</p></template><style scoped>p{}</style><style module>.a{}</style>',
  setupTs: '<script setup lang="ts">const n: string = "é"</script><template><img src="./logo.png"><p>{{ n }}</p></template><style scoped>p{}</style>',
  options: '<script>export default { name: "Opt" }</script><template><p :class="theme.a" /></template><style scoped>p{}</style><style module="theme">.a{}</style>',
  optionsOnly: '<script>export default { name: "NoTpl", render: () => null }</script><style scoped>p{}</style>',
  template: '<template><p>🐱</p></template><style scoped>p{}</style><i18n>{}</i18n>',
  reexport: '<script>export { default } from "./x.js"</script><style scoped>p{}</style>',
};
const out = {};
for (const [name, source] of Object.entries(cases)) {
  for (const ssr of [false, true]) {
    for (const isCustomElement of ssr || name !== "template" ? [false] : [false, true]) {
      const file = "/src/" + name + (isCustomElement ? ".ce" : "") + ".vue";
      const compiled = compileFile(file, source, { ssr, isCustomElement, sourceMap: true });
      parses = 0;
      const { code } = generateOutputWithMap(compiled, {
        requestPath: "./" + name + ".vue", filePath: file, rootContext: "/src", isProduction: false,
      });
      out[file + (ssr ? "?ssr" : "")] = { parses, code };
    }
  }
}
process.stdout.write(JSON.stringify(out));
`;

void test("assembling real native output never parses the emitted module", () => {
  const child = spawnSync(
    process.execPath,
    ["--experimental-test-module-mocks", "--no-warnings", "--input-type=module", "-e", probe],
    { cwd: new URL("../..", import.meta.url), encoding: "utf8" },
  );
  assert.equal(child.status, 0, child.stderr);
  const results: Record<string, { parses: number; code: string }> = JSON.parse(child.stdout);
  assert.equal(Object.keys(results).length, 13);
  for (const [name, { parses, code }] of Object.entries(results)) {
    assert.equal(parses, 0, `${name}: assembling the module must not re-parse it`);
    if (name.startsWith("/src/reexport")) {
      // Negative control: no `_sfc_main` is exported, so nothing is attached.
      assert.doesNotMatch(code, /_sfc_main/, name);
      assert.match(code, /export \{ default \} from "\.\/x\.js"/, name);
      continue;
    }
    const exported = code.indexOf("export default _sfc_main");
    assert.ok(exported >= 0, `${name}: the component stays the default export`);
    assert.match(code, /_sfc_main\.__scopeId = "data-v-[0-9a-f]{8}";|__scopeId: "data-v-/, name);
    assert.ok(code.indexOf("_sfc_main.__file") > exported, `${name}: appended after the export`);
  }
  assert.match(results["/src/template.ce.vue"].code, /_sfc_main\.styles = \[_style_0\];/);
  assert.match(results["/src/template.vue"].code, /block0\(_sfc_main\)/);
  assert.match(results["/src/setup.vue"].code, /_sfc_main\.__cssModules\["\$style"\]/);
  assert.match(results["/src/setupTs.vue"].code, /import _imports_0 from "\.\/logo\.png";/);
});
