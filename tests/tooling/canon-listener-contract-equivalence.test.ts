import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";
import { beforeCurrentListenerLibTypeQualification } from "../../tools/support/compat/content-mapper/lib-type-qualification-listeners-transition.ts";

// The per-event alias chain the generator still emits outside the shared
// contract is the oracle: `$ListenerContract` must produce exactly the chain's
// event, listener, handler and array types, printed the same, for every
// component shape a closed template can bind.
const repo = new URL("../../", import.meta.url);
const read = (path: string) =>
  beforeCurrentListenerLibTypeQualification(path, fs.readFileSync(new URL(path, repo), "utf8"));
const events = "crates/vize_canon/src/virtual_ts/scope/component_events";
const declarations = "crates/vize_canon/src/virtual_ts/helpers/declarations/";

// Format strings exactly as the generator spells them.
const TEMPLATES = {
  library: [
    `${events}.rs`,
    "type {library} = typeof {component_ref} extends {{ __vizeCheck: any }} | {{ readonly __vizeComponentMarker: true }} ? false : typeof {component_ref} extends (...args: any[]) => infer __R ? 0 extends (1 & __R) ? false : '__ctx' extends keyof __R ? true : false : false;",
  ],
  props: [
    `${events}.rs`,
    "type {props_box} = typeof {component_ref} extends {{ new (...args: any[]): {{ $props: infer __P }} }} ? [__P] : typeof {component_ref} extends (props: infer __P, ...args: any[]) => any ? [__P] : [];",
  ],
  emitProps: [
    `${events}.rs`,
    "type {emit_props_box} = typeof {component_ref} extends {{ __vizeEmitProps?: infer __EP }} ? [__EP] : [];",
  ],
  nativeBox: [
    `${events}/native_fallthrough.rs`,
    'type {fallthrough_box} = typeof {component_ref} extends {{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: infer __F }} ? \\"__vizeEmitProps\\" extends keyof {{ [__K in keyof typeof {component_ref} as string extends __K ? never : number extends __K ? never : symbol extends __K ? never : __K]: unknown }} ? {is_any}<__F> extends true ? [] : [__F] : [] : [];',
  ],
  propArgs: [
    `${events}.rs`,
    "type {prop_args} = {library} extends true ? unknown[] : {props_box} extends infer __B ? __B extends [infer __P] ? {prop_listener_args} : unknown[] : never;",
  ],
  nativeArgs: [
    `${events}/native_fallthrough.rs`,
    "type {fallthrough_args} = {fallthrough_box} extends infer __B ? __B extends [infer __F] ? NonNullable<__F> extends infer __P ? ({prop_listener_args}) extends infer __A extends unknown[] ? (__A extends [] ? unknown[] : __A) : unknown[] : unknown[] : unknown[] : never;",
  ],
  staticArgs: [
    `${events}.rs`,
    "type {static_emit_args} = {emit_props_box} extends infer __B ? __B extends [infer __EP] ? __EP extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : unknown[] : unknown[] : never;",
  ],
  emitArgs: [
    `${events}.rs`,
    "type {emit_args} = unknown[] extends {inferred} ? {static_emit_args} : {inferred};",
  ],
  args: [
    `${events}.rs`,
    "type {args_type} = {loose_runtime_args}unknown[] extends {inferred} ? (unknown[] extends {prop_args} ? (unknown[] extends {emit_args} ? {unresolved_args} : {emit_args}) : {prop_args}) : {inferred};",
  ],
  event: [
    `${events}.rs`,
    "type {event_type} = {args_type} extends infer __A extends unknown[] ? [__A] extends [[]] ? any : unknown[] extends __A ? any : __A[0] : never;",
  ],
  nativeReturn: [
    `${events}/native_fallthrough.rs`,
    "type {native_return} = {fallthrough_box} extends infer __B ? __B extends [infer __F] ? NonNullable<__F> extends {{ {prop_key}?: infer __H }} ? Extract<NonNullable<__H>, (...args: any[]) => any> extends (...args: any[]) => infer __R ? __R : any : any : any : never;",
  ],
  publicReturn: [`${events}/listener_return.rs`, "type {return_type}_public = {public_return};"],
  return: [
    `${events}/listener_return.rs`,
    "type {return_type} = unknown[] extends {static_emit_args} ? {return_type}_public : 0 extends (1 & {return_type}_public) ? void : {return_type}_public;",
  ],
  members: [
    `${events}/prop_array_share.rs`,
    "type {props_members} = {props_box} extends infer __B ? __B extends [infer __P] ? [__P] : never : never;",
  ],
  arraysBody: [
    `${events}/prop_array_share.rs`,
    "{is_any}<__P> extends true ? never : __P extends {{ {prop_key}?: infer __H }} ? Extract<NonNullable<__H>, readonly unknown[]> : never",
  ],
  arrays: [
    `${events}/prop_array_share.rs`,
    "type {arrays_type} = {props_members} extends infer __B ? __B extends [infer __P] ? {arrays} : never : never;",
  ],
  listener: [
    `${events}/listener_return.rs`,
    "unknown[] extends {args_type} ? ((...args: any[]) => any) : ((...args: {listener_args_type}) => {return_type})",
  ],
  handler: [
    `${events}.rs`,
    "unknown[] extends {args_type} ? ((...args: any[]) => any) : {listener_type}",
  ],
  looseHandler: [`${events}.rs`, "unknown[] extends {args_type} ? unknown : {listener_type}"],
  contract: [
    `${events}/listener_contract.rs`,
    "type {alias} = {helper}<{library}, {props_box}, {emit_props_box}, {fallthrough_box}, {listener_key}, {raw_listener_key}, {inferred_args}, {loose}>;",
  ],
} as const;

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

