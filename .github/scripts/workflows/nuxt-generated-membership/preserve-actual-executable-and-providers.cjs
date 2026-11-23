const fs = require("node:fs");
for (const p of [
  "node_modules/vue/package.json",
  "tests/node_modules/vue-tsc/package.json",
  "tests/node_modules/typescript/package.json",
  "node_modules/@typescript/typescript-linux-x64/package.json",
])
  if (fs.existsSync(p)) console.log(p, fs.readFileSync(p, "utf8"));
