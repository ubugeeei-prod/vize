export default {
  vue: {
    linter: {
      preset: "incremental",
      rules: { "script/define-macros-order": "warn" },
      ruleOptions: {
        "script/define-macros-order": {
          order: ["defineOptions", "defineModel", "defineProps", "defineEmits"],
        },
      },
    },
  },
};
