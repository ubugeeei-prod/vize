import assert from "node:assert/strict";

import { canonical, identity, jsonBytes } from "./lint-preset-source-successors.ts";

const registryPath = "crates/vize_patina/src/linter/script_rules/registry/rules.rs";
const membershipPath =
  "crates/vize_patina/src/preset/snapshots/vize_patina__preset__tests__lint_preset_rule_membership.snap";

// This branch registers the essential script/no-template-ref-outside-instance.
const rule = "script/no-template-ref-outside-instance";
const presets = ["ecosystem", "essential", "happy_path", "nuxt", "opinionated", "opt_in"];

// The registry text with the rule, then without it. rustfmt reflows both
// import lists around the added names.
const registration = [
  [
    `    RULE_NO_TEMPLATE_REF_OUTSIDE_INSTANCE, RULE_NO_THIS_IN_BEFORE_ROUTE_ENTER,
    RULE_NO_TOP_LEVEL_REF_IN_SCRIPT, RULE_NO_UNSTABLE_NESTED_COMPONENTS,
    RULE_NO_UNUSED_EMIT_DECLARATIONS, RULE_NO_USE_COMPUTED_PROPERTY_LIKE_METHOD,
    RULE_NO_WATCH_AFTER_AWAIT, RULE_NO_WITH_DEFAULTS, RULE_NUXT_CONFIG_KEYS_ORDER,
    RULE_NUXT_NO_CONFIG_TEST_KEY, RULE_NUXT_NO_PAGE_META_RUNTIME_VALUES,
    RULE_NUXT_PREFER_IMPORT_META, RULE_ONE_COMPONENT_PER_FILE, RULE_ORDER_IN_COMPONENTS,
    RULE_PINIA_PREFER_STORE_TO_REFS, RULE_PREFER_COMPUTED, RULE_PREFER_DEFINE_EMITS_BINDING,
    RULE_PREFER_DEFINE_OPTIONS, RULE_PREFER_IMPORT_FROM_VUE, RULE_PREFER_PROP_TYPE_BOOLEAN_FIRST,
`,
    `    RULE_NO_THIS_IN_BEFORE_ROUTE_ENTER, RULE_NO_TOP_LEVEL_REF_IN_SCRIPT,
    RULE_NO_UNSTABLE_NESTED_COMPONENTS, RULE_NO_UNUSED_EMIT_DECLARATIONS,
    RULE_NO_USE_COMPUTED_PROPERTY_LIKE_METHOD, RULE_NO_WATCH_AFTER_AWAIT, RULE_NO_WITH_DEFAULTS,
    RULE_NUXT_CONFIG_KEYS_ORDER, RULE_NUXT_NO_CONFIG_TEST_KEY,
    RULE_NUXT_NO_PAGE_META_RUNTIME_VALUES, RULE_NUXT_PREFER_IMPORT_META,
    RULE_ONE_COMPONENT_PER_FILE, RULE_ORDER_IN_COMPONENTS, RULE_PINIA_PREFER_STORE_TO_REFS,
    RULE_PREFER_COMPUTED, RULE_PREFER_DEFINE_EMITS_BINDING, RULE_PREFER_DEFINE_OPTIONS,
    RULE_PREFER_IMPORT_FROM_VUE, RULE_PREFER_PROP_TYPE_BOOLEAN_FIRST,
`,
  ],
  [
    `    NoSideEffectsInComputed, NoTemplateRefOutsideInstance, NoThisInBeforeRouteEnter,
    NoTopLevelRefInScript, NoUnstableNestedComponents, NoUnusedEmitDeclarations,
    NoUseComputedPropertyLikeMethod, NoWatchAfterAwait, NoWithDefaults, NuxtConfigKeysOrder,
    OneComponentPerFile, OrderInComponents, PiniaPreferStoreToRefs, PreferComputed,
    PreferDefineEmitsBinding, PreferDefineOptions, PreferImportFromVue, PreferImportMeta,
    PreferPropTypeBooleanFirst, PreferRefOverReactive, PreferUseAttrs, PreferUseId, PreferUseSlots,
    PreferUseTemplateRef, RequireDefaultProp, RequireDefineComponent, RequireExplicitEmits,
    RequireExplicitSlots, RequireExpose, RequireFunctionReturnType, RequirePropTypeConstructor,
    RequirePropTypes, RequireRenderReturn, RequireSlotsAsFunctions, RequireSymbolProvide,
    RequireTypedObjectProp, RequireTypedRef, RequireValidDefaultProp, ReturnInComputedProperty,
    ReturnInEmitsValidator, ValidDefineEmits, ValidDefineOptions, ValidDefineProps,
    ValidModelDefinition, ValidNextTick, VueRouterPreferNamedPush, VueTestUtilsNoHtmlSnapshot,
`,
    `    NoSideEffectsInComputed, NoThisInBeforeRouteEnter, NoTopLevelRefInScript,
    NoUnstableNestedComponents, NoUnusedEmitDeclarations, NoUseComputedPropertyLikeMethod,
    NoWatchAfterAwait, NoWithDefaults, NuxtConfigKeysOrder, OneComponentPerFile, OrderInComponents,
    PiniaPreferStoreToRefs, PreferComputed, PreferDefineEmitsBinding, PreferDefineOptions,
    PreferImportFromVue, PreferImportMeta, PreferPropTypeBooleanFirst, PreferRefOverReactive,
    PreferUseAttrs, PreferUseId, PreferUseSlots, PreferUseTemplateRef, RequireDefaultProp,
    RequireDefineComponent, RequireExplicitEmits, RequireExplicitSlots, RequireExpose,
    RequireFunctionReturnType, RequirePropTypeConstructor, RequirePropTypes, RequireRenderReturn,
    RequireSlotsAsFunctions, RequireSymbolProvide, RequireTypedObjectProp, RequireTypedRef,
    RequireValidDefaultProp, ReturnInComputedProperty, ReturnInEmitsValidator, ValidDefineEmits,
    ValidDefineOptions, ValidDefineProps, ValidModelDefinition, ValidNextTick,
    VueRouterPreferNamedPush, VueTestUtilsNoHtmlSnapshot,
`,
  ],
  [
    `    BuiltinScriptRuleEntry { rule_name: RULE_NO_TEMPLATE_REF_OUTSIDE_INSTANCE, profile_name: "patina.script_rule.no_template_ref_outside_instance", category: "Script", fixable: false, presets: ESSENTIAL_SCRIPT_PRESETS, rule: &NoTemplateRefOutsideInstance },
`,
    ``,
  ],
] as const;

