/**
 * MCP tool handlers for code generation.
 *
 * Handles `generate_variants`, `generate_csf`, `generate_docs`,
 * `generate_catalog`, `get_tokens`, and `search_tokens` tool calls.
 */

import fs from "node:fs";
import path from "node:path";
import { ErrorCode, McpError } from "@modelcontextprotocol/sdk/types.js";
import { buildCatalogMarkdown, buildDocumentation, resolveArtReference } from "../../musea.js";
import {
  flattenTokenCategories,
  generateTokensMarkdown,
  parseTokensFromPath,
} from "../../tokens.js";
import type { ServerContext, ToolResult } from "./types.js";

export async function handleGenerateVariants(
  ctx: ServerContext,
  binding: ReturnType<ServerContext["loadNative"]>,
  args: Record<string, unknown> | undefined,
): Promise<ToolResult> {
  const componentRelPath = args?.componentPath as string;
  if (!componentRelPath) {
    throw new McpError(ErrorCode.InvalidParams, "componentPath is required");
  }
  if (!binding.analyzeSfc) {
    throw new McpError(ErrorCode.InternalError, "analyzeSfc not available in native binding");
  }
  if (!binding.generateVariants) {
    throw new McpError(ErrorCode.InternalError, "generateVariants not available in native binding");
  }

  const absolutePath = path.resolve(ctx.projectRoot, componentRelPath);
  const source = await fs.promises.readFile(absolutePath, "utf-8");

  const analysis = binding.analyzeSfc(source, { filename: absolutePath });
  const props = analysis.props.map((prop) => ({
    name: prop.name,
    prop_type: prop.type,
    required: prop.required,
    default_value: prop.default_value,
  }));

  const relPath = `./${path.basename(absolutePath)}`;
  const result = binding.generateVariants(relPath, props, {
    max_variants: args?.maxVariants as number | undefined,
    include_default: args?.includeDefault as boolean | undefined,
    include_boolean_toggles: args?.includeBooleanToggles as boolean | undefined,
    include_enum_variants: args?.includeEnumVariants as boolean | undefined,
  });

  return {
    content: [
      {
        type: "text",
        text: JSON.stringify(
          {
            componentPath: componentRelPath,
            componentName: result.component_name,
            artFileContent: result.art_file_content,
            variants: result.variants.map((variant) => ({
              name: variant.name,
              isDefault: variant.is_default,
              props: variant.props,
              description: variant.description,
            })),
          },
          null,
          2,
        ),
      },
    ],
  };
}

export async function handleGenerateCsf(
  ctx: ServerContext,
  binding: ReturnType<ServerContext["loadNative"]>,
  args: Record<string, unknown> | undefined,
): Promise<ToolResult> {
  const resolved = await resolveArtReference(ctx, args);
  const source = await fs.promises.readFile(resolved.absolutePath, "utf-8");
  const csf = binding.artToCsf(source, { filename: resolved.absolutePath });

  return {
    content: [
      {
        type: "text",
        text: JSON.stringify(
          {
            component: {
              title: resolved.info.title,
              path: resolved.relativePath,
            },
            match: {
              matchedBy: resolved.matchedBy,
              matchValue: resolved.matchValue,
              score: resolved.score,
              reasons: resolved.reasons,
              alternatives: resolved.alternatives,
            },
            filename: csf.filename,
            code: csf.code,
          },
          null,
          2,
        ),
      },
    ],
  };
}

export async function handleGenerateDocs(
  ctx: ServerContext,
  binding: ReturnType<ServerContext["loadNative"]>,
  args: Record<string, unknown> | undefined,
): Promise<ToolResult> {
  const resolved = await resolveArtReference(ctx, args);
  const source = await fs.promises.readFile(resolved.absolutePath, "utf-8");

  if (!binding.generateArtDoc) {
    throw new McpError(ErrorCode.InternalError, "generateArtDoc not available in native binding");
  }

  const doc = binding.generateArtDoc(
    source,
    { filename: resolved.absolutePath },
    {
      include_source: args?.includeSource as boolean | undefined,
      include_templates: args?.includeTemplates as boolean | undefined,
      include_metadata: true,
    },
  );
  const formattedDoc = await buildDocumentation(binding, resolved, source, {
    includeSource: args?.includeSource as boolean | undefined,
    includeTemplates: args?.includeTemplates as boolean | undefined,
  });

  return {
    content: [
      {
        type: "text",
        text: JSON.stringify(
          {
            component: {
              title: resolved.info.title,
              path: resolved.relativePath,
            },
            match: {
              matchedBy: resolved.matchedBy,
              matchValue: resolved.matchValue,
              score: resolved.score,
              reasons: resolved.reasons,
              alternatives: resolved.alternatives,
            },
            markdown: formattedDoc?.markdown ?? doc.markdown,
            title: doc.title,
            category: doc.category,
            variantCount: doc.variant_count,
          },
          null,
          2,
        ),
      },
    ],
  };
}

