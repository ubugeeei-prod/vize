/**
 * Shared utility functions for the Musea Vite plugin.
 */

import fs from "node:fs";
import path from "node:path";

import type { ArtFileInfo } from "./types/index.js";
import { loadNative } from "./native-loader.js";

function normalizeGlobPath(filepath: string): string {
  return filepath.split(path.sep).join("/");
}

function matchesPattern(file: string, pattern: string, root: string): boolean {
  const normalizedPattern = normalizeGlobPath(pattern);
  const candidate = path.isAbsolute(pattern) ? path.resolve(file) : path.relative(root, file);
  return matchGlob(candidate, normalizedPattern);
}

export function shouldProcess(
  file: string,
  include: string[],
  exclude: string[],
  root: string,
): boolean {
  // Check exclude patterns
  for (const pattern of exclude) {
    if (matchesPattern(file, pattern, root)) {
      return false;
    }
  }

  // Check include patterns
  for (const pattern of include) {
    if (matchesPattern(file, pattern, root)) {
      return true;
    }
  }

  return false;
}

export function matchGlob(filepath: string, pattern: string): boolean {
  const normalizedFilepath = normalizeGlobPath(filepath);
  const normalizedPattern = normalizeGlobPath(pattern);

  // Simple glob matching (supports * and **)
  // Use placeholder for ** to avoid * replacement interfering
  const PLACEHOLDER = "<<GLOBSTAR>>";
  const SEGMENT_PLACEHOLDER = "<<GLOBSTAR_SEGMENT>>";
  const regex = normalizedPattern
    .replaceAll("**/", SEGMENT_PLACEHOLDER)
    .replaceAll("**", PLACEHOLDER)
    .replace(/[|\\{}()[\]^$+?.]/g, "\\$&")
    .replace(/\*/g, "[^/]*")
    .replaceAll(SEGMENT_PLACEHOLDER, "(?:.*/)?")
    .replaceAll(PLACEHOLDER, ".*");

  return new RegExp(`^${regex}$`).test(normalizedFilepath);
}

function resolveScanRoot(root: string, pattern: string): string {
  const absolutePattern = path.isAbsolute(pattern) ? pattern : path.resolve(root, pattern);
  const normalizedPattern = normalizeGlobPath(absolutePattern);
  const globIndex = normalizedPattern.search(/[*[{]/);

  if (globIndex === -1) {
    return path.dirname(absolutePattern);
  }

  const staticPrefix = normalizedPattern.slice(0, globIndex);
  if (!staticPrefix) {
    return root;
  }

  if (staticPrefix.endsWith("/")) {
    return path.resolve(staticPrefix.slice(0, -1));
  }

  return path.resolve(path.dirname(staticPrefix));
}

export function resolveScanRoots(root: string, include: string[]): string[] {
  const roots = new Set<string>();

  for (const pattern of include) {
    roots.add(resolveScanRoot(root, pattern));
  }

  if (roots.size === 0) {
    roots.add(root);
  }

  return [...roots];
}

export async function scanArtFiles(
  root: string,
  include: string[],
  exclude: string[],
  scanInlineArt = false,
): Promise<string[]> {
  const files = new Set<string>();
  const scanRoots = resolveScanRoots(root, include);
  const visitedDirs = new Set<string>();

  async function scan(dir: string): Promise<void> {
    const resolvedDir = path.resolve(dir);
    if (visitedDirs.has(resolvedDir)) {
      return;
    }
    visitedDirs.add(resolvedDir);

    let entries: fs.Dirent[];
    try {
      entries = await fs.promises.readdir(resolvedDir, { withFileTypes: true });
    } catch {
      return;
    }

    for (const entry of entries) {
      const fullPath = path.join(resolvedDir, entry.name);

      // Check exclude
      let excluded = false;
      for (const pattern of exclude) {
        if (matchesPattern(fullPath, pattern, root) || matchGlob(entry.name, pattern)) {
          excluded = true;
          break;
        }
      }

      if (excluded) continue;

      if (entry.isDirectory()) {
        await scan(fullPath);
      } else if (entry.isFile() && entry.name.endsWith(".art.vue")) {
        if (shouldProcess(fullPath, include, exclude, root)) {
          files.add(fullPath);
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
          files.add(fullPath);
        }
      }
    }
  }

  for (const scanRoot of scanRoots) {
    await scan(scanRoot);
  }

  return [...files];
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