/** The shared module's declarations as plain top-level ones. */
const shared = (file: string) => {
  const text = read(declarations + file);
  return text
    .slice(text.indexOf("{") + 1, text.lastIndexOf("}"))
    .replaceAll(/^ {2}export /gmu, "")
    .replaceAll(/^ {2}/gmu, "");
};

const prelude = [
  "export {};",
  "type Equal<A, B> = (<G>() => G extends A ? 1 : 2) extends <G>() => G extends B ? 1 : 2 ? true : false;",
  "type Expect<T extends true> = T;",
  "type $IsAny<T> = 0 extends (1 & T) ? true : false;",
  shared("shared-listener-args.d.ts.txt"),
  "type Ctor<P> = new <A = unknown>(props?: A & Partial<P>, ...args: any[]) => { $props: P };",
  "type Sfc<P, EP, F = never> = { readonly __vizeComponentMarker: true; readonly __vizeRawProps?: P; __vizeEmitProps?: EP } & ([F] extends [never] ? {} : { readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: F }) & Ctor<P & EP>;",
  "type Fn<A extends unknown[], R = void> = (...args: A) => R;",
];

const emitProps =
  "{ onSelect?: Fn<[id: number, label: string], any>; 'onUpdate:modelValue'?: Fn<[value: string], any>; onRowClick?: Fn<[row: { id: number }], any> }";
const shapes: Record<string, string> = {
  any: "any",
  unknown: "unknown",
  never: "never",
  empty: "{}",
  sfc: `Sfc<{ label: string }, ${emitProps}>`,
  propListener:
    "Sfc<{ onSelect?: (id: number) => number; onTap?: Fn<[value: string]> | Fn<[value: string]>[] }, {}>",
  arrayOnly: "Sfc<{ onSelect?: Fn<[id: number]>[] }, {}>",
  anyProps: "Sfc<any, {}>",
  nativeRoot:
    "Sfc<{ name?: string }, {}, { onFocus?: Fn<[event: FocusEvent], boolean>; onSelect?: Fn<[]> }>",
  nativeAnyRoot: "Sfc<{ name?: string }, {}, any>",
  loose: "Sfc<{ [key: `on${string}`]: Fn<any[]> | undefined }, {}>",
  kebab: "Sfc<{ 'onRow-click'?: Fn<[row: number]> }, {}>",
  plain: "(new (...args: any[]) => { $props: { onSelect?: Fn<[id: number]> } })",
  functional: "((props: { onSelect?: Fn<[id: string]> }, ctx: any) => any)",
  library:
    "(<T>(props: { value: T; onSelect?: Fn<[value: T]> }, ctx?: any) => { __ctx?: { props: { value: T; onSelect?: Fn<[value: T]> } } })",
  union: `Sfc<${emitProps}, ${emitProps}> | Sfc<{ onSelect?: Fn<[id: string]> }, {}>`,
  optional: `Sfc<${emitProps}, ${emitProps}> | undefined`,
};
const keys: [listener: string, raw: string][] = [
  ['"onSelect"', "never"],
  ['"onUpdate:modelValue"', '"onUpdate:model-value"'],
  ['"onRowClick"', '"onRow-click"'],
  ['"onFocus"', "never"],
  ['"onTap"', "never"],
];
const inferred = [
  undefined,
  "[id: number, label: string]",
  "unknown[]",
  "any",
  "[] | [id: number]",
  "[]",
];
const loose = [undefined, "true", "false", "boolean"];

