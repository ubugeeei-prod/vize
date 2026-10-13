/**
 * Post-build correctness checks that only a large, minified, sourcemapped build
 * can fail.
 *
 * The reference benchmark's production config turns on minification and
 * sourcemaps but never inspects the result. Doing that here is the point: a
 * plugin can win the timing table and still have emitted a bundle whose
 * sourcemaps point at the wrong line, or whose scoped-style attributes were
 * minified away. Both are invisible in a 3-file smoke test and expensive to
 * find later.
 *
 * Checks:
 *
 * 1. **Sourcemap position.** For a spread of components, find the component's
 *    unique token in the minified JS, walk the emitted sourcemap back to an
 *    original position, and assert the original line actually contains the
 *    token. Off-by-N mappings in a virtual-module pipeline show up here.
 * 2. **Scoped style survival.** Every scoped component must contribute both a
 *    `data-v-*` attribute in the JS and the matching attribute selector in the
 *    CSS. Minifiers rewrite attribute selectors; a mismatch means the built app
 *    renders unstyled.
 */

import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

import { isRecord } from "../script-types.ts";
import { decodeMappings, lookupSegment, positionOf } from "./sourcemap.ts";
import type { MappingSegment } from "./sourcemap.ts";

/** One traced token; every status but `ok` is a failure. */
export type TokenCheck = {
  token: string;
  status: string;
  jsPath?: string;
  source?: string | null;
  generated?: { line: number; column: number };
  segment?: MappingSegment;
  originalLine?: string;
};
export type SourcemapCheck = { checked: number; failures: TokenCheck[] };
export type ScopedStyleCheck = {
  jsScopeIdCount: number;
  cssScopeIdCount: number;
  jsOnly: string[];
  cssOnly: string[];
};

// Source map v3 `sources`: a name per source, or null when it is unknown.
const isSourceList = (value: unknown): value is (string | null)[] =>
  Array.isArray(value) &&
  value.every((source: unknown) => source === null || typeof source === "string");

function distFiles(distDir: string, suffix: string): string[] {
  const files: string[] = [];
  const walk = (dir: string): void => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const full = join(dir, entry.name);
      if (entry.isDirectory()) walk(full);
      else if (entry.name.endsWith(suffix)) files.push(full);
    }
  };
  if (existsSync(distDir)) walk(distDir);
  return files;
}

function checkOneToken(jsPath: string, code: string, token: string): TokenCheck {
  const index = code.indexOf(token);
  if (index === -1) {
    return { token, status: "token-missing-from-js", jsPath };
  }

  const mapPath = `${jsPath}.map`;
  if (!existsSync(mapPath)) {
    return { token, status: "map-missing", jsPath };
  }

  const map: unknown = JSON.parse(readFileSync(mapPath, "utf8"));
  // Decoding or indexing anything else would throw a TypeError further down.
  if (!isRecord(map) || typeof map.mappings !== "string" || !isSourceList(map.sources)) {
    throw new Error(`invalid source map: ${mapPath}`);
  }
  const { line, column } = positionOf(code, index);
  const segment = lookupSegment(decodeMappings(map.mappings), line, column);
  if (segment === null || segment.sourceIndex === null) {
    return { token, status: "position-unmapped", jsPath, generated: { line, column } };
  }

  const source = map.sources[segment.sourceIndex];
  const contents = map.sourcesContent;
  const content: unknown = Array.isArray(contents) ? contents[segment.sourceIndex] : undefined;
  if (typeof content !== "string") {
    return { token, status: "sources-content-missing", jsPath, source };
  }

  const originalLine = content.split("\n")[segment.sourceLine];
  if (originalLine === undefined) {
    return { token, status: "original-line-out-of-range", jsPath, source, segment };
  }
  if (!originalLine.includes(token)) {
    return {
      token,
      status: "original-line-mismatch",
      jsPath,
      source,
      segment,
      originalLine: originalLine.trim().slice(0, 120),
    };
  }

  // Landing on a line that contains the token is necessary but not sufficient:
  // it is also satisfied by mapping into a *generated* intermediate module that
  // happens to contain the same literal. A source map is only useful if it
  // names a file the author can open, so require the mapped source to be the
  // `.vue` file. See ubugeeei/vize#3399.
  // A null source threw here before typing too.
  if (!source!.replace(/\?.*$/, "").endsWith(".vue")) {
    return { token, status: "mapped-to-generated-module", jsPath, source };
  }

  return { token, status: "ok", jsPath, source };
}

