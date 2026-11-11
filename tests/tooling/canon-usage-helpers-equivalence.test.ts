import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

// Hoisted projections name a shared `$Usage*<typeof X>` helper where each
// component usage used to declare an inline conditional over `typeof X`. The
// inline forms are the oracle: every helper must be `Equal` to its form for
// any component type, including unions, `never`, `any` and string-indexed
// roots, whose results a naked (distributive) helper would change.
const declarations = ["component-checks.d.ts.txt", "shared-listener-args.d.ts.txt"]
  .map((file) =>
    fs.readFileSync(
      new URL(
        `../../crates/vize_canon/src/virtual_ts/helpers/declarations/${file}`,
        import.meta.url,
      ),
      "utf8",
    ),
  )
  .join("\n");
const propHelpers = fs.readFileSync(
  new URL("../../crates/vize_canon/src/virtual_ts/component_prop_helpers.txt", import.meta.url),
  "utf8",
);
const line = (source: string, name: string): string => {
  const found = source
    .split("\n")
    .map((text) => text.trim().replace(/^export /u, ""))
    .find((text) => text.startsWith(`type ${name}<`));
  assert.ok(found, name);
  return found;
};
const usageHelpers = [
  "$UsageProps",
  "$UsageRequiredFallthrough",
  "$UsageLibrary",
  "$UsagePropsBox",
  "$UsageEmitPropsBox",
  "$UsageNativeFallthroughBox",
] as const;
const helpers = [
  "type $IsAny<T> = 0 extends (1 & T) ? true : false;",
  line(propHelpers, "$RequiredKeys"),
  ...[
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
    "$FallthroughSourceMember",
    "$FallthroughSourceAdmitsEmpty",
  ].map((name) => line(declarations, name)),
  ...usageHelpers.map((name) => line(declarations, name)),
];

// The inline per-usage forms these helpers replace, over a component value `c`.
const inline: Record<(typeof usageHelpers)[number], (c: string) => string> = {
  $UsageProps: (c) =>
    `typeof ${c} extends { __vizeCheck: infer __F } ? ($IsAny<__F> extends true ? $ComponentRawProps<typeof ${c}> : Record<string, unknown>) : $ComponentRawProps<typeof ${c}>`,
  $UsageRequiredFallthrough: (c) =>
    `($FallthroughProps<typeof ${c}> extends infer __F ? [__F extends unknown ? {} extends __F ? never : __F : never] extends [never] ? $RequiredFallthroughProps<typeof ${c}, __F, never> : $RequiredFallthroughProps<typeof ${c}, __F> : never)`,
  $UsageLibrary: (c) =>
    `typeof ${c} extends { __vizeCheck: any } | { readonly __vizeComponentMarker: true } ? false : typeof ${c} extends (...args: any[]) => infer __R ? 0 extends (1 & __R) ? false : '__ctx' extends keyof __R ? true : false : false`,
  $UsagePropsBox: (c) =>
    `typeof ${c} extends { new (...args: any[]): { $props: infer __P } } ? [__P] : typeof ${c} extends (props: infer __P, ...args: any[]) => any ? [__P] : []`,
  $UsageEmitPropsBox: (c) => `typeof ${c} extends { __vizeEmitProps?: infer __EP } ? [__EP] : []`,
  $UsageNativeFallthroughBox: (c) =>
    `typeof ${c} extends { readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: infer __F } ? "__vizeEmitProps" extends keyof { [__K in keyof typeof ${c} as string extends __K ? never : number extends __K ? never : symbol extends __K ? never : __K]: unknown } ? $IsAny<__F> extends true ? [] : [__F] : [] : []`,
};

const generated = (members: string) =>
  `{ new (...args: any[]): { $props: { a?: string; b: number } }; readonly __vizeComponentMarker: true; readonly __vizeHasFallthroughProps: true; ${members} }`;
const nativeRoot = (fallthrough: string, members = "") =>
  `{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: ${fallthrough}; ${members} }`;
