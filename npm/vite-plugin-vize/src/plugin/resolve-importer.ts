/**
 * Importer path utilities for the Vize resolve hook.
 *
 * Extracted to keep resolve.ts within the source-file-lengths budget while
 * staying colocated with the resolver logic. Consumers import only the two
 * public predicates; the normalisation helpers are private to this module.
 */

import path from "node:path";
import fs from "node:fs";
import { classifyVitePluginRequest, splitViteIdQuery } from "@vizejs/native";

import type { VizePluginState } from "./state.ts";

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

function normalizeImporterFilePath(importer: string): string {
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

export function isProjectLocalImporter(
  state: Pick<VizePluginState, "root">,
  importer?: string,
): boolean {
  if (!importer) {
    return false;
  }

  const importerPath = normalizeImporterFilePath(importer);
  if (!path.isAbsolute(importerPath)) {
    return false;
  }

  if (isInsidePath(state.root, importerPath)) {
    return true;
  }

  try {
    return isInsidePath(fs.realpathSync(state.root), fs.realpathSync(importerPath));
  } catch {
    return false;
  }
}

export function isProjectSourceImporter(
  state: Pick<VizePluginState, "root">,
  importer?: string,
): boolean {
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

  if (isInsidePath(state.root, importerPath)) {
    return true;
  }

  try {
    return isInsidePath(fs.realpathSync(state.root), fs.realpathSync(importerPath));
  } catch {
    return false;
  }
}
