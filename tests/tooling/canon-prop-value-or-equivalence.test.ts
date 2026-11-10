import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

// `$PropValueOr<P, K, C, FK>` resolves the fallthrough fallback only for keys P
// does not declare. `$PropValue<P, K, $FallthroughValue<C, FK>>` is the oracle.
const declarations = fs.readFileSync(
  new URL(
    "../../crates/vize_canon/src/virtual_ts/helpers/declarations/component-checks.d.ts.txt",
    import.meta.url,
  ),
  "utf8",
);
const line = (name: string): string => {
  const found = declarations
    .split("\n")
    .map((text) => text.replace(/^export /u, ""))
    .find((text) => text.startsWith(`type ${name}<`));
  assert.ok(found, name);
  return found;
};
const helpers = [
  "type $IsAny<T> = 0 extends (1 & T) ? true : false;",
  ...["$IsGeneratedComponent", "$HasFallthroughProps", "$FallthroughValue", "$PropValue"].map(line),
  line("$PropValueOr"),
];
const component = (fallthrough: string) =>
  `{ readonly __vizeComponentMarker: true; readonly __vizeHasFallthroughProps: true; readonly __vizeFallthroughProps?: ${fallthrough} }`;
const components = [
  component(
    "{ title?: string; 'aria-label'?: string; maxlength?: number | string; tabindex?: number }",
  ),
  component("{ title: string } | { title?: number; id?: string }"),
  component("any"),
  "{ readonly __vizeComponentMarker: true }",
  "{ new (): { $props: { title?: string } } }",
  "any",
  "unknown",
];
const props = [
  "{ title: string; count?: number; gone: never; 'as-child': boolean; asChild?: 'camel' }",
  "{ title: string } | { count: number }",
  "{ title?: string | undefined; ariaLabel?: string }",
  "{ [key: string]: unknown; title: number }",
  "{ readonly [K in 'title' | 'count']?: K }",
  "any",
  "unknown",
  "never",
  "{}",
];
const keys: Array<[string, string]> = [
  ["'title'", "'title'"],
  ["'count'", "'count'"],
  ["'gone'", "'gone'"],
  ["'missing'", "'missing'"],
  ["'asChild'", "'as-child'"],
  ["'ariaLabel'", "'aria-label'"],
  ["'maxlength'", "'maxlength'"],
];

function diagnostics(content: string, exact: boolean): string[] {
  const fixture = fileURLToPath(import.meta.url);
  const options: ts.CompilerOptions = {
    strict: true,
    exactOptionalPropertyTypes: exact,
    noEmit: true,
    target: ts.ScriptTarget.ESNext,
    lib: ["lib.esnext.d.ts"],
    types: [],
  };
  const host = ts.createCompilerHost(options);
  const read = host.readFile.bind(host);
  host.readFile = (file) => (file === fixture ? content : read(file));
  return ts.getPreEmitDiagnostics(ts.createProgram([fixture], options, host)).map((diagnostic) => {
    const where = diagnostic.file?.getLineAndCharacterOfPosition(diagnostic.start ?? 0);
    return `${where ? where.line + 1 : "?"} TS${diagnostic.code} ${ts.flattenDiagnosticMessageText(diagnostic.messageText, " ")}`;
  });
}

const prelude = [
  "export {};",
  "type Equal<A, B> = (<G>() => G extends A ? 1 : 2) extends <G>() => G extends B ? 1 : 2 ? true : false;",
  "type Expect<T extends true> = T;",
  ...helpers,
];

for (const exact of [false, true]) {
  test(`the lazy fallback equals the eager one (exactOptionalPropertyTypes: ${exact})`, () => {
    const rows: string[] = [];
    let n = 0;
    for (const P of props)
      for (const C of components)
        for (const [K, FK] of keys) {
          n++;
          rows.push(
            `type _${n} = Expect<Equal<$PropValueOr<${P}, ${K}, ${C}, ${FK}>, $PropValue<${P}, ${K}, $FallthroughValue<${C}, ${FK}>>>>;`,
          );
          if (K === FK)
            rows.push(
              `type _${n}d = Expect<Equal<$PropValueOr<${P}, ${K}, ${C}>, $PropValue<${P}, ${K}, $FallthroughValue<${C}, ${K}>>>>;`,
            );
          // The kebab/camel pair: the authored spelling first, then the camel alias.
          rows.push(
            `type _${n}k = Expect<Equal<$PropValue<${P}, ${FK}, $PropValueOr<${P}, ${K}, ${C}, ${FK}>>, $PropValue<${P}, ${FK}, $PropValue<${P}, ${K}, $FallthroughValue<${C}, ${FK}>>>>>;`,
          );
        }
    assert.ok(n >= 400);
    assert.deepEqual(diagnostics([...prelude, ...rows].join("\n"), exact), []);
  });
}

test("a value assigned through either alias reports identically", () => {
  const C = components[0]!;
  const P = props[0]!;
  const uses = (alias: (k: string, fk: string) => string) =>
    keys
      .map(
        ([K, FK], i) =>
          `type A${i} = ${alias(K, FK)};\nconst v${i}: A${i} = {} as { wrong: true };`,
      )
      .join("\n");
  const lazy = uses((K, FK) => `$PropValueOr<${P}, ${K}, ${C}, ${FK}>`);
  const eager = uses((K, FK) => `$PropValue<${P}, ${K}, $FallthroughValue<${C}, ${FK}>>`);
  const run = (body: string) => diagnostics([...prelude, body].join("\n"), false);
  assert.deepEqual(run(lazy), run(eager));
  assert.ok(run(eager).length > 0);
});

test("a broken candidate that skips the fallback is caught", () => {
  const broken =
    "type $PropValueOr<P, K extends PropertyKey, C, FK extends PropertyKey = K, __V = P extends unknown ? (K extends keyof P ? P[K] : never) : never> = __V;";
  const P = props[0]!;
  const C = components[0]!;
  const body = `type _x = Expect<Equal<$PropValueOr<${P}, 'maxlength', ${C}>, $PropValue<${P}, 'maxlength', $FallthroughValue<${C}, 'maxlength'>>>>;`;
  const content = [...prelude.slice(0, -1), broken, body].join("\n");
  assert.match(diagnostics(content, false).join("\n"), /TS2344/u);
});
