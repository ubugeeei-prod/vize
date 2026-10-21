import {
  BATCH_SCHEMA,
  applyFixes,
  defineFactProvider,
  defineOutputPlugin,
  definePlugin,
  defineTransformPlugin,
  runBatch,
  runRules,
  type VisitBatch,
} from "@vizejs/plugin-sdk";
import { applyFixes as subpathFixes } from "@vizejs/plugin-sdk/fixes";
import { createSandboxRunner, SANDBOX_IMAGE } from "@vizejs/plugin-sdk/sandbox";
import {
  VISITOR_API_VERSION,
  defineVisitorPlugin,
  negotiateVisitorPlugin,
  runVisitorFixture,
  type VisitorEvent,
} from "@vizejs/plugin-sdk/visitor";

export const plugin = definePlugin({
  name: "packed-types",
  version: "1",
  visit: ["ui.element"] as const,
  demands: ["packed/colors"] as const,
  cacheInputs: [] as const,
  rules: {
    contrast(ctx) {
      for (const node of ctx.nodes)
        ctx.report(node, ctx.facts<string>("packed/colors").get(node.id) ?? "none");
    },
  },
});
const batch: VisitBatch = {
  schema: BATCH_SCHEMA,
  parents: [-1],
  nodes: [{ id: 0, kind: "ui.element" }],
  facts: { "packed/colors": [[0, "red"]] },
};
runBatch(plugin, batch);
runRules(plugin, { nodes: batch.nodes, byId: new Map(), facts: () => new Map(), parent: () => -1 });
applyFixes("blue", [{ start: 0, end: 4, text: "red" }] as const);
subpathFixes("blue", [] as const);

export const provider = defineFactProvider({
  name: "packed",
  version: "1",
  provides: ["packed/colors"] as const,
  cacheInputs: [] as const,
  provide(input) {
    return { "packed/colors": input.nodes.map((node) => [node.id, "red"] as const) };
  },
});
export const transform = defineTransformPlugin({
  name: "packed-transform",
  version: "1",
  cacheInputs: [] as const,
  transform(input) {
    return input.nodes.map((node) => ({
      kind: "replace-static-attribute" as const,
      node: node.id,
      name: "class",
      value: null,
    }));
  },
});
export const formatter = defineOutputPlugin({
  name: "packed-formatter",
  version: "1",
  cacheInputs: [] as const,
  family: "formatter",
  output: () => [{ start: 0, end: 0, text: "\n" }] as const,
});
export const output = defineOutputPlugin({
  name: "packed-output",
  version: "1",
  cacheInputs: [] as const,
  family: "output",
  output: () => [{ placement: "append", comment: "packed" }] as const,
});
export const sandbox = createSandboxRunner({
  name: "packed-sandbox",
  version: "1",
  cacheInputs: [] as const,
  rules: { check: "(ctx) => {}" },
});
export const sandboxProvider = createSandboxRunner({
  name: "packed",
  version: "1",
  family: "provider",
  provides: ["packed/colors"] as const,
  cacheInputs: [] as const,
  callback: "(batch) => ({ 'packed/colors': [] })",
});
export const sandboxTransform = createSandboxRunner({
  name: "packed-transform",
  version: "1",
  family: "transform",
  cacheInputs: [] as const,
  callback: "(batch) => []",
});
export const sandboxFormatter = createSandboxRunner({
  name: "packed-formatter",
  version: "1",
  family: "formatter",
  cacheInputs: [] as const,
  callback: "(batch) => []",
});
export const sandboxOutput = createSandboxRunner({
  name: "packed-output",
  version: "1",
  family: "output",
  cacheInputs: [] as const,
  callback: "(batch) => []",
});
SANDBOX_IMAGE satisfies string;

