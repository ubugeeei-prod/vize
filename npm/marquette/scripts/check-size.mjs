import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const budgets = [
  { entry: "@vizejs/marquette", file: "dist/index.mjs", maximumGzipBytes: 1024 },
  {
    entry: "@vizejs/marquette/validate",
    file: "dist/validate.mjs",
    maximumGzipBytes: 3072,
  },
];

for (const { entry, file, maximumGzipBytes } of budgets) {
  const gzipBytes = gzipSync(fs.readFileSync(path.join(packageRoot, file)), {
    level: 9,
  }).byteLength;

  if (gzipBytes > maximumGzipBytes) {
    throw new Error(
      `${entry} gzip size ${gzipBytes} bytes exceeds ${maximumGzipBytes} byte budget`,
    );
  }

  console.log(JSON.stringify({ entry, gzipBytes, maximumGzipBytes }));
}
