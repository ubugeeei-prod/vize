import fs from "node:fs";
fs.writeFileSync(
  "package.json",
  JSON.stringify({
    private: true,
    type: "module",
    dependencies: {
      vize: "0.608.0",
      "@vizejs/native": "0.608.0",
      vue: "3.5.42",
      typescript: "6.0.3",
      "typescript-native": "npm:typescript@7.1.0-beta",
      "vue-tsc": "3.3.13",
    },
    overrides: {
      "@vue/compiler-dom": "3.5.42",
      "@vue/compiler-core": "3.5.42",
      "@vue/compiler-sfc": "3.5.42",
      "@vue/shared": "3.5.42",
    },
  }),
);
