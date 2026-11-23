export const shared = {
  linter: {
    enabled: true,
    preset: "incremental",
    rules: { "vue/attribute-hyphenation": "warn", "vue/require-v-for-key": "error" },
    ruleOptions: {
      "vue/attribute-hyphenation": "always",
      "vue/component-name-in-template-casing": { casing: "PascalCase" },
      "vue/sfc-element-order": { order: ["script", "template", "style"] },
    },
  },
  entries: [
    {
      files: ["src/Fragment.vue"],
      linter: { rules: { "vue/no-multiple-template-root": "off" } },
    },
  ],
};
