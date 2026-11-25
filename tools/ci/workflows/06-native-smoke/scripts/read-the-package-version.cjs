const requireFromWorkspace = require("node:module").createRequire(
  require("node:path").resolve("package.json"),
);
console.log(`version=${requireFromWorkspace("./npm/native/package.json").version}`);
