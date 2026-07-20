import { readFile } from "node:fs/promises";
import { gzipSync } from "node:zlib";

const maximumGzipBytes = 3 * 1024;
const gzipBytes = gzipSync(
  await readFile(new URL("../dist/index.mjs", import.meta.url)),
).byteLength;

console.log(JSON.stringify({ entry: "@vizeui/core", gzipBytes, maximumGzipBytes }));
if (gzipBytes > maximumGzipBytes) process.exitCode = 1;
