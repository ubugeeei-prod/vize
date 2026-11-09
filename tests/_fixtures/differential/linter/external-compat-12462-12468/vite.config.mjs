export default {
  vue: {
    linter: {
      preset: "incremental",
      rules: {
        "vue/warn-custom-directive": "error",
        "a11y/label-has-for": "error",
        "a11y/anchor-has-content": "error",
        "vue/no-bare-strings-in-template": "error",
      },
      ruleOptions: {
        "vue/no-bare-strings-in-template": {
          allowlist: ["+", "Ctrl+K", "Tip", "Level", "Upload"],
        },
      },
    },
  },
};
