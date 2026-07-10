/**
 * MCP tools module for Musea.
 *
 * Re-exports tool definitions and the handler from submodules:
 * - definitions: Tool schema declarations (name, description, input parameters)
 * - handler: Tool call routing and execution logic
 */

export { toolDefinitions } from "./definitions.js";
export { handleToolCall } from "./handler/index.js";
