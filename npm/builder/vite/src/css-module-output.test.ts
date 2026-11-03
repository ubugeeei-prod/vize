// #11526, #11544: real compiler output carries its module shape and style
// bindings attach to the `_sfc_main` the assembly exported, so generating a
// module never parses the emitted code again. The probe
// runs in a child process because `oxc-parser` must be mocked before the
// output helpers load.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";

const url = (path: string) => new URL(path, import.meta.url).href;

const probe = `
import { mock } from "node:test";
const real = await import("oxc-parser");
let parses = 0;
mock.module("oxc-parser", {
  namedExports: { ...real, parseSync: (...args) => (parses++, real.parseSync(...args)) },
});
const { compileFile } = await import(${JSON.stringify(url("./compiler.ts"))});
const { generateOutput } = await import(${JSON.stringify(url("./utils/index.ts"))});
const cases = {
  setupModule: '<script setup>const n = 1</script><template><p :class="$style.a">{{ n }}</p></template><style module>.a{color:red}</style>',
  optionsModule: '<script>export default { name: "Opt" }</script><template><p :class="classes.a" /></template><style module="classes">.a{color:red}</style>',
  templateModule: '<template><p :class="$style.a" /></template><style module>.a{color:red}</style>',
  customElement: '<script setup>const n = 1</script><template><p>{{ n }}</p></template><style>.a{color:red}</style>',
};
const out = {};
for (const [name, source] of Object.entries(cases)) {
  for (const ssr of [false, true]) {
    if (ssr && name === "customElement") continue;
    const file = "/src/" + name + ".vue";
    const compiled = compileFile(file, new Map(), { sourceMap: false, ssr, vapor: false }, source);
    parses = 0;
    const code = generateOutput(compiled, {
      isProduction: false, isDev: false, ssr, filePath: file, customElement: name === "customElement",
    });
    out[name + (ssr ? "/ssr" : "")] = { parses, code };
  }
}
// A module that exports no \`_sfc_main\` has nothing to attach bindings to.
const shape = { hasDefaultExport: false, hasSfcMainDefined: false, hasNamedRenderExport: false, hasNamedSsrRenderExport: false, defaultExportKeywordEnd: null, defaultExportStart: null, defaultExportEnd: null, defaultExportIsSfcMain: false };
parses = 0;
const noDefault = generateOutput(
  { code: "export const x = 1;\\n", moduleShape: shape, scopeId: "nodefault", hasScoped: false, styles: [{ content: ".a{}", lang: "css", scoped: false, module: true, index: 0 }] },
  { isProduction: false, isDev: false, filePath: "/src/NoDefault.vue" },
);
process.stdout.write(JSON.stringify({ out, noDefault: { parses, code: noDefault } }));
`;

const child = spawnSync(
  process.execPath,
  ["--experimental-test-module-mocks", "--no-warnings", "--input-type=module", "-e", probe],
  { cwd: new URL("..", import.meta.url), encoding: "utf8" },
);
assert.equal(child.status, 0, child.stderr);
type Probe = { parses: number; code: string };
const { out: results, noDefault }: { out: Record<string, Probe>; noDefault: Probe } = JSON.parse(
  child.stdout,
);

for (const [name, { parses, code }] of Object.entries(results)) {
  assert.equal(parses, 0, `${name}: generating the module must not re-parse it`);
  const exported = code.indexOf("export default _sfc_main");
  assert.ok(exported >= 0, `${name}: the component stays the default export`);
  const binding = name.startsWith("customElement")
    ? /_sfc_main\.styles = \[_style_0\];/
    : name.startsWith("optionsModule")
      ? /_sfc_main\.__cssModules\["classes"\] = classes;/
      : /_sfc_main\.__cssModules\["\$style"\] = \$style;/;
  assert.match(code, binding, `${name}: the style binding is attached`);
}
assert.equal(Object.keys(results).length, 7);
// Negative control: nothing is attached to a module that exports no `_sfc_main`.
assert.equal(noDefault.parses, 0);
assert.doesNotMatch(noDefault.code, /_sfc_main/, "no binding without an exported _sfc_main");
assert.match(noDefault.code, /import \$style from/, "its style import is still emitted");

console.log("✅ vite-plugin-vize CSS module output tests passed!");
