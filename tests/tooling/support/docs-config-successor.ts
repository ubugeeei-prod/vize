/** Apply the public config migration to an archived, complete documentation packet.
 * Historical fixtures remain byte-for-byte authorities; only current examples use
 * this adapter. It deliberately changes config ownership, never rule/example data.
 */
export function currentConfigExample(original: string): string {
  let source = original
    .replaceAll('from "vize"', 'from "@vizejs/vite-plugin/vite-plus"')
    .replaceAll(
      'import { defineConfig } from "@vizejs/vite-plugin"',
      'import { defineConfig } from "@vizejs/vite-plugin/vite-plus"',
    )
    .replace(
      /(export default defineConfig\(\{\n)  lint: \{\n    vize: \{/g,
      "$1  vue: {\n    linter: {",
    )
    .replace(/(export default defineConfig\(\{\n)  vize: \{/g, "$1  vue: {")
    .replaceAll("// vize.config.ts", "// vite.config.ts")
    .replaceAll(
      '// vize.config: { compiler: { jsxMode: "vapor" } }',
      '// vite.config.ts: { vue: { compiler: { jsxMode: "vapor" } } }',
    );
  const native =
    /\bexport default defineConfig\(\{\n(?=  (?:compiler|typeChecker|linter|formatter|lsp|musea|vite|dialect|experimentals):)/;
  if (native.test(source)) {
    source = source.replace(native, "export default defineConfig({\n  vue: {\n");
    const start = source.indexOf("  vue: {\n") + "  vue: {\n".length;
    const end = source.lastIndexOf("\n});");
    if (end < start) throw new Error("Unrecognized archived config closure");
    source =
      source.slice(0, start) +
      source.slice(start, end).replace(/^  /gm, "    ") +
      "\n  }," +
      source.slice(end);
  }
  return source;
}

/** Whole rendered Markdown packets use the same finite config-only changes. */
export function currentConfigDocument(original: string): string {
  return original.replace(
    /^(```[^\n]*\n)([\s\S]*?)(^```[^\n]*$)/gm,
    (_, open: string, code: string, close: string) => open + currentConfigExample(code) + close,
  );
}

/** Formatter recipes keep their complete option payload in the fluent API. */
export function currentFormatterExample(original: string): string {
  if (original.startsWith("```json\n")) {
    const body = original.slice("```json\n".length, original.lastIndexOf("\n```"));
    const options = JSON.parse(body);
    if (JSON.stringify(options) !== '{"formatter":{"tabWidth":3}}')
      throw new Error("Unrecognized archived formatter JSON recipe");
    return '```ts\n// vite.config.ts\nimport { defineConfig } from "@vizejs/vite-plugin/vite-plus";\n\nexport default defineConfig({}).withVue({\n  formatter: { tabWidth: 3 },\n});\n```';
  }
  return original
    .replaceAll('from "vize"', 'from "@vizejs/vite-plugin/vite-plus"')
    .replace(
      /export default defineConfig\(\{\n  fmt: \{\n    vize: ([\s\S]*?)\n  \},\n\}\);/g,
      (_, body: string) =>
        "export default defineConfig({}).withVue({\n  formatter: " +
        body.replace(/^  /gm, "") +
        "\n});",
    )
    .replace(
      "export default defineConfig({\n  formatter:",
      "export default defineConfig({}).withVue({\n  formatter:",
    );
}

/** Native compiler-reference packets move to the same fluent project carrier. */
export function currentCompilerReferenceExample(original: string): string {
  if (original.startsWith("// vize.config.ts\n"))
    return original
      .replaceAll("// vize.config.ts", "// vite.config.ts")
      .replace('from "@vizejs/vite-plugin"', 'from "@vizejs/vite-plugin/vite-plus"')
      .replace("export default defineConfig({", "export default defineConfig({}).withVue({");
  if (original === '{\n  "dialect": "petite-vue"\n}')
    return '// vite.config.ts\nimport { defineConfig } from "@vizejs/vite-plugin/vite-plus";\n\nexport default defineConfig({}).withVue({\n  dialect: "petite-vue",\n});';
  return currentConfigExample(original);
}
