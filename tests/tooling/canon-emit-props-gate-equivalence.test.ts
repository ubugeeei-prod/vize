import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";
import { emitGateOracle } from "./support/emit-props-gate-oracle.ts";
import { beforeGenericEventSpecialization } from "./support/generic-event-specialization-source.ts";

// The direct inference-only emit-props call is the oracle: the gated call that
// checks a props-independent literal against `any` must produce exactly the
// type the direct call produces, for every component shape a closed template
// can bind.
const repo = new URL("../../", import.meta.url);
const read = (path: string) => fs.readFileSync(new URL(path, repo), "utf8");
const oracle = emitGateOracle(read);
const declarations = "crates/vize_canon/src/virtual_ts/helpers/declarations/";
const inference = read(`${declarations}emit-inference.d.ts.txt`);
const sharedInference = read(`${declarations}shared-emit-inference.d.ts.txt`);
const ownership = ["$ComponentMetadataKeys", "$ComponentOwnKeys", "$UnownedComponentAttrs"]
  .map((name) => {
    const line = read("crates/vize_canon/src/virtual_ts/component_prop_helpers.txt")
      .split("\n")
      .find((line) => line.startsWith(`type ${name}<`));
    assert.ok(line, name);
    return line;
  })
  .join("\n");
const helpersFor = (emitInference: string) =>
  [
    read(`${declarations}core-types.d.ts.txt`).trimEnd(),
    "type $PublicComponentInputProps = { hidden?: boolean };",
    read(`${declarations}emit-overloads.d.ts.txt`).trimEnd(),
    read(`${declarations}emit-props.d.ts.txt`).trimEnd(),
    ownership,
    emitInference.trimEnd(),
  ].join("\n");
const helpers = helpersFor(inference);

const scope = "crates/vize_canon/src/virtual_ts/scope/";
const events = read(`${scope}component_events.rs`);
const generic = read(`${scope}component_events/generic_inference.rs`);
const gate = read(`${scope}component_events/emit_props_gate.rs`);

// Format strings exactly as the generator spells them.
const TEMPLATES = {
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
} as const;

const aliasSource = gate.match(/const GATE_ALIASES: &str = "((?:[^"\\]|\\.)*)";/u);
assert.ok(aliasSource, "GATE_ALIASES");
const GATE_ALIASES = aliasSource[1]!.replaceAll("\\n", "\n").replaceAll('\\"', '"');

/** Rust `format!` semantics for named placeholders. */
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

// Generated SFC exports, as the projection declares them.
const prelude = [
  "export {};",
  "type Equal<A, B> = (<G>() => G extends A ? 1 : 2) extends <G>() => G extends B ? 1 : 2 ? true : false;",
  "type Expect<T extends true> = T;",
  "type Ctor<P, E> = new <A = unknown>(props?: A & Partial<P>, ...args: any[]) => { $props: P; $emit: E };",
  "type Sfc<P, E, F = {}> = { readonly __vizeComponentMarker: true; readonly __vizeRawProps?: P; readonly __vizeFallthroughProps?: F; __vizeEmitProps?: __EmitProps<E> } & Ctor<P, E>;",
  "type Save<T> = (event: 'save', value: T) => void;",
  "declare const columns: { id: string; type: string }[];",
  "declare const id: string;",
];

const sfc =
  "Sfc<{ value?: string; items?: { a: number }[] }, { (event: 'save', value: number): void; (event: 'update:modelValue', value: string): void }, { onClick?: (event: MouseEvent) => void }>";
const loose = "Sfc<{ column?: { id: string } }, (event: string, ...args: any[]) => void>";
const required = "Sfc<{ value: string }, Save<string>>";
const unmarked = "Ctor<{ value?: string }, Save<boolean>>";
const sameEmit = "Sfc<{ value: string; other?: number }, Save<string>>";
const plain =
  "(new (...args: any[]) => { $props: { value?: string; onSave?: (value: number) => void }; $emit: Save<number> })";
const functional =
  "((props: { value?: string; onSave?: (value: number) => void }, ctx: any) => any)";
const library =
  "(<T>(props: { value: T; onSave?: (value: T) => void }, ctx?: any) => { __ctx?: { props: { value: T; onSave?: (value: T) => void } } })";
