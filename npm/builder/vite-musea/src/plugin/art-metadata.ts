import { parseStaticCallArgument } from "../native-loader.js";
import type { NativeBinding } from "../native-loader.js";
import type { ArtMetadata } from "../types/index.js";

function findTagEnd(source: string, start: number): number {
  let quote: '"' | "'" | undefined;
  for (let index = start; index < source.length; index += 1) {
    const character = source[index];
    if (quote) {
      if (character === quote) quote = undefined;
    } else if (character === '"' || character === "'") {
      quote = character;
    } else if (character === ">") {
      return index + 1;
    }
  }
  return source.length;
}

function findRawTextEnd(source: string, start: number, tagName: string): number {
  const closingTag = new RegExp(`</${tagName}\\s*>`, "giu");
  closingTag.lastIndex = start;
  const match = closingTag.exec(source);
  return match ? match.index + match[0].length : source.length;
}

function maskInactiveMarkup(source: string): string {
  const openingRawTextTag = /<(script|style)(?=[\s/>])/iuy;
  const parts: string[] = [];
  let cursor = 0;
  let searchFrom = 0;

  while (searchFrom < source.length) {
    const start = source.indexOf("<", searchFrom);
    if (start < 0) break;

    let end: number | undefined;
    if (source.startsWith("<!--", start)) {
      const commentEnd = source.indexOf("-->", start + 4);
      end = commentEnd < 0 ? source.length : commentEnd + 3;
    } else {
      openingRawTextTag.lastIndex = start;
      const match = openingRawTextTag.exec(source);
      if (match) {
        const openingTagEnd = findTagEnd(source, openingRawTextTag.lastIndex);
        end = findRawTextEnd(source, openingTagEnd, match[1]);
      }
    }

    if (end === undefined) {
      searchFrom = start + 1;
      continue;
    }
    parts.push(source.slice(cursor, start), " ".repeat(end - start));
    cursor = end;
    searchFrom = end;
  }

  parts.push(source.slice(cursor));
  return parts.join("");
}

function extractArtTagAttributes(source: string): Record<string, string | true> {
  const structuralSource = maskInactiveMarkup(source);
  const artTagMatch = structuralSource.match(/<art\b((?:[^>"']|"[^"]*"|'[^']*')*)>/iu);
  if (!artTagMatch) return {};

  const attributes: Record<string, string | true> = {};
  const attributePattern = /([^\s=/>]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+)))?/g;
  for (const match of artTagMatch[1].matchAll(attributePattern)) {
    const name = match[1];
    if (name && name !== "/") attributes[name] = match[2] ?? match[3] ?? match[4] ?? true;
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

const ART_STATUSES = new Set(["draft", "ready", "deprecated"]);

export function isArtStatus(value: unknown): value is NonNullable<ArtMetadata["status"]> {
  return typeof value === "string" && ART_STATUSES.has(value);
}

export function extractCustomArtMetadata(
  source: string,
): Pick<ArtMetadata, "actionEvents" | "status"> {
  const attrs = extractArtTagAttributes(source);
  const actionEvents = new Set(parseActionEvents(attrs["action-events"]) ?? []);
  const captureMousemove = attrs["capture-mousemove"];
  const authoredStatus = attrs.status;
  const status = isArtStatus(authoredStatus)
    ? authoredStatus
    : attrs.draft === true
      ? "draft"
      : attrs.deprecated === true
        ? "deprecated"
        : undefined;

  if (captureMousemove === true || captureMousemove === "true") actionEvents.add("mousemove");
  return {
    actionEvents: actionEvents.size > 0 ? [...actionEvents] : undefined,
    status,
  };
}

export function sourceWithOptionalArtTitle(source: string): string {
  const match = /<art\b((?:[^>"']|"[^"]*"|'[^']*')*)>/iu.exec(maskInactiveMarkup(source));
  if (!match || /\btitle\s*=/iu.test(match[1] ?? "")) return source;
  const start = match.index;
  return `${source.slice(0, start)}<art title=""${source.slice(start + "<art".length)}`;
}

export type MuseaArtMeta = {
  title: string;
  description: string;
  category: string;
  tags: string[];
  status: string;
  order: number;
  actionEvents: string[];
  parameters: Record<string, unknown>;
};

export function extractDefineArtMeta(source: string): Partial<MuseaArtMeta> {
  const value =
    parseStaticCallArgument(source, "defineArtMeta", 0) ??
    parseStaticCallArgument(source, "defineArt", 1);
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Partial<MuseaArtMeta>)
    : {};
}

const DEFINE_ART_META_KEYS = new Set<keyof MuseaArtMeta>([
  "title",
  "description",
  "category",
  "tags",
  "status",
  "order",
  "actionEvents",
  "parameters",
]);

export function reportDefineArtMetaWarnings(
  metadata: Partial<MuseaArtMeta>,
  filename: string,
  warn: (message: string) => void = console.warn,
): void {
  for (const key of Object.keys(metadata)) {
    if (!DEFINE_ART_META_KEYS.has(key as keyof MuseaArtMeta)) {
      warn(
        `[musea] ${filename}: unknown defineArt metadata key ${JSON.stringify(key)}; ignoring it`,
      );
    }
  }
  if (metadata.status !== undefined && !isArtStatus(metadata.status)) {
    warn(
      `[musea] ${filename}: unknown status ${JSON.stringify(metadata.status)}; ignoring it (expected "draft" | "ready" | "deprecated")`,
    );
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
  for (const warning of warnings) warn(`[musea] ${warning}`);
}
