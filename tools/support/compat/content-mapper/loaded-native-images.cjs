const { createRequire } = require("node:module");
const { resolve } = require("node:path");
const consumer = createRequire(resolve("package.json"));
consumer("@vizejs/native");
process.stdout.write(
  JSON.stringify(Object.keys(require.cache).filter((file) => file.endsWith(".node"))),
);
