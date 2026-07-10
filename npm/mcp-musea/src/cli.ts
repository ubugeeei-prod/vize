#!/usr/bin/env node

/**
 * Musea MCP Server CLI.
 *
 * Usage:
 *   musea-mcp [project-root] [--tokens-path <path>]
 *
 * Environment:
 *   MUSEA_PROJECT_ROOT - Project root directory (default: cwd)
 *   MUSEA_TOKENS_PATH  - Path to design tokens file/directory
 */

import { startServer } from "./index.js";

// Parse CLI arguments
let projectRoot = process.env.MUSEA_PROJECT_ROOT || process.cwd();
let tokensPath = process.env.MUSEA_TOKENS_PATH;

const args = process.argv.slice(2);
for (let i = 0; i < args.length; i++) {
  if (args[i] === "--tokens-path" && i + 1 < args.length) {
    tokensPath = args[++i];
  } else if (!args[i].startsWith("--")) {
    projectRoot = args[i];
  }
}

console.error(`[musea-mcp] Starting server for project: ${projectRoot}`);
if (tokensPath) {
  console.error(`[musea-mcp] Tokens path: ${tokensPath}`);
}

startServer(projectRoot, { tokensPath }).catch((error) => {
  console.error("[musea-mcp] Failed to start:", error);
  process.exit(1);
});
