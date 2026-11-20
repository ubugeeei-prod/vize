import fs from "node:fs";
import path from "node:path";

import {
  extractScriptSetupContent,
  extractScriptSetupIsolated,
  parseScriptSetupForArt,
} from "../art-module.js";
import { resolveArtComponent } from "../art-component.js";
import { artScopeAttribute, stampScopeAttribute } from "../art-style.js";
import { loadNative, parseStaticJsonExpression } from "../native-loader.js";
import type { NativeBinding } from "../native-loader.js";
import type { ArtFileInfo, ArtMetadata } from "../types/index.js";

export interface ArtProcessingContext {
  root: string;
  command: string;
  onError?: (message: string) => void;
}

function extractArtTagAttributes(source: string): Record<string, string | true> {
  const artTagMatch = source.match(/<art\b((?:[^>"']|"[^"]*"|'[^']*')*)>/i);
  if (!artTagMatch) return {};

  const attributes: Record<string, string | true> = {};
  const attrPattern = /([^\s=/>]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'))?/g;

  for (const match of artTagMatch[1].matchAll(attrPattern)) {
    const name = match[1];
    if (!name || name === "/") continue;
    attributes[name] = match[2] ?? match[3] ?? true;
  }

  return attributes;
}

function decodeAttributeValue(value: string): string {
  return value
    .replaceAll("&quot;", '"')
    .replaceAll("&apos;", "'")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&amp;", "&");
}

type VariantRuntimeOptions = {
  name?: string;
  play?: string;
  props?: Record<string, unknown>;
  attrs?: Record<string, unknown>;
  args?: Record<string, unknown>;
  viewport?: { width: number; height: number; deviceScaleFactor?: number };
};

/** Read runtime-only variant attributes that older native bindings do not expose. */
export function extractVariantRuntimeOptions(source: string): VariantRuntimeOptions[] {
  const variants: VariantRuntimeOptions[] = [];
  const tagPattern = /<variant\b((?:[^>"']|"[^"]*"|'[^']*')*)>/gi;
  const attrPattern = /([^\s=/>]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+)))?/g;

  for (const tag of source.matchAll(tagPattern)) {
    const attrs: Record<string, string | true> = {};
    for (const match of (tag[1] ?? "").matchAll(attrPattern)) {
      const name = match[1];
      if (!name || name === "/") continue;
      attrs[name] = match[2] ?? match[3] ?? match[4] ?? true;
    }

    const option: VariantRuntimeOptions = {};
    if (typeof attrs.name === "string") option.name = decodeAttributeValue(attrs.name);
    const play = attrs[":play"] ?? attrs.play;
    if (typeof play === "string") option.play = decodeAttributeValue(play).trim();
    const props = readVariantObject(attrs, "props");
    const variantAttrs = readVariantObject(attrs, "attrs");
    if (props) option.props = props;
    if (variantAttrs) option.attrs = variantAttrs;
    if (typeof attrs.args === "string") {
      try {
        const parsed = JSON.parse(decodeAttributeValue(attrs.args));
        if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) option.args = parsed;
      } catch {
        // Native parsing owns malformed args diagnostics and compatibility.
      }
    }
    if (typeof attrs[":args"] === "string") {
      const parsed = parseStaticJsonExpression(decodeAttributeValue(attrs[":args"]));
      if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
        option.args = parsed as Record<string, unknown>;
      }
    }
    if (typeof attrs.viewport === "string") {
      option.viewport = parseVariantViewport(decodeAttributeValue(attrs.viewport));
    }
    variants.push(option);
  }

  return variants;
}

function readVariantObject(
  attrs: Record<string, string | true>,
  name: string,
): Record<string, unknown> | undefined {
  const bound = attrs[`:${name}`];
  if (typeof bound === "string") {
    const parsed = parseStaticJsonExpression(decodeAttributeValue(bound));
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      return parsed as Record<string, unknown>;
    }
    throw new Error(`Musea variant :${name} must be a statically serializable object literal.`);
  }
  const literal = attrs[name];
  if (typeof literal !== "string") return undefined;
  try {
    const parsed: unknown = JSON.parse(decodeAttributeValue(literal));
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      return parsed as Record<string, unknown>;
    }
    throw new Error("not an object");
  } catch {
    throw new Error(`Musea variant ${name} must be a JSON object.`);
  }
}

const FALLTHROUGH_ATTRIBUTES = new Set(["class", "id", "role", "style", "tabindex"]);

/** Extract statically serializable inputs from one authored target instance. */
export function extractTargetInputs(
  template: string,
  componentName: string | undefined,
): { props?: Record<string, unknown>; attrs?: Record<string, unknown> } {
  if (!componentName) return {};
  const names = [
    componentName,
    componentName.replace(/([a-z0-9])([A-Z])/gu, "$1-$2").toLowerCase(),
  ];
  const escaped = names.map((name) => name.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&")).join("|");
  const matches = [
    ...template.matchAll(
      new RegExp(`<(?:${escaped})(?=\\s|/?>)((?:[^>"']|"[^"]*"|'[^']*')*)/?>`, "gu"),
    ),
  ];
  if (matches.length !== 1) return {};

  const props: Record<string, unknown> = {};
  const attrs: Record<string, unknown> = {};
  const attributes = matches[0]?.[1] ?? "";
  const pattern = /([^\s=/>]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+)))?/gu;
  for (const match of attributes.matchAll(pattern)) {
    const rawName = match[1];
    if (!rawName || rawName === "/" || rawName.startsWith("@") || rawName.startsWith("#")) {
      continue;
    }
    if (/^v-(?:if|else|else-if|for|show|model|slot|on)(?::|$)/u.test(rawName)) continue;

    const isBound = rawName.startsWith(":") || rawName.startsWith("v-bind:");
    const name = rawName.replace(/^:/u, "").replace(/^v-bind:/u, "");
    if (!name || name === "key" || name === "ref" || name === "is" || name === "v-bind") continue;
    const rawValue = decodeAttributeValue(match[2] ?? match[3] ?? match[4] ?? "");
    let value: unknown = rawValue || true;
    if (isBound) {
      try {
        value = parseStaticJsonExpression(rawValue);
      } catch {
        continue;
      }
      if (value === undefined) continue;
    }

    const normalized = name.replace(/-([a-z])/gu, (_whole, letter: string) => letter.toUpperCase());
    const lower = name.toLowerCase();
    const isAttribute =
      lower.startsWith("aria-") || lower.startsWith("data-") || FALLTHROUGH_ATTRIBUTES.has(lower);
    (isAttribute ? attrs : props)[isAttribute ? name : normalized] = value;
  }

  return {
    ...(Object.keys(props).length > 0 ? { props } : {}),
    ...(Object.keys(attrs).length > 0 ? { attrs } : {}),
  };
}

function parseVariantViewport(
  value: string,
): { width: number; height: number; deviceScaleFactor?: number } | undefined {
  try {
    if (value.startsWith("{")) {
      const parsed = JSON.parse(value) as Record<string, unknown>;
      if (typeof parsed.width === "number" && typeof parsed.height === "number") {
        return {
          width: parsed.width,
          height: parsed.height,
          ...(typeof parsed.deviceScaleFactor === "number"
            ? { deviceScaleFactor: parsed.deviceScaleFactor }
            : {}),
        };
      }
    }
  } catch {
    return undefined;
  }
  const match = value.match(/^(\d+)x(\d+)(?:@(\d+(?:\.\d+)?))?$/);
  if (!match) return undefined;
  return {
    width: Number(match[1]),
    height: Number(match[2]),
    ...(match[3] ? { deviceScaleFactor: Number(match[3]) } : {}),
  };
}

function parseActionEvents(value: string | true | undefined): string[] | undefined {
  if (typeof value !== "string") return undefined;

  const events = value
    .split(",")
    .map((eventName) => eventName.trim().toLowerCase())
    .filter(Boolean);

  return events.length > 0 ? [...new Set(events)] : undefined;
}

function extractCustomArtMetadata(source: string): Pick<ArtMetadata, "actionEvents"> {
  const attrs = extractArtTagAttributes(source);
  const actionEvents = new Set(parseActionEvents(attrs["action-events"]) ?? []);
  const captureMousemove = attrs["capture-mousemove"];

  if (captureMousemove === true || captureMousemove === "true") {
    actionEvents.add("mousemove");
  }

  return {
    actionEvents: actionEvents.size > 0 ? [...actionEvents] : undefined,
  };
}

export function sourceWithOptionalArtTitle(source: string): string {
  const match = source.match(/<art\b((?:[^>"']|"[^"]*"|'[^']*')*)>/iu);
  if (!match || /\btitle\s*=/iu.test(match[1] ?? "")) return source;
  return source.replace(/<art\b/iu, '<art title=""');
}

function extractDefineArtMeta(source: string): Partial<MuseaArtMeta> {
  const match = source.match(/\bdefineArtMeta\s*\(\s*(\{[\s\S]*?\})\s*\)/u);
  if (!match?.[1]) return {};
  const value = parseStaticJsonExpression(match[1]);
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Partial<MuseaArtMeta>)
    : {};
}

type MuseaArtMeta = {
  title: string;
  description: string;
  category: string;
  tags: string[];
  status: "draft" | "ready" | "deprecated";
  order: number;
  actionEvents: string[];
};

function extractStyleBlocks(
  source: string,
  filePath: string,
): { blocks: Array<{ content: string; lang: string; scoped: boolean }>; scopeAttr?: string } {
  const styles: Array<{ content: string; lang: string; scoped: boolean }> = [];

  for (const match of source.matchAll(
    /<style\b((?:[^>"']|"[^"]*"|'[^']*')*)>([\s\S]*?)<\/style>/gi,
  )) {
    const attrs = match[1] ?? "";
    const content = match[2]?.trim();
    const lang = attrs.match(/\blang\s*=\s*["']([^"']+)["']/i)?.[1]?.toLowerCase();

    if (!content) continue;
    styles.push({ content, lang: lang ?? "css", scoped: /\bscoped\b/i.test(attrs) });
  }

  if (!styles.some((style) => style.scoped)) {
    return { blocks: styles };
  }

  const scopeAttr = artScopeAttribute(filePath);
  return {
    scopeAttr,
    blocks: styles,
  };
}

function formatArtProcessingError(root: string, filePath: string, error: unknown): string {
  const detail =
    error instanceof Error
      ? error.message || error.name
      : error == null
        ? "unknown error"
        : stringifyUnknown(error);
  const relativePath = path.relative(root, filePath) || filePath;
  return `[musea] Failed to process ${relativePath}: ${detail}`;
}

function stringifyUnknown(value: unknown): string {
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean" || typeof value === "bigint") {
    return value.toString();
  }
  try {
    return JSON.stringify(value) ?? "unknown error";
  } catch {
    return "unknown error";
  }
}

export function reportArtStatusWarnings(
  binding: Pick<NativeBinding, "parseArtStatusWarnings">,
  source: string,
  filename: string,
  warn: (message: string) => void = console.warn,
): void {
  const warnings = binding.parseArtStatusWarnings?.(source, { filename });
  if (warnings == null) return;
  for (const warning of warnings) {
    warn(`[musea] ${warning}`);
  }
}

export async function processMuseaArtFile(
  filePath: string,
  ctx: ArtProcessingContext,
): Promise<ArtFileInfo | null> {
  try {
    const source = await fs.promises.readFile(filePath, "utf-8");
    const binding = loadNative();
    const parsed = binding.parseArt(sourceWithOptionalArtTitle(source), { filename: filePath });
    reportArtStatusWarnings(binding, source, filePath);
    const customMetadata = extractCustomArtMetadata(source);
    const macroMetadata = extractDefineArtMeta(source);
    const variantOptions = extractVariantRuntimeOptions(source);

    if (!parsed.variants || parsed.variants.length === 0) return null;

    const isInline = !filePath.endsWith(".art.vue");
    const styles = isInline
      ? { blocks: [] as Array<{ content: string; lang: string; scoped: boolean }> }
      : extractStyleBlocks(source, filePath);

    const scriptSetupContent =
      !isInline && parsed.hasScriptSetup ? extractScriptSetupContent(source) : undefined;
    const baseVariants = parsed.variants.map((variant) => ({
      name: variant.name,
      template: styles.scopeAttr
        ? stampScopeAttribute(variant.template, styles.scopeAttr)
        : variant.template,
      isDefault: variant.isDefault,
      skipVrt: variant.skipVrt,
    }));
    const provisional: ArtFileInfo = {
      path: filePath,
      metadata: {
        title: macroMetadata.title ?? parsed.metadata.title,
        description: macroMetadata.description ?? parsed.metadata.description,
        component: isInline ? undefined : parsed.metadata.component,
        category: macroMetadata.category ?? parsed.metadata.category,
        tags: macroMetadata.tags ?? parsed.metadata.tags,
        status:
          macroMetadata.status ?? (parsed.metadata.status as "draft" | "ready" | "deprecated"),
        order: macroMetadata.order ?? parsed.metadata.order,
      },
      variants: baseVariants,
      hasScriptSetup: !isInline && parsed.hasScriptSetup,
      scriptSetupContent,
      hasScript: parsed.hasScript,
      styleCount: parsed.styleCount,
      isInline,
      componentPath: isInline ? filePath : undefined,
    };
    const parsedScriptSetup = scriptSetupContent
      ? parseScriptSetupForArt(scriptSetupContent)
      : null;
    const component = resolveArtComponent(provisional, filePath, parsedScriptSetup, {
      root: ctx.root,
    });
    const inferredTitle =
      component.componentTagName ?? path.basename(filePath).replace(/\.art\.vue$/u, "");

    return {
      path: filePath,
      metadata: {
        title:
          macroMetadata.title ||
          parsed.metadata.title ||
          (isInline ? path.basename(filePath, ".vue") : inferredTitle),
        description: macroMetadata.description ?? parsed.metadata.description,
        component: isInline ? undefined : (parsed.metadata.component ?? component.componentSource),
        category: macroMetadata.category ?? parsed.metadata.category,
        tags: macroMetadata.tags ?? parsed.metadata.tags,
        status:
          macroMetadata.status ?? (parsed.metadata.status as "draft" | "ready" | "deprecated"),
        order: macroMetadata.order ?? parsed.metadata.order,
        actionEvents:
          customMetadata.actionEvents ?? macroMetadata.actionEvents ?? parsed.metadata.actionEvents,
      },
      variants: parsed.variants.map((v, index) => {
        const runtime = variantOptions[index]?.name === v.name ? variantOptions[index] : undefined;
        const targetInputs = extractTargetInputs(v.template, component.componentTagName);
        const legacyArgs = runtime?.args ?? v.args;
        const viewport =
          v.viewport ??
          runtime?.viewport ??
          (legacyArgs?.viewport as VariantRuntimeOptions["viewport"] | undefined);
        const legacyProps = legacyArgs
          ? Object.fromEntries(Object.entries(legacyArgs).filter(([name]) => name !== "viewport"))
          : undefined;
        const props = runtime?.props ?? v.props ?? targetInputs.props ?? legacyProps;
        const attrs = runtime?.attrs ?? v.attrs ?? targetInputs.attrs;
        const play = runtime?.play ?? v.play;
        return {
          name: v.name,
          template: styles.scopeAttr
            ? stampScopeAttribute(v.template, styles.scopeAttr)
            : v.template,
          isDefault: v.isDefault,
          skipVrt: v.skipVrt,
          ...(play ? { play } : {}),
          ...(props && Object.keys(props).length > 0 ? { props } : {}),
          ...(attrs && Object.keys(attrs).length > 0 ? { attrs } : {}),
          ...(viewport ? { viewport } : {}),
          ...(legacyArgs && Object.keys(legacyArgs).length > 0 ? { args: legacyArgs } : {}),
        };
      }),
      hasScriptSetup: isInline ? false : parsed.hasScriptSetup,
      scriptSetupContent,
      scriptSetupIsolated:
        !isInline && parsed.hasScriptSetup ? extractScriptSetupIsolated(source) : true,
      hasScript: parsed.hasScript,
      styleCount: parsed.styleCount,
      styleBlocks: styles.blocks.map((style) => style.content),
      styleBlockDetails: styles.blocks,
      isInline,
      componentPath: isInline ? filePath : undefined,
      inlineArtBlockIndex: isInline
        ? binding
            .parseSfc(source, { filename: filePath })
            .customBlocks.findIndex((block) => block.type === "art")
        : undefined,
    };
  } catch (error) {
    const message = formatArtProcessingError(ctx.root, filePath, error);
    if (ctx.command === "build") throw new Error(message);
    (ctx.onError ?? console.error)(message);
    return null;
  }
}
