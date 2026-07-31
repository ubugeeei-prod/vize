// `@vizejs/native`'s TypeScript surface must come from exactly one file: the
// `index.d.ts` that `napi build` generates from the `#[napi]` items in
// `crates/vize_vitrine`. A hand-written second copy cannot be kept in sync --
// `npm/native/types/` drifted until its `SfcCompileOptionsNapi` was missing four
// fields and its `BatchCompileOptionsNapi` eight, while `package.json` shipped
// the directory and no test compared it to the generated file (#3494).
//
// A shipped declaration that is silently a subset of the real API is worse than
// no declaration: valid option objects look like type errors. These tests pin
// that there is one declaration file and that the package ships only it.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const nativeDir = path.join(root, "npm/native");

/** Every `.d.ts` under `npm/native`, repo-relative and sorted. Skips installed dependencies. */
function declarationFiles(dir: string, prefix: string): string[] {
  const found: string[] = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === "node_modules" || entry.name.startsWith(".")) continue;
    const relative = prefix === "" ? entry.name : `${prefix}/${entry.name}`;
    if (entry.isDirectory()) {
      found.push(...declarationFiles(path.join(dir, entry.name), relative));
    } else if (entry.name.endsWith(".d.ts")) {
      found.push(relative);
    }
  }
  return found.sort();
}

test("npm/native declares its types in exactly one generated file", () => {
  assert.deepEqual(declarationFiles(nativeDir, ""), ["index.d.ts"]);
});

test("npm/native ships exactly the generated declaration and the loader", () => {
  const manifest = JSON.parse(fs.readFileSync(path.join(nativeDir, "package.json"), "utf8")) as {
    files: string[];
    types: string;
  };

  assert.deepEqual(manifest.files, [
    "index.js",
    "index.d.ts",
    "native-binding.js",
    "native-targets.js",
  ]);
  assert.equal(manifest.types, "index.d.ts");
});
