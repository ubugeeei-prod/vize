const requireFromWorkspace = require("node:module").createRequire(
  require("node:path").resolve("package.json"),
);
for (const name of ["@typescript/typescript-linux-x64", "vize"])
  console.log(
    "mapper:" + name,
    requireFromWorkspace(`./node_modules/${name}/package.json`).version,
  );
