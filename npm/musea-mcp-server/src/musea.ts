import fs from "node:fs";
import path from "node:path";
import { ErrorCode, McpError } from "@modelcontextprotocol/sdk/types.js";
import type { ArtInfo, NativeBinding, ServerContext } from "./types.js";

type PaletteControl = {
  name: string;
  control: string;
  defaultValue?: unknown;
  description?: string;
  required: boolean;
  options: Array<{ label: string; value: unknown }>;
  range?: { min: number; max: number; step?: number };
  group?: string;
};

export interface ComponentAnalysisResult {
  path: string;
  props: Array<{
    name: string;
    type: string;
    required: boolean;
    defaultValue?: unknown;
  }>;
  emits: string[];
}

export interface PaletteResult {
  title: string;
  controls: PaletteControl[];
  groups: string[];
  json: string;
  typescript: string;
}

export interface DocumentationResult {
  markdown: string;
  title: string;
  category?: string;
  variantCount: number;
}

export interface ArtSearchResult {
  info: ArtInfo;
  relativePath: string;
  score: number;
  reasons: string[];
}

export interface ResolvedArtReference {
  info: ArtInfo;
  absolutePath: string;
  relativePath: string;
  matchedBy: "path" | "title" | "component" | "query" | "ref";
  matchValue: string;
  score: number;
  reasons: string[];
  alternatives: Array<{
    path: string;
    title: string;
    component?: string;
    score: number;
    reasons: string[];
  }>;
}

interface ComponentSourceDescriptor {
  reference?: string;
  absolutePath?: string;
  path?: string;
  exists: boolean;
  error?: string;
}

function normalize(value: string | undefined): string {
  return value?.trim().toLowerCase() ?? "";
}

function normalizePathLike(value: string): string {
  return value.replace(/\\/g, "/").toLowerCase();
}

function tokenize(query: string): string[] {
  return Array.from(
    new Set(
      query
        .toLowerCase()
        .split(/[\s/,_-]+/)
        .map((term) => term.trim())
        .filter(Boolean),
    ),
  );
}

function toProjectPath(projectRoot: string, absolutePath: string): string {
  const relativePath = path.relative(projectRoot, absolutePath);
  if (!relativePath || relativePath.startsWith("..") || path.isAbsolute(relativePath)) {
    return absolutePath;
  }
  return relativePath;
}

function buildResourceUris(
  relativePath: string,
  variantNames: string[],
  hasComponentSource: boolean,
) {
  const encodedPath = encodeURIComponent(relativePath);
  return {
    component: `musea://component/${encodedPath}`,
    docs: `musea://docs/${encodedPath}`,
    source: `musea://source/${encodedPath}`,
    componentSource: hasComponentSource ? `musea://component-source/${encodedPath}` : undefined,
    variants: variantNames.map((variantName) => ({
      name: variantName,
      uri: `musea://variant/${encodedPath}/${encodeURIComponent(variantName)}`,
    })),
  };
}

function addScore(
  reasons: Set<string>,
  reason: string,
  amount: number,
  scoreRef: { value: number },
): void {
  reasons.add(reason);
  scoreRef.value += amount;
}

function getComponentCandidates(info: ArtInfo): string[] {
  const candidates = new Set<string>();
  if (info.component) {
    candidates.add(info.component);
    candidates.add(path.basename(info.component));
    candidates.add(path.basename(info.component, path.extname(info.component)));
  }
  return Array.from(candidates);
}

