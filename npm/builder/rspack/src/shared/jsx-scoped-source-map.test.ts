import assert from "node:assert/strict";
import { test } from "node:test";
import { compileJsx } from "@vizejs/native";
import { decodedMappings, originalPositionFor, TraceMap } from "@jridgewell/trace-mapping";
import { compileJsxModule as compileVite } from "../../../vite/src/compiler.ts";
import { compileJsxModule as compileUnplugin } from "../../../unplugin/src/compiler.ts";
import { normalizeOptions } from "../../../unplugin/src/unplugin.ts";
import { prependMappedJsxCode } from "../../../shared/source-map.ts";
import { compileJsxModule as compileRspack } from "./compiler.ts";

const adapters = [
  { name: "vite", compile: compileVite },
  { name: "rspack", compile: compileRspack },
  {
    name: "unplugin",
    compile: (filename: string, source: string, options: { sourceMap: boolean }) =>
      compileUnplugin(filename, source, normalizeOptions({ ...options, jsxMode: "vdom" })),
  },
];

function sourceFor(lang: "tsx" | "jsx") {
  const parameter = lang === "tsx" ? "props: { label: string }" : "props";
  return [
    'import { ref } from "vue";',
    'const emoji = "🐱"; export const suffix = emoji + "!";',
    "const message = ref(suffix);",
    "export const Extra = () => <span>{message.value}</span>;",
    `const App = (${parameter}) => (`,
    '  <section class="box">',
    '    <style scoped>{`.box { color: red; content: "🧭" }`}</style>',
    "    <p>{props.label}{message.value}</p>",
    "  </section>",
    ");",
    "export default App;",
    "",
  ].join("\n");
}

for (const lang of ["tsx", "jsx"] as const) {
  void test(`${lang} scoped CSS retains full native module maps in every adapter`, () => {
    const source = sourceFor(lang);
    const filename = `/src/Scoped.${lang}`;
    const native = compileJsx(source, { filename, lang, jsxMode: "vdom", sourceMap: true });
    assert.deepEqual(native.errors, []);
    assert.deepEqual(native.warnings, []);
    assert.equal(typeof native.map, "string");
    const original = new TraceMap(native.map!);
    assert.deepEqual(original.sourcesContent, [source]);
    const nativeMap = JSON.parse(native.map!);
    const { mappings: _nativeMappings, ...nativeFields } = nativeMap;
    const nativeLines = decodedMappings(original);
    const css = native.scopedStyles.map((style) => style.css).join("\n");
    assert.notEqual(css, "");

    for (const adapter of adapters) {
      const compiled = adapter.compile(filename, source, { sourceMap: true });
      assert.deepEqual(compiled.warnings, [], adapter.name);
      const prefixLength = compiled.code.length - native.code.length;
      assert.ok(prefixLength > 0, adapter.name);
      const prefix = compiled.code.slice(0, prefixLength);
      assert.equal(compiled.code.slice(prefixLength), native.code, adapter.name);
      assert.equal(prefix.endsWith("\n"), true, adapter.name);
      assert.equal(prefix.split("\n")[1], `export const __vize_css__ = ${JSON.stringify(css)};`);
      assert.equal(typeof compiled.map, "string", adapter.name);
      const { mappings: _mappings, ...fields } = JSON.parse(compiled.map!);
      assert.deepEqual(fields, nativeFields, adapter.name);
      const mapped = new TraceMap(compiled.map!);
      const lines = decodedMappings(mapped);
      const injectedLines = prefix.split("\n").length - 1;
      assert.deepEqual(
        lines.slice(0, injectedLines),
        Array.from({ length: injectedLines }, () => []),
      );
      assert.deepEqual(lines.slice(injectedLines), nativeLines, adapter.name);

      for (const marker of ["export const suffix", "export default App"]) {
        assert.deepEqual(
          trace(compiled.code, mapped, marker),
          {
            source: original.sources[0],
            ...position(source, marker),
            name: null,
          },
          adapter.name,
        );
      }
      assert.deepEqual(
        originalPositionFor(mapped, position(compiled.code, "export const __vize_css__")),
        {
          source: null,
          line: null,
          column: null,
          name: null,
        },
      );
      assert.equal(adapter.compile(filename, source, { sourceMap: false }).map, null);
    }

    const ssr = compileVite(filename, source, { sourceMap: true, ssr: true });
    assert.deepEqual(ssr, { code: native.code, map: native.map, warnings: native.warnings });
  });
}

function position(text: string, marker: string) {
  const offset = text.indexOf(marker);
  assert.notEqual(offset, -1, marker);
  const prefix = text.slice(0, offset);
  return { line: prefix.split("\n").length, column: offset - prefix.lastIndexOf("\n") - 1 };
}

function trace(code: string, map: TraceMap, marker: string) {
  return originalPositionFor(map, position(code, marker));
}

void test("unusable or partial-line prefix maps are omitted without changing emitted code", () => {
  for (const map of [undefined, null, "{", "null", '{"version":2}']) {
    assert.deepEqual(prependMappedJsxCode("module", map, "🐱\n"), {
      code: "🐱\nmodule",
      map: null,
    });
  }
  const map = JSON.stringify({ version: 3, sources: ["source.tsx"], names: [], mappings: "AAAA" });
  assert.deepEqual(prependMappedJsxCode("module", map, "🐱"), { code: "🐱module", map: null });
  assert.deepEqual(prependMappedJsxCode("module", map, ""), { code: "module", map });
});
