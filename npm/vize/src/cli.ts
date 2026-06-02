import { createRequire } from "node:module";
import { dirname } from "node:path";

const require = createRequire(import.meta.url);
const native = require("@vizejs/native") as typeof import("@vizejs/native");

process.env.VIZE_VUE_PACKAGE ??= dirname(require.resolve("vue/package.json"));

native.runCli(process.argv.slice(2));
