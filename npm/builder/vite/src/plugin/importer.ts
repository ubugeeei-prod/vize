import fs from "node:fs";
import path from "node:path";
import { classifyVitePluginRequest, splitViteIdQuery } from "@vizejs/native";

export function isInsidePath(parent: string, child: string): boolean {
  const relative = path.relative(parent, child);
  return (
    relative === "" || (!!relative && !relative.startsWith("..") && !path.isAbsolute(relative))
  );
}

function normalizeNuxtVirtualImporterPath(importer: string): string | null {
  const { request } = splitViteIdQuery(importer);
  for (const prefix of ["/@id/virtual:nuxt:", "virtual:nuxt:"]) {
    if (!request.startsWith(prefix)) {
      continue;
    }

    const encodedPath = request.slice(prefix.length);
    try {
      return decodeURIComponent(encodedPath);
    } catch {
      return encodedPath;
    }
  }

  return null;
}

export function normalizeImporterFilePath(importer: string): string {
  const nuxtVirtualPath = normalizeNuxtVirtualImporterPath(importer);
  if (nuxtVirtualPath) {
    return nuxtVirtualPath;
  }

  const request = classifyVitePluginRequest(importer);
  return (
    request.normalizedFsId ??
    request.strippedVirtualPath ??
    request.vizeVirtualPath ??
    request.normalizedVuePath ??
    splitViteIdQuery(importer).request
  );
}

export function isProjectSourceImporter(root: string, importer?: string): boolean {
  if (!importer) {
    return false;
  }

  const importerPath = normalizeImporterFilePath(importer);
  if (!path.isAbsolute(importerPath)) {
    return false;
  }

  const normalizedImporterPath = importerPath.split(path.sep).join("/");
  if (normalizedImporterPath.includes("/node_modules/")) {
    return false;
  }

  if (isInsidePath(root, importerPath)) {
    return true;
  }

  try {
    return isInsidePath(fs.realpathSync(root), fs.realpathSync(importerPath));
  } catch {
    return false;
  }
}
