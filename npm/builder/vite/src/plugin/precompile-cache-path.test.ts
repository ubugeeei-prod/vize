import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { openPrecompileCache } from "./precompile-cache.ts";

void test("persistent cache identity ignores Windows path separators", (context) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-precompile-path-"));
  context.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const file = path.join(root, "src", "App.vue");
  const forward = file.replaceAll("\\", "/");
  const backward = forward.replaceAll("/", "\\");
  const options = { root, compileOptions: { ssr: false }, env: {} };
  const module = { code: "export default {}", scopeId: "data-v-path", hasScoped: false };

  const first = openPrecompileCache(options);
  first.set(backward, "source-hash", module);
  assert.equal(first.flush(), true);

  const restored = openPrecompileCache(options);
  assert.deepEqual(
    restored.get(forward, "source-hash"),
    module,
    "a manifest written with backslashes must restore through a forward-slash glob path",
  );
});
