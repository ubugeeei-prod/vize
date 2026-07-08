import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { buildPalette } from "./musea.ts";
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
