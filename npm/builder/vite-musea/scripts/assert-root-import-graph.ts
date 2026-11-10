import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const entry = path.join(packageRoot, "dist/index.mjs");
const pending = [entry];
const visited = new Set<string>();
const bareImports = new Set<string>();

const staticImports = [
  /(?:^|\n)\s*import\s+(?!\()(?:(?:[^"'()]*?)\s+from\s+)?["']([^"']+)["']/g,
  /(?:^|\n)\s*export\s+[^"'()]*?\s+from\s+["']([^"']+)["']/g,
];

while (pending.length > 0) {
  const file = pending.pop();
  if (!file || visited.has(file)) continue;
  visited.add(file);
  const source = await readFile(file, "utf8");
  for (const pattern of staticImports) {
    for (const match of source.matchAll(pattern)) {
      const specifier = match[1];
      if (!specifier) continue;
      if (!specifier.startsWith(".")) {
        if (!specifier.startsWith("node:")) bareImports.add(specifier);
        continue;
      }
      pending.push(path.resolve(path.dirname(file), specifier));
    }
  }
}

for (const dependency of ["typescript", "vite", "pngjs"]) {
  assert(
    ![...bareImports].some(
      (specifier) => specifier === dependency || specifier.startsWith(`${dependency}/`),
    ),
    `${dependency} must not be statically reachable from dist/index.mjs`,
  );
}

for (const dedicatedEntry of ["dist/vrt.mjs", "dist/a11y/index.mjs"]) {
  assert(
    !visited.has(path.join(packageRoot, dedicatedEntry)),
    `${dedicatedEntry} must remain behind its dedicated subpath or a dynamic import`,
  );
}

console.log(
  `[musea] root import graph: ${visited.size} local module(s), no TypeScript, Vite, PNG, VRT runner, or a11y runner`,
);
