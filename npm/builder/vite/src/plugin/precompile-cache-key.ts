/**
 * Identity of a persistent pre-compile cache.
 *
 * A stale hit would hand Vite *wrong* compiled output, which is far worse than
 * being slow, so the cache is split into two independent gates and both fail
 * toward recompilation. This module owns them:
 *
 * - **Manifest identity** (`computePrecompileCacheKey`) covers everything
 *   outside the source text that determines the batch output: the manifest
 *   format version, the resolved `@vizejs/native` version, the size and mtime of
 *   the native binary actually loaded, and the fully resolved native batch
 *   compile options (Vue dialect/`vueVersion`, `vapor`, `ssr`, `mode`,
 *   `templateSyntax`, `customRenderer`, runtime module/global names, and every
 *   experimental flag). The key names the manifest file, so a different
 *   configuration reads a different file and a version bump abandons every old
 *   manifest at once.
 * - **Source identity** (`hashPrecompileSource`) is a SHA-256 of the exact
 *   source text that was compiled. `mtime` is never trusted on its own, so a
 *   same-size edit, a `git checkout` that rewrites content while preserving
 *   `mtime`, and a container with a reset clock all miss.
 */

import crypto from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

/**
 * Bump when the persisted entry shape changes; abandons every old manifest.
 *
 * 3: entries carry the compiled module's source map (#3399). The container
 * persists it automatically (it stores "everything but `code`/`css`" by rest
 * destructuring), but a format-2 manifest holds mapless entries, and serving
 * those to a build that asked for maps is exactly the silent regression this
 * gate exists to prevent.
 */
export const PRECOMPILE_CACHE_FORMAT = 3;

/**
 * SHA-256 of the exact source text handed to the compiler.
 *
 * `base64url` rather than hex: the same 256 bits in 43 characters instead of 64,
 * and the index carries one of these per entry.
 */
export function hashPrecompileSource(source: string): string {
  return crypto.createHash("sha256").update(source, "utf8").digest("base64url");
}

/** Key-independent JSON: object key order must not change the hash. */
function stableStringify(value: unknown): string {
  if (Array.isArray(value)) {
    return `[${value.map(stableStringify).join(",")}]`;
  }
  if (typeof value === "object" && value !== null) {
    const entries = Object.entries(value as Record<string, unknown>)
      .filter(([, item]) => item !== undefined)
      .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
    const body = entries
      .map(([key, item]) => `${JSON.stringify(key)}:${stableStringify(item)}`)
      .join(",");
    return `{${body}}`;
  }
  return JSON.stringify(value) ?? "null";
}

function describeBinary(binary: string): string {
  try {
    const stat = fs.statSync(binary);
    return `${path.basename(binary)}:${stat.size}:${stat.mtimeMs}`;
  } catch {
    return `${path.basename(binary)}:unresolved`;
  }
}

/**
 * Identity of the native compiler that produced (or would produce) the output.
 *
 * The package version covers releases; the binary's size and mtime cover local
 * `pnpm --dir npm/native build:debug` rebuilds, which change codegen without
 * changing any version. Hashing the binary itself would be airtight but costs
 * hundreds of milliseconds for a >30 MB artifact, so a rebuild is treated as a
 * new identity — a miss, never a stale hit.
 */
function resolveCompilerIdentity(): unknown {
  const configured = process.env.NAPI_RS_NATIVE_LIBRARY_PATH;
  try {
    const manifestPath = createRequire(import.meta.url).resolve("@vizejs/native/package.json");
    const packageDir = path.dirname(manifestPath);
    const version = (JSON.parse(fs.readFileSync(manifestPath, "utf-8")) as { version?: unknown })
      .version;
    const binaries = configured
      ? [configured]
      : fs
          .readdirSync(packageDir)
          .filter((name) => name.endsWith(".node"))
          .sort()
          .map((name) => path.join(packageDir, name));
    return {
      version: typeof version === "string" ? version : "unknown",
      binaries: binaries.map(describeBinary),
    };
  } catch {
    // Without a resolvable compiler identity there is nothing safe to key on;
    // the literal below is stable, so such a run simply shares one namespace.
    return { version: "unresolved", binaries: [configured ?? "unresolved"] };
  }
}

/**
 * Hash of everything outside the source text that changes compiled output.
 *
 * `compileOptions` must be the object actually handed to the native batch
 * compiler, so a newly added compile option cannot be forgotten here.
 */
export function computePrecompileCacheKey(compileOptions: unknown): string {
  const material = stableStringify({
    format: PRECOMPILE_CACHE_FORMAT,
    compiler: resolveCompilerIdentity(),
    options: compileOptions,
  });
  return crypto.createHash("sha256").update(material, "utf8").digest("hex").slice(0, 32);
}
