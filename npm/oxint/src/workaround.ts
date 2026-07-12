import { extractSfcBlocks } from "./sfc-blocks.js";
import type { LineColumn } from "./model.js";

const SCRIPTLESS_WORKAROUND_MARKER = "oxlint-plugin-vize-scriptless";
const SCRIPTLESS_WORKAROUND_OPEN_TAG_PREFIX = `<script setup lang="ts" data-${SCRIPTLESS_WORKAROUND_MARKER}="`;
const SCRIPTLESS_WORKAROUND_CLOSE_TAG = "</script>";
const SCRIPTLESS_WORKAROUND_FILENAME_ATTR = `data-${SCRIPTLESS_WORKAROUND_MARKER}="`;

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

export function hasScriptLikeBlock(source: string): boolean {
  return extractSfcBlocks(source).some(
    (block) => block.kind === "script" || block.kind === "script-setup",
  );
}

export function appendScriptlessWorkaround(source: string, filename: string): string {
  return `${createWorkaroundScript(source, filename)}${source}`;
}

export function resolveWorkaroundSource(
  source: string,
  fallbackFilename: string,
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
  const strippedSource = source.slice(strippedSourceStart);

  return {
    filename: decodedFilename,
    source: strippedSource,
    usesOriginalLocations: true,
  };
}

function getPrependedWorkaroundBlock(
  source: string,
): { closeTagEnd: number; encodedFilename: string } | null {
  if (!source.startsWith(SCRIPTLESS_WORKAROUND_OPEN_TAG_PREFIX)) {
    return null;
  }

  const [firstBlock] = extractSfcBlocks(source);
  if (!firstBlock || firstBlock.kind !== "script-setup") {
    return null;
  }
  if (!isWhitespaceOnly(firstBlock.content)) {
    return null;
  }

  const contentStart = offsetFromLineColumn(source, firstBlock.contentStart);
  const contentEnd = offsetFromLineColumn(source, firstBlock.contentEnd);
  const openTag = source.slice(0, contentStart);
  const encodedFilename = encodedFilenameFromWorkaroundOpenTag(openTag);
  if (encodedFilename == null) {
    return null;
  }

  return {
    closeTagEnd: contentEnd + SCRIPTLESS_WORKAROUND_CLOSE_TAG.length,
    encodedFilename,
  };
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

function createWorkaroundScript(source: string, filename: string): string {
  return `${SCRIPTLESS_WORKAROUND_OPEN_TAG_PREFIX}${encodeWorkaroundFilename(filename)}">${createWhitespaceMirror(source)}</script>\n`;
}

function createWhitespaceMirror(source: string): string {
  return source.replaceAll(/[^\r\n]/gu, " ");
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
