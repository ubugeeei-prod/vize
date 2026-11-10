import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

// A plain component binding's `__X_Props_N` is `$UsageProps<typeof X>`, and its
// `__X_CheckProps_N` intersects it with `$UsageCheckExtras<C>`. The aliases it
// used to declare, written against `typeof X`, are the oracle: they are
// non-distributive, and `any` takes both branches.
const read = (path: string) =>
  fs.readFileSync(
    new URL(`../../crates/vize_canon/src/virtual_ts/${path}`, import.meta.url),
    "utf8",
  );
const declarations = [
  read("helpers/declarations/component-checks.d.ts.txt"),
  read("component_prop_helpers.txt"),
].join("\n");
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
  ...[
    "$RequiredKeys",
    "$EmitListeners",
    "$IsGeneratedComponent",
    "$HasFallthroughProps",
    "$FallthroughProps",
    "$RequiredFallthroughProps",
    "$PublicProps",
    "$InstanceRawProps",
    "$StaticRawProps",
    "$HasInstanceRawProps",
    "$HasStaticRawProps",
    "$ComponentRawProps",
    "$ComponentCheckProps",
  ].map(line),
];
const candidate = [line("$UsageProps"), line("$UsageCheckExtras")];

const generated = (fields: string) => `{ readonly __vizeComponentMarker: true; ${fields} }`;
const components = [
  // A generated SFC with static raw props, emits and a native-root surface.
  generated(
    "readonly __vizeRawProps?: { title: string; count?: number }; readonly __vizeEmitProps?: { onSave?: (id: number) => void }; readonly __vizeHasFallthroughProps: true; readonly __vizeFallthroughProps?: { id?: string; tabindex?: number }",
  ),
  // A required fallthrough key (`checkRequiredFallthroughAttributes`).
  generated(
    "readonly __vizeRawProps?: { title: string }; readonly __vizeHasFallthroughProps: true; readonly __vizeFallthroughProps?: { id: string; title?: string }",
  ),
  // A union fallthrough surface and an `any` one.
  generated(
    "readonly __vizeHasFallthroughProps: true; readonly __vizeFallthroughProps?: { a: string } | { b?: number }",
  ),
  generated("readonly __vizeHasFallthroughProps: true; readonly __vizeFallthroughProps?: any"),
  // Raw props carried by the instance, and `any` raw props.
  `{ readonly __vizeComponentMarker: true; new (): { readonly __vizeRawProps?: { a: number }; $props: { a: number } } }`,
  generated("readonly __vizeRawProps?: any"),
  // A generic SFC child checks through `__vizeCheck`; `any` and `unknown` checkers.
  generated("__vizeCheck: <T>(props: { item: T; list: T[] }) => void"),
  "{ __vizeCheck: any }",
  "{ __vizeCheck: unknown; new (): { $props: { a?: string } } }",
  // A library generic export, as vue-tsc compiles one.
  "(<T>(props: { item: T }) => { __ctx?: { props: { item: T } } })",
  // Non-generated constructors, functional components and global fallbacks.
  "{ new (): { $props: { title?: string; onClick?: () => void } } }",
  "(props: { a: string; b?: number }) => any",
  "{ new (): { $props: { a?: number } } } & { readonly __vizeComponentMarker: true }",
  // A union target (what a dynamic component resolves to).
  `${generated("readonly __vizeRawProps?: { title: string }")} | { new (): { $props: { href?: string } } }`,
  "'MkA' | 'a'",
  `${generated("__vizeCheck: (props: { x: 1 }) => void")} | ((props: { y: 2 }) => any)`,
  "any",
  "unknown",
  "never",
  "{}",
  "string",
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
  const readFile = host.readFile.bind(host);
  host.readFile = (file) => (file === fixture ? content : readFile(file));
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

// The aliases a usage of `X` declared before, verbatim but for the names.
const oracle = (n: number, name = "__X") => [
  `type ${name}_Props_${n} = typeof X${n} extends { __vizeCheck: infer __F } ? ($IsAny<__F> extends true ? $ComponentRawProps<typeof X${n}> : Record<string, unknown>) : $ComponentRawProps<typeof X${n}>;`,
  `type ${name}_Component_${n} = typeof X${n};`,
  `type ${name}_CheckProps_${n} = ${name}_Props_${n} & $EmitListeners<${name}_Component_${n}> & ($FallthroughProps<${name}_Component_${n}> extends infer __F ? [__F extends unknown ? {} extends __F ? never : __F : never] extends [never] ? $RequiredFallthroughProps<${name}_Component_${n}, __F, never> : $RequiredFallthroughProps<${name}_Component_${n}, __F> : never);`,
];
// What the emitter declares now.
const shared = (n: number, name = "__X") => [
  `type ${name}_Props_${n} = $UsageProps<typeof X${n}>;`,
  `type ${name}_Component_${n} = typeof X${n};`,
  `type ${name}_CheckProps_${n} = ${name}_Props_${n} & $UsageCheckExtras<${name}_Component_${n}>;`,
];

const rows = (helperLines: string[]) =>
  [
    ...prelude,
    ...helperLines,
    ...components.flatMap((C, n) => [
      `declare const X${n}: ${C};`,
      ...oracle(n),
      ...shared(n, "__S"),
      `type _p${n} = Expect<Equal<__S_Props_${n}, __X_Props_${n}>>;`,
      `type _c${n} = Expect<Equal<__S_CheckProps_${n}, __X_CheckProps_${n}>>;`,
    ]),
  ].join("\n");

for (const exact of [false, true]) {
  test(`the shared usage contract equals the per-usage aliases (exactOptionalPropertyTypes: ${exact})`, () => {
    assert.ok(components.length >= 20);
    assert.deepEqual(diagnostics(rows(candidate), exact), []);
  });
}

test("a call checked through either contract prints the same diagnostics", () => {
  const literals = [
    "{}",
    "{ title: 1 }",
    "{ title: 'a', count: 'b' }",
    "{ title: 'a', onSave: (id: string) => {} }",
    "{ title: 'a', id: 2 }",
    "{ item: 1, list: ['a'] }",
  ];
  const program = (aliases: (n: number) => string[]) =>
    [
      ...prelude,
      ...candidate,
      ...components.flatMap((C, n) => [
        `declare const X${n}: ${C};`,
        ...aliases(n),
        `declare function check${n}(props: $ComponentCheckProps<__X_CheckProps_${n}, { readonly tail?: true }>): void;`,
        ...literals.map((literal) => `check${n}(${literal});`),
      ]),
    ].join("\n");
  // Same alias names on both sides, so every message must match byte for byte,
  // including the `__X_CheckProps_N` a TS2345 prints for the parameter.
  const before = diagnostics(
    program((n) => oracle(n)),
    false,
  );
  assert.deepEqual(
    diagnostics(
      program((n) => shared(n)),
      false,
    ),
    before,
  );
  assert.ok(before.length > 20);
  assert.ok(before.some((message) => message.includes("__X_CheckProps_")));
});

test("a distributive candidate is caught; the explicit `any` arm only skips inference", () => {
  const usageProps = line("$UsageProps");
  const distributive = usageProps.replace(
    "[C] extends [{ __vizeCheck: infer __F }]",
    "C extends { __vizeCheck: infer __F }",
  );
  assert.notEqual(distributive, usageProps);
  assert.match(
    diagnostics(rows([distributive, line("$UsageCheckExtras")]), false).join("\n"),
    /TS2344/u,
  );
  // `[any]` matches the checker shape and infers `any`, which reaches the same
  // `$ComponentRawProps<any>` as the arm; the arm answers without inference.
  const withoutAny = usageProps.replace("0 extends (1 & C) ? $ComponentRawProps<C> : ", "");
  assert.notEqual(withoutAny, usageProps);
  assert.deepEqual(diagnostics(rows([withoutAny, line("$UsageCheckExtras")]), false), []);
});