function usage(
  index: number,
  shape: string,
  native: boolean,
  key: [string, string],
  inferredArgs?: string,
  looseFlag?: string,
): string[] {
  const p = `L${index}`;
  const [listener_key, raw_listener_key] = key;
  const prop_key = listener_key.includes(":") ? listener_key : listener_key.slice(1, -1);
  const values: Record<string, string> = {
    component_ref: `${p}_C`,
    library: `${p}_library`,
    props_box: `${p}_props`,
    emit_props_box: `${p}_emit_props`,
    fallthrough_box: `${p}_fallthrough`,
    is_any: "$IsAny",
    prop_key,
    prop_listener_args: `$ListenerArgs<__P, ${listener_key}, ${raw_listener_key}>`,
    prop_args: `${p}_prop_args`,
    fallthrough_args: `${p}_fallthrough_args`,
    static_emit_args: `${p}_static`,
    emit_args: `${p}_emit_args`,
    inferred: inferredArgs === undefined ? "unknown[]" : `${p}_inferred`,
    loose_runtime_args: looseFlag === undefined ? "" : `${p}_loose extends true ? unknown[] : `,
    unresolved_args: native ? `${p}_fallthrough_args` : "unknown[]",
    args_type: `${p}_args`,
    event_type: `${p}_event`,
    native_return: `${p}_native_return`,
    return_type: `${p}_return`,
    unresolved_return: native ? `${p}_native_return` : "any",
    props_members: `${p}_members`,
    arrays_type: `${p}_arrays`,
    listener_args_type: `${p}_args`,
    listener_type: `${p}_listener`,
    alias: `${p}_contract`,
    helper: "$ListenerContract",
    listener_key,
    raw_listener_key,
    inferred_args: inferredArgs === undefined ? "unknown[]" : `${p}_inferred`,
    loose: looseFlag === undefined ? "false" : `${p}_loose`,
  };
  values.public_return = `unknown[] extends ${values.prop_args} ? ${values.unresolved_return} : ${values.props_box} extends infer __B ? __B extends [infer __P] ? (__P extends { ${values.prop_key}?: (...args: any[]) => infer __R } ? __R : any) : any : never`;
  values.arrays = format(TEMPLATES.arraysBody[1], values);
  const f = (name: keyof typeof TEMPLATES) => format(TEMPLATES[name][1], values);
  return [
    `declare const ${p}_C: ${shape};`,
    f("library"),
    f("props"),
    f("emitProps"),
    native ? f("nativeBox") : `type ${p}_fallthrough = [];`,
    f("propArgs"),
    ...(native ? [f("nativeArgs"), f("nativeReturn")] : []),
    ...(inferredArgs === undefined ? [] : [`type ${p}_inferred = ${inferredArgs};`]),
    ...(looseFlag === undefined ? [] : [`type ${p}_loose = ${looseFlag};`]),
    f("staticArgs"),
    f("emitArgs"),
    f("args"),
    f("event"),
    f("publicReturn"),
    f("return"),
    f("members"),
    f("arrays"),
    `type ${p}_listener = ${f("listener")};`,
    `type ${p}_handler = ${f("handler")};`,
    `type ${p}_loose_handler = ${f("looseHandler")};`,
    f("contract"),
    ...(["event", "listener", "handler", "looseHandler", "arrays"] as const).map(
      (member) => `type ${p}_c_${member} = ${p}_contract["${member}"];`,
    ),
    ...(
      [
        ["event", "event"],
        ["listener", "listener"],
        ["handler", "handler"],
        ["loose_handler", "looseHandler"],
        ["arrays", "arrays"],
      ] as const
    ).map(
      ([chain, member]) =>
        `type ${p}_eq_${chain} = Expect<Equal<${p}_${chain}, ${p}_c_${member}>>;`,
    ),
  ];
}

function cases(): { lines: string[]; pairs: [string, string][] } {
  const lines = [...prelude];
  const pairs: [string, string][] = [];
  let index = 0;
  for (const shape of Object.values(shapes))
    for (const native of [false, true])
      for (const key of keys)
        for (const [i, inferredArgs] of inferred.entries()) {
          const looseFlag = loose[i % loose.length];
          lines.push(...usage(index, shape, native, key, inferredArgs, looseFlag));
          for (const [chain, member] of [
            ["event", "event"],
            ["listener", "listener"],
            ["handler", "handler"],
            ["loose_handler", "looseHandler"],
            ["arrays", "arrays"],
          ])
            pairs.push([`L${index}_${chain}`, `L${index}_c_${member}`]);
          index++;
        }
  return { lines, pairs };
}

