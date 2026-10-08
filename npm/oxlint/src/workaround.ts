import { createHash } from "node:crypto";
import fs from "node:fs";
import { isUtf8 } from "node:buffer";
import { extractSfcBlocks } from "./sfc-blocks.ts";
import type { LineColumn } from "./model.js";

const SCRIPTLESS_WORKAROUND_MARKER = "oxlint-plugin-vize-scriptless";
const SCRIPTLESS_WORKAROUND_OPEN_TAG_PREFIX = `<script setup lang="ts" data-${SCRIPTLESS_WORKAROUND_MARKER}="`;
const SCRIPTLESS_WORKAROUND_CLOSE_TAG = "</script>";
const SCRIPTLESS_WORKAROUND_FILENAME_ATTR = `data-${SCRIPTLESS_WORKAROUND_MARKER}="`;
const LOCATION_BRIDGE_MARKER = "/*vize-location-bridge*/";
const ORIGINAL_PAYLOAD_PREFIX = "<!--vize-original-source:";

export const SCRIPTLESS_WORKAROUND_TRACKING = {
  strategy: "synthetic-script-setup-bridge",
  removeWhen:
    "Oxlint JS plugins can pass scriptless Vue/HTML files to native structured input with original Vue positions.",
} as const;

export interface ResolvedWorkaroundSource {
  filename: string;
  source: string;
  usesOriginalLocations: boolean;
}

/** Validate the original bytes before decoding reuse can enter native linting. */
export function readOriginalWorkaroundSource(filename: string): string {
  const bytes = fs.readFileSync(filename);
  if (!isUtf8(bytes)) throw new Error("Vize carrier cannot preserve non-UTF8 original file bytes.");
  return bytes.toString("utf8");
}

export function validateWorkaroundAuthority(
  state: ResolvedWorkaroundSource,
  physical: string,
): void {
  if (
    state.usesOriginalLocations &&
    state.filename !== physical &&
    readOriginalWorkaroundSource(state.filename) !== state.source
  )
    throw new Error("Vize carrier source no longer matches the original file bytes.");
}

export function hasScriptLikeBlock(source: string): boolean {
  return extractSfcBlocks(source).some(
    (block) => block.kind === "script" || block.kind === "script-setup",
  );
}

export function appendScriptlessWorkaround(source: string, filename: string): string {
  return prepareWorkaroundSource(source, filename).source;
}

/** Keep original coordinates in one safe host script and an exact source payload. */
export function prepareWorkaroundSource(source: string, filename: string) {
  const openTag = `${SCRIPTLESS_WORKAROUND_OPEN_TAG_PREFIX}${encodeWorkaroundFilename(filename)}">`;
  const script = `${openTag}${createWhitespaceMirror(source)}${LOCATION_BRIDGE_MARKER}</script>\n`;
  return {
    source: `${script}${ORIGINAL_PAYLOAD_PREFIX}${createHash("sha256").update(filename).update("\0").update(source).digest("hex")}:${Buffer.from(source, "utf8").toString("base64url")}-->`,
    locations: {
      source,
      scriptStart: openTag.length,
      originalStart: Buffer.byteLength(script, "utf8"),
    },
  };
}

export function isLocationBridgeProgram(extractedScript: string): boolean {
  return extractedScript.trimEnd().endsWith(LOCATION_BRIDGE_MARKER);
}

export function resolveWorkaroundSource(
  source: string,
  fallbackFilename: string,
  readOriginalSource?: (filename: string) => string,
): ResolvedWorkaroundSource {
  const workaroundBlock = getPrependedWorkaroundBlock(source);
  if (workaroundBlock == null) {
    return {
      filename: fallbackFilename,
      source,
      usesOriginalLocations: false,
    };
  }

  let strippedSourceStart = workaroundBlock.closeTagEnd;
  if (source.charCodeAt(strippedSourceStart) === 13) {
    strippedSourceStart += 1;
  }
  if (source.charCodeAt(strippedSourceStart) === 10) {
    strippedSourceStart += 1;
  }

  const decodedFilename =
    decodeWorkaroundFilename(workaroundBlock.encodedFilename) ?? fallbackFilename;
  const strippedSource = workaroundBlock.originalSource ?? source.slice(strippedSourceStart);
  if (readOriginalSource && readOriginalSource(decodedFilename) !== strippedSource)
    throw new Error("Vize carrier source no longer matches the original file bytes.");

  return {
    filename: decodedFilename,
    source: strippedSource,
    usesOriginalLocations: true,
  };
}

