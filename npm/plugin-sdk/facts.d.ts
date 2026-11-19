/** Template binding name and the position that introduced it. */
export interface TemplateScopeEntry {
  readonly name: string;
  readonly position: "value" | "key" | "index" | "slot";
}
/** Host classification for a script binding exposed to plugin facts. */
export type BindingKind =
  | "setup-let"
  | "setup-maybe-ref"
  | "setup-ref"
  | "setup-reactive-const"
  | "setup-const"
  | "props"
  | "props-aliased"
  | "data"
  | "options"
  | "literal-const"
  | "js-global-universal"
  | "js-global-browser"
  | "js-global-node"
  | "js-global-deno"
  | "js-global-bun"
  | "vue-global"
  | "external-module";
/** Script-setup marker or resolved classification for one binding. */
export type BindingFact =
  | Readonly<{ scriptSetup: true }>
  | Readonly<{
      kind: BindingKind | null;
      span: readonly [number, number] | null;
      propKey: string | null;
    }>;
/** Unresolved template reference and its authored byte offset. */
export interface UndefinedRefFact {
  readonly name: string;
  readonly offset: number;
  readonly context: string;
}
/** Authored byte span for a binding with no observed reads. */
export interface UnusedBindingFact {
  readonly span: readonly [number, number];
}
/** Half-open UTF-8 byte range shared by source-backed facts. */
export interface FactRange {
  readonly start: number;
  readonly end: number;
}
/** One component prop occurrence, including dynamic-name and value state. */
export interface ComponentPropUsage extends FactRange {
  readonly name: string;
  readonly nameIsDynamic: boolean;
  readonly value: string | null;
  readonly isDynamic: boolean;
}
/** One component event-handler occurrence and its modifiers. */
export interface ComponentEventUsage extends FactRange {
  readonly name: string;
  readonly nameIsDynamic: boolean;
  readonly handler: string | null;
  readonly modifiers: readonly string[];
}
/** One component slot occurrence and its introduced scope variables. */
export interface ComponentSlotUsage extends FactRange {
  readonly name: string;
  readonly nameIsDynamic: boolean;
  readonly scopeVars: readonly string[];
  readonly hasScope: boolean;
}
/** One spread-prop expression applied to a component. */
export interface ComponentSpreadUsage extends FactRange {
  readonly expression: string;
}
/** Complete normalized usage of one component node. */
export interface ComponentUsage extends FactRange {
  readonly name: string;
  readonly props: readonly ComponentPropUsage[];
  readonly events: readonly ComponentEventUsage[];
  readonly slots: readonly ComponentSlotUsage[];
  readonly hasSpreadAttrs: boolean;
  readonly spreadProps: readonly ComponentSpreadUsage[];
  readonly scopeId: number;
  readonly vifGuard: string | null;
}
/** Resolved component tag paired with its optional usage record. */
export interface ComponentUsageSite {
  readonly tag: string;
  readonly usage: ComponentUsage | null;
  readonly usageIndex: number | null;
  readonly nameOrdinal: number | null;
}
/** Import/export identity and every observed site for one component. */
export interface ComponentUsagesFact {
  readonly module: string | null;
  readonly export: string;
  readonly sites: readonly ComponentUsageSite[];
}
/** Stability class inferred for a reactive source. */
export type ReactivityClass = "static" | "props-stable" | "reactive" | "unstable";
/** Confidence verdict attached to an inferred contract. */
export type Verdict = "proven" | "refuted" | "unknown";
/** Effect that contributes to a reactive stability verdict. */
export type ReactivityEffect =
  | "freeze"
  | "capture"
  | "read-prop"
  | "read-reactive"
  | "mutate-local"
  | "mutate-global"
  | "call-unknown"
  | "allocate";
/** Vue API that created a reactive source. */
export type ReactiveSourceKind =
  | "ref"
  | "shallow-ref"
  | "reactive"
  | "shallow-reactive"
  | "computed"
  | "readonly"
  | "shallow-readonly"
  | "to-ref"
  | "to-refs";
