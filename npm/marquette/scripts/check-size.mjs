import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const entryPath = path.join(packageRoot, "dist/index.mjs");
const maximumGzipBytes = 1024;
const gzipBytes = gzipSync(fs.readFileSync(entryPath), { level: 9 }).byteLength;

if (gzipBytes > maximumGzipBytes) {
  throw new Error(
    `@vizejs/marquette gzip size ${gzipBytes} bytes exceeds ${maximumGzipBytes} byte budget`,
  );
}

console.log(
  JSON.stringify({
    entry: "@vizejs/marquette",
    gzipBytes,
    maximumGzipBytes,
  }),
);
