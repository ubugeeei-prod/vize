import { createHash } from "node:crypto";
import path from "node:path";

interface Dependency {
  specifier: string;
  resolveFrom?: string | null;
  meta: Record<string, unknown>;
}

/** Immutable metadata crosses worker boundaries through Parcel's own graph. */
export function resolveVirtualModule(
  dependency: Dependency,
): { filePath: string; code: string; sideEffects: true } | undefined {
  const entry = dependency.meta.vizeVirtualModule;
  if (!entry || typeof entry !== "object" || Array.isArray(entry)) return;
  const { filePath, code, digest } = entry as Record<string, unknown>;
  if (
    typeof code !== "string" ||
    typeof filePath !== "string" ||
    typeof digest !== "string" ||
    !/^[a-f0-9]{64}$/.test(digest) ||
    dependency.specifier !== `vize-generated-${digest}.js` ||
    typeof dependency.resolveFrom !== "string" ||
    !path.isAbsolute(dependency.resolveFrom) ||
    filePath !== `${dependency.resolveFrom}.vize-${digest}.js` ||
    createHash("sha256").update(code).digest("hex") !== digest
  ) {
    throw new Error("Invalid immutable Vize component module metadata.");
  }
  return { filePath, code, sideEffects: true };
}
