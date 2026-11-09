import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

// `$UnownedComponentAttrs` computes its owned key set once, outside the key mapping.
// The previous per-key definition is the oracle: same keys, types and modifiers.
// Generated exports forward only concrete roots and non-generic props, so a
// deferred instantiation (which would print the defaulted key set) never occurs.
const helpers = fs.readFileSync(
  new URL("../../crates/vize_canon/src/virtual_ts/component_prop_helpers.txt", import.meta.url),
  "utf8",
);
const line = (name: string): string => {
  const found = helpers.split("\n").find((text) => text.startsWith(`type ${name}<`));
  assert.ok(found, name);
  return found;
};
const current = [
  "interface VNodeProps { key?: PropertyKey; ref?: unknown }",
  "interface AllowedComponentProps { class?: unknown; style?: unknown }",
  "interface ComponentCustomProps {}",
  "type $IsAny<T> = 0 extends (1 & T) ? true : false;",
  "type $PublicComponentInputProps = VNodeProps & AllowedComponentProps & ComponentCustomProps;",
  line("$ComponentMetadataKeys"),
  line("$ComponentOwnKeys"),
  line("$UnownedComponentAttrs"),
];
const previous = [
  "type OldUnownedComponentAttrs<F, P = {}, E = {}> = $IsAny<F> extends true ? F : unknown extends F ? F : F extends unknown ? F extends null | undefined ? F : { [K in keyof F as K extends $ComponentOwnKeys<P, E> ? never : K]: F[K] } : never;",
];

const fallthrough = [
  "{}",
  "Partial<HTMLInputElement>",
  "Partial<HTMLDivElement> & Partial<HTMLSpanElement>",
  "{ readonly id?: string; title: string; class?: string; key?: string; onClick?: (event: MouseEvent) => void }",
  "{ [key: string]: unknown; id?: string }",
  "Record<string, unknown>",
  "{ a?: 1 } | { b: 2 }",
  "{ a?: 1 } | null | undefined",
  "any",
  "unknown",
  "never",
];
const owners = [
  ["{}", "{}"],
  ["{ title: string; id?: number }", "{}"],
  ["{ title: string } | { label: string }", "{ onClick?: () => void }"],
  ["any", "{}"],
  ["{ value?: string }", "any"],
  ["Partial<{ id: string; onClick: () => void }>", "{ onFocus?: () => void } | undefined"],
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
  const read = host.readFile.bind(host);
  host.readFile = (file) => (file === fixture ? content : read(file));
  return ts.getPreEmitDiagnostics(ts.createProgram([fixture], options, host)).map((diagnostic) => {
    const where = diagnostic.file?.getLineAndCharacterOfPosition(diagnostic.start ?? 0);
    const text = ts.flattenDiagnosticMessageText(diagnostic.messageText, " ");
    return `${where ? `${where.line + 1}:${where.character + 1}` : "?"} TS${diagnostic.code} ${text}`;
  });
}

const prelude = [
  "export {};",
  "type Equal<A, B> = (<G>() => G extends A ? 1 : 2) extends <G>() => G extends B ? 1 : 2 ? true : false;",
  "type Expect<T extends true> = T;",
];

function equalities(helpers: string[]): string {
  return [
    ...prelude,
    ...helpers,
    ...previous,
    ...fallthrough.flatMap((f, i) =>
      owners.map(
        ([p, e], j) =>
          `type _attrs${i}_${j} = Expect<Equal<$UnownedComponentAttrs<${f}, ${p}, ${e}>, OldUnownedComponentAttrs<${f}, ${p}, ${e}>>>;`,
      ),
    ),
    ...fallthrough.map(
      (f, i) =>
        `type _defaults${i} = Expect<Equal<$UnownedComponentAttrs<${f}>, OldUnownedComponentAttrs<${f}>>>;`,
    ),
  ].join("\n");
}

function usage(attrs: string): string {
  return [
    ...prelude,
    ...current,
    ...previous,
    `type Attrs<F, P> = ${attrs}<F, P>;`,
    "declare const input: Attrs<Partial<HTMLInputElement>, { title: string; id?: number }>;",
    "const readonlyId: string | undefined = input.id;",
    "const owned: unknown = input.title;",
    "const classValue = input.class;",
    "const wrong: number = input.value;",
    "export const results = [readonlyId, owned, classValue, wrong];",
  ].join("\n");
}

void test("the hoisted owned key set keeps every unowned attribute surface identical", () => {
  assert.deepEqual(diagnostics(equalities(current)), []);
  const fresh = diagnostics(usage("$UnownedComponentAttrs"));
  const old = diagnostics(usage("OldUnownedComponentAttrs")).map((text) =>
    text.replaceAll("OldUnownedComponentAttrs", "$UnownedComponentAttrs"),
  );
  assert.deepEqual(fresh, old);
  assert.ok(fresh.length > 0);
});

void test("the oracle rejects a key set that drops an owned key", () => {
  const broken = [...current];
  broken[7] =
    "type $UnownedComponentAttrs<F, P = {}, E = {}, __O = keyof $PublicComponentInputProps> = $IsAny<F> extends true ? F : unknown extends F ? F : F extends unknown ? F extends null | undefined ? F : { [K in keyof F as K extends __O ? never : K]: F[K] } : never;";
  assert.ok(diagnostics(equalities(broken)).some((text) => / TS2344 /u.test(text)));
});
