require("node:fs").writeFileSync(
  process.env.RUNNER_TEMP + "/partition-provider/package.json",
  JSON.stringify({
    private: true,
    dependencies: {
      vue: "3.6.0-rc.6",
      typescript: "6.0.3",
      "native-typescript": "npm:typescript@7.1.0-beta",
      "vue-language-tools-fixture-vue": "npm:vue@3.6.0-rc.6",
      "vue-component-type-helpers": "3.3.14",
    },
  }),
);