function check(content: string, pairs: [string, string][], exactOptionalPropertyTypes: boolean) {
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
  const program = ts.createProgram([fixture], options, host);
  const diagnostics = ts.getPreEmitDiagnostics(program).map((diagnostic) => {
    const line = diagnostic.file?.getLineAndCharacterOfPosition(diagnostic.start ?? 0).line ?? 0;
    return `TS${diagnostic.code} ${ts.flattenDiagnosticMessageText(diagnostic.messageText, " ")} @ ${content.split("\n")[line]?.slice(0, 140)}`;
  });
  const checker = program.getTypeChecker();
  const source = program.getSourceFile(fixture)!;
  const printed = new Map<string, string>();
  source.forEachChild((node) => {
    if (ts.isTypeAliasDeclaration(node))
      printed.set(
        node.name.text,
        checker.typeToString(
          checker.getTypeAtLocation(node.name),
          undefined,
          ts.TypeFormatFlags.NoTruncation,
        ),
      );
  });
  const differences = pairs
    .filter(([chain, contract]) => printed.get(chain) !== printed.get(contract))
    .map(([chain, contract]) => `${chain}: ${printed.get(chain)} !== ${printed.get(contract)}`);
  return { diagnostics, differences };
}

void test("the oracle reads the generator's own text", () => {
  for (const [name, [file, template]] of Object.entries(TEMPLATES))
    assert.ok(read(file).includes(template), `${name} is no longer the generator's text`);
});

void test("the shared contract has exactly the chain's types and printed text", () => {
  const { lines, pairs } = cases();
  assert.ok(pairs.length >= 1_000, `${pairs.length} pairs`);
  for (const exactOptionalPropertyTypes of [false, true]) {
    const { diagnostics, differences } = check(lines.join("\n"), pairs, exactOptionalPropertyTypes);
    assert.deepEqual(diagnostics, [], `exactOptionalPropertyTypes=${exactOptionalPropertyTypes}`);
    assert.deepEqual(differences, [], `exactOptionalPropertyTypes=${exactOptionalPropertyTypes}`);
  }
});

void test("declared emits use void without widening authored listener returns", () => {
  const lines = [
    ...prelude,
    "type Synthesized = $ListenerContract<false, [{ onSave?: Fn<[value: string], any> }], [{ onSave?: Fn<[value: string], any> }], [], 'onSave', never, unknown[], false>['listener'];",
    "type Authored = $ListenerContract<false, [{ onSave?: Fn<[value: string], number> }], [{ onSave?: Fn<[value: string], any> }], [], 'onSave', never, unknown[], false>['listener'];",
    "type Unresolved = $ListenerContract<false, [{ onSave?: Fn<[value: string], any> }], [], [], 'onSave', never, unknown[], false>['listener'];",
    "type SynthesizedIsVoid = Expect<Equal<Synthesized, Fn<[value: string], void>>>;",
    "type AuthoredKeepsNumber = Expect<Equal<Authored, Fn<[value: string], number>>>;",
    "type UnresolvedKeepsAny = Expect<Equal<Unresolved, Fn<[value: string], any>>>;",
  ];
  for (const exactOptionalPropertyTypes of [false, true]) {
    const { diagnostics } = check(lines.join("\n"), [], exactOptionalPropertyTypes);
    assert.deepEqual(diagnostics, [], `exactOptionalPropertyTypes=${exactOptionalPropertyTypes}`);
  }
});

void test("the oracle rejects a contract that changes a listener", () => {
  const { lines, pairs } = cases();
  for (const [name, from, to] of [
    [
      "drops the inferred payload",
      "unknown[] extends __Inferred ? (",
      "unknown[] extends unknown[] ? (",
    ],
    [
      "distributes the event over a union",
      "[__Args] extends [infer __A extends unknown[]]",
      "__Args extends infer __A extends unknown[]",
    ],
    [
      "widens the authored listener return",
      "0 extends (1 & __Public) ? void : __Public",
      "0 extends (1 & __Public) ? any : __Public",
    ],
  ] as const) {
    const broken = lines.map((line) => line.replace(from, to)).join("\n");
    assert.notEqual(broken, lines.join("\n"), name);
    const { diagnostics, differences } = check(broken, pairs, false);
    assert.ok(diagnostics.length > 0 || differences.length > 0, name);
  }
});
