import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

// A hoisted module reads listener contracts through `$Listener*` helpers; a
// standalone document still inlines the generator's text. The inline text is
// the oracle: every helper must compute exactly the type it replaces.
const repo = new URL("../../", import.meta.url);
const read = (path: string) => fs.readFileSync(new URL(path, repo), "utf8");
const helpers = read(
  "crates/vize_canon/src/virtual_ts/helpers/declarations/listener-contracts.d.ts.txt",
)
  .replace(/^declare module "vize:virtual-ts" \{\n/u, "")
  .replace(/\}\n$/u, "")
  .replace(/^export /gmu, "");
const generator = [
  "crates/vize_canon/src/virtual_ts/scope/component_events.rs",
  "crates/vize_canon/src/virtual_ts/scope/component_events/native_fallthrough.rs",
  "crates/vize_canon/src/virtual_ts/scope/component_events/listener_return.rs",
]
  .map(read)
  .join("\n");

// Standalone format strings exactly as the generator spells them.
const INLINE = {
  library:
    "type {library} = typeof {component_ref} extends {{ __vizeCheck: any }} | {{ readonly __vizeComponentMarker: true }} ? false : typeof {component_ref} extends (...args: any[]) => infer __R ? 0 extends (1 & __R) ? false : '__ctx' extends keyof __R ? true : false : false;",
  props:
    "type {props_box} = typeof {component_ref} extends {{ new (...args: any[]): {{ $props: infer __P }} }} ? [__P] : typeof {component_ref} extends (props: infer __P, ...args: any[]) => any ? [__P] : [];",
  emit: "type {emit_props_box} = typeof {component_ref} extends {{ __vizeEmitProps?: infer __EP }} ? [__EP] : [];",
  native:
    'type {fallthrough_box} = typeof {component_ref} extends {{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: infer __F }} ? \\"__vizeEmitProps\\" extends keyof {{ [__K in keyof typeof {component_ref} as string extends __K ? never : number extends __K ? never : symbol extends __K ? never : __K]: unknown }} ? {is_any}<__F> extends true ? [] : [__F] : [] : [];',
  explicit:
    "type {fallthrough_box} = typeof {component_ref} extends {{ readonly __vizeFallthroughProps?: infer __F }} ? {is_any}<__F> extends true ? [] : [__F] : [];",
  chain:
    "__P extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : __P extends {{ {prop_key}?: infer __H }} ? [Extract<NonNullable<__H>, (...args: any) => any>] extends [never] ? {missing_prop_args} : Extract<NonNullable<__H>, (...args: any) => any> extends (...args: infer __A) => any ? __A : {missing_prop_args} : {missing_prop_args}",
  raw: "__P extends {{ {raw_key}?: (...args: infer __A) => any }} ? __A : unknown[]",
  propArgs:
    "type {prop_args} = {library} extends true ? unknown[] : {props_box} extends infer __B ? __B extends [infer __P] ? {prop_listener_args} : unknown[] : never;",
  fallthroughArgs:
    "type {fallthrough_args} = {fallthrough_box} extends infer __B ? __B extends [infer __F] ? NonNullable<__F> extends infer __P ? ({prop_listener_args}) extends infer __A extends unknown[] ? (__A extends [] ? unknown[] : __A) : unknown[] : unknown[] : unknown[] : never;",
  staticEmit:
    "type {static_emit_args} = {emit_props_box} extends infer __B ? __B extends [infer __EP] ? __EP extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[] : unknown[] : never;",
  nativeReturn:
    "type {native_return} = {fallthrough_box} extends infer __B ? __B extends [infer __F] ? NonNullable<__F> extends {{ {prop_key}?: infer __H }} ? Extract<NonNullable<__H>, (...args: any[]) => any> extends (...args: any[]) => infer __R ? __R : any : any : any : never;",
  return:
    "type {return_type} = unknown[] extends {prop_args} ? {unresolved_return} : {props_box} extends infer __B ? __B extends [infer __P] ? (__P extends {{ {prop_key}?: (...args: any[]) => infer __R }} ? __R : any) : any : never;",
  arrays:
    "__P extends {{ {prop_key}?: infer __H }} ? Extract<NonNullable<__H>, readonly unknown[]> : never",
  arraysAlias:
    "type {arrays_type} = {props_box} extends infer __B ? __B extends [infer __P] ? {arrays} : never : never;",
};