export const visitor = defineVisitorPlugin({
  apiVersion: VISITOR_API_VERSION,
  name: "packed-visitor",
  version: "1",
  capabilities: ["sfc", "virtual-ts", "devtools-trace"] as const,
  cacheInputs: [] as const,
  visitors: {
    sfc(event) {
      event.payload
        .blocks satisfies readonly import("@vizejs/plugin-sdk/visitor").VisitorSfcBlock[];
    },
    "virtual-ts"(event) {
      event.payload.mappings[0]?.generated.encoding satisfies "utf8" | undefined;
      // @ts-expect-error virtual TypeScript events expose no lint diagnostics
      void event.payload.diagnostics;
    },
    "devtools-trace"(event) {
      event.payload.events[0]?.durationNs satisfies number | undefined;
    },
  },
});
negotiateVisitorPlugin(visitor, {
  apiVersion: VISITOR_API_VERSION,
  capabilities: ["sfc", "virtual-ts", "devtools-trace"],
});
const virtualEvent: VisitorEvent<"virtual-ts"> = {
  apiVersion: VISITOR_API_VERSION,
  capability: "virtual-ts",
  file: "Button.vue",
  payload: {
    sourceFile: "Button.vue",
    virtualFile: "Button.vue.ts",
    code: "export {}",
    mappings: [],
  },
};
runVisitorFixture(visitor, "virtual-ts", virtualEvent);
// @ts-expect-error fixture capability must be declared by this plugin
runVisitorFixture(visitor, "style", {} as VisitorEvent<"style">);

import type {
  RuleContext,
  ScopeEntry,
  FactGroups,
  FactKey,
  FactValue,
  FactEntry,
  BindingFact,
  UndefinedRefFact,
  UnusedBindingFact,
  ComponentUsagesFact,
  ReactivityFact,
  ProvideInjectFact,
  RaceConditionFact,
  SignatureContract,
  PropContract,
  EmitContract,
  SlotContract,
  ReactivityContract,
  ComponentContract,
} from "@vizejs/plugin-sdk";
export function typedFacts(ctx: RuleContext) {
  // @ts-expect-error native visit records are immutable
  ctx.nodes[0].name = "changed";
  // @ts-expect-error static attribute pairs are deeply readonly
  ctx.nodes[0].attrs![0][1] = "changed";
  // @ts-expect-error scoped binding facts are deeply readonly
  ctx.facts("templateScopes").get(0)![0].name = "changed";
  const _scopes: ReadonlyMap<number, readonly ScopeEntry[]> = ctx.facts("templateScopes");
  const _bindings: ReadonlyMap<string, BindingFact> = ctx.facts("bindings");
  const refs: ReadonlyMap<number, UndefinedRefFact> = ctx.facts("undefined-refs");
  const unused: ReadonlyMap<string, UnusedBindingFact> = ctx.facts("unused-bindings");
  const unusedBinding: FactValue<"unused-bindings"> = unused.get("unusedLocal")!;
  unusedBinding.span satisfies readonly [number, number];
  // @ts-expect-error unused bindings use binding names as keys
  unused.get(0);
  // @ts-expect-error native declaration spans are readonly tuples
  unusedBinding.span[0] = 0;
  const usages: ReadonlyMap<string, ComponentUsagesFact> = ctx.facts("component-usages");
  const reactive: FactGroups["reactivity"] = ctx.facts("reactivity");
  const provide: FactGroups["provide-inject"] = ctx.facts("provide-inject");
  const _races: ReadonlyMap<number, RaceConditionFact> = ctx.facts("race-conditions");
  const signatures: ReadonlyMap<string, SignatureContract> = ctx.facts("component-signature");
  const props: ReadonlyMap<string, PropContract> = ctx.facts("prop-types");
  const emits: ReadonlyMap<string, EmitContract> = ctx.facts("emit-types");
  const _slots: ReadonlyMap<string, SlotContract> = ctx.facts("slot-types");
  const _classes: ReadonlyMap<string, ReactivityContract> = ctx.facts("reactivity-classes");
  const _components: ReadonlyMap<string, ComponentContract> = ctx.facts("component-references");
  const _custom: ReadonlyMap<string | number, { color: string }> = ctx.facts<{ color: string }>(
    "packed/colors",
  );
  const fallback: ReadonlyMap<string | number, unknown> = ctx.facts("unregistered/group");
  const _explicit: ReadonlyMap<string | number, PropContract> =
    ctx.facts<PropContract>("prop-types");
  const _entry: FactEntry<"undefined-refs"> = [
    0,
    { name: "missing", offset: 5, context: "interpolation" },
  ];
  const _factKey: FactKey<"undefined-refs"> = 0;
  const _factValue: FactValue<"prop-types"> = props.get("title")!;
  const source: ReactivityFact = reactive.get("source:0")!;
  if (source.kind === "source") {
    source.sourceKind satisfies
      | "ref"
      | "shallow-ref"
      | "reactive"
      | "shallow-reactive"
      | "computed"
      | "readonly"
      | "shallow-readonly"
      | "to-ref"
      | "to-refs";
    // @ts-expect-error native nested arrays are readonly
    source.effects.push("freeze");
  }
  const flow: ProvideInjectFact = provide.get("inject:0")!;
  if (flow.kind === "inject" && flow.pattern.kind === "indirect")
    flow.pattern.injectVar satisfies string;
  const prop = props.get("title")!;
  prop.required satisfies boolean | null;
  prop.default satisfies string | null;
  prop.type_dependencies.complete satisfies boolean;
  prop.type_dependencies.declarations[0].unresolved_reason satisfies string | null;
  // @ts-expect-error alpha authored fields are readonly
  prop.type = "boolean";
  // @ts-expect-error nested dependency values are readonly
  prop.type_dependencies.declarations[0].body = "unknown";
  // @ts-expect-error nested dependency arrays are readonly
  prop.type_dependencies.declarations[0].extends.push("Other");
  // @ts-expect-error wrong ordinal key type
  refs.get("0");
  // @ts-expect-error primary source keys retain their native prefix
  reactive.get("0");
  // @ts-expect-error ordinal key helper is numeric
  const _wrongKey: FactKey<"race-conditions"> = "0";
  // @ts-expect-error declared maps are readonly
  props.set("other", prop);
  // @ts-expect-error unknown custom groups do not gain inferred value shapes
  void fallback.get(0)!.color;
  const signature = signatures.get("Component.vue")!;
  signature.schema satisfies 1;
  signature.generic satisfies string | null;
  signature.props_complete satisfies boolean;
  signature.with_defaults satisfies string | null;
  signature.prop_order satisfies readonly string[];
  signature.slot_order satisfies readonly string[];
  // @ts-expect-error schema is exactly one
  const _schema: SignatureContract["schema"] = 2;
  // @ts-expect-error signature authored order is readonly
  signature.prop_order.push("extra");
  const emit = emits.get("change")!;
  emit.overload_payloads satisfies readonly (string | null)[];
  emit.unresolved_type_arguments satisfies string | null;
  emit.validator_signatures satisfies readonly string[];
  emit.validator_type_annotations satisfies readonly string[];
  // @ts-expect-error active runtime type annotation lists are readonly
  emit.validator_type_annotations.push("Unknown");
  // @ts-expect-error unknown overload type arguments remain nullable
  const _completeArguments: string = emit.unresolved_type_arguments;
  // @ts-expect-error emit type argument facts are readonly
  emit.unresolved_type_arguments = null;
  // @ts-expect-error runtime validator headers are readonly
  emit.validator_signatures.push("(payload: never): boolean");
  // @ts-expect-error ordered overload rows are readonly
  emit.overload_payloads[0] = null;
  const usage = usages.get('[null,"Card"]')!;
  usage.sites[0].usageIndex satisfies number | null;
  usage.sites[0].nameOrdinal satisfies number | null;
  // @ts-expect-error primary nested component records are readonly
  usage.sites[0].usage!.props[0].value = "changed";
}

