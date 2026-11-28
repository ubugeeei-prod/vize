import { registerHooks } from "node:module";
// The plugin reads the native module shape and has no JS parser to fall back
// on: resolving `oxc-parser` from anything this probe loads is a failure.
let oxcLoads = 0;
registerHooks({
  resolve(specifier, context, nextResolve) {
    if (specifier === "oxc-parser" || specifier.startsWith("oxc-parser/")) {
      oxcLoads++;
      throw new Error(`[probe] ${context.parentURL ?? "entry"} loads ${specifier}`);
    }
    return nextResolve(specifier, context);
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
  // JSX stays in the module for the bundler to lower; the shape is still native.
  jsxModule:
    '<script setup lang="tsx">const R = () => <b>{1 as number}</b></script><template><p :class="$style.a"><R /></p></template><style module>.a{color:red}</style>',
};
const out: Record<string, { oxcLoads: number; shaped: boolean; code: string }> = {};
for (const [name, source] of Object.entries(cases)) {
  for (const ssr of [false, true]) {
    if (ssr && name === "customElement") continue;
    const file = "/src/" + name + ".vue";
    const compiled = compileFile(file, new Map(), { sourceMap: false, ssr, vapor: false }, source);
    const code = generateOutput(compiled, {
      isProduction: false,
      isDev: false,
      ssr,
      filePath: file,
      customElement: name === "customElement",
    });
    out[name + (ssr ? "/ssr" : "")] = { oxcLoads, shaped: !!compiled.moduleShape, code };
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
process.stdout.write(JSON.stringify({ out, noDefault: { oxcLoads, code: noDefault } }));
