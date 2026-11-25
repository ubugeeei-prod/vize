import assert from "node:assert/strict";

import { canonical, identity, jsonBytes } from "./lint-preset-source-successors.ts";

const registryPath = "crates/vize_patina/src/linter/script_rules/registry/rules.rs";
const membershipPath =
  "crates/vize_patina/src/preset/snapshots/vize_patina__preset__tests__lint_preset_rule_membership.snap";

// This branch registers the opt-in script/consistent-type-imports.
const rule = "script/consistent-type-imports";

// The registry text with the rule, then without it. rustfmt reflows both
// import lists around the added names.
const registration = [
  [
    `    RULE_CONSISTENT_TYPE_IMPORTS, RULE_CUSTOM_EVENT_NAME_CASING, RULE_DEFINE_EMITS_DECLARATION,
    RULE_DEFINE_MACROS_ORDER, RULE_DEFINE_PROPS_DECLARATION, RULE_DEFINE_PROPS_DESTRUCTURING,
    RULE_JSX_USES_VARS, RULE_NO_ARROW_FUNCTIONS_IN_WATCH, RULE_NO_ASYNC_IN_COMPUTED,
    RULE_NO_BOOLEAN_DEFAULT, RULE_NO_COMPUTED_PROPERTIES_IN_DATA,
    RULE_NO_DEEP_DESTRUCTURE_IN_PROPS, RULE_NO_DEPRECATED_DATA_OBJECT_DECLARATION,
    RULE_NO_DEPRECATED_DELETE_SET, RULE_NO_DEPRECATED_DESTROYED_LIFECYCLE,
    RULE_NO_DEPRECATED_DOLLAR_LISTENERS_API, RULE_NO_DEPRECATED_DOLLAR_SCOPEDSLOTS_API,
    RULE_NO_DEPRECATED_EVENTS_API, RULE_NO_DEPRECATED_MODEL_DEFINITION,
    RULE_NO_DEPRECATED_PROPS_DEFAULT_THIS, RULE_NO_DEPRECATED_VUE_CONFIG_KEYCODES,
    RULE_NO_DUPE_KEYS, RULE_NO_DUPLICATE_ATTR_INHERITANCE, RULE_NO_EXPORT_IN_SCRIPT_SETUP,
    RULE_NO_EXPOSE_AFTER_AWAIT, RULE_NO_GET_CURRENT_INSTANCE, RULE_NO_IMPORT_COMPILER_MACROS,
    RULE_NO_INTERNAL_IMPORTS, RULE_NO_LIFECYCLE_AFTER_AWAIT, RULE_NO_MULTIPLE_SLOT_ARGS,
    RULE_NO_NEXT_TICK, RULE_NO_OPTIONS_API, RULE_NO_POTENTIAL_COMPONENT_OPTION_TYPO,
    RULE_NO_REACTIVE_DESTRUCTURE, RULE_NO_REF_AS_OPERAND, RULE_NO_REF_OBJECT_REACTIVITY_LOSS,
    RULE_NO_REQUIRED_PROP_WITH_DEFAULT, RULE_NO_RESERVED_IDENTIFIERS, RULE_NO_RESERVED_KEYS,
    RULE_NO_RESERVED_PROPS, RULE_NO_RESTRICTED_GLOBALS, RULE_NO_RESTRICTED_MEMBERS,
    RULE_NO_SETUP_PROPS_REACTIVITY_LOSS, RULE_NO_SIDE_EFFECTS_IN_COMPUTED,
    RULE_NO_THIS_IN_BEFORE_ROUTE_ENTER, RULE_NO_TOP_LEVEL_REF_IN_SCRIPT,
    RULE_NO_UNSTABLE_NESTED_COMPONENTS, RULE_NO_UNUSED_EMIT_DECLARATIONS,
    RULE_NO_USE_COMPUTED_PROPERTY_LIKE_METHOD, RULE_NO_WATCH_AFTER_AWAIT, RULE_NO_WITH_DEFAULTS,
    RULE_NUXT_CONFIG_KEYS_ORDER, RULE_NUXT_NO_CONFIG_TEST_KEY,
    RULE_NUXT_NO_PAGE_META_RUNTIME_VALUES, RULE_NUXT_PREFER_IMPORT_META,
    RULE_ONE_COMPONENT_PER_FILE, RULE_ORDER_IN_COMPONENTS, RULE_PINIA_PREFER_STORE_TO_REFS,
    RULE_PREFER_COMPUTED, RULE_PREFER_DEFINE_EMITS_BINDING, RULE_PREFER_DEFINE_OPTIONS,
    RULE_PREFER_IMPORT_FROM_VUE, RULE_PREFER_PROP_TYPE_BOOLEAN_FIRST,
`,
    `    RULE_CUSTOM_EVENT_NAME_CASING, RULE_DEFINE_EMITS_DECLARATION, RULE_DEFINE_MACROS_ORDER,
    RULE_DEFINE_PROPS_DECLARATION, RULE_DEFINE_PROPS_DESTRUCTURING, RULE_JSX_USES_VARS,
    RULE_NO_ARROW_FUNCTIONS_IN_WATCH, RULE_NO_ASYNC_IN_COMPUTED, RULE_NO_BOOLEAN_DEFAULT,
    RULE_NO_COMPUTED_PROPERTIES_IN_DATA, RULE_NO_DEEP_DESTRUCTURE_IN_PROPS,
    RULE_NO_DEPRECATED_DATA_OBJECT_DECLARATION, RULE_NO_DEPRECATED_DELETE_SET,
    RULE_NO_DEPRECATED_DESTROYED_LIFECYCLE, RULE_NO_DEPRECATED_DOLLAR_LISTENERS_API,
    RULE_NO_DEPRECATED_DOLLAR_SCOPEDSLOTS_API, RULE_NO_DEPRECATED_EVENTS_API,
    RULE_NO_DEPRECATED_MODEL_DEFINITION, RULE_NO_DEPRECATED_PROPS_DEFAULT_THIS,
    RULE_NO_DEPRECATED_VUE_CONFIG_KEYCODES, RULE_NO_DUPE_KEYS, RULE_NO_DUPLICATE_ATTR_INHERITANCE,
    RULE_NO_EXPORT_IN_SCRIPT_SETUP, RULE_NO_EXPOSE_AFTER_AWAIT, RULE_NO_GET_CURRENT_INSTANCE,
    RULE_NO_IMPORT_COMPILER_MACROS, RULE_NO_INTERNAL_IMPORTS, RULE_NO_LIFECYCLE_AFTER_AWAIT,
    RULE_NO_MULTIPLE_SLOT_ARGS, RULE_NO_NEXT_TICK, RULE_NO_OPTIONS_API,
    RULE_NO_POTENTIAL_COMPONENT_OPTION_TYPO, RULE_NO_REACTIVE_DESTRUCTURE, RULE_NO_REF_AS_OPERAND,
    RULE_NO_REF_OBJECT_REACTIVITY_LOSS, RULE_NO_REQUIRED_PROP_WITH_DEFAULT,
    RULE_NO_RESERVED_IDENTIFIERS, RULE_NO_RESERVED_KEYS, RULE_NO_RESERVED_PROPS,
    RULE_NO_RESTRICTED_GLOBALS, RULE_NO_RESTRICTED_MEMBERS, RULE_NO_SETUP_PROPS_REACTIVITY_LOSS,
    RULE_NO_SIDE_EFFECTS_IN_COMPUTED, RULE_NO_THIS_IN_BEFORE_ROUTE_ENTER,
    RULE_NO_TOP_LEVEL_REF_IN_SCRIPT, RULE_NO_UNSTABLE_NESTED_COMPONENTS,
    RULE_NO_UNUSED_EMIT_DECLARATIONS, RULE_NO_USE_COMPUTED_PROPERTY_LIKE_METHOD,
    RULE_NO_WATCH_AFTER_AWAIT, RULE_NO_WITH_DEFAULTS, RULE_NUXT_CONFIG_KEYS_ORDER,
    RULE_NUXT_NO_CONFIG_TEST_KEY, RULE_NUXT_NO_PAGE_META_RUNTIME_VALUES,
    RULE_NUXT_PREFER_IMPORT_META, RULE_ONE_COMPONENT_PER_FILE, RULE_ORDER_IN_COMPONENTS,
    RULE_PINIA_PREFER_STORE_TO_REFS, RULE_PREFER_COMPUTED, RULE_PREFER_DEFINE_EMITS_BINDING,
    RULE_PREFER_DEFINE_OPTIONS, RULE_PREFER_IMPORT_FROM_VUE, RULE_PREFER_PROP_TYPE_BOOLEAN_FIRST,
`,
  ],
  [
    `    ComponentOptionsNameCasing, ConsistentPropsEmitsKeyQuotes, ConsistentTypeImports,
    CustomEventNameCasing, DefineEmitsDeclaration, DefineMacrosOrder, DefinePropsDeclaration,
    DefinePropsDestructuring, JsxUsesVars, NoArrowFunctionsInWatch, NoAsyncInComputed,
    NoBooleanDefault, NoComputedPropertiesInData, NoDeepDestructureInProps,
    NoDeprecatedDataObjectDeclaration, NoDeprecatedDeleteSet, NoDeprecatedDestroyedLifecycle,
    NoDeprecatedDollarListenersApi, NoDeprecatedDollarScopedSlotsApi, NoDeprecatedEventsApi,
    NoDeprecatedModelDefinition, NoDeprecatedPropsDefaultThis, NoDeprecatedVueConfigKeycodes,
    NoDupeKeys, NoDuplicateAttrInheritance, NoExportInScriptSetup, NoExposeAfterAwait,
    NoGetCurrentInstance, NoImportCompilerMacros, NoInternalImports, NoLifecycleAfterAwait,
    NoMultipleSlotArgs, NoNextTick, NoNuxtConfigTestKey, NoOptionsApi, NoPageMetaRuntimeValues,
`,
    `    ComponentOptionsNameCasing, ConsistentPropsEmitsKeyQuotes, CustomEventNameCasing,
    DefineEmitsDeclaration, DefineMacrosOrder, DefinePropsDeclaration, DefinePropsDestructuring,
    JsxUsesVars, NoArrowFunctionsInWatch, NoAsyncInComputed, NoBooleanDefault,
    NoComputedPropertiesInData, NoDeepDestructureInProps, NoDeprecatedDataObjectDeclaration,
    NoDeprecatedDeleteSet, NoDeprecatedDestroyedLifecycle, NoDeprecatedDollarListenersApi,
    NoDeprecatedDollarScopedSlotsApi, NoDeprecatedEventsApi, NoDeprecatedModelDefinition,
    NoDeprecatedPropsDefaultThis, NoDeprecatedVueConfigKeycodes, NoDupeKeys,
    NoDuplicateAttrInheritance, NoExportInScriptSetup, NoExposeAfterAwait, NoGetCurrentInstance,
    NoImportCompilerMacros, NoInternalImports, NoLifecycleAfterAwait, NoMultipleSlotArgs,
    NoNextTick, NoNuxtConfigTestKey, NoOptionsApi, NoPageMetaRuntimeValues,
`,
  ],
  [
    `static CONSISTENT_TYPE_IMPORTS_RULE: ConsistentTypeImports = ConsistentTypeImports::new();
`,
    ``,
  ],
  [
    `    BuiltinScriptRuleEntry { rule_name: RULE_CONSISTENT_TYPE_IMPORTS, profile_name: "patina.script_rule.consistent_type_imports", category: "Script", fixable: true, presets: OPT_IN_SCRIPT_PRESETS, rule: &CONSISTENT_TYPE_IMPORTS_RULE },
`,
    ``,
  ],
] as const;

