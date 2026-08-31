/**
 * Resolve a Vue source file that stays inside the project after realpath.
 */

import fs from "node:fs";
import { ErrorCode, McpError } from "@modelcontextprotocol/sdk/types.js";
import { resolveProjectPath } from "./musea.js";

/**
 * Require a `.vue` suffix on both the request path and the realpath target.
 * A planted `Evil.vue` → `.env` symlink must not be readable.
 */
export function resolveProjectVueFile(
  projectRoot: string,
  inputPath: string,
  label = "path",
): string {
  if (!inputPath.endsWith(".vue")) {
    throw new McpError(ErrorCode.InvalidParams, `${label} must be a .vue file`);
  }

  const resolved = resolveProjectPath(projectRoot, inputPath, label);
  let real: string;
  try {
    real = fs.realpathSync.native(resolved);
  } catch {
    throw new McpError(ErrorCode.InvalidParams, `${label} must be a readable .vue file`);
  }
  if (!real.endsWith(".vue")) {
    throw new McpError(ErrorCode.InvalidParams, `${label} must be a .vue file`);
  }

  return resolved;
}
