import assert from "node:assert/strict";

import { canonical, identity, jsonBytes, sha } from "./lint-preset-source-successors.ts";

export function currentPresetCases(body: Buffer, essentialCases: string[], manifest: any): any {
  identity(body, {
    bytes: 12915,
    sha256: "d347790320c83bbfb0ae513de8781559312766f48ac5c07483ce753146d579ea",
  });
  const value = JSON.parse(body.toString());
  assert.deepEqual(
    value.cases.map(({ id }: { id: string }) => id),
    ["else-for", "deep-props", ...essentialCases],
  );
  const { essentialExtensionAuthority, essentialBaselineAuthority, ...original } = value;
  assert(essentialExtensionAuthority && essentialBaselineAuthority);
  original.cases = value.cases.slice(0, 2);
  identity(jsonBytes(original), manifest.cases);
  for (const [index, checksum] of [
    "c99b63e767e42a975eb39bdfe14bb25b9fa2328165b0d29d49a9a27d9aec57b8",
    "7588faadaa8522d8fdc7783193decbe0ff3fcb611d183d08cb371e5f7af02399",
  ].entries())
    assert.equal(sha(jsonBytes(value.cases[index + 2])), checksum);
  return value;
}

// Remove only the exact 2026-11-06 general-default additions before older
// byte-identity audits replay their frozen source ledgers.
export function inverseLatestDefaultExpansion(name: string, body: Buffer): Buffer {
  if (name === "crates/vize_patina/src/rules/script/define_macros_order.rs") {
    const text = body.toString();
    const after = `        Statement::ImportDeclaration(_)
        | Statement::TSEnumDeclaration(_)
        | Statement::TSGlobalDeclaration(_)
        | Statement::TSModuleDeclaration(_)`;
    const before = `        Statement::ImportDeclaration(_)
        | Statement::TSEnumDeclaration(_)
        | Statement::TSModuleDeclaration(_)`;
    assert.equal(text.split(after).length, 2);
    return Buffer.from(text.replace(after, before));
  }
  if (name === "crates/vize_patina/src/linter/script_rules/registry.rs") {
    const text = body.toString();
    const after = `    RULE_REQUIRE_SLOTS_AS_FUNCTIONS, RULE_REQUIRE_TYPED_OBJECT_PROP, RULE_REQUIRE_TYPED_REF,
    RULE_REQUIRE_VALID_DEFAULT_PROP, RULE_RETURN_IN_COMPUTED_PROPERTY,
    RULE_RETURN_IN_EMITS_VALIDATOR, RULE_VALID_DEFINE_EMITS, RULE_VALID_DEFINE_OPTIONS,
    RULE_VALID_DEFINE_PROPS, RULE_VALID_NEXT_TICK, RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
    RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,`;
    const before = `    RULE_REQUIRE_SLOTS_AS_FUNCTIONS, RULE_REQUIRE_TYPED_REF, RULE_REQUIRE_VALID_DEFAULT_PROP,
    RULE_RETURN_IN_COMPUTED_PROPERTY, RULE_RETURN_IN_EMITS_VALIDATOR, RULE_VALID_DEFINE_EMITS,
    RULE_VALID_DEFINE_OPTIONS, RULE_VALID_DEFINE_PROPS, RULE_VALID_NEXT_TICK,
    RULE_VUE_ROUTER_PREFER_NAMED_PUSH, RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,`;
    assert.equal(text.split(after).length, 2);
    return Buffer.from(text.replace(after, before));
  }
  if (name === "crates/vize_patina/src/rules/opinionated/vue.rs") {
    const text = body.toString();
    const after = `pub(crate) mod authored_element;
mod component_name_in_template_casing;
mod component_registration;`;
    const before = `mod authored_element;
mod component_name_in_template_casing;`;
    assert.equal(text.split(after).length, 2);
    return Buffer.from(text.replace(after, before));
  }
  if (name === "crates/vize_patina/src/rule/happy_path.rs") {
    let text = body.toString();
    const strictSlotDefault = `        // Match eslint-plugin-vue's strict default. Component libraries which
        // intentionally encode dots in slot names can opt into
        // \`allowModifiers: true\`.
        registry.register(Box::new(crate::rules::vue::ValidVSlot));`;
    const legacySlotDefault = `        // Vue's parser records dots in static slot names as directive
        // modifiers. Component libraries intentionally use names such as
        // \`item.name\`, so the default preset keeps the historical behavior
        // of accepting them. Projects that need the stricter eslint-plugin-vue
        // policy can still set \`allowModifiers: false\` explicitly.
        registry.register(Box::new(
            crate::rules::vue::ValidVSlot::allowing_static_name_dots(),
        ));`;
    assert.equal(text.split(strictSlotDefault).length, 2);
    text = text.replace(strictSlotDefault, legacySlotDefault);
    const after = `        registry.register(Box::new(crate::rules::vue::NoUnusedProperties::default()));
        registry.register(Box::new(crate::rules::vue::NoUndefinedRefs));
        registry.register(Box::new(crate::rules::facts::NoUnusedSetupBindings));
        #[cfg(not(target_arch = "wasm32"))]`;
    const before = `        registry.register(Box::new(crate::rules::vue::NoUnusedProperties::default()));
        #[cfg(not(target_arch = "wasm32"))]`;
    assert.equal(text.split(after).length, 2);
    return Buffer.from(text.replace(after, before));
  }
  if (name === "crates/vize_patina/src/linter/script_rules/registry/rules.rs") {
    let text = body.toString();
    const replacements = [
      [
        `    RULE_COMPONENT_OPTIONS_NAME_CASING, RULE_CONSISTENT_PROPS_EMITS_KEY_QUOTES,
    RULE_CUSTOM_EVENT_NAME_CASING, RULE_DEFINE_EMITS_DECLARATION, RULE_DEFINE_MACROS_ORDER,
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
    RULE_PREFER_DEFINE_OPTIONS, RULE_PREFER_IMPORT_FROM_VUE, RULE_PREFER_PROP_TYPE_BOOLEAN_FIRST,`,
        `    RULE_COMPONENT_OPTIONS_NAME_CASING, RULE_CUSTOM_EVENT_NAME_CASING,
    RULE_DEFINE_EMITS_DECLARATION, RULE_DEFINE_MACROS_ORDER, RULE_DEFINE_PROPS_DECLARATION,
    RULE_DEFINE_PROPS_DESTRUCTURING, RULE_JSX_USES_VARS, RULE_NO_ARROW_FUNCTIONS_IN_WATCH,
    RULE_NO_ASYNC_IN_COMPUTED, RULE_NO_BOOLEAN_DEFAULT, RULE_NO_COMPUTED_PROPERTIES_IN_DATA,
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
    RULE_PREFER_IMPORT_FROM_VUE, RULE_PREFER_PROP_TYPE_BOOLEAN_FIRST,`,
      ],
      [
        `    ComponentOptionsNameCasing, ConsistentPropsEmitsKeyQuotes, CustomEventNameCasing,
    DefineEmitsDeclaration, DefineMacrosOrder, DefinePropsDeclaration, DefinePropsDestructuring,
    JsxUsesVars, NoArrowFunctionsInWatch, NoAsyncInComputed, NoBooleanDefault,
    NoComputedPropertiesInData, NoDeepDestructureInProps, NoDeprecatedDataObjectDeclaration,
    NoDeprecatedDeleteSet, NoDeprecatedDestroyedLifecycle, NoDeprecatedDollarListenersApi,
    NoDeprecatedDollarScopedSlotsApi, NoDeprecatedEventsApi, NoDeprecatedModelDefinition,
    NoDeprecatedPropsDefaultThis, NoDeprecatedVueConfigKeycodes, NoDupeKeys,
    NoDuplicateAttrInheritance, NoExportInScriptSetup, NoExposeAfterAwait, NoGetCurrentInstance,
    NoImportCompilerMacros, NoInternalImports, NoLifecycleAfterAwait, NoMultipleSlotArgs,
    NoNextTick, NoNuxtConfigTestKey, NoOptionsApi, NoPageMetaRuntimeValues,`,
        `    ComponentOptionsNameCasing, CustomEventNameCasing, DefineEmitsDeclaration, DefineMacrosOrder,
    DefinePropsDeclaration, DefinePropsDestructuring, JsxUsesVars, NoArrowFunctionsInWatch,
    NoAsyncInComputed, NoBooleanDefault, NoComputedPropertiesInData, NoDeepDestructureInProps,
    NoDeprecatedDataObjectDeclaration, NoDeprecatedDeleteSet, NoDeprecatedDestroyedLifecycle,
    NoDeprecatedDollarListenersApi, NoDeprecatedDollarScopedSlotsApi, NoDeprecatedEventsApi,
    NoDeprecatedModelDefinition, NoDeprecatedPropsDefaultThis, NoDeprecatedVueConfigKeycodes,
    NoDupeKeys, NoDuplicateAttrInheritance, NoExportInScriptSetup, NoExposeAfterAwait,
    NoGetCurrentInstance, NoImportCompilerMacros, NoInternalImports, NoLifecycleAfterAwait,
    NoMultipleSlotArgs, NoNextTick, NoNuxtConfigTestKey, NoOptionsApi, NoPageMetaRuntimeValues,`,
      ],
      [
        `    RULE_REQUIRE_DEFINE_COMPONENT, RULE_REQUIRE_EXPLICIT_EMITS, RULE_REQUIRE_EXPLICIT_SLOTS,
    RULE_REQUIRE_EXPOSE, RULE_REQUIRE_FUNCTION_RETURN_TYPE, RULE_REQUIRE_PROP_TYPE_CONSTRUCTOR,
    RULE_REQUIRE_PROP_TYPES, RULE_REQUIRE_RENDER_RETURN, RULE_REQUIRE_SLOTS_AS_FUNCTIONS,
    RULE_REQUIRE_SYMBOL_PROVIDE, RULE_REQUIRE_TYPED_OBJECT_PROP, RULE_REQUIRE_TYPED_REF,
    RULE_REQUIRE_VALID_DEFAULT_PROP, RULE_RETURN_IN_COMPUTED_PROPERTY,
    RULE_RETURN_IN_EMITS_VALIDATOR, RULE_VALID_DEFINE_EMITS, RULE_VALID_DEFINE_OPTIONS,
    RULE_VALID_DEFINE_PROPS, RULE_VALID_MODEL_DEFINITION, RULE_VALID_NEXT_TICK,
    RULE_VUE_ROUTER_PREFER_NAMED_PUSH, RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,`,
        `    RULE_REQUIRE_EXPLICIT_EMITS, RULE_REQUIRE_EXPLICIT_SLOTS, RULE_REQUIRE_EXPOSE,
    RULE_REQUIRE_FUNCTION_RETURN_TYPE, RULE_REQUIRE_PROP_TYPE_CONSTRUCTOR, RULE_REQUIRE_PROP_TYPES,
    RULE_REQUIRE_RENDER_RETURN, RULE_REQUIRE_SLOTS_AS_FUNCTIONS, RULE_REQUIRE_SYMBOL_PROVIDE,
    RULE_REQUIRE_TYPED_OBJECT_PROP, RULE_REQUIRE_TYPED_REF, RULE_REQUIRE_VALID_DEFAULT_PROP,
    RULE_RETURN_IN_COMPUTED_PROPERTY, RULE_RETURN_IN_EMITS_VALIDATOR, RULE_VALID_DEFINE_EMITS,
    RULE_VALID_DEFINE_OPTIONS, RULE_VALID_DEFINE_PROPS, RULE_VALID_MODEL_DEFINITION,
    RULE_VALID_NEXT_TICK, RULE_VUE_ROUTER_PREFER_NAMED_PUSH, RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,`,
      ],
      [
        `    RequireDefineComponent, RequireExplicitEmits, RequireExplicitSlots, RequireExpose,
    RequireFunctionReturnType, RequirePropTypeConstructor, RequirePropTypes, RequireRenderReturn,
    RequireSlotsAsFunctions, RequireSymbolProvide, RequireTypedObjectProp, RequireTypedRef,
    RequireValidDefaultProp, ReturnInComputedProperty, ReturnInEmitsValidator, ValidDefineEmits,
    ValidDefineOptions, ValidDefineProps, ValidModelDefinition, ValidNextTick,
    VueRouterPreferNamedPush, VueTestUtilsNoHtmlSnapshot,`,
        `    RequireExplicitEmits, RequireExplicitSlots, RequireExpose, RequireFunctionReturnType,
    RequirePropTypeConstructor, RequirePropTypes, RequireRenderReturn, RequireSlotsAsFunctions,
    RequireSymbolProvide, RequireTypedObjectProp, RequireTypedRef, RequireValidDefaultProp,
    ReturnInComputedProperty, ReturnInEmitsValidator, ValidDefineEmits, ValidDefineOptions,
    ValidDefineProps, ValidModelDefinition, ValidNextTick, VueRouterPreferNamedPush,
    VueTestUtilsNoHtmlSnapshot,`,
      ],
      [
        `    BuiltinScriptRuleEntry { rule_name: RULE_DEFINE_PROPS_DESTRUCTURING, profile_name: "patina.script_rule.define_props_destructuring", category: "Script", fixable: false, presets: HAPPY_PATH_SCRIPT_PRESETS, rule: &DefinePropsDestructuring },`,
        `    BuiltinScriptRuleEntry { rule_name: RULE_DEFINE_PROPS_DESTRUCTURING, profile_name: "patina.script_rule.define_props_destructuring", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &DefinePropsDestructuring },`,
      ],
      [
        `    BuiltinScriptRuleEntry { rule_name: RULE_REQUIRE_DEFINE_COMPONENT, profile_name: "patina.script_rule.require_define_component", category: "Script", fixable: false, presets: HAPPY_PATH_SCRIPT_PRESETS, rule: &RequireDefineComponent },\n`,
        "",
      ],
      [
        `    BuiltinScriptRuleEntry { rule_name: RULE_CONSISTENT_PROPS_EMITS_KEY_QUOTES, profile_name: "patina.script_rule.consistent_props_emits_key_quotes", category: "Script", fixable: false, presets: OPINIONATED_ONLY_SCRIPT_PRESETS, rule: &ConsistentPropsEmitsKeyQuotes },\n`,
        "",
      ],
      [
        `    BuiltinScriptRuleEntry { rule_name: RULE_NO_THIS_IN_BEFORE_ROUTE_ENTER, profile_name: "patina.script_rule.no_this_in_before_route_enter", category: "Script", fixable: false, presets: HAPPY_PATH_SCRIPT_PRESETS, rule: &NoThisInBeforeRouteEnter },`,
        `    BuiltinScriptRuleEntry { rule_name: RULE_NO_THIS_IN_BEFORE_ROUTE_ENTER, profile_name: "patina.script_rule.no_this_in_before_route_enter", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &NoThisInBeforeRouteEnter },`,
      ],
    ];
    for (const [after, before] of replacements) {
      assert.equal(text.split(after).length, 2, after);
      text = text.replace(after, before);
    }
    return Buffer.from(text);
  }
  if (name === "tests/_fixtures/patina-eslint-vue-rule-map.json") {
    const value = canonical(body);
    for (const rule of ["vue/define-props-destructuring", "vue/no-this-in-before-route-enter"]) {
      assert.deepEqual(value.entries[rule].patinaPresets, [
        "ecosystem",
        "general-recommended",
        "nuxt",
        "opinionated",
      ]);
      value.entries[rule].patinaPresets = [];
    }
    return jsonBytes(value);
  }
  if (
    name ===
    "crates/vize_patina/src/preset/snapshots/vize_patina__preset__tests__lint_preset_rule_membership.snap"
  ) {
    const text = body.toString();
    const at = text.indexOf("{");
    const table = canonical(Buffer.from(text.slice(at)));
    const broad = ["happy_path", "ecosystem", "opinionated", "nuxt"];
    const promoted = [
      "script/define-props-destructuring",
      "vue/no-undefined-refs",
      "vue/no-unused-setup-bindings",
      "vue/no-this-in-before-route-enter",
    ];
    for (const preset of broad)
      for (const rule of promoted) {
        assert.equal(table[preset].filter((name: string) => name === rule).length, 1);
        table[preset].splice(table[preset].indexOf(rule), 1);
      }
    for (const preset of [...broad, "opt_in"]) {
      const rule = "script/require-define-component";
      assert.equal(table[preset].filter((name: string) => name === rule).length, 1);
      table[preset].splice(table[preset].indexOf(rule), 1);
    }
    for (const preset of ["opinionated", "opt_in"]) {
      const rule = "script/consistent-props-emits-key-quotes";
      assert.equal(table[preset].filter((name: string) => name === rule).length, 1);
      table[preset].splice(table[preset].indexOf(rule), 1);
    }
    return Buffer.concat([Buffer.from(text.slice(0, at)), jsonBytes(table)]);
  }
  return body;
}
