import { createRequire } from "node:module";
import path from "node:path";

createRequire(path.resolve("tests/package.json")).resolve("vue/compiler-sfc");
