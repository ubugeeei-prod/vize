import { mock } from "node:test";
const real = await import("oxc-parser");
let parses = 0;
mock.module("oxc-parser", {
  namedExports: {
    ...real,
    parseSync: (...args: Parameters<typeof real.parseSync>) => {
      parses++;
      return real.parseSync(...args);
    },
  },
});
const { compileFile } = await import("../src/compiler.ts");
const { generateOutput } = await import("../src/utils/index.ts");
const cases = {
  setupModule:
    '<script setup>const n = 1</script><template><p :class="$style.a">{{ n }}</p></template><style module>.a{color:red}</style>',
  optionsModule:
    '<script>export default { name: "Opt" }</script><template><p :class="classes.a" /></template><style module="classes">.a{color:red}</style>',
  templateModule: '<template><p :class="$style.a" /></template><style module>.a{color:red}</style>',
  customElement:
    "<script setup>const n = 1</script><template><p>{{ n }}</p></template><style>.a{color:red}</style>",
};
const out: Record<string, { parses: number; code: string }> = {};
for (const [name, source] of Object.entries(cases)) {
  for (const ssr of [false, true]) {
    if (ssr && name === "customElement") continue;
    const file = "/src/" + name + ".vue";
    const compiled = compileFile(file, new Map(), { sourceMap: false, ssr, vapor: false }, source);
    parses = 0;
    const code = generateOutput(compiled, {
      isProduction: false,
      isDev: false,
      ssr,
      filePath: file,
      customElement: name === "customElement",
    });
    out[name + (ssr ? "/ssr" : "")] = { parses, code };
  }
}
// A module that exports no `_sfc_main` has nothing to attach bindings to.
const shape = {
  hasDefaultExport: false,
  hasSfcMainDefined: false,
  hasNamedRenderExport: false,
  hasNamedSsrRenderExport: false,
  defaultExportKeywordEnd: null,
  defaultExportStart: null,
  defaultExportEnd: null,
  defaultExportIsSfcMain: false,
};
parses = 0;
const noDefault = generateOutput(
  {
    code: "export const x = 1;\n",
    moduleShape: shape,
    scopeId: "nodefault",
    hasScoped: false,
    styles: [{ content: ".a{}", lang: "css", scoped: false, module: true, index: 0 }],
  },
  { isProduction: false, isDev: false, filePath: "/src/NoDefault.vue" },
);
process.stdout.write(JSON.stringify({ out, noDefault: { parses, code: noDefault } }));
