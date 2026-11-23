const { createRequire } = require("node:module");
const path = require("node:path");
const workspace = createRequire(path.resolve("package.json"));
console.log(workspace.resolve("vue/package.json"));