const components: Record<string, string> = {
  generated: generated(
    "readonly __vizeFallthroughProps?: { title: string; id?: string }; __vizeEmitProps?: { onSave?: () => void }; readonly __vizeNativeFallthroughRoot: true; __vizeCheck: (props: { a?: string }) => void; readonly __vizeRawProps?: { a?: string; b: number }",
  ),
  optionalFallthrough: generated(
    "readonly __vizeFallthroughProps?: { title?: string }; readonly __vizeNativeFallthroughRoot: true; __vizeCheck: any",
  ),
  anyFallthrough: generated(
    "readonly __vizeFallthroughProps?: any; readonly __vizeNativeFallthroughRoot: true; __vizeEmitProps?: {}; __vizeCheck: unknown",
  ),
  generatedUnion: `${generated("readonly __vizeFallthroughProps?: { title: string }; __vizeEmitProps?: { onA?: () => void }; readonly __vizeNativeFallthroughRoot: true; __vizeCheck: any")} | ${generated("readonly __vizeFallthroughProps?: { id: number }; __vizeEmitProps?: { onB?: () => void }; readonly __vizeNativeFallthroughRoot: true")}`,
  mixedUnion: `${generated("readonly __vizeFallthroughProps?: { title: string }; __vizeEmitProps?: {}; readonly __vizeNativeFallthroughRoot: true")} | ((props: { x: string }, ctx: any) => { __ctx?: { props: { x: string } } })`,
  stringIndexedRoot: `{ [key: string]: unknown; ${nativeRoot("{ title: string }", "__vizeEmitProps?: {}").slice(1)}`,
  numberIndexedRoot: `{ [key: number]: unknown; ${nativeRoot("{ title: string }", "readonly __vizeEmitProps?: {}").slice(1)}`,
  templateIndexedRoot: `{ [key: symbol]: unknown; [key: \`__vize\${string}\`]: unknown; ${nativeRoot("{ title: string }").slice(1)}`,
  indexedRootUnion: `{ [key: string]: unknown; ${nativeRoot("{ title: string }").slice(1)} | ${nativeRoot("{ id: string }", "__vizeEmitProps?: {}")}`,
  intersectedRoot: `(${nativeRoot("{ title: string }")} & { __vizeEmitProps?: {} } & { [key: string]: unknown })`,
  rootWithoutEmits: nativeRoot("{ title: string }"),
  // The raw source beside the owned-key map: the shortcut must equal the map.
  sourceOptional: generated(
    "readonly __vizeFallthroughSource?: { title?: string; a?: string }; readonly __vizeFallthroughProps?: { title?: string }",
  ),
  sourceRequired: generated(
    "readonly __vizeFallthroughSource?: { title: string; id?: string }; readonly __vizeFallthroughProps?: { title: string; id?: string }",
  ),
  sourceOwnedRequired: generated(
    "readonly __vizeFallthroughSource?: { b: number; title?: string }; readonly __vizeFallthroughProps?: { title?: string }",
  ),
  sourceAny: generated(
    "readonly __vizeFallthroughSource?: any; readonly __vizeFallthroughProps?: any",
  ),
  sourceUnknown: generated(
    "readonly __vizeFallthroughSource?: unknown; readonly __vizeFallthroughProps?: { title: string }",
  ),
  sourceUnion: `${generated("readonly __vizeFallthroughSource?: { title?: string }; readonly __vizeFallthroughProps?: { title?: string }")} | ${generated("readonly __vizeFallthroughSource?: { id: number }; readonly __vizeFallthroughProps?: { id: number }")}`,
  sourceOptionalUnion: `${generated("readonly __vizeFallthroughSource?: { title?: string }; readonly __vizeFallthroughProps?: { title?: string }")} | ${generated("readonly __vizeFallthroughSource?: { id?: number }; readonly __vizeFallthroughProps?: { id?: number }")}`,
  sourceBesideOldChild: `${generated("readonly __vizeFallthroughSource?: { title?: string }; readonly __vizeFallthroughProps?: { title?: string }")} | ${generated("readonly __vizeFallthroughProps?: { id: number }")}`,
  sourceWithoutFallthrough:
    "{ new (...args: any[]): { $props: {} }; readonly __vizeComponentMarker: true; readonly __vizeFallthroughSource?: { title: string } }",
  anyRoot: nativeRoot("any", "__vizeEmitProps?: {}"),
  libraryGeneric: "(props: { x: string }, ctx: any) => { __ctx?: { props: { x: string } } }",
  anyReturn: "(props: { x: string }) => any",
  functional: "(props: { x: string }, ctx: any) => null",
  constructor: "{ new (): { $props: { a: string } } }",
  never: "never",
  any: "any",
  unknown: "unknown",
  string: "string",
  null: "null",
};