/** Authored operation that loses or obscures reactive ownership. */
export type ReactivityLoss =
  | Readonly<{
      kind: "reactive-destructure" | "ref-value-destructure";
      sourceName: string;
      destructuredProps: readonly string[];
    }>
  | Readonly<{ kind: "ref-value-extract"; sourceName: string; targetName: string }>
  | Readonly<{
      kind: "reactive-property-extract";
      sourceName: string;
      propName: string;
      targetName: string;
    }>
  | Readonly<{ kind: "props-destructure"; destructuredProps: readonly string[] }>
  | Readonly<{
      kind: "function-argument-extract";
      sourceName: string;
      argumentName: string;
      calleeName: string;
    }>
  | Readonly<{
      kind: "getter-call-extract";
      contextName: string;
      getterName: string;
      targetName: string;
      calleeName: string;
      sourceName: string;
    }>
  | Readonly<{
      kind: "plain-value-alias";
      sourceName: string;
      aliasName: string;
      targetName: string;
    }>
  | Readonly<{ kind: "reactive-spread" | "reactive-reassign"; sourceName: string }>;
/** Reactive source classification or a detected reactivity-loss site. */
export type ReactivityFact =
  | Readonly<{
      kind: "source";
      name: string;
      sourceKind: ReactiveSourceKind;
      declarationOffset: number;
      class: ReactivityClass;
      verdict: Verdict;
      effects: readonly ReactivityEffect[];
    }>
  | Readonly<{
      kind: "loss";
      loss: ReactivityLoss;
      start: number;
      end: number;
    }>;

/** Normalized string or symbol key used by provide/inject. */
export interface ProvideKey {
  readonly kind: "string" | "symbol";
  readonly name: string;
}
/** Shape through which an injected value is consumed. */
export type InjectPattern =
  | Readonly<{ kind: "simple" }>
  | Readonly<{ kind: "object" | "array"; props: readonly string[] }>
  | Readonly<{ kind: "indirect"; injectVar: string; props: readonly string[]; offset: number }>;
/** Provide, inject, or composable-boundary fact from one source file. */
export type ProvideInjectFact =
  | Readonly<{
      kind: "provide";
      id: number;
      key: ProvideKey;
      value: string;
      valueType: string | null;
      fromComposable: string | null;
      start: number;
      end: number;
    }>
  | Readonly<{
      kind: "inject";
      key: ProvideKey;
      localName: string;
      defaultValue: string | null;
      expectedType: string | null;
      pattern: InjectPattern;
      fromComposable: string | null;
      start: number;
      end: number;
    }>
  | Readonly<{
      kind: "composable";
      name: string;
      source: string;
      localName: string | null;
      usesProvide: boolean;
      usesInject: boolean;
      usesReactivity: boolean;
      start: number;
      end: number;
    }>;

/** Async scheduling pattern that may race with a later mutation. */
export type RaceRisk =
  | Readonly<{
      kind: "async-watcher";
      watcherName: string;
      asyncOperation: string;
      mutatedTargets: readonly string[];
    }>
  | Readonly<{
      kind: "async-watch-effect" | "promise-continuation";
      asyncOperation: string;
      mutatedTargets: readonly string[];
    }>
  | Readonly<{
      kind: "async-lifecycle";
      hookName: string;
      asyncOperation: string;
      mutatedTargets: readonly string[];
    }>
  | Readonly<{ kind: "scheduled"; schedulerName: string; mutatedTargets: readonly string[] }>;
/** Source-backed asynchronous race-risk observation. */
export interface RaceConditionFact extends FactRange {
  readonly kind: RaceRisk;
}

/** Closed reason why complete public type information was unavailable. */
export type UnresolvedTypeReason =
  | "external-world-unavailable"
  | "unbound"
  | "missing-module"
  | "incomplete-module"
  | "unsupported-declaration"
  | "unsupported-qualification"
  | "ambiguous-export"
  | "resolution-cycle"
  | "resolution-limit";
