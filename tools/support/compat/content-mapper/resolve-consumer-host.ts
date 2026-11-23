import path from "node:path";
import { pathToFileURL } from "node:url";

const consumer = process.argv[2];
if (!consumer) throw new Error("Expected a consumer project directory");
const entry = path.resolve(consumer, "node_modules/typescript/lib/getExePath.js");
const { default: getExePath } = await import(pathToFileURL(entry).href);
console.log(getExePath());