function scoreArtInfo(projectRoot: string, info: ArtInfo, query: string): ArtSearchResult | null {
  const normalizedQuery = normalize(query);
  if (!normalizedQuery) return null;

  const relativePath = toProjectPath(projectRoot, info.path);
  const relativePathNorm = normalizePathLike(relativePath);
  const titleNorm = normalize(info.title);
  const descriptionNorm = normalize(info.description);
  const categoryNorm = normalize(info.category);
  const tagNorms = info.tags.map((tag) => normalize(tag));
  const variantNorms = info.variantNames.map((variantName) => normalize(variantName));
  const componentCandidates = getComponentCandidates(info);
  const componentNorms = componentCandidates.map((component) => normalizePathLike(component));
  const reasons = new Set<string>();
  const scoreRef = { value: 0 };

  if (relativePathNorm === normalizePathLike(query)) {
    addScore(reasons, "exact path match", 220, scoreRef);
  } else if (relativePathNorm.includes(normalizePathLike(query))) {
    addScore(reasons, "path match", 70, scoreRef);
  }

  if (titleNorm === normalizedQuery) {
    addScore(reasons, "exact title match", 200, scoreRef);
  } else if (titleNorm.startsWith(normalizedQuery)) {
    addScore(reasons, "title prefix match", 140, scoreRef);
  } else if (titleNorm.includes(normalizedQuery)) {
    addScore(reasons, "title match", 110, scoreRef);
  }

  if (categoryNorm === normalizedQuery) {
    addScore(reasons, "exact category match", 120, scoreRef);
  } else if (categoryNorm.includes(normalizedQuery)) {
    addScore(reasons, "category match", 55, scoreRef);
  }

  if (descriptionNorm.includes(normalizedQuery)) {
    addScore(reasons, "description match", 60, scoreRef);
  }

  for (const tag of tagNorms) {
    if (tag === normalizedQuery) {
      addScore(reasons, "exact tag match", 130, scoreRef);
      break;
    }
    if (tag.includes(normalizedQuery)) {
      addScore(reasons, "tag match", 80, scoreRef);
      break;
    }
  }

  for (const variant of variantNorms) {
    if (variant === normalizedQuery) {
      addScore(reasons, "exact variant match", 130, scoreRef);
      break;
    }
    if (variant.includes(normalizedQuery)) {
      addScore(reasons, "variant match", 90, scoreRef);
      break;
    }
  }

  for (const component of componentNorms) {
    if (component === normalizePathLike(query)) {
      addScore(reasons, "exact component match", 180, scoreRef);
      break;
    }
    if (component.includes(normalizePathLike(query))) {
      addScore(reasons, "component match", 100, scoreRef);
      break;
    }
  }

  const terms = tokenize(query);
  for (const term of terms) {
    if (term === normalizedQuery) continue;
    if (titleNorm.includes(term)) {
      addScore(reasons, `title contains "${term}"`, 18, scoreRef);
    }
    if (descriptionNorm.includes(term)) {
      addScore(reasons, `description contains "${term}"`, 10, scoreRef);
    }
    if (categoryNorm.includes(term)) {
      addScore(reasons, `category contains "${term}"`, 10, scoreRef);
    }
    if (tagNorms.some((tag) => tag.includes(term))) {
      addScore(reasons, `tag contains "${term}"`, 16, scoreRef);
    }
    if (variantNorms.some((variant) => variant.includes(term))) {
      addScore(reasons, `variant contains "${term}"`, 14, scoreRef);
    }
    if (componentNorms.some((component) => component.includes(term))) {
      addScore(reasons, `component contains "${term}"`, 14, scoreRef);
    }
  }

  if (scoreRef.value <= 0) {
    return null;
  }

  return {
    info,
    relativePath,
    score: scoreRef.value,
    reasons: Array.from(reasons).slice(0, 5),
  };
}

function compareArtResults(left: ArtSearchResult, right: ArtSearchResult): number {
  return (
    right.score - left.score ||
    (left.info.order ?? Number.MAX_SAFE_INTEGER) - (right.info.order ?? Number.MAX_SAFE_INTEGER) ||
    left.info.title.localeCompare(right.info.title)
  );
}

export async function searchArtInfos(
  ctx: ServerContext,
  query: string,
  filters?: {
    category?: string;
    tag?: string;
    status?: string;
    component?: string;
    limit?: number;
  },
): Promise<ArtSearchResult[]> {
  const arts = Array.from((await ctx.scanArtFiles()).values());
  const category = normalize(filters?.category);
  const tag = normalize(filters?.tag);
  const status = normalize(filters?.status);
  const componentFilter = normalize(filters?.component);

  const filtered = arts.filter((info) => {
    if (category && normalize(info.category) !== category) return false;
    if (tag && !info.tags.some((item) => normalize(item) === tag)) return false;
    if (status && normalize(info.status) !== status) return false;
    if (
      componentFilter &&
      !getComponentCandidates(info).some((candidate) =>
        normalizePathLike(candidate).includes(componentFilter),
      )
    ) {
      return false;
    }
    return true;
  });

  const results = filtered
    .map((info) => scoreArtInfo(ctx.projectRoot, info, query))
    .filter((result): result is ArtSearchResult => result != null)
    .sort(compareArtResults);

  return results.slice(0, filters?.limit ?? 10);
}