// Remove only that exact registration and its preset membership before the
// older ledgers replay. Whole-file identities keep any later edit fatal until
// it receives its own finite successor.
export function inverseTemplateRefOutsideInstance(name: string, body: Buffer): Buffer {
  if (name === registryPath) {
    identity(body, {
      bytes: 29090,
      sha256: "18921f24c06286d36633f29593a157b62041ae826cc2c9f6b3270f7f4bd82fb1",
    });
    let text = body.toString();
    for (const [after, before] of registration) {
      assert.equal(text.split(after).length, 2, name);
      text = text.replace(after, before);
    }
    const predecessor = Buffer.from(text);
    identity(predecessor, {
      bytes: 28763,
      sha256: "aa70fe2906e950e3218b888faabdc1e2636d19427f58b294039d1082200cb0de",
    });
    return predecessor;
  }
  if (name !== membershipPath) return body;
  identity(body, {
    bytes: 40388,
    sha256: "84bc6a8ea779d92098fa7bfa89e3f1f7eda7c868834b544a95fb6eb6559665e0",
  });
  const text = body.toString();
  const at = text.indexOf("{");
  const table = canonical(Buffer.from(text.slice(at)));
  for (const [preset, rules] of Object.entries(table)) {
    if (!Array.isArray(rules)) continue;
    const count = rules.filter((entry) => entry === rule).length;
    assert.equal(count, presets.includes(preset) ? 1 : 0, preset);
    if (count === 1) rules.splice(rules.indexOf(rule), 1);
  }
  const predecessor = Buffer.concat([Buffer.from(text.slice(0, at)), jsonBytes(table)]);
  identity(predecessor, {
    bytes: 40106,
    sha256: "720fee52a74a32d5685d5a2fa77b9d65c6cbfd08ddc7d1424ac050d7749621ed",
  });
  return predecessor;
}
