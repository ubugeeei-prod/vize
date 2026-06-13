import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const native = require("@vizejs/native") as typeof import("@vizejs/native");

native.runCli(process.argv.slice(2));
