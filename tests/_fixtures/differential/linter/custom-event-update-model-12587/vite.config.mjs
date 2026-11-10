export default {
  vue: {
    linter: {
      preset: "incremental",
      rules: { "script/custom-event-name-casing": "error" },
      ruleOptions: {
        "script/custom-event-name-casing": { casing: "kebab-case" },
      },
    },
  },
};
