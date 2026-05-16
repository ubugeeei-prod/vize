/**
 * MCP tool handlers for the component registry.
 *
 * Handles `list_components`, `get_component`, `get_variant`, `search_components`,
 * and `recommend_components` tool calls.
 */

import fs from "node:fs";
import { ErrorCode, McpError } from "@modelcontextprotocol/sdk/types.js";
import {
  analyzeResolvedComponent,
  buildComponentDetails,
  getProjectPath,
  resolveArtReference,
  searchArtInfos,
} from "../../musea.js";
import type { ServerContext, ToolResult } from "./types.js";

function buildResourceUris(relativePath: string, variantNames: string[], hasComponent: boolean) {
  const encodedPath = encodeURIComponent(relativePath);
  return {
    component: `musea://component/${encodedPath}`,
    docs: `musea://docs/${encodedPath}`,
    source: `musea://source/${encodedPath}`,
    componentSource: hasComponent ? `musea://component-source/${encodedPath}` : undefined,
    variants: variantNames.map((variantName) => ({
      name: variantName,
      uri: `musea://variant/${encodedPath}/${encodeURIComponent(variantName)}`,
    })),
  };
}

export async function handleListComponents(
  ctx: ServerContext,
  args: Record<string, unknown> | undefined,
): Promise<ToolResult> {
  const arts = Array.from((await ctx.scanArtFiles()).values());
  const category = typeof args?.category === "string" ? args.category.toLowerCase() : undefined;
  const tag = typeof args?.tag === "string" ? args.tag.toLowerCase() : undefined;
  const status = typeof args?.status === "string" ? args.status.toLowerCase() : undefined;
  const component = typeof args?.component === "string" ? args.component.toLowerCase() : undefined;
  const limit = typeof args?.limit === "number" ? args.limit : undefined;
  const includeVariants = args?.includeVariants === true;
  const sortBy = typeof args?.sortBy === "string" ? args.sortBy : "title";

  let results = arts.filter((info) => {
    if (category && info.category?.toLowerCase() !== category) return false;
    if (tag && !info.tags.some((item) => item.toLowerCase() === tag)) return false;
    if (status && info.status.toLowerCase() !== status) return false;
    if (
      component &&
      ![
        info.component,
        info.component ? info.component.split("/").at(-1) : undefined,
        info.component
          ? info.component
              .split("/")
              .at(-1)
              ?.replace(/\.\w+$/, "")
          : undefined,
      ]
        .filter((value): value is string => Boolean(value))
        .some((value) => value.toLowerCase().includes(component))
    ) {
      return false;
    }
    return true;
  });

  results = results.sort((left, right) => {
    if (sortBy === "category") {
      return (
        (left.category || "").localeCompare(right.category || "") ||
        left.title.localeCompare(right.title)
      );
    }
    if (sortBy === "status") {
      return left.status.localeCompare(right.status) || left.title.localeCompare(right.title);
    }
    if (sortBy === "variants") {
      return right.variantCount - left.variantCount || left.title.localeCompare(right.title);
    }
    return left.title.localeCompare(right.title);
  });

  if (typeof limit === "number") {
    results = results.slice(0, limit);
  }

  return {
    content: [
      {
        type: "text",
        text: JSON.stringify(
          results.map((info) => {
            const relativePath = getProjectPath(ctx.projectRoot, info.path);
            return {
              path: relativePath,
              title: info.title,
              description: info.description,
              component: info.component,
              category: info.category,
              status: info.status,
              order: info.order,
              tags: info.tags,
              variantCount: info.variantCount,
              variantNames: info.variantNames,
              defaultVariant: info.defaultVariant,
              variants: includeVariants
                ? info.variantNames.map((variantName) => ({
                    name: variantName,
                    isDefault: variantName === info.defaultVariant,
                    uri: `musea://variant/${encodeURIComponent(relativePath)}/${encodeURIComponent(
                      variantName,
                    )}`,
                  }))
                : undefined,
              resources: buildResourceUris(
                relativePath,
                info.variantNames,
                Boolean(info.component),
              ),
            };
          }),
          null,
          2,
        ),
      },
    ],
  };
}

