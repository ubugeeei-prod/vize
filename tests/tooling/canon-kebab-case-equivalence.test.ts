import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

// The shared preamble's `$KebabCase` ends in one step once the rest of the key
// is lowercase; embedded projections keep the char-by-char form. That form is
// the oracle: both must be `Equal` for literals, template and intrinsic
// patterns, unions, `never`, `any` and every mapped key remap that uses them.
const read = (name: string) =>
  fs.readFileSync(
    new URL(`../../crates/vize_canon/src/virtual_ts/helpers/declarations/${name}`, import.meta.url),
    "utf8",
  );
const embedded = read("prop-keys.d.ts.txt").split("\n");
const shared = read("shared-prop-keys.d.ts.txt").split("\n");
const kebab = (lines: string[]) => lines.find((line) => line.startsWith("type $KebabCase<"))!;

const shapes = [
  '""', '"a"', '"abc"', '"aB"', '"fooBar"', '"FooBar"', '"ABC"', '"onUpdate:modelValue"',
  '"update:model-value"', '"data-fooBar"', '"aria-label"', '"x1Y"', '"ÄbC"', '"émile"',
  '"İstanbul"', '"ΣΑΣ"', '"ß"', '"a😀B"', '"123"', '"a-B-c"', '"--x"', '"onClick"',
  '"aVeryLongCamelCasedPropertyNameWithManyWords"', "`data-${string}`", "`aria${Capitalize<string>}`",
  "`on${Capitalize<string>}`", "`${number}`", "`a${number}B`", "`ab${Lowercase<string>}`",
  "`aB${string}`", "`${string}Foo`", "`${bigint}`", "`x${Uppercase<string>}`", "string", "never",
  "any", '"aB" | "c"', '"x" | "yZ" | `data-${string}`', "Uppercase<string>", "Lowercase<string>",
  "Capitalize<string>", "Uncapitalize<string>",
];

function diagnostics(content: string): string[] {
  const fixture = fileURLToPath(import.meta.url);
  const options: ts.CompilerOptions = {
    strict: true,
    noEmit: true,
    target: ts.ScriptTarget.ESNext,
    lib: ["lib.esnext.d.ts", "lib.dom.d.ts"],
    types: [],
  };
  const host = ts.createCompilerHost(options);
  const original = host.readFile.bind(host);
  host.readFile = (file) => (file === fixture ? content : original(file));
  return ts.getPreEmitDiagnostics(ts.createProgram([fixture], options, host)).map((diagnostic) => {
    const where = diagnostic.file?.getLineAndCharacterOfPosition(diagnostic.start ?? 0);
    return `${where ? where.line + 1 : "?"} TS${diagnostic.code} ${ts.flattenDiagnosticMessageText(diagnostic.messageText, " ")}`;
  });
}

const program = (candidate: string, rows: string[]) =>
  [
    "export {};",
    "type Equal<A, B> = (<G>() => G extends A ? 1 : 2) extends <G>() => G extends B ? 1 : 2 ? true : false;",
    "type Expect<T extends true> = T;",
    kebab(embedded).replaceAll("$KebabCase", "$KebabCaseOracle"),
    candidate,
    'type P = { fooBar: 1; title: 2; "data-x": 5; onUpdateModelValue: 4; ariaLabel: 6; [k: `data-${string}`]: 5; [k: `aria${Capitalize<string>}`]: 6 };',
    ...rows,
  ].join("\n");

const rows = [
  ...shapes.flatMap((shape, index) => [
    `type _K${index} = Expect<Equal<$KebabCase<${shape}>, $KebabCaseOracle<${shape}>>>;`,
    `type _A${index} = Expect<Equal<$KebabCase<${shape}, "pre">, $KebabCaseOracle<${shape}, "pre">>>;`,
  ]),
  "type _M1 = Expect<Equal<{ [K in keyof P & string as $KebabCase<K>]: P[K] }, { [K in keyof P & string as $KebabCaseOracle<K>]: P[K] }>>;",
  "type _M2 = Expect<Equal<{ [K in keyof P & string as $KebabCase<K> extends K ? never : $KebabCase<K>]: P[K] }, { [K in keyof P & string as $KebabCaseOracle<K> extends K ? never : $KebabCaseOracle<K>]: P[K] }>>;",
  "type _M3 = Expect<Equal<$KebabCase<keyof HTMLElementEventMap>, $KebabCaseOracle<keyof HTMLElementEventMap>>>;",
  "type _M4 = Expect<Equal<{ [K in keyof CSSStyleDeclaration & string as $KebabCase<K>]: 1 }, { [K in keyof CSSStyleDeclaration & string as $KebabCaseOracle<K>]: 1 }>>;",
];

test("the shared prop keys differ from the embedded ones only in the kebab-case step", () => {
  assert.equal(shared.length, embedded.length);
  const changed = shared.flatMap((line, index) => (line === embedded[index] ? [] : [index]));
  assert.deepEqual(changed, [embedded.indexOf(kebab(embedded))]);
});

test("the shared kebab-case step equals the embedded char-by-char form", () => {
  assert.deepEqual(diagnostics(program(kebab(shared), rows)), []);
});

test("a fast path that lowercases a mixed key is caught", () => {
  const wrong = 'type $KebabCase<S extends string, Acc extends string = ""> = S extends Lowercase<S> ? `${Acc}${S}` : `${Acc}${Lowercase<S>}`;';
  const found = diagnostics(program(wrong, rows));
  assert.ok(found.length > 0);
  for (const diagnostic of found) assert.match(diagnostic, /TS2344/u);
});