const resolvesEmits =
  "({ readonly __vizeComponentMarker: true; __vizeCheck: <T>(props: { value: T }) => void; __vizeResolveProps?: <T>(props: { value: T }) => { value: T }; __vizeResolveEmitProps?: <T>(props: Partial<{ value: T }> & Record<string, unknown>) => __EmitProps<Save<T>> } & Ctor<{ value: unknown }, Save<unknown>>)";
const resolvesProps =
  "({ readonly __vizeComponentMarker: true; __vizeCheck: <T>(props: { value: T }) => void; __vizeResolveProps?: <T>(props: { value: T; onSave?: (value: T) => void }) => { value: T; onSave?: (value: T) => void } } & Ctor<{ value: unknown }, (event: string, ...args: any[]) => void>)";
const columnsUnion = Array.from(
  { length: 6 },
  (_, index) =>
    `Sfc<{ column?: { id: string }; isStacked?: boolean }, (event: 'headerWheel', value: ${index}) => void>`,
).join(" | ");

const shapes: Record<string, string> = {
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
const literals: Record<string, string[]> = {
  text: ['"value": "text"'],
  wrong: ['"value": 1', '"class": { _shadow: true }'],
  nested: ['"items": [{ a: 1 }]'],
  find: ['"column": columns.find(c => c.id === id)!', '"isStacked": columns.length > 1'],
  callback: ['"onSave": (value) => { void value; }', '"value": "text"'],
};
const keys = ["onSave", "onHeaderWheel"];
// Every literal for the shapes whose routes read or reduce it; one elsewhere.
const fullLiterals = new Set(["sfc", "resolvesEmits", "library", "deck", "genericUnion"]);
const literalsFor = (name: string) =>
  fullLiterals.has(name) ? Object.values(literals) : [literals.find!];

function usage(
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

function program(aliases: string, only?: string, historical = false): string {
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
    const line = diagnostic.file?.getLineAndCharacterOfPosition(diagnostic.start ?? 0).line ?? 0;
    const text = content.split("\n")[line]?.slice(0, 120);
    return `TS${diagnostic.code} ${ts.flattenDiagnosticMessageText(diagnostic.messageText, " ")} @ ${text}`;
  });
}

void test("the oracle reads the generator's own text", () => {
  for (const [name, [source, template]] of Object.entries(TEMPLATES)) {
    assert.ok(source.includes(template), `${name} is no longer the generator's text`);
  }
});

void test("the gate mirrors every $EmitPropsFactory route", () => {
  const factory = inference
    .split("\n")
    .find((line) => line.startsWith("type $EmitPropsRoutes<C, All> ="));
  assert.ok(factory);
  // The shared module declares the same factory for hoisted modules.
  for (const name of [
    "$EmitPropsFactory",
    "$EmitPropsRoutes",
    "$UnionEmit",
    "$NativeEmitPropsFactory",
    "$ConstructorEmitPropsFactory",
    "$MemberConstructorEmitPropsFactory",
    "$LooseEmitPropsFactory",
    "$EmitIsWide",
  ]) {
    const local = inference.split("\n").find((line) => line.startsWith(`type ${name}<`));
    const shared = sharedInference.split("\n").find((line) =>
      line
        .trimStart()
        .replace(/^export /u, "")
        .startsWith(`type ${name}<`),
    );
    assert.equal(shared?.trim().replace(/^export /u, ""), local, name);
  }
  // A generic resolver reads the props (`true`); every other route does not.
  // The `any` route stays in `$EmitPropsFactory`; the member routes thread the whole argument.
  const mirror = (
    "0 extends (1 & C) ? false : " + factory.slice("type $EmitPropsRoutes<C, All> = ".length, -1)
  )
    .replaceAll("(component: C) => F", "true")
    .replaceAll("$NativeEmitPropsFactory<C, All>", "__vize_emit_native_input<C>")
    .replaceAll("infer F", "infer X")
    .replaceAll("F extends", "X extends");
  assert.ok(GATE_ALIASES.includes(`true extends (${mirror}) ? F :`), mirror);
});

void test("a gated emit-props call has exactly the direct call's type", () => {
  for (const exactOptionalPropertyTypes of [false, true]) {
    assert.deepEqual(
      diagnostics(program(GATE_ALIASES), exactOptionalPropertyTypes),
      [],
      `exactOptionalPropertyTypes=${exactOptionalPropertyTypes}`,
    );
  }
});