function buildAlternatives(results: ArtSearchResult[]): ResolvedArtReference["alternatives"] {
  return results.slice(1, 4).map((result) => ({
    path: result.relativePath,
    title: result.info.title,
    component: result.info.component,
    score: result.score,
    reasons: result.reasons,
  }));
}

export async function resolveArtReference(
  ctx: ServerContext,
  args: Record<string, unknown> | undefined,
): Promise<ResolvedArtReference> {
  const arts = Array.from((await ctx.scanArtFiles()).values());
  const pathArg = typeof args?.path === "string" ? args.path : undefined;
  const titleArg = typeof args?.title === "string" ? args.title : undefined;
  const componentArg = typeof args?.component === "string" ? args.component : undefined;
  const queryArg = typeof args?.query === "string" ? args.query : undefined;
  const refArg = typeof args?.ref === "string" ? args.ref : undefined;

  if (pathArg) {
    const resolvedPath = path.isAbsolute(pathArg)
      ? pathArg
      : path.resolve(ctx.projectRoot, pathArg);
    const normalizedResolvedPath = normalizePathLike(resolvedPath);
    const normalizedRelativePath = normalizePathLike(path.relative(ctx.projectRoot, resolvedPath));
    const directMatch = arts.find((info) => {
      const infoRelativePath = normalizePathLike(path.relative(ctx.projectRoot, info.path));
      return (
        normalizePathLike(info.path) === normalizedResolvedPath ||
        infoRelativePath === normalizedRelativePath
      );
    });

    if (directMatch) {
      return {
        info: directMatch,
        absolutePath: directMatch.path,
        relativePath: toProjectPath(ctx.projectRoot, directMatch.path),
        matchedBy: "path",
        matchValue: pathArg,
        score: 999,
        reasons: ["exact path match"],
        alternatives: [],
      };
    }
  }

  if (titleArg) {
    const matches = arts.filter((info) => normalize(info.title) === normalize(titleArg));
    if (matches.length > 0) {
      const primary = matches[0];
      return {
        info: primary,
        absolutePath: primary.path,
        relativePath: toProjectPath(ctx.projectRoot, primary.path),
        matchedBy: "title",
        matchValue: titleArg,
        score: 950,
        reasons: ["exact title match"],
        alternatives: matches.slice(1, 4).map((info) => ({
          path: toProjectPath(ctx.projectRoot, info.path),
          title: info.title,
          component: info.component,
          score: 900,
          reasons: ["exact title match"],
        })),
      };
    }
  }

  if (componentArg) {
    const normalizedComponent = normalizePathLike(componentArg);
    const matches = arts.filter((info) =>
      getComponentCandidates(info).some(
        (candidate) => normalizePathLike(candidate) === normalizedComponent,
      ),
    );
    if (matches.length > 0) {
      const primary = matches[0];
      return {
        info: primary,
        absolutePath: primary.path,
        relativePath: toProjectPath(ctx.projectRoot, primary.path),
        matchedBy: "component",
        matchValue: componentArg,
        score: 930,
        reasons: ["exact component match"],
        alternatives: matches.slice(1, 4).map((info) => ({
          path: toProjectPath(ctx.projectRoot, info.path),
          title: info.title,
          component: info.component,
          score: 880,
          reasons: ["exact component match"],
        })),
      };
    }
  }

  const queryValue = queryArg ?? refArg ?? pathArg ?? titleArg ?? componentArg;
  if (!queryValue) {
    throw new McpError(
      ErrorCode.InvalidParams,
      "Provide one of: path, title, component, query, or ref",
    );
  }

  const results = await searchArtInfos(ctx, queryValue, { limit: 4 });
  if (results.length === 0) {
    throw new McpError(
      ErrorCode.InvalidParams,
      `No component matched "${queryValue}". Try list_components or search_components first.`,
    );
  }

  const primary = results[0];
  return {
    info: primary.info,
    absolutePath: primary.info.path,
    relativePath: primary.relativePath,
    matchedBy: queryArg ? "query" : "ref",
    matchValue: queryValue,
    score: primary.score,
    reasons: primary.reasons,
    alternatives: buildAlternatives(results),
  };
}

