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
  if (name === "crates/vize_patina/src/rule/happy_path.rs") {
    const text = body.toString();
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
      ["EventNameCasing::Lowercase", "EventNameCasing::CamelCase"],
      [
        `    BuiltinScriptRuleEntry { rule_name: RULE_DEFINE_PROPS_DESTRUCTURING, profile_name: "patina.script_rule.define_props_destructuring", category: "Script", fixable: false, presets: HAPPY_PATH_SCRIPT_PRESETS, rule: &DefinePropsDestructuring },`,
        `    BuiltinScriptRuleEntry { rule_name: RULE_DEFINE_PROPS_DESTRUCTURING, profile_name: "patina.script_rule.define_props_destructuring", category: "Script", fixable: false, presets: OPT_IN_SCRIPT_PRESETS, rule: &DefinePropsDestructuring },`,
      ],
      [
        `    BuiltinScriptRuleEntry { rule_name: RULE_REQUIRE_DEFINE_COMPONENT, profile_name: "patina.script_rule.require_define_component", category: "Script", fixable: false, presets: HAPPY_PATH_SCRIPT_PRESETS, rule: &RequireDefineComponent },\n`,
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
    return Buffer.concat([Buffer.from(text.slice(0, at)), jsonBytes(table)]);
  }
  return body;
}
