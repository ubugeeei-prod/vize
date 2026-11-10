import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import ts from "typescript";
import { beforeListenerLibTypeQualification } from "../../../tools/support/compat/content-mapper/lib-type-qualification-listeners-transition.ts";
import { emitGateOracle } from "./emit-props-gate-oracle.ts";

// Shared by the emit-props gate oracles: the generator's own templates, the
// component shapes a closed template can bind, and one TypeScript check.
export const repo = new URL("../../../", import.meta.url);
export const read = (path: string) =>
  beforeListenerLibTypeQualification(path, fs.readFileSync(new URL(path, repo), "utf8"));
export const oracle = emitGateOracle(read);
export const declarations = "crates/vize_canon/src/virtual_ts/helpers/declarations/";
export const inference = read(`${declarations}emit-inference.d.ts.txt`);
export const sharedInference = read(`${declarations}shared-emit-inference.d.ts.txt`);
export const ownership = ["$ComponentMetadataKeys", "$ComponentOwnKeys", "$UnownedComponentAttrs"]
  .map((name) => {
    const line = read("crates/vize_canon/src/virtual_ts/component_prop_helpers.txt")
      .split("\n")
      .find((line) => line.startsWith(`type ${name}<`));
    assert.ok(line, name);
    return line;
  })
  .join("\n");
export const helpersFor = (emitInference: string) =>
  [
    read(`${declarations}core-types.d.ts.txt`).trimEnd(),
    "type $PublicComponentInputProps = { hidden?: boolean };",
    read(`${declarations}emit-overloads.d.ts.txt`).trimEnd(),
    read(`${declarations}emit-props.d.ts.txt`).trimEnd(),
    ownership,
    emitInference.trimEnd(),
  ].join("\n");
export const helpers = helpersFor(inference);

export const scope = "crates/vize_canon/src/virtual_ts/scope/";
export const events = read(`${scope}component_events.rs`);
export const generic = read(`${scope}component_events/generic_inference.rs`);
export const gate = read(`${scope}component_events/emit_props_gate.rs`);

// Format strings exactly as the generator spells them.
export const TEMPLATES = {
  library: [
    events,
    "type {library} = typeof {component_ref} extends {{ __vizeCheck: any }} | {{ readonly __vizeComponentMarker: true }} ? false : typeof {component_ref} extends (...args: any[]) => infer __R ? 0 extends (1 & __R) ? false : '__ctx' extends keyof __R ? true : false : false;",
  ],
  resolver: [
    generic,
    "type {resolver_type} = {library} extends true ? (component: typeof {component_ref}) => typeof {component_ref} : {emit_props_factory}<typeof {component_ref}>;",
  ],
  direct: [
    generic,
    "const {emit_props} = (undefined as unknown as {resolver_type})({component_ref})(",
  ],
  gated: [
    gate,
    "const __vize_emit_factory_{usage_idx} = (undefined as unknown as __vize_emit_resolver<{library}, typeof {component_ref}>)({component_ref}); const {emit_props} = (undefined as unknown as __vize_emit_input<{library}, typeof {component_ref}, typeof __vize_emit_factory_{usage_idx}>)(",
  ],
  inferred: [generic, oracle.current.inferred.replace("? __L :", "? {loose_args} :")],
  guardedDirect: [generic, "if ({guard}) return (undefined as unknown as {resolver_type})({})("],
  guardedFactory: [
    gate,
    "const __vize_emit_factory_{usage_idx} = (undefined as unknown as __vize_emit_resolver<{library}, typeof {component_ref}>)({component_ref});",
  ],
  guardedGated: [
    gate,
    "if ({guard}) return (undefined as unknown as __vize_emit_input<{library}, typeof {component_ref}, typeof __vize_emit_factory_{usage_idx}>)(",
  ],
} as const;

export const aliasSource = gate.match(/const GATE_ALIASES: &str = "((?:[^"\\]|\\.)*)";/u);
assert.ok(aliasSource, "GATE_ALIASES");
export const GATE_ALIASES = aliasSource[1]!.replaceAll("\\n", "\n").replaceAll('\\"', '"');