/**
 * @param tokens unique per-component tokens (e.g. `node-00000`) to trace
 * @returns `{ checked, failures }` — `failures` is empty when every token
 *   round-tripped through the sourcemap to a line that contains it
 */
export function verifySourcemaps(appDir: string, tool: string, tokens: string[]): SourcemapCheck {
  const distDir = join(appDir, `dist-${tool}`);
  const jsFiles = distFiles(distDir, ".js");
  const sources = jsFiles.map((jsPath) => ({ jsPath, code: readFileSync(jsPath, "utf8") }));

  const failures: TokenCheck[] = [];
  let checked = 0;

  for (const token of tokens) {
    const holder = sources.find(({ code }) => code.includes(token));
    if (!holder) {
      failures.push({ token, status: "token-missing-from-build" });
      continue;
    }
    checked += 1;
    const result = checkOneToken(holder.jsPath, holder.code, token);
    if (result.status !== "ok") {
      failures.push(result);
    }
  }

  return { checked, failures };
}

const SCOPE_ID_PATTERN = /data-v-[0-9a-f]{6,}/g;

/**
 * Every scope id that reaches the JS must also reach the CSS, and vice versa.
 *
 * A scoped SFC compiles to a `__scopeId`/`data-v-*` attribute in the JS and a
 * `[data-v-*]` selector in the CSS. If minification, CSS extraction, or chunk
 * splitting drops one side, the component renders with no styles and nothing in
 * the build output says so.
 */
export function verifyScopedStyles(appDir: string, tool: string): ScopedStyleCheck {
  const distDir = join(appDir, `dist-${tool}`);
  const jsScopeIds = new Set<string>();
  const cssScopeIds = new Set<string>();

  for (const jsPath of distFiles(distDir, ".js")) {
    for (const match of readFileSync(jsPath, "utf8").matchAll(SCOPE_ID_PATTERN)) {
      jsScopeIds.add(match[0]);
    }
  }
  for (const cssPath of distFiles(distDir, ".css")) {
    for (const match of readFileSync(cssPath, "utf8").matchAll(SCOPE_ID_PATTERN)) {
      cssScopeIds.add(match[0]);
    }
  }

  const jsOnly = [...jsScopeIds].filter((id) => !cssScopeIds.has(id));
  const cssOnly = [...cssScopeIds].filter((id) => !jsScopeIds.has(id));

  return {
    jsScopeIdCount: jsScopeIds.size,
    cssScopeIdCount: cssScopeIds.size,
    jsOnly,
    cssOnly,
  };
}

/**
 * Spread of `count` component tokens across a corpus of `componentCount`.
 *
 * Only variant 0 (`index % 4 === 0`, see `tools/benchmarks/scripts/scale/component.ts`) embeds a
 * `node-NNNNN` token, so indices are snapped down to a multiple of 4.
 */
export function sampleTokens(componentCount: number, count = 6): string[] {
  const tokens: string[] = [];
  const step = Math.max(4, Math.floor(componentCount / count));
  for (let index = 0; index < componentCount && tokens.length < count; index += step) {
    const aligned = index - (index % 4);
    const token = `node-${String(aligned).padStart(5, "0")}`;
    if (!tokens.includes(token)) tokens.push(token);
  }
  return tokens;
}
