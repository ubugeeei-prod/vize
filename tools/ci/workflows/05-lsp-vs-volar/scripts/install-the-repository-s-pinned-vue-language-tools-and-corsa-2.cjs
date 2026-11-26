const requireFromWorkspace = require("node:module").createRequire(
  require("node:path").resolve("package.json"),
);
for (const name of [
  "@vue/language-server",
  "@vue/typescript-plugin",
  "@vue/language-core",
  "@volar/language-server",
  "typescript",
  "@typescript/typescript-linux-x64",
  "tsrs",
])
  console.log(name, requireFromWorkspace(`./node_modules/${name}/package.json`).version);