export async function handleGetComponent(
  ctx: ServerContext,
  binding: ReturnType<ServerContext["loadNative"]>,
  args: Record<string, unknown> | undefined,
): Promise<ToolResult> {
  const resolved = await resolveArtReference(ctx, args);
  const details = await buildComponentDetails(ctx, binding, resolved, {
    includeAnalysis: args?.includeAnalysis !== false,
    includePalette: args?.includePalette !== false,
    includeDocumentation: args?.includeDocumentation === true,
  });

  return {
    content: [{ type: "text", text: JSON.stringify(details, null, 2) }],
  };
}

export async function handleGetVariant(
  ctx: ServerContext,
  binding: ReturnType<ServerContext["loadNative"]>,
  args: Record<string, unknown> | undefined,
): Promise<ToolResult> {
  const variantName = args?.variant as string | undefined;
  if (!variantName) {
    throw new McpError(ErrorCode.InvalidParams, "variant is required");
  }

  const resolved = await resolveArtReference(ctx, args);
  const source = await fs.promises.readFile(resolved.absolutePath, "utf-8");
  const parsed = binding.parseArt(source, { filename: resolved.absolutePath });

  const variant = parsed.variants.find(
    (item) => item.name.toLowerCase() === variantName.toLowerCase(),
  );
  if (!variant) {
    throw new McpError(
      ErrorCode.InvalidParams,
      `Variant "${variantName}" not found in "${resolved.info.title}"`,
    );
  }

  const analysis =
    args?.includeAnalysis === true ? await analyzeResolvedComponent(ctx, binding, resolved) : null;

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
            variant: {
              name: variant.name,
              template: variant.template,
              isDefault: variant.is_default,
              skipVrt: variant.skip_vrt,
            },
            relatedVariants: parsed.variants.map((item) => item.name),
            componentAnalysis:
              analysis?.analysis == null
                ? undefined
                : {
                    path: analysis.analysis.path,
                    props: analysis.analysis.props,
                    emits: analysis.analysis.emits,
                  },
          },
          null,
          2,
        ),
      },
    ],
  };
}

export async function handleSearchComponents(
  ctx: ServerContext,
  args: Record<string, unknown> | undefined,
): Promise<ToolResult> {
  const query = args?.query as string | undefined;
  if (!query) {
    throw new McpError(ErrorCode.InvalidParams, "query is required");
  }

  const results = await searchArtInfos(ctx, query, {
    category: args?.category as string | undefined,
    tag: args?.tag as string | undefined,
    status: args?.status as string | undefined,
    component: args?.component as string | undefined,
    limit: (args?.limit as number | undefined) ?? 10,
  });

  return {
    content: [
      {
        type: "text",
        text: JSON.stringify(
          results.map((result) => ({
            score: result.score,
            reasons: result.reasons,
            path: result.relativePath,
            title: result.info.title,
            description: result.info.description,
            component: result.info.component,
            category: result.info.category,
            status: result.info.status,
            tags: result.info.tags,
            variantCount: result.info.variantCount,
            variantNames: result.info.variantNames,
            defaultVariant: result.info.defaultVariant,
            resources: buildResourceUris(
              result.relativePath,
              result.info.variantNames,
              Boolean(result.info.component),
            ),
          })),
          null,
          2,
        ),
      },
    ],
  };
}

export async function handleRecommendComponents(
  ctx: ServerContext,
  args: Record<string, unknown> | undefined,
): Promise<ToolResult> {
  const task = args?.task as string | undefined;
  if (!task) {
    throw new McpError(ErrorCode.InvalidParams, "task is required");
  }

  const results = await searchArtInfos(ctx, task, {
    category: args?.category as string | undefined,
    tag: args?.tag as string | undefined,
    status: args?.status as string | undefined,
    component: args?.component as string | undefined,
    limit: (args?.limit as number | undefined) ?? 5,
  });

  return {
    content: [
      {
        type: "text",
        text: JSON.stringify(
          {
            task,
            recommendations: results.map((result) => ({
              title: result.info.title,
              path: result.relativePath,
              component: result.info.component,
              category: result.info.category,
              status: result.info.status,
              tags: result.info.tags,
              variantNames: result.info.variantNames,
              defaultVariant: result.info.defaultVariant,
              score: result.score,
              why: result.reasons,
              resources: buildResourceUris(
                result.relativePath,
                result.info.variantNames,
                Boolean(result.info.component),
              ),
            })),
          },
          null,
          2,
        ),
      },
    ],
  };
}
