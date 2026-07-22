import { readFile } from "node:fs/promises";
import { gzipSync } from "node:zlib";

const budgets = new Map([
  ["index.mjs", 4 * 1024],
  ["temporal.mjs", 2 * 1024],
]);

for (const [file, maximumGzipBytes] of budgets) {
  const gzipBytes = gzipSync(
    await readFile(new URL(`../dist/${file}`, import.meta.url)),
  ).byteLength;

  console.log(
    JSON.stringify({
      entry: `@vizejs/composable/${file.replace(/\.mjs$/, "")}`,
      gzipBytes,
      maximumGzipBytes,
    }),
  );
  if (gzipBytes > maximumGzipBytes) process.exitCode = 1;
}
