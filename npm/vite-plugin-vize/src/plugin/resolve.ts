import path from "node:path";
import fs from "node:fs";
import { createRequire } from "node:module";

import type { VizePluginState } from "./state.ts";
import {
  LEGACY_VIZE_PREFIX,
  VIRTUAL_CSS_MODULE,
  RESOLVED_CSS_MODULE,
  isVizeVirtual,
  isVizeSsrVirtual,
  toVirtualId,
  fromVirtualId,
  normalizeFsIdForBuild,
} from "../virtual.ts";

export function resolveVuePath(state: VizePluginState, id: string, importer?: string): string {
  let resolved: string;
  // Handle Vite's /@fs/ prefix for absolute filesystem paths
  if (id.startsWith("/@fs/")) {
    resolved = id.slice(4); // Remove '/@fs' prefix, keep the absolute path
  } else if (id.startsWith("/") && !fs.existsSync(id)) {
    // Check if it's a web-root relative path (starts with / but not a real absolute path)
    // These are relative to the project root, not the filesystem root
    // Remove leading slash and resolve relative to root
    resolved = path.resolve(state.root, id.slice(1));
  } else if (path.isAbsolute(id)) {
    resolved = id;
  } else if (importer) {
    // If importer is a virtual module, extract the real path
    const realImporter = isVizeVirtual(importer) ? fromVirtualId(importer) : importer;
    resolved = path.resolve(path.dirname(realImporter), id);
  } else {
    // Relative path without importer - resolve from root
    resolved = path.resolve(state.root, id);
  }
  // Ensure we always return an absolute path
  if (!path.isAbsolute(resolved)) {
    resolved = path.resolve(state.root, resolved);
  }
  return path.normalize(resolved);
}

interface ResolveContext {
  resolve(
    id: string,
    importer?: string,
    options?: { skipSelf: boolean },
  ): Promise<{ id: string } | null>;
}

function hasQueryParam(id: string, name: string): boolean {
  const query = id.split("?")[1];
  return query ? new URLSearchParams(query).has(name) : false;
}

function hasMacroQuery(id: string): boolean {
  const query = id.split("?")[1];
  return query ? new URLSearchParams(query).get("macro") === "true" : false;
}

function isMacroVirtualId(id: string): boolean {
  return id.startsWith("\0") && (hasMacroQuery(id) || hasQueryParam(id, "definePage"));
}

function normalizeRequireBase(importer?: string): string | null {
  if (!importer) {
    return null;
  }

  let normalized = importer;
  if (isVizeVirtual(normalized)) {
    normalized = fromVirtualId(normalized);
  } else if (isMacroVirtualId(normalized)) {
    normalized = normalized.slice(1).split("?")[0] ?? "";
  }

  return normalized.split("?")[0] ?? null;
}

function resolveBareImportWithNode(
  state: Pick<VizePluginState, "root">,
  id: string,
  importer?: string,
): string | null {
  const [request, queryPart] = id.split("?");
  const querySuffix = queryPart ? `?${queryPart}` : "";
  const candidates = [normalizeRequireBase(importer), path.join(state.root, "package.json")].filter(
    (candidate): candidate is string => candidate != null,
  );

  const seen = new Set<string>();
  for (const candidate of candidates) {
    if (seen.has(candidate)) {
      continue;
    }
    seen.add(candidate);

    try {
      const requireFromBase = createRequire(candidate);
      const resolved = requireFromBase.resolve(request);
      return `${resolved}${querySuffix}`;
    } catch {
      // Continue to the next base candidate.
    }
  }

  return null;
}

