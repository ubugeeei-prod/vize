/**
 * Shared types for MCP tool handlers.
 */

import type { ServerContext } from "../../types.js";

export type ToolResult = { content: Array<{ type: "text"; text: string }> };

export type { ServerContext };
