import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import { generateTokensHtml } from "./tokens/generator.ts";
import { parseTokens } from "./tokens/parser.ts";
import {
  buildTokenMap,
  deleteTokenAtPath,
  resolveReferences,
  setTokenAtPath,
} from "./tokens/resolver.ts";

async function makeAgentTempDir() {
  const root = path.resolve("target", "vize-tests");
  await fs.promises.mkdir(root, { recursive: true });
  return fs.promises.mkdtemp(path.join(root, "musea-tokens-"));
}

void test("parseTokens merges token directories into canonical reference paths", async () => {
  const tempDir = await makeAgentTempDir();

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

void test("token mutations reject prototype-polluting paths", () => {
  const data: Record<string, unknown> = {};

  assert.throws(() => setTokenAtPath(data, "__proto__.polluted", { value: "yes" }), /not allowed/);
  assert.throws(() => deleteTokenAtPath(data, "constructor.prototype"), /not allowed/);
  assert.equal(({} as Record<string, unknown>).polluted, undefined);
});

void test("token parsing keeps prototype-like token names as data", async () => {
  const tempDir = await makeAgentTempDir();

  try {
    await fs.promises.writeFile(
      path.join(tempDir, "colors.tokens.json"),
      '{"color":{"__proto__":{"value":"#fff","type":"color"}}}',
      "utf-8",
    );

    const categories = await parseTokens(tempDir);
    const tokenMap = buildTokenMap(categories);
    const colorCategory = categories[0];

    assert.ok(colorCategory);
    assert.equal(Object.getPrototypeOf(colorCategory.tokens), null);
    assert.equal(Object.getPrototypeOf(tokenMap), null);
    assert.equal(tokenMap["color.__proto__"]?.value, "#fff");
    assert.equal(({} as Record<string, unknown>).value, undefined);
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("generateTokensHtml escapes untrusted token text and filters unsafe color styles", () => {
  const html = generateTokensHtml([
    {
      name: `Colors <img src=x onerror=alert(1)>`,
      tokens: {
        [`bad"><script>alert(1)</script>`]: {
          value: `url(javascript:alert(1)); color:red`,
          type: "color",
          description: `<b onclick=alert(1)>owned</b>`,
        },
      },
    },
  ]);

  assert.match(html, /Colors &lt;img src=x onerror=alert\(1\)&gt;/);
  assert.match(html, /bad&quot;&gt;&lt;script&gt;alert\(1\)&lt;\/script&gt;/);
  assert.match(html, /&lt;b onclick=alert\(1\)&gt;owned&lt;\/b&gt;/);
  assert.doesNotMatch(html, /<script>|<b onclick=|javascript:/);
  assert.doesNotMatch(html, /class="color-swatch"|style="background:/);
});
