require("node:fs").writeFileSync(
  "package.json",
  JSON.stringify({
    private: true,
    dependencies: {
      vue: "3.5.44",
      typescript: "6.0.3",
      "typescript-native": "npm:typescript@7.1.0-beta",
      ajv: "8.17.1",
    },
  }),
);
