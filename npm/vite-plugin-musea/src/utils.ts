/**
 * Shared utility functions for the Musea Vite plugin.
 */

import fs from "node:fs";
import path from "node:path";

import type { ArtFileInfo } from "./types/index.js";
import { loadNative } from "./native-loader.js";

export function shouldProcess(
  file: string,
  include: string[],
  exclude: string[],
  root: string,
): boolean {
  const relative = path.relative(root, file);

  // Check exclude patterns
  for (const pattern of exclude) {
    if (matchGlob(relative, pattern)) {
      return false;
    }
  }

  // Check include patterns
  for (const pattern of include) {
    if (matchGlob(relative, pattern)) {
      return true;
    }
  }

  return false;
}

export function matchGlob(filepath: string, pattern: string): boolean {
  // Simple glob matching (supports * and **)
  // Use placeholder for ** to avoid * replacement interfering
  const PLACEHOLDER = "<<GLOBSTAR>>";
  const regex = pattern
    .replaceAll("**", PLACEHOLDER)
    .replace(/\./g, "\\.")
    .replace(/\*/g, "[^/]*")
    .replaceAll(PLACEHOLDER, ".*");

  return new RegExp(`^${regex}$`).test(filepath);
}

export async function scanArtFiles(
  root: string,
  include: string[],
  exclude: string[],
  scanInlineArt = false,
): Promise<string[]> {
  const files: string[] = [];

  async function scan(dir: string): Promise<void> {
    const entries = await fs.promises.readdir(dir, { withFileTypes: true });

    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      const relative = path.relative(root, fullPath);

      // Check exclude
      let excluded = false;
      for (const pattern of exclude) {
        if (matchGlob(relative, pattern) || matchGlob(entry.name, pattern)) {
          excluded = true;
          break;
        }
      }

      if (excluded) continue;

      if (entry.isDirectory()) {
        await scan(fullPath);
      } else if (entry.isFile() && entry.name.endsWith(".art.vue")) {
        // Check include
        for (const pattern of include) {
          if (matchGlob(relative, pattern)) {
            files.push(fullPath);
            break;
          }
        }
      } else if (
        scanInlineArt &&
        entry.isFile() &&
        entry.name.endsWith(".vue") &&
        !entry.name.endsWith(".art.vue")
      ) {
        // Inline art: check if .vue file contains <art block
        const content = await fs.promises.readFile(fullPath, "utf-8");
        if (content.includes("<art")) {
          files.push(fullPath);
        }
      }
    }
  }

  await scan(root);
  return files;
}

export async function generateStorybookFiles(
  artFiles: Map<string, ArtFileInfo>,
  root: string,
  outDir: string,
): Promise<void> {
  const binding = loadNative();
  const outputDir = path.resolve(root, outDir);

  // Ensure output directory exists
  await fs.promises.mkdir(outputDir, { recursive: true });

  for (const [filePath, _art] of artFiles) {
    try {
      const source = await fs.promises.readFile(filePath, "utf-8");
      const csf = binding.artToCsf(source, { filename: filePath });

      const outputPath = path.join(outputDir, csf.filename);
      await fs.promises.writeFile(outputPath, csf.code, "utf-8");

      console.log(`[musea] Generated: ${path.relative(root, outputPath)}`);
    } catch (e) {
      console.error(`[musea] Failed to generate CSF for ${filePath}:`, e);
    }
  }
}

export function toPascalCase(str: string): string {
  const normalized = str
    .normalize("NFKD")
    .replace(/[^\p{L}\p{N}]+/gu, " ")
    .trim();
  const pascal = normalized
    .split(/\s+/)
    .filter(Boolean)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join("");

  if (!pascal) {
    return "Variant";
  }

  return /^[\p{L}_$]/u.test(pascal) ? pascal : `Variant${pascal}`;
}

export function escapeTemplate(str: string): string {
  return str.replace(/\\/g, "\\\\").replace(/'/g, "\\'").replace(/\n/g, "\\n");
}

export function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#x27;");
}

/**
 * Build the theme config object from plugin options for runtime injection.
 */
export function buildThemeConfig(
  theme?:
    | string
    | { name: string; base?: "dark" | "light"; colors: Record<string, string> }
    | Array<{ name: string; base?: "dark" | "light"; colors: Record<string, string> }>,
):
  | {
      default: string;
      custom?: Record<string, { base?: "dark" | "light"; colors: Record<string, string> }>;
    }
  | undefined {
  if (!theme) return undefined;

  if (typeof theme === "string") {
    // 'dark' | 'light' | 'system'
    return { default: theme };
  }

  // Single custom theme or array of custom themes
  const themes = Array.isArray(theme) ? theme : [theme];
  const custom: Record<string, { base?: "dark" | "light"; colors: Record<string, string> }> = {};
  for (const t of themes) {
    custom[t.name] = {
      base: t.base,
      colors: t.colors as Record<string, string>,
    };
  }
  return {
    default: themes[0].name,
    custom,
  };
}
