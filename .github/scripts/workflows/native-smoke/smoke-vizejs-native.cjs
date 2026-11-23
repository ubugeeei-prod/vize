const { createRequire } = require("node:module");
const { resolve } = require("node:path");

const requireFromWorkspace = createRequire(resolve("package.json"));
const native = requireFromWorkspace("./npm/native");
if (typeof native.compileSfc !== "function") throw new Error("compileSfc missing");
