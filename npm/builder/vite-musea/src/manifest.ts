/**
 * Manifest module generation for Musea.
 *
 * Generates the virtual module that exposes all discovered art files
 * as a JSON manifest.
 */

import type { ArtFileInfo } from "./types/index.js";
import { sortedArts } from "./art-order.js";

/**
 * Generate the virtual manifest module code containing all art file metadata.
 */
export function generateManifestModule(artFiles: Map<string, ArtFileInfo>): string {
  const arts = sortedArts(artFiles.values());
  return `export const arts = ${JSON.stringify(arts, null, 2)};`;
}
