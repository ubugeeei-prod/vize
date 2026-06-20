/**
 * CLI utility functions for art file scanning and parsing.
 *
 * Extracted from cli.ts to keep file sizes manageable.
 */

import fs from "node:fs";
import path from "node:path";

import type { ArtFileInfo } from "../types/index.js";

/** Recursively scan a directory for .art.vue files. */
export async function scanArtFiles(root: string): Promise<string[]> {
  const files: string[] = [];

  async function scan(dir: string): Promise<void> {
    const entries = await fs.promises.readdir(dir, { withFileTypes: true });

    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);

      // Skip node_modules and dist
      if (entry.name === "node_modules" || entry.name === "dist") {
        continue;
      }

      if (entry.isDirectory()) {
        await scan(fullPath);
      } else if (entry.isFile() && entry.name.endsWith(".art.vue")) {
        files.push(fullPath);
      }
    }
  }

  await scan(root);
  return files;
}

/** Parse a single .art.vue file into an ArtFileInfo structure. */
export async function parseArtFile(filePath: string): Promise<ArtFileInfo | null> {
  try {
    const source = await fs.promises.readFile(filePath, "utf-8");

    // Simple parsing - in production, use @vizejs/native
    const titleMatch = source.match(/<art[^>]*\stitle=["']([^"']+)["']/);
    const componentMatch = source.match(/<art[^>]*\scomponent=["']([^"']+)["']/);
    const categoryMatch = source.match(/<art[^>]*\scategory=["']([^"']+)["']/);

    const variants: ArtFileInfo["variants"] = [];
    const variantRegex = /<variant\s+([^>]*)>([\s\S]*?)<\/variant>/g;
    let match;

    while ((match = variantRegex.exec(source)) !== null) {
      const attrs = match[1];
      const template = match[2].trim();

      const nameMatch = attrs.match(/name=["']([^"']+)["']/);
      const isDefault = /\bdefault\b/.test(attrs);
      const skipVrt = /\bskip-vrt\b/.test(attrs);

      if (nameMatch) {
        variants.push({
          name: nameMatch[1],
          template,
          isDefault,
          skipVrt,
        });
      }
    }

    return {
      path: filePath,
      metadata: {
        title: titleMatch?.[1] || path.basename(filePath, ".art.vue"),
        component: componentMatch?.[1],
        category: categoryMatch?.[1],
        tags: [],
        status: "ready",
      },
      variants,
      hasScriptSetup: /<script\s+setup/.test(source),
      hasScript: /<script(?!\s+setup)/.test(source),
      styleCount: (source.match(/<style/g) || []).length,
    };
  } catch (error) {
    console.error(`Failed to parse ${filePath}:`, error);
    return null;
  }
}
