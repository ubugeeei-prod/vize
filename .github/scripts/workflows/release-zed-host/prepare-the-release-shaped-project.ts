import fs from "node:fs";
import path from "node:path";
const root = process.argv[2];
fs.writeFileSync(
  path.join(root, "package.json"),
  `${JSON.stringify(
    {
      name: "vize-release-zed-smoke",
      version: "0.0.0",
      private: true,
      type: "module",
      dependencies: { vue: "3.5.34" },
      devDependencies: { typescript: "6.0.3", vite: "8.0.0" },
    },
    null,
    2,
  )}\n`,
);
fs.writeFileSync(path.join(root, "vite.config.ts"), "export default { vue: {} };\n");
fs.writeFileSync(
  path.join(root, "src/Lint.vue"),
  `<script setup lang="ts">
const items = [1, 2]
</script>
<template><ul><li v-for="item in items">{{ item }}</li></ul></template>
`,
);