/** Rust `format!` semantics for the placeholders above. */
function format(template: string, values: Record<string, string>): string {
  return template
    .replaceAll('\\"', '"')
    .replace(/\{\{|\}\}|\{([a-z_]+)\}/gu, (token, name?: string) => {
      if (token === "{{") return "{";
      if (token === "}}") return "}";
      const value = values[name!];
      assert.ok(value !== undefined, name);
      return value;
    });
}

const shapes: Record<string, string> = {
  any: "any",
  unknown: "unknown",
  never: "never",
  ctor: 'new (...args: any[]) => { $props: { onSave?: (x: number) => void; onClick?: (e: MouseEvent) => number; "onUpdate:modelValue"?: (v: string) => void } }',
  required: "new () => { $props: { onSave: (x: number, y: string) => boolean } }",
  arrayOrFn: "new () => { $props: { onSave?: ((x: number) => void) | ((x: number) => void)[] } }",
  arrayOnly: "new () => { $props: { onSave?: ((x: number) => void)[] } }",
  nonFunction: "new () => { $props: { onSave?: string } }",
  unionProps:
    "new () => { $props: { onSave?: (x: number) => void } | { onSave?: (x: string) => void; onClick?: () => void } }",
  unionCtor:
    "(new () => { $props: { onSave?: (x: number) => void } }) | (new () => { $props: { onClick?: (e: Event) => void } })",
  functional:
    '(props: { onSave?: (x: string) => void; "onFoo-bar"?: (n: 1) => void }, ctx: any) => any',
  library:
    "(props: { onSave?: (x: number) => void }) => { __ctx?: { props: { onSave?: (x: number) => void } } }",
  anyFunction: "(props: any) => any",
  checked: "{ __vizeCheck: any; new (): { $props: { onSave?: (x: number) => void } } }",
  marker:
    "{ readonly __vizeComponentMarker: true; new (): { $props: { onSave?: (x: number) => void } }; readonly __vizeFallthroughProps?: { onClick?: (e: MouseEvent) => void } }",
  nativeRoot:
    '{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: { onClick?: (e: MouseEvent) => void; onSave?: () => void; "onFoo-bar"?: (x: 2) => void }; __vizeEmitProps?: { onSave?: (x: number) => void }; new (): { $props: { onSave?: (x: number) => void } } }',
  nativeRootAny:
    "{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: any; __vizeEmitProps?: {}; new (): { $props: {} } }",
  nativeRootNoEmit:
    "{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: { onClick?: (e: MouseEvent) => void }; new (): { $props: {} } }",
  nativeRootIndex:
    "{ [key: string]: any; readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: { onClick?: (e: MouseEvent) => void } }",
  nativeRootUnion:
    "{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: { onClick?: (e: MouseEvent) => void }; __vizeEmitProps?: {} } | { readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: { onClick?: (e: KeyboardEvent) => void }; __vizeEmitProps?: {} }",
  emitOnly: "{ __vizeEmitProps?: { onSave?: (x: number) => void; onClick?: (...a: []) => void } }",
  emitUnion:
    "{ __vizeEmitProps?: { onSave?: (x: number) => void } | { onSave?: (x: string) => void } }",
  indexProps: "new () => { $props: { [key: string]: unknown } }",
  anyProps: "new () => { $props: any }",
  neverProps: "new () => { $props: never }",
  intersection:
    "(new () => { $props: { onSave?: (x: number) => void } }) & { __vizeEmitProps?: { onSave?: (x: 1) => void } }",
  record: "Record<string, any>",
  overloaded:
    "{ new (): { $props: { onSave?: (x: number) => void } }; (props: { onClick?: () => void }): any }",
  kebabProps: 'new () => { $props: { "onFoo-bar"?: (x: number) => void; onFooBar?: never } }',
};
// [prop_key as generated, key type, raw key type]
const keys: [string, string, string | null][] = [
  ["onSave", '"onSave"', null],
  ["onClick", '"onClick"', null],
  ['"onUpdate:modelValue"', '"onUpdate:modelValue"', null],
  ["onFooBar", '"onFooBar"', '"onFoo-bar"'],
];
const ALIASES = ["library", "props", "emit", "native", "explicit"] as const;
const EVENT_ALIASES = ["pa", "fa", "sea", "rnf", "ret", "arr"] as const;