function getPrependedWorkaroundBlock(
  source: string,
): { closeTagEnd: number; encodedFilename: string; originalSource?: string } | null {
  if (!source.startsWith(SCRIPTLESS_WORKAROUND_OPEN_TAG_PREFIX)) {
    return null;
  }

  const [firstBlock] = extractSfcBlocks(source);
  if (!firstBlock || firstBlock.kind !== "script-setup") {
    return null;
  }
  if (
    !firstBlock.content.endsWith(LOCATION_BRIDGE_MARKER) ||
    !isWhitespaceOnly(firstBlock.content.slice(0, -LOCATION_BRIDGE_MARKER.length))
  ) {
    return null;
  }

  const contentStart = offsetFromLineColumn(source, firstBlock.contentStart);
  const contentEnd = offsetFromLineColumn(source, firstBlock.contentEnd);
  const openTag = source.slice(0, contentStart);
  const encodedFilename = encodedFilenameFromWorkaroundOpenTag(openTag);
  if (encodedFilename == null) {
    return null;
  }

  const closeTagEnd = contentEnd + SCRIPTLESS_WORKAROUND_CLOSE_TAG.length;
  const strippedSourceStart = sourceStartAfterSyntheticBlock(source, closeTagEnd);
  if (strippedSourceStart == null) {
    return null;
  }
  const tail = source.slice(strippedSourceStart);
  let originalSource: string | undefined;
  if (tail.startsWith(ORIGINAL_PAYLOAD_PREFIX)) {
    const payload = /^<!--vize-original-source:([a-f0-9]{64}):([A-Za-z0-9_-]*)-->$/u.exec(tail);
    if (payload == null) throw new Error("Invalid Vize original-source carrier payload.");
    const originalFilename = decodeWorkaroundFilename(encodedFilename);
    if (originalFilename == null || encodeWorkaroundFilename(originalFilename) !== encodedFilename)
      throw new Error("Invalid Vize original-source carrier filename.");
    originalSource = Buffer.from(payload[2], "base64url").toString("utf8");
    if (
      Buffer.from(originalSource, "utf8").toString("base64url") !== payload[2] ||
      createHash("sha256")
        .update(originalFilename)
        .update("\0")
        .update(originalSource)
        .digest("hex") !== payload[1] ||
      firstBlock.content !== createWhitespaceMirror(originalSource) + LOCATION_BRIDGE_MARKER
    )
      throw new Error(
        "Vize original-source carrier bytes do not match their complete source mirror.",
      );
  } else if (firstBlock.content !== createWhitespaceMirror(tail) + LOCATION_BRIDGE_MARKER) {
    return null;
  }

  return { closeTagEnd, encodedFilename, originalSource };
}

function sourceStartAfterSyntheticBlock(source: string, closeTagEnd: number): number | null {
  if (source.charCodeAt(closeTagEnd) === 13 && source.charCodeAt(closeTagEnd + 1) === 10) {
    return closeTagEnd + 2;
  }
  if (source.charCodeAt(closeTagEnd) === 10) {
    return closeTagEnd + 1;
  }

  return null;
}

function encodedFilenameFromWorkaroundOpenTag(openTag: string): string | null {
  if (!openTag.startsWith(SCRIPTLESS_WORKAROUND_OPEN_TAG_PREFIX)) {
    return null;
  }

  const encodedFilenameStart =
    openTag.indexOf(SCRIPTLESS_WORKAROUND_FILENAME_ATTR) +
    SCRIPTLESS_WORKAROUND_FILENAME_ATTR.length;
  const encodedFilenameEnd = openTag.indexOf('"', encodedFilenameStart);
  if (
    encodedFilenameStart < SCRIPTLESS_WORKAROUND_FILENAME_ATTR.length ||
    encodedFilenameEnd === -1
  ) {
    return null;
  }

  return openTag.slice(encodedFilenameStart, encodedFilenameEnd);
}

function createWhitespaceMirror(source: string): string {
  // Oxlint loc columns and Patina's NAPI locations both count UTF-16 units.
  // Keep both halves of an astral character, rather than collapsing it to one space.
  return source.replaceAll(/[^\r\n]/gu, (character) => (character.length === 2 ? "  " : " "));
}

function encodeWorkaroundFilename(filename: string): string {
  return Buffer.from(filename, "utf8").toString("base64url");
}

function decodeWorkaroundFilename(encoded: string): string | null {
  try {
    return Buffer.from(encoded, "base64url").toString("utf8");
  } catch {
    return null;
  }
}

function isWhitespaceOnly(value: string): boolean {
  return /^\s*$/u.test(value);
}

function offsetFromLineColumn(source: string, loc: LineColumn): number {
  let line = 1;
  let lineStart = 0;

  for (let index = 0; index < source.length && line < loc.line; index += 1) {
    if (source.charCodeAt(index) === 10) {
      line += 1;
      lineStart = index + 1;
    }
  }

  return lineStart + loc.column - 1;
}

if (import.meta.vitest) {
  const { describe, expect, it } = import.meta.vitest;

  describe("scriptless workaround helpers", () => {
    it("detects script blocks", () => {
      expect(hasScriptLikeBlock("<template><div /></template>")).toBe(false);
      expect(
        hasScriptLikeBlock("<template><div /></template>\n<script setup>const x = 1</script>"),
      ).toBe(true);
    });

    it("appends and resolves the workaround payload", () => {
      const source = "<template>\n  <div>hello</div>\n</template>\n";
      const filename = "/Users/example/Hello.vue";
      const appended = appendScriptlessWorkaround(source, filename);

      expect(appended).toMatch(/oxlint-plugin-vize-scriptless/u);
      expect(resolveWorkaroundSource(appended, "/Users/example/fallback.vue")).toEqual({
        filename,
        source,
        usesOriginalLocations: true,
      });
    });

    it("does not strip real script setup blocks that mimic the marker", () => {
      const fallbackFilename = "/Users/example/fallback.vue";
      const source = `${SCRIPTLESS_WORKAROUND_OPEN_TAG_PREFIX}${encodeWorkaroundFilename("/Users/example/Real.vue")}">const userCode = true;</script>\n<template />`;

      expect(resolveWorkaroundSource(source, fallbackFilename)).toEqual({
        filename: fallbackFilename,
        source,
        usesOriginalLocations: false,
      });
    });
  });
}
