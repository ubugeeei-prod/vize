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
const { compileFile } = await import("../src/shared/compiler.ts");
const { generateOutputWithMap } = await import("../src/shared/output.ts");
const cases = {
  setup:
    '<script setup>const n = "日本 🐱"</script><template><p :class="$style.a">{{ n }}</p></template><style scoped>p{}</style><style module>.a{}</style>',
  setupTs:
    '<script setup lang="ts">const n: string = "é"</script><template><img src="./logo.png"><p>{{ n }}</p></template><style scoped>p{}</style>',
  options:
    '<script>export default { name: "Opt" }</script><template><p :class="theme.a" /></template><style scoped>p{}</style><style module="theme">.a{}</style>',
  optionsOnly:
    '<script>export default { name: "NoTpl", render: () => null }</script><style scoped>p{}</style>',
  template: "<template><p>🐱</p></template><style scoped>p{}</style><i18n>{}</i18n>",
  reexport: '<script>export { default } from "./x.js"</script><style scoped>p{}</style>',
};
const out: Record<string, { parses: number; code: string }> = {};
for (const [name, source] of Object.entries(cases)) {
  for (const ssr of [false, true]) {
    for (const isCustomElement of ssr || name !== "template" ? [false] : [false, true]) {
      const file = "/src/" + name + (isCustomElement ? ".ce" : "") + ".vue";
      const compiled = compileFile(file, source, { ssr, isCustomElement, sourceMap: true });
      parses = 0;
      const { code } = generateOutputWithMap(compiled, {
        requestPath: "./" + name + ".vue",
        filePath: file,
        rootContext: "/src",
        isProduction: false,
      });
      out[file + (ssr ? "?ssr" : "")] = { parses, code };
    }
  }
}
process.stdout.write(JSON.stringify(out));