export async function resolveIdHook(
  ctx: ResolveContext,
  state: VizePluginState,
  id: string,
  importer?: string,
  options?: { ssr?: boolean },
): Promise<string | { id: string } | null | undefined> {
  const isBuild = state.server === null;
  const isSsrRequest = !!options?.ssr || (importer ? isVizeSsrVirtual(importer) : false);

  // Skip all virtual module IDs
  if (id.startsWith("\0")) {
    // This is one of our .vue.ts virtual modules -- pass through
    if (isVizeVirtual(id)) {
      if (isSsrRequest && !isVizeSsrVirtual(id)) {
        return toVirtualId(fromVirtualId(id), true);
      }
      return null;
    }
    // Legacy: handle old \0vize: prefixed non-vue files
    if (id.startsWith(LEGACY_VIZE_PREFIX)) {
      const rawPath = id.slice(LEGACY_VIZE_PREFIX.length);
      const cleanPath = rawPath.endsWith(".ts") ? rawPath.slice(0, -3) : rawPath;
      if (!cleanPath.endsWith(".vue")) {
        state.logger.log(`resolveId: redirecting legacy virtual ID to ${cleanPath}`);
        return cleanPath;
      }
    }
    // Redirect non-vue files that accidentally got \0 prefix.
    // This happens when Vite's import analysis resolves dynamic imports
    // relative to virtual module paths -- the \0 prefix leaks into the
    // resolved path and appears as __x00__ in browser URLs.
    const cleanPath = id.slice(1); // strip \0
    if (cleanPath.startsWith("/") && !cleanPath.endsWith(".vue.ts")) {
      // Strip query params for existence check
      const [pathPart, queryPart] = cleanPath.split("?");
      const querySuffix = queryPart ? `?${queryPart}` : "";
      state.logger.log(
        `resolveId: redirecting \0-prefixed non-vue ID to ${pathPart}${querySuffix}`,
      );
      const redirected = pathPart + querySuffix;
      return isBuild ? normalizeFsIdForBuild(redirected) : redirected;
    }
    return null;
  }

  // Handle stale vize: prefix (without \0) from cached resolutions
  if (id.startsWith("vize:")) {
    let realPath = id.slice("vize:".length);
    if (realPath.endsWith(".ts")) {
      realPath = realPath.slice(0, -3);
    }
    state.logger.log(`resolveId: redirecting stale vize: ID to ${realPath}`);
    const resolved = await ctx.resolve(realPath, importer, { skipSelf: true });
    if (resolved && isBuild && resolved.id.startsWith("/@fs/")) {
      return { ...resolved, id: normalizeFsIdForBuild(resolved.id) };
    }
    return resolved;
  }

  // Handle virtual CSS module for production extraction
  if (id === VIRTUAL_CSS_MODULE) {
    return RESOLVED_CSS_MODULE;
  }

  if (isBuild && id.startsWith("/@fs/")) {
    return normalizeFsIdForBuild(id);
  }

  // Handle route macro queries.
  // - ?macro=true is used by Nuxt page macros.
  // - ?definePage is used by Vue Router file-based routing.
  // Nuxt's router generates `import { default } from "page.vue?macro=true"` to extract
  // route metadata. Without @vitejs/plugin-vue, Vize must resolve this query so the
  // load hook can return compile-time macro artifact modules.
  if (hasMacroQuery(id) || hasQueryParam(id, "definePage")) {
    const filePath = id.split("?")[0];
    const querySuffix = id.slice(id.indexOf("?"));
    const resolved = resolveVuePath(state, filePath, importer);
    if (resolved && fs.existsSync(resolved)) {
      return `\0${resolved}${querySuffix}`;
    }
  }

  // Handle virtual style imports:
  //   Component.vue?vue&type=style&index=0&lang=scss
  //   Component.vue?vue&type=style&index=0&lang=scss&module
  if (id.includes("?vue&type=style") || id.includes("?vue=&type=style")) {
    const params = new URLSearchParams(id.split("?")[1]);
    const lang = params.get("lang") || "css";
    if (params.has("module")) {
      // For CSS Modules, append .module.{lang} suffix so Vite's CSS pipeline
      // automatically treats it as a CSS module and returns the class mapping.
      return `\0${id}.module.${lang}`;
    }
    // Append .{lang} suffix so Vite's CSS pipeline recognizes the file type
    // and applies the appropriate preprocessor (SCSS, Less, etc.).
    return `\0${id}.${lang}`;
  }

  // If importer is a vize virtual module or macro module, resolve imports against the real path
  const isMacroImporter = importer ? isMacroVirtualId(importer) : false;
  if (importer && (isVizeVirtual(importer) || isMacroImporter)) {
    const cleanImporter = isMacroImporter
      ? importer.slice(1).split("?")[0]!
      : fromVirtualId(importer);

    state.logger.log(`resolveId from virtual: id=${id}, cleanImporter=${cleanImporter}`);

    // Subpath imports (e.g., #imports/entry from Nuxt)
    if (id.startsWith("#")) {
      try {
        return await ctx.resolve(id, cleanImporter, { skipSelf: true });
      } catch {
        return null;
      }
    }

    // For non-vue files, resolve relative to the real importer
    if (!id.endsWith(".vue")) {
      // For bare module specifiers (not relative, not absolute),
      // resolve them from the real importer path so that Vite can find
      // packages in the correct node_modules directory.
      if (!id.startsWith("./") && !id.startsWith("../") && !id.startsWith("/")) {
        const matchesAlias = state.cssAliasRules.some(
          (rule) => id === rule.find || id.startsWith(rule.find + "/"),
        );
        if (!matchesAlias) {
          try {
            const resolved = await ctx.resolve(id, cleanImporter, { skipSelf: true });
            if (resolved) {
              state.logger.log(
                `resolveId: resolved bare ${id} to ${resolved.id} via Vite resolver`,
              );
              if (isBuild && resolved.id.startsWith("/@fs/")) {
                return { ...resolved, id: normalizeFsIdForBuild(resolved.id) };
              }
              return resolved;
            }
          } catch {
            // Fall through
          }

          const nodeResolved = resolveBareImportWithNode(state, id, cleanImporter);
          if (nodeResolved) {
            state.logger.log(`resolveId: resolved bare ${id} to ${nodeResolved} via Node fallback`);
            return nodeResolved;
          }
        }
        return null;
      }

      // Delegate to Vite's full resolver pipeline with the real importer
      try {
        const resolved = await ctx.resolve(id, cleanImporter, { skipSelf: true });
        if (resolved) {
          state.logger.log(`resolveId: resolved ${id} to ${resolved.id} via Vite resolver`);
          if (isBuild && resolved.id.startsWith("/@fs/")) {
            return { ...resolved, id: normalizeFsIdForBuild(resolved.id) };
          }
          return resolved;
        }
      } catch {
        // Fall through to manual resolution
      }

      // Fallback: manual resolution for relative imports
      if (id.startsWith("./") || id.startsWith("../")) {
        const [pathPart, queryPart] = id.split("?");
        const querySuffix = queryPart ? `?${queryPart}` : "";

        const resolved = path.resolve(path.dirname(cleanImporter), pathPart);
        for (const ext of ["", ".ts", ".tsx", ".js", ".jsx", ".json"]) {
          const candidate = resolved + ext;
          if (fs.existsSync(candidate) && fs.statSync(candidate).isFile()) {
            const finalPath = candidate + querySuffix;
            state.logger.log(`resolveId: resolved relative ${id} to ${finalPath}`);
            return finalPath;
          }
        }
        if (fs.existsSync(resolved) && fs.statSync(resolved).isDirectory()) {
          for (const indexFile of ["/index.ts", "/index.tsx", "/index.js", "/index.jsx"]) {
            const candidate = resolved + indexFile;
            if (fs.existsSync(candidate)) {
              const finalPath = candidate + querySuffix;
              state.logger.log(`resolveId: resolved directory ${id} to ${finalPath}`);
              return finalPath;
            }
          }
        }
      }

      return null;
    }
  }

  // Handle .vue file imports
  if (id.endsWith(".vue")) {
    const handleNodeModules = state.initialized
      ? (state.mergedOptions.handleNodeModulesVue ?? true)
      : true;

    if (!handleNodeModules && id.includes("node_modules")) {
      state.logger.log(`resolveId: skipping node_modules import ${id}`);
      return null;
    }

    const resolved = resolveVuePath(state, id, importer);
    const isNodeModulesPath = resolved.includes("node_modules");

    if (!handleNodeModules && isNodeModulesPath) {
      state.logger.log(`resolveId: skipping node_modules path ${resolved}`);
      return null;
    }

    if (state.filter && !isNodeModulesPath && !state.filter(resolved)) {
      state.logger.log(`resolveId: skipping filtered path ${resolved}`);
      return null;
    }

    const hasCache = state.cache.has(resolved);
    const fileExists = fs.existsSync(resolved);
    state.logger.log(
      `resolveId: id=${id}, resolved=${resolved}, hasCache=${hasCache}, fileExists=${fileExists}, importer=${importer ?? "none"}`,
    );

    // Return virtual module ID: \0/path/to/Component.vue.ts
    if (hasCache || fileExists) {
      return toVirtualId(resolved, isSsrRequest);
    }

    // Vite fallback for aliased imports
    if (!fileExists && !path.isAbsolute(id)) {
      const viteResolved = await ctx.resolve(id, importer, { skipSelf: true });
      if (viteResolved && viteResolved.id.endsWith(".vue")) {
        const realPath = viteResolved.id;
        const isResolvedNodeModules = realPath.includes("node_modules");
        if (
          (isResolvedNodeModules ? handleNodeModules : state.filter(realPath)) &&
          (state.cache.has(realPath) || fs.existsSync(realPath))
        ) {
          state.logger.log(`resolveId: resolved via Vite fallback ${id} to ${realPath}`);
          return toVirtualId(realPath, isSsrRequest);
        }
      }
    }
  }

  return null;
}