void test("the oracle rejects gates that drop a props-dependent route or the factory result", () => {
  const single =
    "[R extends unknown ? (<G>() => G extends R ? 1 : 2) extends <G>() => G extends U ? 1 : 2 ? true : false : false] extends [true] ? (props: any) => R : F";
  for (const [name, broken, witness] of [
    [
      "generic resolvers read as props-independent",
      GATE_ALIASES.replaceAll(
        "X extends (...args: any[]) => any ? true",
        "X extends (...args: any[]) => any ? false",
      ),
      "resolvesEmits",
    ],
    [
      "library exports read as props-independent",
      GATE_ALIASES.replace("[L] extends [true] ? F : true extends", "true extends"),
      "library",
    ],
    [
      "the cheap call drops the factory result",
      GATE_ALIASES.replace("(props: any) => R : F", "(props: any) => {} : F"),
      "sfc",
    ],
    [
      "a union whose members return different types skips subtype reduction",
      GATE_ALIASES.replace(single, "(props: any) => R"),
      oracle.current.subtypeWitness,
    ],
  ] as const) {
    assert.notEqual(broken, GATE_ALIASES, name);
    const failures = diagnostics(program(broken, witness), false);
    assert.ok(
      failures.some((text) => text.startsWith("TS2344 ") && text.includes(`C_${witness}_`)),
      `${name}: ${failures.slice(0, 3).join("\n")}`,
    );
  }
});

void test("the historical deferred counter stays frozen and generic templates keep the direct call", () => {
  const deferred = (historical: boolean) =>
    [
      ...prelude,
      historical ? oracle.historical.helpers : helpers,
      GATE_ALIASES.replaceAll("{emit_props_factory}", "$EmitPropsFactory"),
      `export function template<K extends ${resolvesEmits} | ${sfc}>() {`,
      ...usage(
        "deferred",
        "K",
        literals.text!,
        0,
        undefined,
        historical ? oracle.historical.inferred : undefined,
      ).map((line) => `  ${line}`),
      "}",
    ].join("\n");
  const failures = diagnostics(deferred(true), false);
  assert.ok(
    failures.some((text) => text.startsWith("TS2344 ") && text.includes("C_deferred_0_")),
    failures.join("\n"),
  );
  // The marker-aware observer removes this historical counterexample. This
  // does not enable the gate for deferred templates: its producer law stays.
  assert.deepEqual(diagnostics(deferred(false), false), []);
  assert.ok(gate.includes("!generic_component && !legacy_vue2"));
});

void test("the gate takes the cheap call exactly where the factory ignores the props", () => {
  // Unmarked constructors now retain their props-dependent higher-order factory.
  const native = ["unmarked", "plain"];
  assertRoutes(
    oracle.current.cheap.filter((name) => !native.includes(name)),
    [...oracle.current.direct, ...native],
    helpers,
  );
});

function assertRoutes(
  cheap: readonly string[],
  direct: readonly string[],
  helperText: string,
  aliases = GATE_ALIASES,
): void {
  const lines = [
    ...prelude,
    helperText,
    aliases.replaceAll("{emit_props_factory}", "$EmitPropsFactory"),
  ];
  for (const name of [...cheap, ...direct]) {
    const component_ref = `C_${name}`;
    const library = `${component_ref}_library`;
    const input = `__vize_emit_input<${library}, typeof ${component_ref}, typeof ${component_ref}_factory>`;
    // A cheap call reads its literal as `any`; a direct one is the factory itself.
    const route = cheap.includes(name)
      ? `$IsAny<Parameters<${input}>[0]>`
      : `Equal<${input}, typeof ${component_ref}_factory>`;
    lines.push(
      `const ${component_ref} = undefined as unknown as ${shapes[name]};`,
      format(TEMPLATES.library[1], { component_ref, library }),
      "// @ts-ignore Unmapped inference call.",
      `const ${component_ref}_factory = (undefined as unknown as __vize_emit_resolver<${library}, typeof ${component_ref}>)(${component_ref});`,
      `type ${component_ref}_route = Expect<${route}>;`,
    );
  }
  for (const exactOptionalPropertyTypes of [false, true]) {
    assert.deepEqual(diagnostics(lines.join("\n"), exactOptionalPropertyTypes), []);
  }
}

