import fs from "node:fs";
import path from "node:path";

import { extractScriptSetupContent, extractScriptSetupIsolated } from "../art-module.js";
import { loadNative } from "../native-loader.js";
import type { ArtFileInfo, ArtMetadata } from "../types/index.js";

export interface ArtProcessingContext {
  root: string;
  command: string;
  onError?: (message: string) => void;
}

function extractArtTagAttributes(source: string): Record<string, string | true> {
  const artTagMatch = source.match(/<art\b([\s\S]*?)>/i);
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

function extractStyleBlocks(source: string): string[] {
  const styles: string[] = [];

  for (const match of source.matchAll(/<style\b([^>]*)>([\s\S]*?)<\/style>/gi)) {
    const attrs = match[1] ?? "";
    const content = match[2]?.trim();
    const lang = attrs.match(/\blang\s*=\s*["']([^"']+)["']/i)?.[1]?.toLowerCase();

    if (!content) continue;
    if (lang && lang !== "css") continue;

    styles.push(content);
  }

  return styles;
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

export async function processMuseaArtFile(
  filePath: string,
  ctx: ArtProcessingContext,
): Promise<ArtFileInfo | null> {
  try {
    const source = await fs.promises.readFile(filePath, "utf-8");
    const binding = loadNative();
    const parsed = binding.parseArt(source, { filename: filePath });
    const customMetadata = extractCustomArtMetadata(source);

    if (!parsed.variants || parsed.variants.length === 0) return null;

    const isInline = !filePath.endsWith(".art.vue");

    return {
      path: filePath,
      metadata: {
        title: parsed.metadata.title || (isInline ? path.basename(filePath, ".vue") : ""),
        description: parsed.metadata.description,
        component: isInline ? undefined : parsed.metadata.component,
        category: parsed.metadata.category,
        tags: parsed.metadata.tags,
        status: parsed.metadata.status as "draft" | "ready" | "deprecated",
        order: parsed.metadata.order,
        actionEvents: customMetadata.actionEvents ?? parsed.metadata.actionEvents,
      },
      variants: parsed.variants.map((v) => ({
        name: v.name,
        template: v.template,
        isDefault: v.isDefault,
        skipVrt: v.skipVrt,
      })),
      hasScriptSetup: isInline ? false : parsed.hasScriptSetup,
      scriptSetupContent:
        !isInline && parsed.hasScriptSetup ? extractScriptSetupContent(source) : undefined,
      scriptSetupIsolated:
        !isInline && parsed.hasScriptSetup ? extractScriptSetupIsolated(source) : true,
      hasScript: parsed.hasScript,
      styleCount: parsed.styleCount,
      styleBlocks: isInline ? [] : extractStyleBlocks(source),
      isInline,
      componentPath: isInline ? filePath : undefined,
    };
  } catch (error) {
    const message = formatArtProcessingError(ctx.root, filePath, error);
    if (ctx.command === "build") throw new Error(message);
    (ctx.onError ?? console.error)(message);
    return null;
  }
}
