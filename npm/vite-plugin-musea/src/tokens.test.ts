import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { parseTokens } from "./tokens/parser.ts";
import { buildTokenMap, resolveReferences } from "./tokens/resolver.ts";

void test("parseTokens merges token directories into canonical reference paths", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-tokens-"));

  await fs.promises.writeFile(
    path.join(tempDir, "colors.tokens.json"),
    JSON.stringify({
      color: {
        primitive: {
          gray: {
            50: { value: "#f7f7f7" },
          },
        },
      },
    }),
    "utf-8",
  );

  await fs.promises.writeFile(
    path.join(tempDir, "semantic.tokens.json"),
    JSON.stringify({
      color: {
        semantic: {
          surface: { value: "{color.primitive.gray.50}" },
        },
      },
    }),
    "utf-8",
  );

  const categories = await parseTokens(tempDir);
  const tokenMap = buildTokenMap(categories);
  resolveReferences(categories, tokenMap);
  const resolvedTokenMap = buildTokenMap(categories);

  assert.equal(resolvedTokenMap["color.semantic.surface"]?.$reference, "color.primitive.gray.50");
  assert.equal(resolvedTokenMap["color.semantic.surface"]?.$resolvedValue, "#f7f7f7");

  await fs.promises.rm(tempDir, { recursive: true, force: true });
});
