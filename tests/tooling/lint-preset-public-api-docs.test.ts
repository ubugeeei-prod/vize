import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  inversePatinaPublicApiDocs,
  publicApiDocSourcePaths,
} from "./support/lint-preset-latest-default-expansion.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("public API docs have one exact finite inverse", () => {
  for (const name of publicApiDocSourcePaths) {
    const live = fs.readFileSync(path.join(root, name));
    inversePatinaPublicApiDocs(name, live);
    for (const changed of [
      live.subarray(0, live.length - 1),
      Buffer.concat([live, Buffer.from("\n")]),
      Buffer.concat([Buffer.from("// unrelated\n"), live]),
    ])
      assert.throws(() => inversePatinaPublicApiDocs(name, changed));
  }
});
