export default {
  vue: {
    linter: {
      preset: "incremental",
      rules: {
        "script/require-explicit-slots": "error",
        "vue/slot-name-casing": "error",
        "vue/no-useless-template-attributes": "error",
        "a11y/interactive-supports-focus": "error",
        "a11y/no-static-element-interactions": "error",
        "a11y/click-events-have-key-events": "error",
      },
      ruleOptions: { "vue/slot-name-casing": "kebab-case" },
    },
  },
};