/** Rust `format!` semantics for named placeholders. */
export function format(template: string, values: Record<string, string>): string {
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

// Generated SFC exports, as the projection declares them.
export const prelude = [
  "export {};",
  "type Equal<A, B> = (<G>() => G extends A ? 1 : 2) extends <G>() => G extends B ? 1 : 2 ? true : false;",
  "type Expect<T extends true> = T;",
  "type Ctor<P, E> = new <A = unknown>(props?: A & Partial<P>, ...args: any[]) => { $props: P; $emit: E };",
  "type Sfc<P, E, F = {}> = { readonly __vizeComponentMarker: true; readonly __vizeRawProps?: P; readonly __vizeFallthroughProps?: F; __vizeEmitProps?: __EmitProps<E> } & Ctor<P, E>;",
  "type Save<T> = (event: 'save', value: T) => void;",
  "declare const columns: { id: string; type: string }[];",
  "declare const id: string;",
  "declare const pick: boolean;",
];

export const sfc =
  "Sfc<{ value?: string; items?: { a: number }[] }, { (event: 'save', value: number): void; (event: 'update:modelValue', value: string): void }, { onClick?: (event: MouseEvent) => void }>";
export const loose = "Sfc<{ column?: { id: string } }, (event: string, ...args: any[]) => void>";
export const required = "Sfc<{ value: string }, Save<string>>";
export const unmarked = "Ctor<{ value?: string }, Save<boolean>>";
export const sameEmit = "Sfc<{ value: string; other?: number }, Save<string>>";
export const plain =
  "(new (...args: any[]) => { $props: { value?: string; onSave?: (value: number) => void }; $emit: Save<number> })";
export const functional =
  "((props: { value?: string; onSave?: (value: number) => void }, ctx: any) => any)";
export const library =
  "(<T>(props: { value: T; onSave?: (value: T) => void }, ctx?: any) => { __ctx?: { props: { value: T; onSave?: (value: T) => void } } })";
export const resolvesEmits =
  "({ readonly __vizeComponentMarker: true; __vizeCheck: <T>(props: { value: T }) => void; __vizeResolveProps?: <T>(props: { value: T }) => { value: T }; __vizeResolveEmitProps?: <T>(props: Partial<{ value: T }> & Record<string, unknown>) => __EmitProps<Save<T>> } & Ctor<{ value: unknown }, Save<unknown>>)";
export const resolvesProps =
  "({ readonly __vizeComponentMarker: true; __vizeCheck: <T>(props: { value: T }) => void; __vizeResolveProps?: <T>(props: { value: T; onSave?: (value: T) => void }) => { value: T; onSave?: (value: T) => void } } & Ctor<{ value: unknown }, (event: string, ...args: any[]) => void>)";
export const columnsUnion = Array.from(
  { length: 6 },
  (_, index) =>
    `Sfc<{ column?: { id: string }; isStacked?: boolean }, (event: 'headerWheel', value: ${index}) => void>`,
).join(" | ");

export const shapes: Record<string, string> = {
  any: "any",
  unknown: "unknown",
  never: "never",
  string: "string",
  empty: "{}",
  record: "Record<string, any>",
  sfc,
  loose,
  required,
  unmarked,
  plain,
  overloaded:
    "{ new (props: { value?: 1 }): { $emit: Save<1> }; new (props: { value?: 2 }): { $emit: Save<2> } }",
  abstract: "(abstract new () => { $emit: Save<number> })",
  noEmit: "(new () => { $props: {} })",
  functional,
  library,
  resolvesEmits,
  resolvesProps,
  resolverOnly: "{ __vizeResolveEmitProps?: <T>(props: { value: T }) => __EmitProps<Save<T>> }",
  resolverUndefined: `{ __vizeResolveEmitProps?: undefined } & ${unmarked}`,
  resolverAny: `{ __vizeResolveEmitProps?: any } & ${unmarked}`,
  resolverNever: `{ __vizeResolveEmitProps?: never } & ${unmarked}`,
  intersection: `${plain} & { __vizeEmitProps?: { onSave?: (value: 1) => void } }`,
  deck: `${sfc} | ${loose} | ${required}`,
  specificUnion: `${sfc} | ${unmarked}`,
  sameEmitUnion: `${required} | ${sameEmit}`,
  constructorUnion: `${unmarked} | ${plain}`,
  emptyUnion: `${sfc} | ${functional} | undefined`,
  genericUnion: `${sfc} | ${resolvesEmits}`,
  genericPair: `${resolvesEmits} | ${resolvesProps}`,
  functionalUnion: `${sfc} | ${functional}`,
  libraryUnion: `${library} | ${sfc}`,
  optional: `${sfc} | undefined`,
  nativeUnion: `${sfc} | string`,
  columns: columnsUnion,
};

// Each literal stays on single lines, every one under a generated ignore.
export const literals: Record<string, string[]> = {
  text: ['"value": "text"'],
  wrong: ['"value": 1', '"class": { _shadow: true }'],
  nested: ['"items": [{ a: 1 }]'],
  find: ['"column": columns.find(c => c.id === id)!', '"isStacked": columns.length > 1'],
  callback: ['"onSave": (value) => { void value; }', '"value": "text"'],
};
export const keys = ["onSave", "onHeaderWheel"];
// Every literal for the shapes whose routes read or reduce it; one elsewhere.
export const fullLiterals = new Set(["sfc", "resolvesEmits", "library", "deck", "genericUnion"]);
export const literalsFor = (name: string) =>
  fullLiterals.has(name) ? Object.values(literals) : [literals.find!];

export function usage(
  name: string,
  shape: string,
  literal: string[],
  index: number,
  wrap: (lines: string[]) => string[] = (lines) => lines,
  inferredTemplate: string = TEMPLATES.inferred[1],
): string[] {
  const component_ref = `C_${name}_${index}`;
  const library = `${component_ref}_library`;
  const resolver_type = `${component_ref}_resolver`;
  const body = () => [
    ...literal.flatMap((entry) => [
      "  // @ts-ignore Inference-only prop; the canonical component-props call owns diagnostics.",
      `  ${entry},`,
    ]),
    "});",
  ];
  const direct = `${component_ref}_direct`;
  const gated = `${component_ref}_gated`;
  const values = {
    component_ref,
    library,
    resolver_type,
    emit_props_factory: "$EmitPropsFactory",
    loose_args: "__L",
  };
  const inferred = (emit_props: string, prop_key: string, alias: string) =>
    format(inferredTemplate, {
      ...values,
      emit_props,
      prop_key,
      inferred_args: alias,
    });
  return [
    `const ${component_ref} = undefined as unknown as ${shape};`,
    ...wrap([
      format(TEMPLATES.library[1], values),
      format(TEMPLATES.resolver[1], values),
      "// @ts-ignore Unmapped inference call.",
      format(TEMPLATES.direct[1], { ...values, emit_props: direct }) + "{",
      ...body(),
      "// @ts-ignore Unmapped inference call.",
      format(TEMPLATES.gated[1], { ...values, emit_props: gated, usage_idx: `${name}_${index}` }) +
        "{",
      ...body(),
      `type ${component_ref}_eq = Expect<Equal<typeof ${direct}, typeof ${gated}>>;`,
      ...keys.flatMap((key, k) => [
        inferred(direct, key, `${component_ref}_k${k}_direct`),
        inferred(gated, key, `${component_ref}_k${k}_gated`),
        `type ${component_ref}_k${k}_eq = Expect<Equal<${component_ref}_k${k}_direct, ${component_ref}_k${k}_gated>>;`,
      ]),
    ]),
  ];
}

/** A `v-if` use: the gated factory runs before the guard, the direct call inside it. */
export function guardedUsage(
  name: string,
  shape: string,
  literal: string[],
  index: number,
): string[] {
  const component_ref = `G_${name}_${index}`;
  const library = `${component_ref}_library`;
  const resolver_type = `${component_ref}_resolver`;
  const values = {
    component_ref,
    library,
    resolver_type,
    emit_props_factory: "$EmitPropsFactory",
    loose_args: "__L",
    guard: "pick",
    usage_idx: `g_${name}_${index}`,
  };
  const body = (indent: string) => [
    ...literal.flatMap((entry) => [
      `${indent}  // @ts-ignore Inference-only prop; the canonical component-props call owns diagnostics.`,
      `${indent}  ${entry},`,
    ]),
    `${indent}});`,
  ];
  const iife = (emit_props: string, call: string) => [
    `const ${emit_props} = (() => {`,
    "  // @ts-ignore Inference-only guard; authored binding checks own diagnostics.",
    `  ${call}{`,
    ...body("  "),
    "  return undefined as never;",
    "})();",
  ];
  const direct = `${component_ref}_direct`;
  const gated = `${component_ref}_gated`;
  const inferred = (emit_props: string, prop_key: string, alias: string) =>
    format(TEMPLATES.inferred[1], { ...values, emit_props, prop_key, inferred_args: alias });
  return [
    `const ${component_ref} = undefined as unknown as ${shape};`,
    format(TEMPLATES.library[1], values),
    format(TEMPLATES.resolver[1], values),
    ...iife(
      direct,
      format(TEMPLATES.guardedDirect[1].replace("({})", `(${component_ref})`), values),
    ),
    "// @ts-ignore Unmapped inference factory.",
    format(TEMPLATES.guardedFactory[1], values),
    ...iife(gated, format(TEMPLATES.guardedGated[1], values)),
    `type ${component_ref}_eq = Expect<Equal<typeof ${direct}, typeof ${gated}>>;`,
    ...keys.flatMap((key, k) => [
      inferred(direct, key, `${component_ref}_k${k}_direct`),
      inferred(gated, key, `${component_ref}_k${k}_gated`),
      `type ${component_ref}_k${k}_eq = Expect<Equal<${component_ref}_k${k}_direct, ${component_ref}_k${k}_gated>>;`,
    ]),
  ];
}

export function program(aliases: string, only?: string, historical = false): string {
  return [
    ...prelude,
    historical ? oracle.historical.helpers : helpers,
    aliases.replaceAll("{emit_props_factory}", "$EmitPropsFactory"),
    ...Object.entries(shapes)
      .filter(([name]) => only === undefined || name === only)
      .flatMap(([name, shape]) =>
        literalsFor(name).flatMap((literal, index) =>
          usage(
            name,
            shape,
            literal,
            index,
            undefined,
            historical ? oracle.historical.inferred : undefined,
          ),
        ),
      ),
  ].join("\n");
}

export function diagnostics(content: string, exactOptionalPropertyTypes: boolean): string[] {
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
    const line = diagnostic.file?.getLineAndCharacterOfPosition(diagnostic.start ?? 0).line ?? 0;
    const text = content.split("\n")[line]?.slice(0, 120);
    return `TS${diagnostic.code} ${ts.flattenDiagnosticMessageText(diagnostic.messageText, " ")} @ ${text}`;
  });
}