export async function handleGenerateCatalog(
  ctx: ServerContext,
  binding: ReturnType<ServerContext["loadNative"]>,
  args: Record<string, unknown> | undefined,
): Promise<ToolResult> {
  const arts = await ctx.scanArtFiles();

  if (binding.generateArtCatalog) {
    const sources: string[] = [];
    for (const [filePath] of arts) {
      const source = await fs.promises.readFile(filePath, "utf-8");
      sources.push(source);
    }

    const catalog = binding.generateArtCatalog(sources, {
      include_source: args?.includeSource as boolean | undefined,
      include_templates: args?.includeTemplates as boolean | undefined,
      include_metadata: true,
    });

    return {
      content: [
        {
          type: "text",
          text: JSON.stringify(
            {
              markdown: catalog.markdown,
              componentCount: catalog.component_count,
              categories: catalog.categories,
              tags: catalog.tags,
            },
            null,
            2,
          ),
        },
      ],
    };
  }

  const allArts = Array.from(arts.values());
  return {
    content: [
      {
        type: "text",
        text: JSON.stringify(
          {
            markdown: buildCatalogMarkdown(allArts, ctx.projectRoot),
            componentCount: allArts.length,
            categories: Array.from(new Set(allArts.map((art) => art.category || "Uncategorized"))),
            tags: Array.from(new Set(allArts.flatMap((art) => art.tags))),
          },
          null,
          2,
        ),
      },
    ],
  };
}

export async function handleGetTokens(
  ctx: ServerContext,
  args: Record<string, unknown> | undefined,
): Promise<ToolResult> {
  const inputPath = args?.tokensPath as string | undefined;
  const format = (args?.format as string) ?? "json";

  let resolvedPath: string | null;
  if (inputPath) {
    resolvedPath = path.resolve(ctx.projectRoot, inputPath);
  } else {
    resolvedPath = await ctx.resolveTokensPath();
  }

  if (!resolvedPath) {
    throw new McpError(
      ErrorCode.InvalidParams,
      "No tokens path provided and none auto-detected. Looked for: tokens/, design-tokens/, style-dictionary/ directories.",
    );
  }

  const categories = await parseTokensFromPath(resolvedPath);
  const flattened = flattenTokenCategories(categories);

  if (format === "markdown") {
    return {
      content: [
        {
          type: "text",
          text: generateTokensMarkdown(categories),
        },
      ],
    };
  }

  return {
    content: [
      {
        type: "text",
        text: JSON.stringify(
          {
            source: path.relative(ctx.projectRoot, resolvedPath),
            categoryCount: categories.length,
            tokenCount: flattened.length,
            categories,
          },
          null,
          2,
        ),
      },
    ],
  };
}

export async function handleSearchTokens(
  ctx: ServerContext,
  args: Record<string, unknown> | undefined,
): Promise<ToolResult> {
  const query = args?.query as string | undefined;
  if (!query) {
    throw new McpError(ErrorCode.InvalidParams, "query is required");
  }

  const inputPath = args?.tokensPath as string | undefined;
  const typeFilter = typeof args?.type === "string" ? args.type.toLowerCase() : undefined;
  const limit = typeof args?.limit === "number" ? args.limit : 20;
  const resolvedPath = inputPath
    ? path.resolve(ctx.projectRoot, inputPath)
    : await ctx.resolveTokensPath();

  if (!resolvedPath) {
    throw new McpError(
      ErrorCode.InvalidParams,
      "No tokens path provided and none auto-detected. Looked for: tokens/, design-tokens/, style-dictionary/ directories.",
    );
  }

  const flattened = flattenTokenCategories(await parseTokensFromPath(resolvedPath));
  const normalizedQuery = query.toLowerCase();

  const allMatches = flattened.filter((token) => {
    if (typeFilter && token.type?.toLowerCase() !== typeFilter) {
      return false;
    }
    return (
      token.name.toLowerCase().includes(normalizedQuery) ||
      token.path.toLowerCase().includes(normalizedQuery) ||
      token.categoryPath.some((segment) => segment.toLowerCase().includes(normalizedQuery)) ||
      String(token.value).toLowerCase().includes(normalizedQuery) ||
      token.description?.toLowerCase().includes(normalizedQuery)
    );
  });
  const matches = allMatches.slice(0, limit);

  return {
    content: [
      {
        type: "text",
        text: JSON.stringify(
          {
            query,
            source: path.relative(ctx.projectRoot, resolvedPath),
            totalMatches: allMatches.length,
            matches,
          },
          null,
          2,
        ),
      },
    ],
  };
}
