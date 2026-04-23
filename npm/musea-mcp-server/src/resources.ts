import fs from "node:fs";
import path from "node:path";
import { ErrorCode, McpError } from "@modelcontextprotocol/sdk/types.js";
import {
  buildCatalogMarkdown,
  buildComponentDetails,
  buildDocumentation,
  buildIndexSummary,
  getProjectPath,
  resolveArtReference,
} from "./musea.js";
import { flattenTokenCategories, generateTokensMarkdown, parseTokensFromPath } from "./tokens.js";
import type { ServerContext } from "./types.js";

function componentUri(relativePath: string): string {
  return `musea://component/${encodeURIComponent(relativePath)}`;
}

function docsUri(relativePath: string): string {
  return `musea://docs/${encodeURIComponent(relativePath)}`;
}

function sourceUri(relativePath: string): string {
  return `musea://source/${encodeURIComponent(relativePath)}`;
}

function componentSourceUri(relativePath: string): string {
  return `musea://component-source/${encodeURIComponent(relativePath)}`;
}

function variantUri(relativePath: string, variantName: string): string {
  return `musea://variant/${encodeURIComponent(relativePath)}/${encodeURIComponent(variantName)}`;
}

export async function listResources(ctx: ServerContext) {
  const arts = Array.from((await ctx.scanArtFiles()).values());
  const resources = [
    {
      uri: "musea://index",
      name: "Component Index",
      description: "JSON summary of the project component registry",
      mimeType: "application/json",
    },
    {
      uri: "musea://catalog",
      name: "Component Catalog",
      description: "Markdown catalog for the whole design system",
      mimeType: "text/markdown",
    },
  ];

  for (const info of arts) {
    const relativePath = getProjectPath(ctx.projectRoot, info.path);

    resources.push({
      uri: componentUri(relativePath),
      name: info.title,
      description:
        info.description ||
        `${info.category || "Component"} • ${info.variantCount} variant(s) • ${info.status}`,
      mimeType: "application/json",
    });

    resources.push({
      uri: docsUri(relativePath),
      name: `${info.title} — Documentation`,
      description: `Generated Markdown docs for ${info.title}`,
      mimeType: "text/markdown",
    });

    resources.push({
      uri: sourceUri(relativePath),
      name: `${info.title} — Art Source`,
      description: `Raw .art.vue source for ${info.title}`,
      mimeType: "text/plain",
    });

    if (info.component) {
      resources.push({
        uri: componentSourceUri(relativePath),
        name: `${info.title} — Component Source`,
        description: `Resolved Vue component source for ${info.title}`,
        mimeType: "text/plain",
      });
    }

    for (const variantName of info.variantNames) {
      resources.push({
        uri: variantUri(relativePath, variantName),
        name: `${info.title} — ${variantName}`,
        description: `Variant details for ${variantName}`,
        mimeType: "application/json",
      });
    }
  }

  const resolvedTokensPath = await ctx.resolveTokensPath();
  if (resolvedTokensPath) {
    resources.push({
      uri: "musea://tokens",
      name: "Design Tokens",
      description: "Project design tokens as JSON",
      mimeType: "application/json",
    });
    resources.push({
      uri: "musea://tokens/markdown",
      name: "Design Tokens — Markdown",
      description: "Project design tokens as Markdown tables",
      mimeType: "text/markdown",
    });
  }

  return { resources };
}

