import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { assertBundledHelpers } from "./prepare-consumer.mjs";

void test("public bundle inspection visits nested loader entries and rejects private imports", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "vize-turbopack-packaging-"));
  try {
    await fs.mkdir(path.join(root, "loader"));
    await fs.writeFile(
      path.join(root, "index.mjs"),
      'export { default } from "./loader/index.mjs";',
    );
    const entry = path.join(root, "loader/index.mjs");
    await fs.writeFile(entry, "export default function loader() {};");
    await assertBundledHelpers(root);
    for (const statement of [
      'import { MappedModule } from "@vizejs/builder-shared/mapped-module";',
      'const helper = require("@vizejs/builder-shared/mapped-module");',
      'const helper = import("@vizejs/builder-shared/mapped-module");',
    ]) {
      await fs.writeFile(entry, statement);
      await assert.rejects(assertBundledHelpers(root));
    }
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});
