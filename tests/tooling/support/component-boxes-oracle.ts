import assert from "node:assert/strict";
import fs from "node:fs";
import ts from "typescript";
import {
  format,
  helpers as gateHelpers,
  prelude as gatePrelude,
  repo,
  shapes as gateShapes,
} from "./emit-props-gate-fixture.ts";

// The per-use component boxes as the generator spells them inline. The shared
// `$Component*` helpers must equal each of them for every shape a template can bind.
const scope = "crates/vize_canon/src/virtual_ts/scope/";
export const BOX_TEMPLATES = {
  library: [
    `${scope}component_events/component_boxes.rs`,
    "type {library} = typeof {component_ref} extends {{ __vizeCheck: any }} | {{ readonly __vizeComponentMarker: true }} ? false : typeof {component_ref} extends (...args: any[]) => infer __R ? 0 extends (1 & __R) ? false : '__ctx' extends keyof __R ? true : false : false;",
  ],
  props: [
    `${scope}component_events/component_boxes.rs`,
    "type {props_box} = typeof {component_ref} extends {{ new (...args: any[]): {{ $props: infer __P }} }} ? [__P] : typeof {component_ref} extends (props: infer __P, ...args: any[]) => any ? [__P] : [];",
  ],
  emitProps: [
    `${scope}component_events/component_boxes.rs`,
    "type {emit_props_box} = typeof {component_ref} extends {{ __vizeEmitProps?: infer __EP }} ? [__EP] : [];",
  ],
  nativeFallthrough: [
    `${scope}component_events/native_fallthrough.rs`,
    'type {fallthrough_box} = typeof {component_ref} extends {{ readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: infer __F }} ? \\"__vizeEmitProps\\" extends keyof {{ [__K in keyof typeof {component_ref} as string extends __K ? never : number extends __K ? never : symbol extends __K ? never : __K]: unknown }} ? {is_any}<__F> extends true ? [] : [__F] : [] : [];',
  ],
  eventsResolver: [
    `${scope}component_event_navigation.rs`,
    "type __vize_events_resolver_{idx} = typeof {component_ref} extends {{ __vizeResolveEvents?: infer __F }} ? (__F extends (...args: any[]) => any ? __F : (props: any) => {component_events}<typeof {component_ref}>) : (props: any) => {component_events}<typeof {component_ref}>;",
  ],
} as const;

/** Each box and the shared helper that replaces it. */
export const BOX_HELPERS = {
  library: "$ComponentLibraryFlag",
  props: "$ComponentPropsBox",
  emitProps: "$ComponentEmitPropsBox",
  nativeFallthrough: "$ComponentNativeFallthroughBox",
  eventsResolver: "$ComponentEventsResolver",
} as const;

// The listener-contract shapes (#12475), spelled with their own constructors.
const listenerPrelude = [
  "type LCtor<P> = new <A = unknown>(props?: A & Partial<P>, ...args: any[]) => { $props: P };",
  "type LSfc<P, EP, F = never> = { readonly __vizeComponentMarker: true; __vizeEmitProps?: EP } & ([F] extends [never] ? {} : { readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: F }) & LCtor<P>;",
  "type LFn<A extends unknown[], R = void> = (...args: A) => R;",
];
const emitProps =
  "{ onSelect?: LFn<[id: number, label: string]>; 'onUpdate:modelValue'?: LFn<[value: string]>; onRowClick?: LFn<[row: { id: number }]> }";
const listenerShapes: Record<string, string> = {
  sfc: `LSfc<{ label: string } & ${emitProps}, ${emitProps}>`,
  propListener:
    "LSfc<{ onSelect?: (id: number) => number; onTap?: LFn<[value: string]> | LFn<[value: string]>[] }, {}>",
  arrayOnly: "LSfc<{ onSelect?: LFn<[id: number]>[] }, {}>",
  anyProps: "LSfc<any, {}>",
  nativeRoot:
    "LSfc<{ name?: string }, {}, { onFocus?: LFn<[event: FocusEvent], boolean>; onSelect?: LFn<[]> }>",
  nativeAnyRoot: "LSfc<{ name?: string }, {}, any>",
  nativeEmitRoot: "LSfc<{ name?: string }, {}, { onFocus?: LFn<[]> }> & { __vizeEmitProps?: {} }",
  loose: "LSfc<{ [key: `on${string}`]: LFn<any[]> | undefined }, {}>",
  kebab: "LSfc<{ 'onRow-click'?: LFn<[row: number]> }, {}>",
  union: `LSfc<${emitProps}, ${emitProps}> | LSfc<{ onSelect?: LFn<[id: string]> }, {}>`,
  nativeUnion: `LSfc<{ a?: 1 }, {}, { onFocus?: LFn<[]> }> | LSfc<{ b?: 2 }, {}, { onBlur?: LFn<[]> }>`,
  optional: `LSfc<${emitProps}, ${emitProps}> | undefined`,
  resolvesEvents:
    "{ __vizeResolveEvents?: <T>(props: { value: T }) => { pick: [value: T] } } & LCtor<{ value: unknown }>",
  resolvesEventsValue: "{ __vizeResolveEvents?: { pick: [] } } & LCtor<{}>",
  resolvesEventsUnion:
    "({ __vizeResolveEvents?: (props: any) => { a: [] } } & LCtor<{}>) | ({ __vizeResolveEvents?: (props: any) => { b: [] } } & LCtor<{}>)",
  // Real component exports carry a string index: a union reads its members' shared
  // properties through it, so `keyof` must not distribute.
  indexedNative:
    "LSfc<{ a?: 1 }, {}, { onFocus?: LFn<[event: FocusEvent]> }> & Record<string, any>",
  indexedNativeUnion:
    "(LSfc<{ a?: 1 }, { onPick?: LFn<[]> }, { onFocus?: LFn<[event: FocusEvent]> }> & Record<string, any>) | ({ readonly __vizeComponentMarker: true; readonly __vizeNativeFallthroughRoot: true; readonly __vizeFallthroughProps?: { onFocus?: LFn<[event: FocusEvent]> } } & LCtor<{ b?: 2 }> & Record<string, any>)",
  indexedEmitUnion:
    "(LSfc<{ a?: 1 }, { onPick?: LFn<[id: number]> }> & Record<string, any>) | (LCtor<{ b?: 2 }> & { [key: string]: unknown })",
  indexedEventsUnion:
    "({ __vizeResolveEvents?: (props: any) => { a: [] } } & LCtor<{}> & Record<string, any>) | (LCtor<{}> & Record<string, any>)",
};
export const SHAPES: Record<string, string> = { ...gateShapes, ...listenerShapes };

