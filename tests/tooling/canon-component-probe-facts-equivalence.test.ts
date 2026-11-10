import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

// A named Content Mapper component intersects static probe facts after its
// constructor: `__vizeInstance` is what every constructor signature returns
// for an unbound call, and `__vizeInput` holds the `$ComponentInput` arguments
// whose unbound form is the constructor's `props` parameter. The shared helpers
// read the facts first. Their previous constructor-only forms are the oracle:
// every helper must stay `Equal` for components with and without facts,
// unions of both, other libraries' constructors, `never`, `any` and `unknown`.
const read = (file: string) =>
  fs.readFileSync(
    new URL(`../../crates/vize_canon/src/virtual_ts/helpers/declarations/${file}`, import.meta.url),
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
const checks = read("component-checks.d.ts.txt");
const embeddedEmits = read("emit-inference.d.ts.txt");
const sharedEmits = read("shared-emit-inference.d.ts.txt");

const factHelpers = [
  ["$PublicProps", checks],
  ["$InstanceRawProps", checks],
  ["$HasInstanceRawProps", checks],
  ["$UnionEmit", embeddedEmits],
  ["$NativeEmitPropsFactory", embeddedEmits],
] as const;

// The constructor-only forms before the facts, renamed beside the current ones.
const previous = [
  "type $PublicPropsBefore<C> = C extends abstract new (...args: any[]) => infer __I ? $IsAny<__I> extends true ? Record<string, unknown> : __I extends { $props: infer __P } ? __P : {} : C extends (props: infer __P, ...args: any[]) => any ? __P : {};",
  "type $InstanceRawPropsBefore<C> = C extends { new (): { readonly __vizeRawProps?: infer __P } } ? $IsAny<__P> extends true ? $PublicPropsBefore<C> : __P : $PublicPropsBefore<C>;",
  "type $HasInstanceRawPropsBefore<C> = $IsGeneratedComponent<C> extends true ? C extends { new (): { readonly __vizeRawProps?: infer __P } } ? $IsAny<__P> extends true ? false : true : false : false;",
  "type $UnionEmitBefore<All> = (All extends new (...args: any[]) => { $emit: infer E } ? () => E : never) extends () => infer U ? U : never;",
  "type $NativeEmitPropsFactoryBefore<C, All = C> = C extends new (...args: any[]) => { $emit: any } ? C extends { readonly __vizeComponentMarker: true } ? C extends new (props: infer P, ...args: any[]) => any ? $MemberConstructorEmitPropsFactory<P, $UnionEmitBefore<All>, C extends { readonly __vizeFallthroughProps?: infer F } ? F : {}, $EmitOwnedKeys<C>> : never : <P, E>(component: new (props: P, ...args: any[]) => { $emit: E }) => (props: P) => $DeferredEmitProps<E> : $LooseEmitPropsFactory<C, $UnionEmitBefore<All>>;",
];

// Opaque stand-ins keep every argument visible to `Equal` without Vue.
const support = [
  "type $IsAny<T> = 0 extends (1 & T) ? true : false;",
  "type $IsGeneratedComponent<C> = C extends { readonly __vizeComponentMarker: true } ? true : false;",
  "type $ComponentInputProps<T, Declared = {}> = { readonly input: T; readonly declared: Declared };",
  // The authored-input guard is TypeScript's own business; only the unbound
  // input reaches a probe, so the stand-in accepts every authored input.
  "type $ComponentInput<T, Declared, Authored> = $ComponentInputProps<T, Declared>;",
  "type $MemberConstructorEmitPropsFactory<P, E, F, O extends PropertyKey> = { readonly member: [P, E, F, O] };",
  "type $EmitOwnedKeys<C> = C extends { readonly __vizeRawProps?: infer P } ? keyof P : never;",
  "type $DeferredEmitProps<E> = { readonly deferred: E };",
  "type $LooseEmitPropsFactory<C, E = never> = { readonly loose: [C, E] };",
];

// A generated component: metadata, the generic constructor, Vue options and,
// for a named Content Mapper type, the trailing facts.
const instance = (props: string, emit: string) =>
  `{ $props: ${props}; readonly __vizeRawProps?: ${props}; $emit: ${emit}; $slots: {} }`;
const generated = (props: string, declared: string, emit: string, facts: boolean) => {
  const self = instance(props, emit);
  return [
    `{ readonly __vizeComponentMarker: true; readonly __vizeRawProps?: ${props}; readonly __vizeFallthroughProps?: { title?: string } }`,
    `(new <$AuthoredProps = unknown>(props?: $AuthoredProps & $ComponentInput<${props}, ${declared}, $AuthoredProps>, ...args: any[]) => ${self})`,
    "{ [key: string]: any; name?: string }",
    ...(facts
      ? [`{ readonly __vizeInstance: ${self}; readonly __vizeInput?: [${props}, ${declared}] }`]
      : []),
  ].join(" & ");
};
const button = (facts: boolean) =>
  generated("{ label?: string }", "{ onPress?: () => void }", "(event: 'press') => void", facts);
const card = (facts: boolean) =>
  generated(
    "{ title: string; size?: number }",
    "{}",
    "(event: string, ...args: any[]) => void",
    facts,
  );
const components: Record<string, string> = {
  facts: button(true),
  requiredFacts: card(true),
  constructorOnly: button(false),
  factsUnion: `(${button(true)}) | (${card(true)})`,
  mixedUnion: `(${button(true)}) | (${card(false)})`,
  libraryUnion: `(${button(true)}) | (new () => { $props: { x: string }; $emit: () => void })`,
  vueConstructor: "{ new (...args: any[]): { $props: { x: string }; $emit: (e: 'x') => void } }",
  functional: "(props: { x: string }, ctx: any) => null",
  never: "never",
  any: "any",
  unknown: "unknown",
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
  const original = host.readFile.bind(host);
  host.readFile = (file) => (file === fixture ? content : original(file));
  return ts.getPreEmitDiagnostics(ts.createProgram([fixture], options, host)).map((diagnostic) => {
    const where = diagnostic.file?.getLineAndCharacterOfPosition(diagnostic.start ?? 0);
    return `${where ? where.line + 1 : "?"} TS${diagnostic.code} ${ts.flattenDiagnosticMessageText(diagnostic.messageText, " ")}`;
  });
}

const prelude = (helperLines: readonly string[]) => [
  "export {};",
  "type Equal<A, B> = (<G>() => G extends A ? 1 : 2) extends <G>() => G extends B ? 1 : 2 ? true : false;",
  "type Expect<T extends true> = T;",
  ...support,
  ...helperLines,
  ...previous,
  ...Object.entries(components).map(([name, type]) => `type ${name}_ = ${type};`),
];
const current = factHelpers.map(([name, source]) => line(source, name));
// `$NativeEmitPropsFactory` is reached only through `$EmitPropsRoutes`, behind
// `$EmitPropsFactory`'s `$IsAny` guard, so it never sees `any` itself.
const unguarded = new Set(["$NativeEmitPropsFactory:any"]);
const rows = (names = Object.keys(components)) =>
  factHelpers.flatMap(([helper]) =>
    names
      .filter((name) => !unguarded.has(`${helper}:${name}`))
      .map(
        (name) =>
          `type _${helper.slice(1)}_${name} = Expect<Equal<${helper}<${name}_>, ${helper}Before<${name}_>>>;`,
      ),
  );

test("the embedded and shared emit inference helpers keep one fact policy", () => {
  for (const name of ["$UnionEmit", "$NativeEmitPropsFactory"])
    assert.equal(line(embeddedEmits, name), line(sharedEmits, name), name);
});

test("each fact-reading helper equals its constructor-only form", () => {
  const body = rows();
  assert.equal(body.length, factHelpers.length * Object.keys(components).length - unguarded.size);
  assert.deepEqual(diagnostics([...prelude(current), ...body].join("\n")), []);
});

test("facts that disagree with the constructor are caught", () => {
  // The oracle is not vacuous: a fact that names another instance or input
  // changes every fact-reading helper for the components that carry it.
  const wrong = (component: string) =>
    component
      .replace(
        "readonly __vizeInstance: { $props: { label?: string }",
        "readonly __vizeInstance: { $props: { other?: string }",
      )
      .replace(
        "readonly __vizeRawProps?: { label?: string }; $emit",
        "readonly __vizeRawProps?: { other?: string }; $emit",
      )
      .replace(
        "$emit: (event: 'press') => void; $slots: {} }; readonly __vizeInput",
        "$emit: (event: 'other') => void; $slots: {} }; readonly __vizeInput",
      )
      .replace(
        "readonly __vizeInput?: [{ label?: string }",
        "readonly __vizeInput?: [{ other?: string }",
      );
  const altered = wrong(components.facts!);
  assert.notEqual(altered, components.facts);
  const content = [
    ...prelude(current),
    `type drifted_ = ${altered};`,
    ...factHelpers
      .filter(([helper]) => helper !== "$HasInstanceRawProps")
      .map(
        ([helper]) =>
          `type _${helper.slice(1)} = Expect<Equal<${helper}<drifted_>, ${helper}Before<drifted_>>>;`,
      ),
  ].join("\n");
  const found = diagnostics(content);
  assert.equal(found.length, 4, found.join("\n"));
  for (const diagnostic of found) assert.match(diagnostic, /TS2344/u);
});