function shapeProgram(name: string, shape: string): string[] {
  const C = `C_${name}`;
  const p = (alias: string) => `_${name}_${alias}`;
  const lines = [`declare const ${C}: ${shape};`];
  const component = { component_ref: C, is_any: "$IsAny" };
  lines.push(
    format(INLINE.library, { ...component, library: p("library") }),
    format(INLINE.props, { ...component, props_box: p("props") }),
    format(INLINE.emit, { ...component, emit_props_box: p("emit") }),
    format(INLINE.native, { ...component, fallthrough_box: p("native") }),
    format(INLINE.explicit, { ...component, fallthrough_box: p("explicit") }),
    `type ${p("library2")} = $ListenerLibrary<typeof ${C}>;`,
    `type ${p("props2")} = $ListenerPropsBox<typeof ${C}>;`,
    `type ${p("emit2")} = $ListenerEmitPropsBox<typeof ${C}>;`,
    `type ${p("native2")} = $ListenerNativeFallthroughBox<typeof ${C}>;`,
    `type ${p("explicit2")} = $ListenerFallthroughBox<typeof ${C}>;`,
    ...ALIASES.map((a) => `type ${p(a)}_eq = Expect<Equal<${p(a)}, ${p(`${a}2`)}>>;`),
  );
  keys.forEach(([propKey, key, raw], index) => {
    const e = (alias: string) => p(`k${index}_${alias}`);
    const missing = raw ? format(INLINE.raw, { raw_key: raw }) : "unknown[]";
    const chain = format(INLINE.chain, { prop_key: propKey, missing_prop_args: missing });
    const keyed = (helper: string, contract: string) =>
      raw ? `${helper}Raw<${contract}, ${key}, ${raw}>` : `${helper}<${contract}, ${key}>`;
    const values = {
      library: p("library"),
      props_box: p("props"),
      emit_props_box: p("emit"),
      fallthrough_box: p("native"),
      prop_key: propKey,
      prop_listener_args: chain,
    };
    lines.push(
      format(INLINE.propArgs, { ...values, prop_args: e("pa") }),
      format(INLINE.fallthroughArgs, { ...values, fallthrough_args: e("fa") }),
      format(INLINE.staticEmit, { ...values, static_emit_args: e("sea") }),
      format(INLINE.nativeReturn, { ...values, native_return: e("rnf") }),
      format(INLINE.return, {
        ...values,
        return_type: e("ret"),
        prop_args: e("pa"),
        unresolved_return: e("rnf"),
      }),
      format(INLINE.arraysAlias, {
        ...values,
        arrays_type: e("arr"),
        arrays: format(INLINE.arrays, values),
      }),
      `type ${e("pa2")} = ${keyed("$ListenerPropArgs", `${p("library2")}, ${p("props2")}`)};`,
      `type ${e("fa2")} = ${keyed("$ListenerFallthroughArgs", p("native2"))};`,
      `type ${e("sea2")} = $ListenerStaticEmitArgs<${p("emit2")}, ${key}>;`,
      `type ${e("rnf2")} = $ListenerNativeReturn<${p("native2")}, ${key}>;`,
      `type ${e("ret2")} = $ListenerReturn<${e("pa2")}, ${e("rnf2")}, ${p("props2")}, ${key}>;`,
      `type ${e("arr2")} = $ListenerPropArrays<${p("props2")}, ${key}>;`,
      ...EVENT_ALIASES.map((a) => `type ${e(a)}_eq = Expect<Equal<${e(a)}, ${e(`${a}2`)}>>;`),
    );
  });
  return lines;
}

const prelude = [
  "export {};",
  "type $IsAny<T> = 0 extends (1 & T) ? true : false;",
  "type Equal<A, B> = (<G>() => G extends A ? 1 : 2) extends <G>() => G extends B ? 1 : 2 ? true : false;",
  "type Expect<T extends true> = T;",
];

