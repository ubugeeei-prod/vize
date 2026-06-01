/**
 * MCP tool call handler for Musea.
 *
 * Routes incoming tool calls to the appropriate handler logic based on
 * the tool name, using the native Rust binding and server context.
 */

import { ErrorCode, McpError } from "@modelcontextprotocol/sdk/types.js";
import type { ServerContext, ToolResult } from "./types.js";

import { handleAnalyzeComponent, handleGetPalette } from "./analysis.js";
import {
  handleListComponents,
  handleGetComponent,
  handleGetVariant,
  handleSearchComponents,
  handleRecommendComponents,
} from "./registry.js";
import {
  handleGenerateVariants,
  handleGenerateCsf,
  handleGenerateDocs,
  handleGenerateCatalog,
  handleGetTokens,
  handleSearchTokens,
} from "./generation.js";

export type { ToolResult };

export async function handleToolCall(
  ctx: ServerContext,
  name: string,
  args: Record<string, unknown> | undefined,
): Promise<ToolResult> {
  let binding: ReturnType<ServerContext["loadNative"]> | undefined;
  const loadBinding = () => (binding ??= ctx.loadNative());

  switch (name) {
    // --- Component analysis -------------------------------------------------
    case "analyze_component":
      return handleAnalyzeComponent(ctx, loadBinding(), args);
    case "get_palette":
      return handleGetPalette(ctx, loadBinding(), args);

    // --- Component registry -------------------------------------------------
    case "list_components":
      return handleListComponents(ctx, args);
    case "get_component":
      return handleGetComponent(ctx, loadBinding(), args);
    case "get_variant":
      return handleGetVariant(ctx, loadBinding(), args);
    case "search_components":
      return handleSearchComponents(ctx, args);
    case "recommend_components":
      return handleRecommendComponents(ctx, args);

    // --- Code generation / docs / tokens ------------------------------------
    case "generate_variants":
      return handleGenerateVariants(ctx, loadBinding(), args);
    case "generate_csf":
      return handleGenerateCsf(ctx, loadBinding(), args);
    case "generate_docs":
      return handleGenerateDocs(ctx, loadBinding(), args);
    case "generate_catalog":
      return handleGenerateCatalog(ctx, loadBinding(), args);
    case "get_tokens":
      return handleGetTokens(ctx, args);
    case "search_tokens":
      return handleSearchTokens(ctx, args);

    default:
      throw new McpError(ErrorCode.MethodNotFound, `Unknown tool: ${name}`);
  }
}