/** Authored or resolved dependency reachable from a public type. */
export interface TypeDependency {
  readonly name: string;
  readonly kind: "alias" | "interface" | "imported" | "unresolved";
  readonly module: string | null;
  readonly export: string | null;
  /** Authored type text, including literal whitespace. */
  readonly body: string | null;
  readonly parameters: string | null;
  readonly extends: readonly string[];
  readonly unresolved_reason: UnresolvedTypeReason | null;
}
/** Resolution state and dependencies for a public type contract. */
export interface TypeEnvironment {
  /** False means some reachable type information could not be resolved. */
  readonly complete: boolean;
  readonly declarations: readonly TypeDependency[];
}
/** Name and optional type for a value exposed from a component instance. */
export interface ExposeContract {
  readonly name: string;
  readonly type: string | null;
}
/** Complete public component signature discovered from authored source. */
export interface SignatureContract {
  readonly schema: 1;
  readonly name: string;
  readonly declared_name: string | null;
  readonly generic: string | null;
  readonly component_shape: "unspecified" | "class-api";
  readonly script_setup: boolean;
  readonly prop_type_arguments: string | null;
  readonly emit_type_arguments: string | null;
  readonly slot_type_arguments: string | null;
  /** Public names in authored completion order, without dump key encoding. */
  readonly props_complete: boolean;
  readonly with_defaults: string | null;
  readonly prop_order: readonly string[];
  readonly slot_order: readonly string[];
  readonly exposes_complete: boolean;
  readonly exposes: readonly ExposeContract[];
  readonly type_dependencies: TypeEnvironment;
}
/** Resolved public prop type, requirement, default, and dependencies. */
export interface PropContract {
  readonly schema: 1;
  readonly name: string;
  readonly type: string | null;
  readonly required: boolean | null;
  readonly default: string | null;
  readonly model_modifiers: string | null;
  readonly type_dependencies: TypeEnvironment;
}
/** Resolved public event payloads, validators, and dependencies. */
export interface EmitContract {
  readonly schema: 1;
  readonly name: string;
  readonly payload: string | null;
  readonly overload_payloads: readonly (string | null)[];
  readonly unresolved_type_arguments: string | null;
  readonly validator_signatures: readonly string[];
  readonly validator_type_annotations: readonly string[];
  readonly type_dependencies: TypeEnvironment;
}
/** Resolved public slot props and their type dependencies. */
export interface SlotContract {
  readonly schema: 1;
  readonly name: string;
  readonly props: string | null;
  readonly type_dependencies: TypeEnvironment;
}
/** Public spelling of a supported Vue reactive constructor. */
export type AlphaReactiveKind =
  | "ref"
  | "shallowRef"
  | "reactive"
  | "shallowReactive"
  | "computed"
  | "readonly"
  | "shallowReadonly"
  | "toRef"
  | "toRefs";
/** Public reactivity contract for one component binding. */
export interface ReactivityContract {
  readonly schema: 1;
  readonly name: string;
  readonly type: string | null;
  readonly kind: AlphaReactiveKind | null;
  readonly class: ReactivityClass | null;
  readonly verdict: Verdict;
  readonly effects: readonly ReactivityEffect[];
  readonly type_dependencies: TypeEnvironment;
}
/** Module and export identity for a component dependency. */
export interface ComponentContract {
  readonly schema: 1;
  readonly module: string | null;
  readonly export: string;
}

/** Exact native group names and map key/value contracts. */
export interface FactGroups {
  readonly templateScopes: ReadonlyMap<number, readonly TemplateScopeEntry[]>;
  readonly bindings: ReadonlyMap<string, BindingFact>;
  readonly "undefined-refs": ReadonlyMap<number, UndefinedRefFact>;
  readonly "unused-bindings": ReadonlyMap<string, UnusedBindingFact>;
  /** Keys are serialized JSON tuples [module | null, export]. */
  readonly "component-usages": ReadonlyMap<string, ComponentUsagesFact>;
  readonly reactivity: ReadonlyMap<`source:${number}` | `loss:${number}`, ReactivityFact>;
  readonly "provide-inject": ReadonlyMap<
    `provide:${number}` | `inject:${number}` | `composable:${number}`,
    ProvideInjectFact
  >;
  readonly "race-conditions": ReadonlyMap<number, RaceConditionFact>;
  /** Alpha keys are canonical dump declaration keys. */
  readonly "component-signature": ReadonlyMap<string, SignatureContract>;
  readonly "prop-types": ReadonlyMap<string, PropContract>;
  readonly "emit-types": ReadonlyMap<string, EmitContract>;
  readonly "slot-types": ReadonlyMap<string, SlotContract>;
  readonly "reactivity-classes": ReadonlyMap<string, ReactivityContract>;
  readonly "component-references": ReadonlyMap<string, ComponentContract>;
}
/** Key type used by one named host fact group. */
export type FactKey<K extends keyof FactGroups> =
  FactGroups[K] extends ReadonlyMap<infer Key, unknown> ? Key : never;
/** Value type used by one named host fact group. */
export type FactValue<K extends keyof FactGroups> =
  FactGroups[K] extends ReadonlyMap<unknown, infer Value> ? Value : never;
/** Serialized key/value entry used by one named host fact group. */
export type FactEntry<K extends keyof FactGroups> = readonly [FactKey<K>, FactValue<K>];
