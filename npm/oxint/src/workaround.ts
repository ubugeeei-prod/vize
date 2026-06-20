import { extractSfcBlocks } from "./sfc-blocks.js";

const SCRIPTLESS_WORKAROUND_MARKER = "oxlint-plugin-vize-scriptless";
const SCRIPTLESS_WORKAROUND_OPEN_TAG_PREFIX = `<script setup lang="ts" data-${SCRIPTLESS_WORKAROUND_MARKER}="`;
const SCRIPTLESS_WORKAROUND_CLOSE_TAG = "</script>";

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
  if (!source.startsWith(SCRIPTLESS_WORKAROUND_OPEN_TAG_PREFIX)) {
    return {
      filename: fallbackFilename,
      source,
      usesOriginalLocations: false,
    };
  }

  const encodedFilenameStart = SCRIPTLESS_WORKAROUND_OPEN_TAG_PREFIX.length;
  const encodedFilenameEnd = source.indexOf(`">`, encodedFilenameStart);
  if (encodedFilenameEnd === -1) {
    return {
      filename: fallbackFilename,
      source,
      usesOriginalLocations: false,
    };
  }

  const closeTagStart = source.indexOf(SCRIPTLESS_WORKAROUND_CLOSE_TAG, encodedFilenameEnd + 2);
  if (closeTagStart === -1) {
    return {
      filename: fallbackFilename,
      source,
      usesOriginalLocations: false,
    };
  }

  let strippedSourceStart = closeTagStart + SCRIPTLESS_WORKAROUND_CLOSE_TAG.length;
  if (source.charCodeAt(strippedSourceStart) === 13) {
    strippedSourceStart += 1;
  }
  if (source.charCodeAt(strippedSourceStart) === 10) {
    strippedSourceStart += 1;
  }

  const decodedFilename =
    decodeWorkaroundFilename(source.slice(encodedFilenameStart, encodedFilenameEnd)) ??
    fallbackFilename;
  const strippedSource = source.slice(strippedSourceStart);

  return {
    filename: decodedFilename,
    source: strippedSource,
    usesOriginalLocations: true,
  };
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
  });
}