function diagnostics(content: string, exactOptionalPropertyTypes: boolean): string[] {
  const fixture = fileURLToPath(import.meta.url);
  const options: ts.CompilerOptions = {
    strict: true,
    noEmit: true,
    target: ts.ScriptTarget.ESNext,
    lib: ["lib.esnext.d.ts", "lib.dom.d.ts"],
    types: [],
    exactOptionalPropertyTypes,
  };
  const host = ts.createCompilerHost(options);
  const readFile = host.readFile.bind(host);
  host.readFile = (file) => (file === fixture ? content : readFile(file));
  return ts.getPreEmitDiagnostics(ts.createProgram([fixture], options, host)).map((diagnostic) => {
    const text = ts.flattenDiagnosticMessageText(diagnostic.messageText, " ");
    return `TS${diagnostic.code} ${text}`;
  });
}

const equalities = (helperText: string) =>
  [
    ...prelude,
    helperText,
    ...Object.entries(shapes).flatMap(([name, shape]) => shapeProgram(name, shape)),
  ].join("\n");

void test("the generator's standalone text is the oracle these helpers replace", () => {
  for (const [name, template] of Object.entries(INLINE)) {
    assert.equal(
      generator.split(template).length - 1 > 0,
      true,
      `${name} is no longer the generator's inline text`,
    );
  }
});

void test("every shared listener helper computes the inline contract it replaces", () => {
  for (const exactOptionalPropertyTypes of [false, true]) {
    assert.deepEqual(
      diagnostics(equalities(helpers), exactOptionalPropertyTypes),
      [],
      `exactOptionalPropertyTypes=${exactOptionalPropertyTypes}`,
    );
  }
});

void test("the oracle rejects helpers that distribute or map homomorphically", () => {
  for (const [before, after] of [
    ["type $ListenerLibrary<C> = NoInfer<C> extends", "type $ListenerLibrary<C> = C extends"],
    [
      "type $ListenerPropsBox<C> = NoInfer<C> extends { new",
      "type $ListenerPropsBox<C> = C extends { new",
    ],
    ["[__K in $ListenerKeysOf<C> as", "[__K in keyof C as"],
  ] as const) {
    assert.equal(helpers.split(before).length, 2, before);
    const broken = diagnostics(equalities(helpers.replace(before, after)), false);
    assert.ok(
      broken.some((text) => text.startsWith("TS2344 ")),
      after,
    );
  }
});

// Generic components: the helper and the inline forms must report the same
// diagnostics for the same handlers, including when the component type itself
// is a type parameter and the contract stays deferred.
function usage(form: "inline" | "helper", component: string): string {
  const C = "component";
  const box = { component_ref: C, library: "Lib", props_box: "Box", prop_key: "onSave" };
  const chain = format(INLINE.chain, { prop_key: "onSave", missing_prop_args: "unknown[]" });
  const aliases =
    form === "inline"
      ? [
          format(INLINE.library, box),
          format(INLINE.props, box),
          format(INLINE.propArgs, { ...box, prop_args: "Args", prop_listener_args: chain }),
          format(INLINE.return, {
            ...box,
            return_type: "Ret",
            prop_args: "Args",
            unresolved_return: "any",
          }),
        ]
      : [
          `type Lib = $ListenerLibrary<typeof ${C}>;`,
          `type Box = $ListenerPropsBox<typeof ${C}>;`,
          'type Args = $ListenerPropArgs<Lib, Box, "onSave">;',
          'type Ret = $ListenerReturn<Args, any, Box, "onSave">;',
        ];
  return [
    ...prelude,
    helpers,
    `export function check<T extends string, K extends new () => { $props: { onSave?: (x: T) => boolean } }>(value: T, ${C}: ${component}) {`,
    ...aliases.map((line) => `  ${line}`),
    "  type Listener = unknown[] extends Args ? ((...args: any[]) => any) : ((...args: Args) => Ret);",
    "  const ok: Listener = (x: T) => x.length > 0;",
    "  const wrongArgument: Listener = (x: number) => x > 0;",
    "  const wrongReturn: Listener = (x: T) => x;",
    "  const payload: number = (null as unknown as Args)[0];",
    `  return [ok, wrongArgument, wrongReturn, payload, value, ${C}] as const;`,
    "}",
  ].join("\n");
}

void test("generic components keep identical listener diagnostics through the helpers", () => {
  for (const component of [
    "new () => { $props: { onSave?: (x: T, n: number) => boolean } }",
    "K",
  ]) {
    const inline = diagnostics(usage("inline", component), false);
    assert.deepEqual(diagnostics(usage("helper", component), false), inline, component);
    assert.ok(inline.length > 0, component);
  }
});
