import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { parseCssTokenSource } from "./css-tokens.ts";
import { categoriesFromNativeResult } from "./tokens.ts";

test("categoriesFromNativeResult parses the JSON string returned by the native binding", () => {
  const categories = categoriesFromNativeResult(
    '[{"name":"color","tokens":{"brand":{"value":"#ff0000","type":"color"}}}]',
  );

  assert.equal(categories[0]?.name, "color");
  assert.equal(categories[0]?.tokens.brand?.value, "#ff0000");
  assert.equal(Object.getPrototypeOf(categories[0]?.tokens), null);
});

test("parseCssTokenSource keeps custom properties outside Tailwind namespaces", () => {
  const categories = parseCssTokenSource(`:root {
    --color-primary: #1976d2;
    --brand-color-primary: #1976d2;
    --brand-color-accent: var(--brand-color-primary);
    --space-2: 8px;
    --elevation-overlay: 10;
  }`);
  const brand = categories.find((category) => category.name === "brand");
  const space = categories.find((category) => category.name === "space");
  const elevation = categories.find((category) => category.name === "elevation");
  const color = categories.find((category) => category.name === "color");

  assert.equal(color?.tokens.primary?.value, "#1976d2");
  assert.equal(brand?.tokens["color-primary"]?.value, "#1976d2");
  assert.equal(brand?.tokens["color-accent"]?.$reference, "brand.color-primary");
  assert.equal(space?.tokens["2"]?.value, "8px");
  assert.equal(space?.tokens["2"]?.type, "dimension");
  assert.equal(elevation?.tokens.overlay?.value, "10");
  assert.equal(elevation?.tokens.overlay?.type, "number");
});

test("parseCssTokenFiles reads a css tokensPath", async () => {
  const { parseTokensFromPath } = await import("./tokens.ts");
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "musea-mcp-tokens-"));
  try {
    const cssPath = path.join(dir, "tokens.css");
    fs.writeFileSync(cssPath, ":root { --brand-color-primary: #ff0000; }\n");
    const categories = await parseTokensFromPath(cssPath);
    const brand = categories.find((category) => category.name === "brand");
    assert.equal(brand?.tokens["color-primary"]?.value, "#ff0000");
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});
