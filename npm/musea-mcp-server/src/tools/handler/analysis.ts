/**
 * MCP tool handlers for component analysis.
 *
 * Handles `analyze_component` and `get_palette` tool calls.
 */

import fs from "node:fs";
import path from "node:path";
import { ErrorCode, McpError } from "@modelcontextprotocol/sdk/types.js";
import { analyzeResolvedComponent, buildPalette, resolveArtReference } from "../../musea.js";
import type { ServerContext, ToolResult } from "./types.js";

export async function handleAnalyzeComponent(
  ctx: ServerContext,
  binding: ReturnType<ServerContext["loadNative"]>,
  args: Record<string, unknown> | undefined,
): Promise<ToolResult> {
  const directPath = args?.path as string | undefined;
  if (directPath?.endsWith(".vue") && !directPath.endsWith(".art.vue")) {
    if (!binding.analyzeSfc) {
      throw new McpError(ErrorCode.InternalError, "analyzeSfc not available in native binding");
    }

    const absolutePath = path.resolve(ctx.projectRoot, directPath);
    const source = await fs.promises.readFile(absolutePath, "utf-8");
    const analysis = binding.analyzeSfc(source, { filename: absolutePath });

    return {
      content: [
        {
          type: "text",
          text: JSON.stringify(
            {
              path: directPath,
              props: analysis.props.map((prop) => ({
                name: prop.name,
                type: prop.type,
                required: prop.required,
                defaultValue: prop.default_value,
              })),
              emits: analysis.emits,
            },
            null,
            2,
          ),
        },
      ],
    };
  }

  const resolved = await resolveArtReference(ctx, args);
  const { source, analysis } = await analyzeResolvedComponent(ctx, binding, resolved);

  if (!analysis) {
    throw new McpError(
      ErrorCode.InvalidParams,
      source.error ?? `Could not analyze component source for "${resolved.info.title}"`,
    );
  }

  return {
    content: [
      {
        type: "text",
        text: JSON.stringify(
          {
            component: {
              title: resolved.info.title,
              artPath: resolved.relativePath,
              componentReference: resolved.info.component,
              componentPath: source.path,
            },
            match: {
              matchedBy: resolved.matchedBy,
              matchValue: resolved.matchValue,
              score: resolved.score,
              reasons: resolved.reasons,
              alternatives: resolved.alternatives,
            },
            props: analysis.props,
            emits: analysis.emits,
          },
          null,
          2,
        ),
      },
    ],
  };
}

export async function handleGetPalette(
  ctx: ServerContext,
  binding: ReturnType<ServerContext["loadNative"]>,
  args: Record<string, unknown> | undefined,
): Promise<ToolResult> {
  const resolved = await resolveArtReference(ctx, args);
  const source = await fs.promises.readFile(resolved.absolutePath, "utf-8");
  const palette = await buildPalette(ctx, binding, resolved, source);

  if (!palette) {
    throw new McpError(
      ErrorCode.InvalidParams,
      `Could not infer a palette for "${resolved.info.title}"`,
    );
  }

  return {
    content: [
      {
        type: "text",
        text: JSON.stringify(
          {
            component: {
              title: resolved.info.title,
              path: resolved.relativePath,
              componentReference: resolved.info.component,
            },
            match: {
              matchedBy: resolved.matchedBy,
              matchValue: resolved.matchValue,
              score: resolved.score,
              reasons: resolved.reasons,
              alternatives: resolved.alternatives,
            },
            title: palette.title,
            controls: palette.controls,
            groups: palette.groups,
            json: palette.json,
            typescript: palette.typescript,
          },
          null,
          2,
        ),
      },
    ],
  };
}
