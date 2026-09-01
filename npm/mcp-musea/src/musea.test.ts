import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { analyzeResolvedComponent, buildDocumentation, buildPalette } from "./musea.ts";
import type { NativeBinding, ServerContext } from "./types.ts";

test("buildPalette prints fallback TypeScript through the AST printer", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-mcp-palette-"));
  try {
    const artPath = path.join(root, "FancyButton.art.vue");
    const componentPath = path.join(root, "FancyButton.vue");
    fs.writeFileSync(componentPath, "<template><button /></template>\n");

    const ctx: ServerContext = {
      projectRoot: root,
      loadNative() {
        throw new Error("native binding should be provided by the test");
      },
      scanArtFiles: async () => new Map(),
      resolveTokensPath: async () => null,
    };
    const binding = {
      parseArt() {
        throw new Error("parseArt should not be called");
      },
      artToCsf() {
        throw new Error("artToCsf should not be called");
      },
      parseDesignTokensFromPath() {
        return [];
      },
      flattenDesignTokenCategories() {
        return [];
      },
      generateDesignTokensMarkdown() {
        return "";
      },
      analyzeSfc() {
        return {
          props: [
            {
              name: "tone",
              type: '"brand" | "neutral"',
              required: false,
            },
            {
              name: "disabled",
              type: "boolean",
              required: true,
            },
            {
              name: "model-value",
              type: "string",
              required: false,
            },
          ],
          emits: [],
        };
      },
    } satisfies NativeBinding;

    const palette = await buildPalette(
      ctx,
      binding,
      {
        info: {
          path: artPath,
          title: "Fancy Button",
          component: "./FancyButton.vue",
          tags: [],
          status: "ready",
          variantCount: 0,
          variantNames: [],
        },
        absolutePath: artPath,
        relativePath: "FancyButton.art.vue",
        matchedBy: "path",
        matchValue: "FancyButton.art.vue",
        score: 1,
        reasons: [],
        alternatives: [],
      },
      "",
    );

    assert.equal(
      palette?.typescript,
      [
        "export interface FancyButtonProps {",
        '    tone?: "brand" | "neutral";',
        "    disabled: boolean;",
        '    "model-value"?: string;',
        "}",
        "",
      ].join("\n"),
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("buildDocumentation rewrites Self tags inside template fences", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-mcp-docs-"));
  try {
    const artPath = path.join(root, "FancyButton.art.vue");
    const binding = {
      parseArt() {
        throw new Error("parseArt should not be called");
      },
      artToCsf() {
        throw new Error("artToCsf should not be called");
      },
      generateArtDoc() {
        return {
          markdown: [
            "Inline <Self /> should stay prose.",
            "",
            "```vue",
            '    <Self tone="brand">',
            "      <Self />",
            "      <Selfish />",
            "    </Self>",
            "```",
            "",
            "```ts",
            "const tag = '<Self />';",
            "```",
            "",
          ].join("\n"),
          filename: artPath,
          title: "FancyButton",
          variant_count: 1,
        };
      },
      parseDesignTokensFromPath() {
        return [];
      },
      flattenDesignTokenCategories() {
        return [];
      },
      generateDesignTokensMarkdown() {
        return "";
      },
    } satisfies NativeBinding;

    const documentation = await buildDocumentation(
      binding,
      {
        info: {
          path: artPath,
          title: "FancyButton",
          tags: [],
          status: "ready",
          variantCount: 0,
          variantNames: [],
        },
        absolutePath: artPath,
        relativePath: "FancyButton.art.vue",
        matchedBy: "path",
        matchValue: "FancyButton.art.vue",
        score: 1,
        reasons: [],
        alternatives: [],
      },
      "",
      { includeTemplates: true },
    );

    assert.ok(documentation);
    assert.match(documentation.markdown, /Inline <Self \/> should stay prose\./);
    assert.match(documentation.markdown, /<FancyButton tone="brand">/);
    assert.match(documentation.markdown, /<FancyButton \/>/);
    assert.match(documentation.markdown, /<\/FancyButton>/);
    assert.match(documentation.markdown, /<Selfish \/>/);
    assert.match(documentation.markdown, /const tag = '<Self \/>';/);
    assert.doesNotMatch(documentation.markdown, /<Self tone=/);
    assert.doesNotMatch(documentation.markdown, /<\/Self>\n```/);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("analyzeResolvedComponent omits out-of-project component source paths", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-mcp-component-"));
  const outside = path.join(path.dirname(root), `${path.basename(root)}-secret`);

  try {
    fs.mkdirSync(outside);
    const secretPath = path.join(outside, "Secret.vue");
    fs.writeFileSync(secretPath, "<template><button /></template>\n");
    const artPath = path.join(root, "Card.art.vue");
    fs.writeFileSync(artPath, "");

    const ctx: ServerContext = {
      projectRoot: root,
      loadNative() {
        throw new Error("native binding should not load");
      },
      scanArtFiles: async () => new Map(),
      resolveTokensPath: async () => null,
    };

    const result = await analyzeResolvedComponent(
      ctx,
      {
        parseArt() {
          throw new Error("parseArt should not be called");
        },
        artToCsf() {
          throw new Error("artToCsf should not be called");
        },
        parseDesignTokensFromPath() {
          return [];
        },
        flattenDesignTokenCategories() {
          return [];
        },
        generateDesignTokensMarkdown() {
          return "";
        },
      } satisfies NativeBinding,
      {
        info: {
          path: artPath,
          title: "Card",
          component: "../" + path.basename(outside) + "/Secret.vue",
          tags: [],
          status: "ready",
          variantCount: 0,
          variantNames: [],
        },
        absolutePath: artPath,
        relativePath: "Card.art.vue",
        matchedBy: "path",
        matchValue: "Card.art.vue",
        score: 1,
        reasons: [],
        alternatives: [],
      },
    );

    assert.equal(result.source.exists, false);
    assert.equal(result.source.absolutePath, undefined);
    assert.equal(result.source.path, undefined);
    assert.match(result.source.error ?? "", /outside the project root/);
    assert.doesNotMatch(
      JSON.stringify(result),
      new RegExp(outside.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
    fs.rmSync(outside, { recursive: true, force: true });
  }
});

test("analyzeResolvedComponent rejects an in-project .vue symlink to a non-vue file", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-mcp-component-link-"));

  try {
    const envPath = path.join(root, ".env");
    fs.writeFileSync(envPath, "SECRET=1\n");
    fs.symlinkSync(envPath, path.join(root, "Evil.vue"));
    const artPath = path.join(root, "Card.art.vue");
    fs.writeFileSync(artPath, "");

    const ctx: ServerContext = {
      projectRoot: root,
      loadNative() {
        throw new Error("native binding should not load");
      },
      scanArtFiles: async () => new Map(),
      resolveTokensPath: async () => null,
    };

    const result = await analyzeResolvedComponent(
      ctx,
      {
        parseArt() {
          throw new Error("parseArt should not be called");
        },
        artToCsf() {
          throw new Error("artToCsf should not be called");
        },
        parseDesignTokensFromPath() {
          return [];
        },
        flattenDesignTokenCategories() {
          return [];
        },
        generateDesignTokensMarkdown() {
          return "";
        },
      } satisfies NativeBinding,
      {
        info: {
          path: artPath,
          title: "Card",
          component: "./Evil.vue",
          tags: [],
          status: "ready",
          variantCount: 0,
          variantNames: [],
        },
        absolutePath: artPath,
        relativePath: "Card.art.vue",
        matchedBy: "path",
        matchValue: "Card.art.vue",
        score: 1,
        reasons: [],
        alternatives: [],
      },
    );

    assert.equal(result.source.exists, false);
    assert.equal(result.source.absolutePath, undefined);
    assert.equal(result.source.path, undefined);
    assert.match(result.source.error ?? "", /must be a \.vue file/);
    assert.doesNotMatch(JSON.stringify(result), /SECRET=1/);
    assert.doesNotMatch(JSON.stringify(result), /\.env/);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