export async function readResource(ctx: ServerContext, uri: string) {
  if (uri === "musea://index") {
    const arts = Array.from((await ctx.scanArtFiles()).values());
    return {
      contents: [
        {
          uri,
          mimeType: "application/json",
          text: JSON.stringify(buildIndexSummary(ctx, arts), null, 2),
        },
      ],
    };
  }

  if (uri === "musea://catalog") {
    const binding = ctx.loadNative();
    const arts = Array.from((await ctx.scanArtFiles()).values());

    if (binding.generateArtCatalog) {
      const sources = await Promise.all(arts.map((art) => fs.promises.readFile(art.path, "utf-8")));
      const catalog = binding.generateArtCatalog(sources, { include_metadata: true });
      return {
        contents: [{ uri, mimeType: "text/markdown", text: catalog.markdown }],
      };
    }

    return {
      contents: [
        {
          uri,
          mimeType: "text/markdown",
          text: buildCatalogMarkdown(arts, ctx.projectRoot),
        },
      ],
    };
  }

  if (uri.startsWith("musea://component/")) {
    const relativePath = decodeURIComponent(uri.slice("musea://component/".length));
    const binding = ctx.loadNative();
    const details = await buildComponentDetails(
      ctx,
      binding,
      await resolveArtReference(ctx, { path: relativePath }),
      {
        includeAnalysis: true,
        includePalette: true,
        includeDocumentation: false,
      },
    );

    return {
      contents: [
        {
          uri,
          mimeType: "application/json",
          text: JSON.stringify(details, null, 2),
        },
      ],
    };
  }

  if (uri.startsWith("musea://docs/")) {
    const relativePath = decodeURIComponent(uri.slice("musea://docs/".length));
    const binding = ctx.loadNative();
    const resolved = await resolveArtReference(ctx, { path: relativePath });
    const source = await fs.promises.readFile(resolved.absolutePath, "utf-8");
    const documentation = await buildDocumentation(binding, resolved, source);

    if (!documentation) {
      throw new McpError(ErrorCode.InternalError, "generateArtDoc not available in native binding");
    }

    return {
      contents: [{ uri, mimeType: "text/markdown", text: documentation.markdown }],
    };
  }

  if (uri.startsWith("musea://source/")) {
    const relativePath = decodeURIComponent(uri.slice("musea://source/".length));
    const absolutePath = path.resolve(ctx.projectRoot, relativePath);
    const source = await fs.promises.readFile(absolutePath, "utf-8");

    return {
      contents: [{ uri, mimeType: "text/plain", text: source }],
    };
  }

  if (uri.startsWith("musea://component-source/")) {
    const relativePath = decodeURIComponent(uri.slice("musea://component-source/".length));
    const binding = ctx.loadNative();
    const details = (await buildComponentDetails(
      ctx,
      binding,
      await resolveArtReference(ctx, { path: relativePath }),
      {
        includeAnalysis: false,
        includePalette: false,
        includeDocumentation: false,
      },
    )) as {
      componentSource?: { path?: string; exists?: boolean; error?: string };
    };

    const componentSource = details.componentSource;
    if (!componentSource?.path || componentSource.exists !== true) {
      throw new McpError(
        ErrorCode.InvalidRequest,
        componentSource?.error ?? "Component source not available for this art file",
      );
    }

    const absolutePath = path.resolve(ctx.projectRoot, componentSource.path);
    const source = await fs.promises.readFile(absolutePath, "utf-8");
    return {
      contents: [{ uri, mimeType: "text/plain", text: source }],
    };
  }

  if (uri.startsWith("musea://variant/")) {
    const rest = uri.slice("musea://variant/".length);
    const separatorIndex = rest.indexOf("/");
    if (separatorIndex === -1) {
      throw new McpError(ErrorCode.InvalidRequest, `Unknown resource URI: ${uri}`);
    }

    const relativePath = decodeURIComponent(rest.slice(0, separatorIndex));
    const variantName = decodeURIComponent(rest.slice(separatorIndex + 1));
    const binding = ctx.loadNative();
    const resolved = await resolveArtReference(ctx, { path: relativePath });
    const source = await fs.promises.readFile(resolved.absolutePath, "utf-8");
    const parsed = binding.parseArt(source, { filename: resolved.absolutePath });
    const variant = parsed.variants.find(
      (item) => item.name.toLowerCase() === variantName.toLowerCase(),
    );

    if (!variant) {
      throw new McpError(
        ErrorCode.InvalidRequest,
        `Variant "${variantName}" not found in "${resolved.info.title}"`,
      );
    }

    return {
      contents: [
        {
          uri,
          mimeType: "application/json",
          text: JSON.stringify(
            {
              component: {
                title: resolved.info.title,
                path: resolved.relativePath,
                componentReference: resolved.info.component,
              },
              variant: {
                name: variant.name,
                template: variant.template,
                isDefault: variant.is_default,
                skipVrt: variant.skip_vrt,
              },
              relatedVariants: parsed.variants.map((item) => item.name),
            },
            null,
            2,
          ),
        },
      ],
    };
  }

  if (uri === "musea://tokens" || uri === "musea://tokens/markdown") {
    const resolvedTokensPath = await ctx.resolveTokensPath();
    if (!resolvedTokensPath) {
      throw new McpError(ErrorCode.InternalError, "No tokens path configured or auto-detected");
    }

    const categories = await parseTokensFromPath(resolvedTokensPath);
    if (uri === "musea://tokens/markdown") {
      return {
        contents: [
          {
            uri,
            mimeType: "text/markdown",
            text: generateTokensMarkdown(categories),
          },
        ],
      };
    }

    return {
      contents: [
        {
          uri,
          mimeType: "application/json",
          text: JSON.stringify(
            {
              source: path.relative(ctx.projectRoot, resolvedTokensPath),
              categoryCount: categories.length,
              tokenCount: flattenTokenCategories(categories).length,
              categories,
            },
            null,
            2,
          ),
        },
      ],
    };
  }

  throw new McpError(ErrorCode.InvalidRequest, `Unknown resource URI: ${uri}`);
}
