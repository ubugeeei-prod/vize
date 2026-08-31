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
  return [
    ...new Set(
      patterns.flatMap((pattern) =>
        globSync(pattern, { cwd, exclude: [".yarn/**", "**/node_modules/**"] })
          .filter((entry) => statSync(resolve(cwd, entry)).isFile())
          .map((entry) => entry.replaceAll("\\", "/")),
      ),
    ),
  ]
    .map((file) => ({ file, bytes: Buffer.from(file) }))
    .sort((left, right) => Buffer.compare(left.bytes, right.bytes))
    .map(({ file }) => file);
}

export function collectTypecheckerAuthoredPaths(cwd) {
  return collectVueInputPaths(cwd, typecheckerAuthoredGlobs);
}
