const requireFromWorkspace = require("node:module").createRequire(
  require("node:path").resolve("package.json"),
);
console.log(
  requireFromWorkspace("./tools/benchmarks/typecheck-hosts/package.json").dependencies.tsrs,
);
