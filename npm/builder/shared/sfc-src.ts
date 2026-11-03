import path from "node:path";

/** The bundler's answers for one SFC's `src` module requests: request -> file. */
export type SfcSrcResolutions = ReadonlyMap<string, string>;

/**
 * Whether a block `src` is a path relative to the SFC. Any other `src`
 * (package, alias, absolute) is a module request that the bundler resolves,
 * as in Vue's SFC spec and `@vitejs/plugin-vue`.
 */
export function isRelativeSfcSrc(src: string): boolean {
  return src.startsWith("./") || src.startsWith("../");
}

/** The file a block `src` names: next to the SFC, or where the bundler resolved it. */
export function sfcSrcFile(
  importer: string,
  tag: string,
  src: string,
  resolutions: SfcSrcResolutions | undefined,
): string {
  if (isRelativeSfcSrc(src)) return path.resolve(path.dirname(importer), src);
  const file = resolutions?.get(src);
  if (file !== undefined) return file;
  throw new Error(
    `[vize] <${tag} src="${src}"> not found: the bundler resolved no file for this module request from ${importer}`,
  );
}

/** Resolve an SFC's `src` module requests with the bundler's own resolver. */
export async function resolveSfcSrcRequests(
  importer: string,
  requests: readonly string[],
  resolve: (request: string, importer: string) => Promise<string | null | undefined>,
): Promise<Map<string, string>> {
  const resolutions = new Map<string, string>();
  for (const request of new Set(requests)) {
    if (isRelativeSfcSrc(request)) continue;
    // Virtual ids and queries cannot be read from disk; they stay unresolved.
    const file = (await resolve(request, importer))?.replace(/[?#].*$/, "");
    if (file && path.isAbsolute(file)) resolutions.set(request, file);
  }
  return resolutions;
}
