import assert from "node:assert/strict";
import { test } from "node:test";
import {
  assertEmitGateRejectionPaths,
  emitGateRejections,
} from "./support/canon-emit-props-gate-rejections.ts";
import { emitGateOracle } from "./support/emit-props-gate-oracle.ts";
import { beforeGenericEventSpecialization } from "./support/generic-event-specialization-source.ts";
import {
  read,
  oracle,
  inference,
  sharedInference,
  helpers,
  scope,
  gate,
  TEMPLATES,
  GATE_ALIASES,
  CLOSED_EMIT_ALIASES,
  SHARED_CLOSED_EMIT,
  gateAliases,
  sharedGateAliases,
  format,
  prelude,
  sfc,
  resolvesEmits,
  shapes,
  literals,
  usage,
  program,
  diagnostics,
} from "./support/emit-props-gate-fixture.ts";

// The direct inference-only emit-props call is the oracle: the gated call that
// checks a props-independent literal against `any` must produce exactly the
// type the direct call produces, for every component shape a closed template
// can bind.
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
  // Standalone modules declare the closed check; hoisted ones name the shared helpers.
  for (const bind of [gateAliases, sharedGateAliases]) {
    for (const exactOptionalPropertyTypes of [false, true]) {
      assert.deepEqual(
        diagnostics(program(GATE_ALIASES, undefined, false, bind), exactOptionalPropertyTypes),
        [],
        `${bind.name} exactOptionalPropertyTypes=${exactOptionalPropertyTypes}`,
      );
    }
  }
});

void test("the shared closed helpers are the standalone aliases under their SDK names", () => {
  const renamed = CLOSED_EMIT_ALIASES.replace(
    "type __vize_emit_closed_factory<E>",
    "type $ClosedEmitFactory<E>",
  )
    .replace("type __vize_emit_closed<C, A = C>", "type $ClosedEmit<C, A = C>")
    .trimEnd()
    .split("\n");
  assert.deepEqual(SHARED_CLOSED_EMIT.split("\n"), renamed);
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
      GATE_ALIASES.replace(
        "[L] extends [true] ? F : {closed}<C> extends [unknown] ? F : true extends",
        "{closed}<C> extends [unknown] ? F : true extends",
      ),
      "library",
    ],
    [
      "the cheap call drops the factory result",
      GATE_ALIASES.replace("(props: any) => R : F", "(props: any) => {} : F"),
      "sameEmitUnion",
    ],
    [
      "the closed factory drops the component's emits",
      gateAliases().replace(
        "(component: unknown) => (props: any) => __EmitProps<E>",
        "(component: unknown) => (props: any) => {}",
      ),
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
      gateAliases(),
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

void test("only a single non-generic SFC skips the full factory", () => {
  const closed = ["sfc", "loose", "required"];
  const lines = [...prelude, helpers, gateAliases(), SHARED_CLOSED_EMIT];
  for (const [name, shape] of Object.entries(shapes)) {
    lines.push(
      `type Closed_${name} = Expect<Equal<__vize_emit_closed<${shape}> extends [unknown] ? true : false, ${closed.includes(name)}>>;`,
      `type Shared_${name} = Expect<Equal<$ClosedEmit<${shape}> extends [unknown] ? true : false, ${closed.includes(name)}>>;`,
    );
  }
  for (const exactOptionalPropertyTypes of [false, true]) {
    assert.deepEqual(diagnostics(lines.join("\n"), exactOptionalPropertyTypes), []);
  }
});

function assertRoutes(
  cheap: readonly string[],
  direct: readonly string[],
  helperText: string,
  aliases = GATE_ALIASES,
): void {
  const lines = [...prelude, helperText, gateAliases(aliases)];
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
  assertEmitGateRejectionPaths(oracle.current.sourceHashes);
  for (const [path, reason] of emitGateRejections) {
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
  const lines = [...prelude, helpers, gateAliases()];
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
