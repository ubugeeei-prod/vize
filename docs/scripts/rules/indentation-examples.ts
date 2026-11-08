import type { RuleExample, RuleMetadata } from "./types.ts";

// Authored from the complete original Rust docs and manual CSS carrier for #8363.
// These references do not come from generated Markdown or observed browser output.
export const ruleIndentationExamples = [
  {
    name: "vue/no-deprecated-slot-attribute",
    sourcePath: "crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs",
    expected: {
      bad: {
        language: "vue",
        source:
          '<template>\n  <Foo>\n    <template slot="header"><h1>Title</h1></template>\n    <div :slot="name">Title</div>\n  </Foo>\n</template>',
      },
      good: {
        language: "vue",
        source:
          "<template>\n  <Foo>\n    <template v-slot:header><h1>Title</h1></template>\n  </Foo>\n</template>",
      },
      evidence: "crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs",
    },
  },
  {
    name: "script/component-options-name-casing",
    sourcePath: "crates/vize_patina/src/rules/script/component_options_name_casing.rs",
    expected: {
      bad: {
        language: "vue",
        source: '<script lang="ts">\nexport default {\n  components: { myComponent }\n}\n</script>',
      },
      good: {
        language: "vue",
        source: '<script lang="ts">\nexport default {\n  components: { MyComponent }\n}\n</script>',
      },
      evidence: "crates/vize_patina/src/rules/script/component_options_name_casing.rs",
    },
  },
  {
    name: "petite-vue/valid-v-scope",
    sourcePath: "crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs",
    expected: {
      bad: {
        language: "html",
        source:
          '<!doctype html>\n<html><body>\n<div v-scope="count"></div>\n<div v-scope="foo()"></div>\n<div v-scope="a + b"></div>\n<div v-scope="123"></div>\n<script src="https://unpkg.com/petite-vue" init></script>\n</body></html>',
      },
      good: {
        language: "html",
        source:
          '<!doctype html>\n<html><body>\n<div v-scope></div>\n<div v-scope="{}"></div>\n<div v-scope="{ count: 0 }"></div>\n<div v-scope="({ count: 0 })"></div>\n<script src="https://unpkg.com/petite-vue" init></script>\n</body></html>',
      },
      evidence: "crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs",
    },
  },
  {
    name: "css/no-important",
    sourcePath: "docs/scripts/rules/manual-3.ts",
    expected: {
      bad: {
        language: "vue",
        source: "<style scoped>\n.button {\n  color: red !important;\n}\n</style>",
      },
      good: {
        language: "vue",
        source: "<style scoped>\n.button {\n  color: var(--button-color);\n}\n</style>",
      },
      evidence: "docs/content/rules/css.md",
    },
  },
] as const satisfies readonly {
  name: string;
  sourcePath: string;
  expected: RuleExample;
}[];

export function indentationRuleMetadata(
  fixture: (typeof ruleIndentationExamples)[number],
): RuleMetadata {
  return {
    name: fixture.name,
    category: "Essential",
    defaultSeverity: "error",
    description: fixture.name,
    fixable: false,
    implementationLine: 1,
    implementationPath: fixture.sourcePath,
    metaType: "RuleMeta",
    presets: [],
  };
}
