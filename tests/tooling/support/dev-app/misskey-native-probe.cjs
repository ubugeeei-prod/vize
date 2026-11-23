const path = require("node:path");
const { createRequire } = require("node:module");
const backend = createRequire(path.resolve("packages/backend/package.json"));
backend("re2");