function diagnostics(content: string): string[] {
  const fixture = fileURLToPath(import.meta.url);
  const options: ts.CompilerOptions = {
    strict: true,
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

const prelude = (helperLines: readonly string[]) => [
  "export {};",
  "type Equal<A, B> = (<G>() => G extends A ? 1 : 2) extends <G>() => G extends B ? 1 : 2 ? true : false;",
  "type Expect<T extends true> = T;",
  ...helperLines,
  ...Object.entries(components).map(([name, type]) => `declare const ${name}_: ${type};`),
];

const rows = (helper: (typeof usageHelpers)[number], names = Object.keys(components)) =>
  names.map(
    (name) =>
      `type _${helper.slice(1)}_${name} = Expect<Equal<${helper}<typeof ${name}_>, ${inline[helper](`${name}_`)}>>;`,
  );

test("each shared usage helper equals its inline per-usage form", () => {
  const body = usageHelpers.flatMap((helper) => rows(helper));
  assert.equal(body.length, usageHelpers.length * Object.keys(components).length);
  assert.deepEqual(diagnostics([...prelude(helpers), ...body].join("\n")), []);
});

test("a distributive candidate is caught on unions, never and indexed roots", () => {
  // A naked check type distributes over unions and maps `never` to `never`;
  // a homomorphic `keyof C` mapping does the same. Both must be refused.
  const candidates: Array<[(typeof usageHelpers)[number], string, string[]]> = [
    [
      "$UsageLibrary",
      "type $UsageLibrary<C> = C extends { __vizeCheck: any } | { readonly __vizeComponentMarker: true } ? false : C extends (...args: any[]) => infer __R ? 0 extends (1 & __R) ? false : '__ctx' extends keyof __R ? true : false : false;",
      ["mixedUnion", "never"],
    ],
    [
      "$UsageNativeFallthroughBox",
      'type $UsageNativeFallthroughBox<C> = NoInfer<C> extends { readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: infer __F } ? "__vizeEmitProps" extends keyof { [__K in keyof C as string extends __K ? never : number extends __K ? never : symbol extends __K ? never : __K]: unknown } ? $IsAny<__F> extends true ? [] : [__F] : [] : [];',
      ["indexedRootUnion", "never"],
    ],
  ];
  for (const [helper, candidate, failing] of candidates) {
    const replaced = helpers.map((text) => (text.startsWith(`type ${helper}<`) ? candidate : text));
    assert.notDeepEqual(replaced, helpers, helper);
    const found = diagnostics([...prelude(replaced), ...rows(helper, failing)].join("\n"));
    assert.equal(found.length, failing.length, `${helper}: ${found.join("\n")}`);
    for (const diagnostic of found) assert.match(diagnostic, /TS2344/u);
  }
});

test("the fallthrough source shortcut refuses a child that names no source", () => {
  // Without the `unknown` guard a child without the member (an older
  // declaration) infers `unknown`, admits `{}` and loses its required keys.
  const unguarded =
    "type $FallthroughSourceMember<C> = C extends { readonly __vizeHasFallthroughProps: true; readonly __vizeFallthroughSource?: infer __S } ? [NonNullable<__S> extends infer __N ? __N extends unknown ? {} extends __N ? never : __N : never : never] extends [never] ? true : false : false;";
  const replaced = helpers.map((text) =>
    text.startsWith("type $FallthroughSourceMember<") ? unguarded : text,
  );
  assert.notDeepEqual(replaced, helpers);
  const failing = ["generated"];
  const found = diagnostics(
    [...prelude(replaced), ...rows("$UsageRequiredFallthrough", failing)].join("\n"),
  );
  assert.equal(found.length, failing.length, found.join("\n"));
  for (const diagnostic of found) assert.match(diagnostic, /TS2344/u);
});
