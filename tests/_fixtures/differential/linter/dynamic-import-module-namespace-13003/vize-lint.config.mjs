export default {
  vue: {
    linter: {
      preset: "incremental",
      rules: {
        "type/no-unsafe-argument": "error",
        "type/no-unsafe-assignment": "error",
        "type/no-unsafe-call": "error",
        "type/no-unsafe-member-access": "error",
        "type/no-unsafe-return": "error",
      },
      typeAware: true,
    },
  },
};