export function immutableHookInputs(
  transformBatch: import("@vizejs/plugin-sdk").StaticAttributeTransformBatch,
  outputBatch: import("@vizejs/plugin-sdk").OutputBatch,
  providerBatch: import("@vizejs/plugin-sdk").FactProviderBatch,
) {
  // @ts-expect-error native transform attributes are deeply readonly
  transformBatch.nodes[0].attrs[0].value = "changed";
  // @ts-expect-error native compiled artifacts are immutable
  outputBatch.compiled.code = "changed";
  // @ts-expect-error provider context fields are immutable
  providerBatch.parents = [];
  // @ts-expect-error static transforms support the actual bounded host whitelist
  const _unsupported: import("@vizejs/plugin-sdk").StaticAttributeName = "disabled";
}

export function refusedHookFamilies() {
  defineOutputPlugin({
    name: "bad-formatter",
    version: "1",
    family: "formatter",
    // @ts-expect-error formatter callbacks return byte edits, not comment additions
    output: () => [{ placement: "append", comment: "bad" }],
  });
  // @ts-expect-error output callbacks return additions, not byte edits
  defineOutputPlugin({
    name: "bad-output",
    version: "1",
    family: "output",
    output: () => [{ start: 0, end: 0, text: "bad" }],
  });
}