export function resolveComponentSourcePath(
  artAbsolutePath: string,
  componentReference?: string,
): string | null {
  if (!componentReference) {
    return null;
  }

  if (path.isAbsolute(componentReference)) {
    return componentReference;
  }

  return path.resolve(path.dirname(artAbsolutePath), componentReference);
}

async function getComponentSourceDescriptor(
  ctx: ServerContext,
  resolved: ResolvedArtReference,
): Promise<ComponentSourceDescriptor> {
  const componentPath = resolveComponentSourcePath(resolved.absolutePath, resolved.info.component);
  if (!componentPath) {
    return {
      reference: resolved.info.component,
      exists: false,
      error: "This art file does not declare a component source.",
    };
  }

  try {
    await fs.promises.access(componentPath, fs.constants.R_OK);
    return {
      reference: resolved.info.component,
      absolutePath: componentPath,
      path: toProjectPath(ctx.projectRoot, componentPath),
      exists: true,
    };
  } catch {
    return {
      reference: resolved.info.component,
      absolutePath: componentPath,
      path: toProjectPath(ctx.projectRoot, componentPath),
      exists: false,
      error: `Component source not found: ${toProjectPath(ctx.projectRoot, componentPath)}`,
    };
  }
}

function normalizeDefaultValue(value: unknown): unknown {
  if (value === "true") return true;
  if (value === "false") return false;
  if (
    typeof value === "string" &&
    ((value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'")))
  ) {
    return value.slice(1, -1);
  }
  return value;
}

function inferControlType(type: string): string {
  const normalizedType = type.toLowerCase();
  if (normalizedType === "boolean") return "boolean";
  if (normalizedType === "number") return "number";
  if (normalizedType.includes("|") && !normalizedType.includes("=>")) return "select";
  return "text";
}

function extractOptionsFromType(type: string): Array<{ label: string; value: unknown }> {
  const options: Array<{ label: string; value: unknown }> = [];
  for (const match of type.matchAll(/["']([^"']+)["']/g)) {
    options.push({ label: match[1], value: match[1] });
  }
  return options;
}

function buildPaletteFromAnalysis(title: string, analysis: ComponentAnalysisResult): PaletteResult {
  const controls = analysis.props.map((prop) => {
    const control = inferControlType(prop.type);
    return {
      name: prop.name,
      control,
      defaultValue: normalizeDefaultValue(prop.defaultValue),
      description: undefined,
      required: prop.required,
      options: control === "select" ? extractOptionsFromType(prop.type) : [],
      range: undefined,
      group: undefined,
    };
  });

  return {
    title,
    controls,
    groups: [],
    json: JSON.stringify({ title, controls }, null, 2),
    typescript: `export interface ${title.replace(/\s+/g, "")}Props {\n${controls
      .map((control) => {
        const type =
          control.control === "boolean"
            ? "boolean"
            : control.control === "number"
              ? "number"
              : control.control === "select" && control.options.length > 0
                ? control.options.map((option) => `"${String(option.value)}"`).join(" | ")
                : "string";
        return `  ${control.name}${control.required ? "" : "?"}: ${type};`;
      })
      .join("\n")}\n}\n`,
  };
}

export async function analyzeResolvedComponent(
  ctx: ServerContext,
  binding: NativeBinding,
  resolved: ResolvedArtReference,
): Promise<{ source: ComponentSourceDescriptor; analysis: ComponentAnalysisResult | null }> {
  const sourceDescriptor = await getComponentSourceDescriptor(ctx, resolved);
  if (!sourceDescriptor.exists || !sourceDescriptor.absolutePath) {
    return { source: sourceDescriptor, analysis: null };
  }

  if (!binding.analyzeSfc) {
    return {
      source: {
        ...sourceDescriptor,
        error: "analyzeSfc is not available in the native binding.",
      },
      analysis: null,
    };
  }

  const source = await fs.promises.readFile(sourceDescriptor.absolutePath, "utf-8");
  const analysis = binding.analyzeSfc(source, { filename: sourceDescriptor.absolutePath });
  return {
    source: sourceDescriptor,
    analysis: {
      path: sourceDescriptor.path ?? sourceDescriptor.absolutePath,
      props: analysis.props.map((prop) => ({
        name: prop.name,
        type: prop.type,
        required: prop.required,
        defaultValue: prop.default_value,
      })),
      emits: analysis.emits,
    },
  };
}

function formatGeneratedMarkdown(markdown: string, componentName: string): string {
  let formatted = markdown
    .replace(/<Self(\s|>|\/)/g, `<${componentName}$1`)
    .replace(/<\/Self>/g, `</${componentName}>`);

  formatted = formatted.replace(
    /```(\w*)\n([\s\S]*?)```/g,
    (_match: string, lang: string, code: string) => {
      const lines = code.split("\n");
      let minIndent = Infinity;

      for (const line of lines) {
        if (line.trim()) {
          const indent = line.match(/^(\s*)/)?.[1].length ?? 0;
          minIndent = Math.min(minIndent, indent);
        }
      }

      if (minIndent === Infinity) {
        minIndent = 0;
      }

      const normalizedLines = minIndent > 0 ? lines.map((line) => line.slice(minIndent)) : lines;

      return `\`\`\`${lang}\n${normalizedLines.join("\n")}\`\`\``;
    },
  );

  return formatted;
}

export async function buildPalette(
  ctx: ServerContext,
  binding: NativeBinding,
  resolved: ResolvedArtReference,
  source: string,
): Promise<PaletteResult | null> {
  let palette: PaletteResult | null = null;

  if (binding.generateArtPalette) {
    const generated = binding.generateArtPalette(source, { filename: resolved.absolutePath });
    palette = {
      title: generated.title,
      controls: generated.controls.map((control) => ({
        name: control.name,
        control: control.control,
        defaultValue: control.default_value,
        description: control.description,
        required: control.required,
        options: control.options,
        range: control.range,
        group: control.group,
      })),
      groups: generated.groups,
      json: generated.json,
      typescript: generated.typescript,
    };
  }

  if (palette && palette.controls.length > 0) {
    return palette;
  }

  const { analysis } = await analyzeResolvedComponent(ctx, binding, resolved);
  if (!analysis || analysis.props.length === 0) {
    return palette;
  }

  return buildPaletteFromAnalysis(resolved.info.title, analysis);
}

export async function buildDocumentation(
  binding: NativeBinding,
  resolved: ResolvedArtReference,
  source: string,
  options?: {
    includeSource?: boolean;
    includeTemplates?: boolean;
  },
): Promise<DocumentationResult | null> {
  if (!binding.generateArtDoc) {
    return null;
  }

  const doc = binding.generateArtDoc(
    source,
    { filename: resolved.absolutePath },
    {
      include_source: options?.includeSource,
      include_templates: options?.includeTemplates,
      include_metadata: true,
    },
  );

  return {
    markdown: formatGeneratedMarkdown(doc.markdown, resolved.info.title || "Component"),
    title: doc.title,
    category: doc.category,
    variantCount: doc.variant_count,
  };
}

export async function buildComponentDetails(
  ctx: ServerContext,
  binding: NativeBinding,
  resolved: ResolvedArtReference,
  options?: {
    includeAnalysis?: boolean;
    includePalette?: boolean;
    includeDocumentation?: boolean;
  },
): Promise<Record<string, unknown>> {
  const source = await fs.promises.readFile(resolved.absolutePath, "utf-8");
  const parsed = binding.parseArt(source, { filename: resolved.absolutePath });
  const componentState = await analyzeResolvedComponent(ctx, binding, resolved);
  const palette =
    options?.includePalette === false ? null : await buildPalette(ctx, binding, resolved, source);
  const documentation =
    options?.includeDocumentation === true
      ? await buildDocumentation(binding, resolved, source)
      : null;
  const resourceUris = buildResourceUris(
    resolved.relativePath,
    parsed.variants.map((variant) => variant.name),
    Boolean(componentState.source.reference),
  );

  return {
    path: resolved.relativePath,
    match: {
      matchedBy: resolved.matchedBy,
      matchValue: resolved.matchValue,
      score: resolved.score,
      reasons: resolved.reasons,
      alternatives: resolved.alternatives,
    },
    metadata: parsed.metadata,
    variants: parsed.variants.map((variant) => ({
      name: variant.name,
      template: variant.template,
      isDefault: variant.is_default,
      skipVrt: variant.skip_vrt,
    })),
    defaultVariant: parsed.variants.find((variant) => variant.is_default)?.name,
    variantNames: parsed.variants.map((variant) => variant.name),
    hasScriptSetup: parsed.has_script_setup,
    hasScript: parsed.has_script,
    styleCount: parsed.style_count,
    componentSource: componentState.source,
    componentAnalysis:
      options?.includeAnalysis === false
        ? undefined
        : (componentState.analysis ?? {
            path: componentState.source.path,
            props: [],
            emits: [],
            error: componentState.source.error,
          }),
    palette,
    documentation,
    resources: resourceUris,
  };
}

export function buildCatalogMarkdown(arts: ArtInfo[], projectRoot: string): string {
  const grouped = new Map<string, ArtInfo[]>();
  for (const art of arts) {
    const category = art.category || "Uncategorized";
    const list = grouped.get(category) ?? [];
    list.push(art);
    grouped.set(category, list);
  }

  let markdown = "# Musea Component Catalog\n\n";
  for (const [category, items] of Array.from(grouped.entries()).sort(([left], [right]) =>
    left.localeCompare(right),
  )) {
    markdown += `## ${category}\n\n`;
    for (const item of items.sort((left, right) => left.title.localeCompare(right.title))) {
      const relativePath = toProjectPath(projectRoot, item.path);
      markdown += `- **${item.title}** \`${relativePath}\``;
      if (item.description) {
        markdown += ` — ${item.description}`;
      }
      markdown += "\n";
      if (item.variantNames.length > 0) {
        markdown += `  Variants: ${item.variantNames.join(", ")}\n`;
      }
      if (item.tags.length > 0) {
        markdown += `  Tags: ${item.tags.join(", ")}\n`;
      }
    }
    markdown += "\n";
  }
  return markdown;
}

export function buildIndexSummary(ctx: ServerContext, arts: ArtInfo[]): Record<string, unknown> {
  const categories = new Map<string, number>();
  const tags = new Map<string, number>();

  for (const art of arts) {
    categories.set(
      art.category || "Uncategorized",
      (categories.get(art.category || "Uncategorized") ?? 0) + 1,
    );
    for (const tag of art.tags) {
      tags.set(tag, (tags.get(tag) ?? 0) + 1);
    }
  }

  return {
    componentCount: arts.length,
    categories: Array.from(categories.entries())
      .map(([name, count]) => ({ name, count }))
      .sort((left, right) => left.name.localeCompare(right.name)),
    tags: Array.from(tags.entries())
      .map(([name, count]) => ({ name, count }))
      .sort((left, right) => right.count - left.count || left.name.localeCompare(right.name)),
    components: arts
      .slice()
      .sort((left, right) => left.title.localeCompare(right.title))
      .map((art) => ({
        path: toProjectPath(ctx.projectRoot, art.path),
        title: art.title,
        description: art.description,
        component: art.component,
        category: art.category,
        status: art.status,
        tags: art.tags,
        variantCount: art.variantCount,
        variantNames: art.variantNames,
        defaultVariant: art.defaultVariant,
      })),
  };
}

export function getProjectPath(projectRoot: string, absolutePath: string): string {
  return toProjectPath(projectRoot, absolutePath);
}
