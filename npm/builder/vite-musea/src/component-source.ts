import path from "node:path";

import { resolveInsideAny } from "./security.js";
import type { ArtFileInfo } from "./types/index.js";

export function allowedSourceRoots(root: string, scanRoots: string[] = []): string[] {
  return [...new Set([root, ...scanRoots].map((sourceRoot) => path.resolve(sourceRoot)))];
}

export function resolveComponentSourcePath(
  art: ArtFileInfo,
  artPath: string,
  sourceRoots: string[],
): string | null {
  const componentPath =
    art.isInline && art.componentPath
      ? art.componentPath
      : art.metadata.component
        ? path.isAbsolute(art.metadata.component)
          ? art.metadata.component
          : path.resolve(path.dirname(artPath), art.metadata.component)
        : null;

  return componentPath ? resolveInsideAny(sourceRoots, componentPath, "component path") : null;
}
