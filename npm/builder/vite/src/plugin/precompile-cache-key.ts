/**
 * Identity of a persistent pre-compile cache.
 *
 * A stale hit would hand Vite *wrong* compiled output, which is far worse than
 * being slow, so the cache is split into two independent gates and both fail
 * toward recompilation. This module owns them:
 *
 * - **Manifest identity** (`computePrecompileCacheKey`) covers everything
 *   outside the source text that determines the batch output: the manifest
 *   format version, the resolved `@vizejs/native` version, the immutable token
 *   embedded in the actual native image, and the resolved native batch
 *   compile options (Vue dialect/`vueVersion`, `vapor`, `ssr`, `mode`,
 *   `templateSyntax`, `customRenderer`, `customElements`, runtime module/global
 *   names, and every experimental flag). The key names the manifest file, so a different
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

/**
 * Bump when the persisted entry shape changes; abandons every old manifest.
 *
 * 4: warned sources are excluded so their diagnostics replay on every build.
 *    Older manifests included them without warnings and must be abandoned.
 * 3: entries carry the compiled module's source map (#3399). The container
 * persists it automatically (it stores "everything but `code`/`css`" by rest
 * destructuring), but a format-2 manifest holds mapless entries, and serving
 * those to a build that asked for maps is exactly the silent regression this
 * gate exists to prevent.
 */
export const PRECOMPILE_CACHE_FORMAT = 4;

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

const unresolvedCompilerNonce = crypto.randomUUID();
type BindingIdentity = { kind: "native"; token: string } | { kind: "process"; nonce: string };

/**
 * Identity of the native compiler that produced (or would produce) the output.
 *
 * The successful loader reads the linked image's token once, including pnpm
 * siblings and path/universal fallbacks. OS image reuse or file replacement
 * cannot relabel code already in memory. Unknown providers get a process namespace,
 * so an old wrapper or unknown backend cannot reuse another process's output.
 */
function resolveCompilerIdentity(): unknown {
  try {
    const requireNative = createRequire(import.meta.url);
    const manifestPath = requireNative.resolve("@vizejs/native/package.json");
    const version = (JSON.parse(fs.readFileSync(manifestPath, "utf-8")) as { version?: unknown })
      .version;
    const { getBindingIdentity } = requireNative("@vizejs/native/native-binding-identity.js") as {
      getBindingIdentity: (binding: unknown) => BindingIdentity;
    };
    const identity = getBindingIdentity(requireNative("@vizejs/native"));
    return {
      version: typeof version === "string" ? version : "unknown",
      binding:
        identity.kind === "native" ? { kind: identity.kind, token: identity.token } : identity,
    };
  } catch {
    return { version: "unresolved", binding: { kind: "process", nonce: unresolvedCompilerNonce } };
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