// Remove only that exact registration and its opt-in membership before the
// older ledgers replay. Whole-file identities keep any later edit fatal until
// it receives its own finite successor.
export function inverseConsistentTypeImports(name: string, body: Buffer): Buffer {
  if (name === registryPath) {
    identity(body, {
      bytes: 28763,
      sha256: "aa70fe2906e950e3218b888faabdc1e2636d19427f58b294039d1082200cb0de",
    });
    let text = body.toString();
    for (const [after, before] of registration) {
      assert.equal(text.split(after).length, 2, name);
      text = text.replace(after, before);
    }
    const predecessor = Buffer.from(text);
    identity(predecessor, {
      bytes: 28379,
      sha256: "9e5d71af9eb6bf367e7ac7b563035ec583826c62108e931eba4c8ed671ca867c",
    });
    return predecessor;
  }
  if (name !== membershipPath) return body;
  identity(body, {
    bytes: 39954,
    sha256: "4e27911d86ffe7fa03b7dfa8b183f352536b9b10892d83468d0f9581efaafcf3",
  });
  const text = body.toString();
  const at = text.indexOf("{");
  const table = canonical(Buffer.from(text.slice(at)));
  for (const [preset, rules] of Object.entries(table)) {
    if (!Array.isArray(rules)) continue;
    const count = rules.filter((entry) => entry === rule).length;
    assert.equal(count, preset === "opt_in" ? 1 : 0, preset);
    if (count === 1) rules.splice(rules.indexOf(rule), 1);
  }
  const predecessor = Buffer.concat([Buffer.from(text.slice(0, at)), jsonBytes(table)]);
  identity(predecessor, {
    bytes: 39916,
    sha256: "87ca2b8cd7d533045bcbb09e79e2cf4eba5f9b11f30b39a63b8fad0ef423b51c",
  });
  return predecessor;
}
