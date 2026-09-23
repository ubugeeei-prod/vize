import { Buffer } from "node:buffer";
import { globSync, statSync } from "node:fs";
import { resolve } from "node:path";

const typecheckerAuthoredGlobs = [
  "vue",
  "ts",
  "tsx",
  "mts",
  "cts",
  "js",
  "jsx",
  "mjs",
  "cjs",
].flatMap((extension) => [`**/*.${extension}`, `**/.*/**/*.${extension}`]);

export function collectVueInputPaths(cwd, patterns) {
  return collectInputPaths(cwd, patterns);
}

export function collectInputPaths(cwd, patterns) {
  return [
    ...new Set(
      patterns.flatMap((pattern) =>
        expandHiddenRecursiveGlobs(pattern).flatMap((expanded) =>
          globSync(expanded, { cwd, exclude: [".yarn/**", "**/node_modules/**"] }),
        ),
      ),
    ),
  ]
    .filter((entry) => statSync(resolve(cwd, entry)).isFile())
    .map((entry) => entry.replaceAll("\\", "/"))
    .map((file) => ({ file, bytes: Buffer.from(file) }))
    .sort((left, right) => Buffer.compare(left.bytes, right.bytes))
    .map(({ file }) => file);
}

function expandHiddenRecursiveGlobs(pattern) {
  const normalized = pattern.replaceAll("\\", "/");
  if (!normalized.includes("**/")) return [pattern];
  return [
    pattern,
    ...new Set(
      recursiveGlobIndexes(normalized).map((index) => {
        const prefix = normalized.slice(0, index);
        const suffix = normalized.slice(index + "**/".length);
        return `${prefix}**/.*/**/${suffix}`;
      }),
    ),
  ];
}

function recursiveGlobIndexes(pattern) {
  const indexes = [];
  let start = 0;
  while (true) {
    const index = pattern.indexOf("**/", start);
    if (index === -1) return indexes;
    indexes.push(index);
    start = index + 3;
  }
}

export function collectTypecheckerAuthoredPaths(cwd) {
  return collectVueInputPaths(cwd, typecheckerAuthoredGlobs);
}
