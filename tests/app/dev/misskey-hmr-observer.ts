import { isVizeClientModuleUrl } from "./vize-client-module.ts";

/** Match the accepted SFC, not its style module or a `via` importer. */
export function isMisskeyAuthoredHotUpdate(
  message: string,
  moduleSuffix: string,
  origin: string,
): boolean {
  const match = /^\[vite\] hot updated: (\S+)(?: via \S+)?$/.exec(message);
  if (!match) return false;
  try {
    const acceptedUrl = new URL(match[1], origin);
    return isVizeClientModuleUrl(acceptedUrl.href, moduleSuffix, origin);
  } catch {
    return false;
  }
}
