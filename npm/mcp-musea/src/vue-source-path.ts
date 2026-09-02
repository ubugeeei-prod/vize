/**
 * Confine MCP component reads to real `.vue` files.
 *
 * Request paths that merely *end with* `.vue` are not enough: a planted
 * `Evil.vue` → `.env` symlink would otherwise be read by generate/analyze
 * and by art-declared component sources.
 */

import fs from "node:fs";
import path from "node:path";
import { ErrorCode, McpError } from "@modelcontextprotocol/sdk/types.js";

function realpathNearest(targetPath: string): string {
  let current = path.resolve(targetPath);
  const missingParts: string[] = [];

  while (true) {
    try {
      const real = fs.realpathSync.native(current);
      return missingParts.length > 0 ? path.join(real, ...missingParts.reverse()) : real;
    } catch {
      const parent = path.dirname(current);
      if (parent === current) {
        return path.resolve(targetPath);
      }
      missingParts.push(path.basename(current));
      current = parent;
    }
  }
}

export function isVueSourcePath(candidatePath: string): boolean {
  return realpathNearest(candidatePath).toLowerCase().endsWith(".vue");
}

export function assertVueSourcePath(candidatePath: string, label = "path"): void {
  if (!isVueSourcePath(candidatePath)) {
    throw new McpError(ErrorCode.InvalidParams, `${label} must be a .vue file`);
  }
}
