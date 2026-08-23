/**
 * CLI utility functions for art file scanning and parsing.
 *
 * Extracted from cli.ts to keep file sizes manageable.
 */

import fs from "node:fs";
import path from "node:path";

import { processMuseaArtFile } from "../plugin/art-processing.js";
import type { ArtFileInfo } from "../types/index.js";

/** Recursively scan a directory for .art.vue files. */
export async function scanArtFiles(root: string): Promise<string[]> {
  const files: string[] = [];

  async function scan(dir: string): Promise<void> {
    let entries: fs.Dirent[];
    try {
      entries = await fs.promises.readdir(dir, { withFileTypes: true });
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "EACCES") {
        return;
      }
      throw error;
    }

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
  return processMuseaArtFile(filePath, {
    root: path.dirname(filePath),
    command: "serve",
  });
}
