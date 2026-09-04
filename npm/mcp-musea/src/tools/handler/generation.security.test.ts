import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

test("generate_variants only reads Vue component sources", () => {
  const source = fs.readFileSync(
    path.resolve(path.dirname(fileURLToPath(import.meta.url)), "generation.ts"),
    "utf8",
  );
  const start = source.indexOf("export async function handleGenerateVariants");
  const end = source.indexOf("export async function handleGenerateCsf");
  const block = source.slice(start, end);

  assert.notEqual(start, -1, "generate_variants handler must exist");
  assert.notEqual(end, -1, "generate_csf handler must follow generate_variants");
  assert.match(
    block,
    /assertVueSourcePath\(absolutePath, "componentPath"\)/,
    "generate_variants must reject non-vue paths and .vue symlinks before reading",
  );

  const helper = fs.readFileSync(
    path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../vue-source-path.ts"),
    "utf8",
  );
  assert.match(helper, /must be a \.vue file/, "the rejection must name the .vue requirement");
});