void test("the immutable pre-marker oracle retains its original routes and union refusal", () => {
  const oldGate = beforeGenericEventSpecialization(
    read,
    `${scope}component_events/emit_props_gate.rs`,
    gate,
  );
  const oldAliases = oldGate.match(/const GATE_ALIASES: &str = "((?:[^"\\]|\\.)*)";/u);
  assert.ok(oldAliases);
  assertRoutes(
    oracle.historical.cheap,
    oracle.historical.direct,
    oracle.historical.helpers,
    oldAliases[1]!.replaceAll("\\n", "\n").replaceAll('\\"', '"'),
  );
  const single =
    "[R extends unknown ? (<G>() => G extends R ? 1 : 2) extends <G>() => G extends U ? 1 : 2 ? true : false : false] extends [true] ? (props: any) => R : F";
  const broken = GATE_ALIASES.replace(single, "(props: any) => R");
  assert.notEqual(broken, GATE_ALIASES);
  const witness = oracle.historical.subtypeWitness;
  for (const exactOptionalPropertyTypes of [false, true]) {
    assert.deepEqual(
      diagnostics(program(GATE_ALIASES, undefined, true), exactOptionalPropertyTypes),
      [],
    );
    const failures = diagnostics(program(broken, witness, true), exactOptionalPropertyTypes);
    assert.ok(
      failures.some((text) => text.startsWith("TS2344 ") && text.includes(`C_${witness}_`)),
    );
  }
});

void test("a successor refuses unknown producer bytes and altered historical context", () => {
  const rejections = [
    ["tests/_fixtures/tooling/emit-props-gate-11697.json", /immutable emit gate oracle context/u],
    [`${declarations}emit-inference.d.ts.txt`, /generic specialization image/u],
    [`${declarations}shared-emit-inference.d.ts.txt`, /generic specialization image/u],
    [`${scope}component_events/generic_inference.rs`, /integrated static listener source/u],
    [`${scope}component_events/emit_props_gate.rs`, /generic specialization image/u],
    [
      `${declarations}core-types.d.ts.txt`,
      /selected emit gate producer: crates\/vize_canon\/src\/virtual_ts\/helpers\/declarations\/core-types\.d\.ts\.txt/u,
    ],
    [`${declarations}emit-overloads.d.ts.txt`, /canon leftovers source identity/u],
    [`${declarations}emit-props.d.ts.txt`, /canon leftovers source identity/u],
    [
      "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt",
      /canon leftovers source identity/u,
    ],
    [`${scope}component_events.rs`, /listener relation source/u],
  ] as const;
  assert.deepEqual(
    rejections.map(([path]) => path),
    [
      "tests/_fixtures/tooling/emit-props-gate-11697.json",
      ...Object.keys(oracle.current.sourceHashes),
    ],
  );
  for (const [path, reason] of rejections) {
    assert.throws(
      () => emitGateOracle((selected) => read(selected) + (selected === path ? " " : "")),
      reason,
    );
    assert.throws(
      () => emitGateOracle((selected) => (selected === path ? "" : read(selected))),
      reason,
    );
  }
});

void test("a gated result still feeds the listener args, so the call stays", () => {
  // `_emit_args`/`_args` read `inferred` unless `unknown[] extends inferred`.
  const lines = [
    ...prelude,
    helpers,
    GATE_ALIASES.replaceAll("{emit_props_factory}", "$EmitPropsFactory"),
  ];
  const read = {
    sfc: "[value: number]",
    unmarked: "[value: boolean]",
    specificUnion: "[value: number]",
  };
  const discarded = ["loose", "columns"];
  for (const name of [...Object.keys(read), ...discarded]) {
    lines.push(...usage(name, shapes[name]!, literals.find!, 0));
    const inferred = `C_${name}_0_k0_gated`;
    lines.push(
      name in read
        ? `type ${name}_read = Expect<Equal<${inferred}, ${read[name as keyof typeof read]}>>;`
        : `type ${name}_discarded = Expect<unknown[] extends ${inferred} ? true : false>;`,
    );
  }
  assert.deepEqual(diagnostics(lines.join("\n"), false), []);
});