/** The shared helpers' declarations, as they appear in the SDK asset. */
export function helperDeclarations(asset: string): string {
  const lines = asset
    .split("\n")
    .filter((line) =>
      Object.values(BOX_HELPERS).some((name) => line.startsWith(`export type ${name}<`)),
    )
    .map((line) => line.replace(/^export /u, ""));
  assert.equal(lines.length, Object.keys(BOX_HELPERS).length, "one declaration per helper");
  return lines.join("\n");
}

export function boxCases(declarations: string): { content: string; pairs: [string, string][] } {
  const lines = [
    ...gatePrelude,
    gateHelpers,
    ...listenerPrelude,
    // Both sides pass the same argument; the stub keeps it exact.
    "type $ComponentEvents<C> = { readonly events: [C] };",
    declarations,
  ];
  const pairs: [string, string][] = [];
  for (const [index, shape] of Object.values(SHAPES).entries()) {
    const p = `B${index}`;
    const values: Record<string, string> = {
      component_ref: `${p}_C`,
      library: `${p}_library`,
      props_box: `${p}_props`,
      emit_props_box: `${p}_emitProps`,
      fallthrough_box: `${p}_nativeFallthrough`,
      is_any: "$IsAny",
      idx: String(index),
      component_events: "$ComponentEvents",
    };
    lines.push(`declare const ${p}_C: ${shape};`);
    for (const [name, [, template]] of Object.entries(BOX_TEMPLATES)) {
      const inline = name === "eventsResolver" ? `__vize_events_resolver_${index}` : `${p}_${name}`;
      lines.push(format(template, values));
      const helper = BOX_HELPERS[name as keyof typeof BOX_HELPERS];
      lines.push(`type ${p}_h_${name} = ${helper}<typeof ${p}_C>;`);
      lines.push(`type ${p}_eq_${name} = Expect<Equal<${inline}, ${p}_h_${name}>>;`);
      pairs.push([inline, `${p}_h_${name}`]);
    }
  }
  return { content: lines.join("\n"), pairs };
}

export function checkBoxes(
  content: string,
  pairs: [string, string][],
  exactOptionalPropertyTypes: boolean,
) {
  const fixture = "/component-boxes-oracle.ts";
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
  const fileExists = host.fileExists.bind(host);
  host.readFile = (file) => (file === fixture ? content : readFile(file));
  host.fileExists = (file) => file === fixture || fileExists(file);
  const getSourceFile = host.getSourceFile.bind(host);
  host.getSourceFile = (file, language) =>
    file === fixture ? ts.createSourceFile(file, content, language) : getSourceFile(file, language);
  const program = ts.createProgram([fixture], options, host);
  const diagnostics = ts.getPreEmitDiagnostics(program).map((diagnostic) => {
    const line = diagnostic.file?.getLineAndCharacterOfPosition(diagnostic.start ?? 0).line ?? 0;
    return `TS${diagnostic.code} ${ts.flattenDiagnosticMessageText(diagnostic.messageText, " ")} @ ${content.split("\n")[line]?.slice(0, 160)}`;
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
    .filter(([inline, helper]) => printed.get(inline) !== printed.get(helper))
    .map(([inline, helper]) => `${inline}: ${printed.get(inline)} !== ${printed.get(helper)}`);
  return { diagnostics, differences };
}

/// The current repository bytes: the boxes and their templates are this layer's own text.
export const readSource = (path: string) => fs.readFileSync(new URL(path, repo), "utf8");
