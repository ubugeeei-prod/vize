const requireFromWorkspace = require("node:module").createRequire(
  require("node:path").resolve("package.json"),
);
console.log(
  "@vue/compiler-sfc " +
    requireFromWorkspace("./tools/benchmarks/scripts/node_modules/@vue/compiler-sfc/package.json")
      .version,
);
