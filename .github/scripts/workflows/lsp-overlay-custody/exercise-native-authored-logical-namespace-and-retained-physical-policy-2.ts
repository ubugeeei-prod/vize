import { resolve } from "node:path";
import { createRequire } from "node:module";
console.log(createRequire(resolve("tests/package.json")).resolve("typescript/lib/tsc.js"));
