import path from "node:path";
import fs from "node:fs";
import { createRequire } from "node:module";
import {
  classifyVitePluginRequest,
  createViteBareImportBases,
  createViteBareImportCandidates,
  isViteBareSpecifier,
  normalizeViteRequireBase,
  normalizeViteResolvedVuePath,
  resolveViteAliasRequest,
  resolveViteRelativeImport,
  resolveViteVuePath,
  splitViteIdQuery,
} from "@vizejs/native";

import type { VizePluginState } from "./state.ts";
import {
  LEGACY_VIZE_PREFIX,
  VIRTUAL_CSS_MODULE,
  RESOLVED_CSS_MODULE,
  toVirtualId,
} from "../virtual.ts";
import { toNativeCssAliasRule } from "../utils/css.ts";

export function resolveVuePath(state: VizePluginState, id: string, importer?: string): string {
  return resolveViteVuePath(state.root, id, importer);
}

const EMPTY_NATIVE_ALIAS_RULES: ReturnType<typeof toNativeCssAliasRule>[] = [];

interface ResolveContext {
  resolve(
    id: string,
    importer?: string,
    options?: { skipSelf: boolean },
  ): Promise<{ id: string; external?: boolean } | null>;
}

function resolveAliasRequest(
  state: Pick<VizePluginState, "cssAliasRules">,
  id: string,
): string | null {
  return resolveViteAliasRequest(id, nativeCssAliasRules(state));
}

function resolveBareImportWithNode(
  state: Pick<VizePluginState, "root">,
  id: string,
  importer?: string,
): string | null {
  const { request, querySuffix } = splitViteIdQuery(id);
  for (const candidate of createViteBareImportBases(state.root, importer)) {
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

function resolveBareImportCandidatesWithNode(
  state: Pick<VizePluginState, "root" | "cssAliasRules">,
  id: string,
  importer?: string,
  resolvedId?: string,
): string | null {
  for (const candidate of createViteBareImportCandidates(
    id,
    nativeCssAliasRules(state),
    resolvedId,
  )) {
    const resolved = resolveBareImportWithNode(state, candidate, importer);
    if (resolved) {
      return resolved;
    }
  }

  return null;
}

function findPackageRoot(resolvedFile: string): string | null {
  let current = path.dirname(resolvedFile);
  while (current !== path.dirname(current)) {
    if (fs.existsSync(path.join(current, "package.json"))) {
      return current;
    }
    current = path.dirname(current);
  }
  return null;
}

function resolveVueBundlerEntryWithNode(
  state: Pick<VizePluginState, "root">,
  id: string,
  importer?: string,
): string | null {
  const { request, querySuffix } = splitViteIdQuery(id);
  if (request !== "vue") {
    return null;
  }

  const packageJson = resolveBareImportWithNode(state, "vue/package.json", importer);
  const resolvedVue = packageJson ? null : resolveBareImportWithNode(state, "vue", importer);
  const packageRoot = packageJson
    ? path.dirname(packageJson)
    : resolvedVue
      ? findPackageRoot(resolvedVue)
      : null;
  if (!packageRoot) {
    return null;
  }

  for (const relativeEntry of [
    "dist/vue.runtime.esm-bundler.js",
    "dist/vue.esm-bundler.js",
    "index.mjs",
  ]) {
    const entry = path.join(packageRoot, relativeEntry);
    if (fs.existsSync(entry)) {
      return `${entry}${querySuffix}`;
    }
  }

  return null;
}

function isVueRuntimeRequest(id: string): boolean {
  return splitViteIdQuery(id).request === "vue";
}

function resolveSsrExternalVueRequest(id: string): string | null {
  const { request, querySuffix } = splitViteIdQuery(id);
  if (querySuffix) {
    return null;
  }

  if (request === "@vue/server-renderer" || request === "vue/server-renderer") {
    return "vue/server-renderer";
  }

  if (request === "vue" || request.startsWith("vue/dist/")) {
    return "vue";
  }

  if (request.startsWith("@vue/")) {
    return request;
  }

  return null;
}

function isVuePackageEntry(id: string): boolean {
  const { request } = splitViteIdQuery(id);
  const normalized = request.split(path.sep).join("/");
  return (
    normalized.endsWith("/node_modules/vue/index.js") ||
    normalized.endsWith("/node_modules/vue/dist/vue.runtime.esm-bundler.js") ||
    normalized.endsWith("/node_modules/vue/dist/vue.esm-bundler.js") ||
    normalized.includes("/node_modules/.pnpm/vue@") ||
    normalized.includes("/node_modules/.pnpm/@vue+")
  );
}

function isOptimizedVueDependency(id: string): boolean {
  const { request } = splitViteIdQuery(id);
  const normalized = request.split(path.sep).join("/");
  return normalized.includes("/node_modules/.vite/deps/vue.");
}

function normalizeResolvedVuePath(id: string): string | null {
  return normalizeViteResolvedVuePath(id);
}

function nativeCssAliasRules(
  state: Pick<VizePluginState, "cssAliasRules">,
): ReturnType<typeof toNativeCssAliasRule>[] {
  return state.cssAliasRules.length === 0
    ? EMPTY_NATIVE_ALIAS_RULES
    : state.cssAliasRules.map(toNativeCssAliasRule);
}

function isPotentialVizeResolveId(id: string): boolean {
  return (
    id.startsWith("\0") ||
    id.startsWith("vize:") ||
    id.startsWith("/@fs") ||
    id === VIRTUAL_CSS_MODULE ||
    id.endsWith(".vue") ||
    id.includes(".vue?") ||
    id.includes("?macro=true") ||
    id.includes("?definePage")
  );
}

function isPotentialVizeImporter(importer: string | undefined): boolean {
  return importer !== undefined && (importer.startsWith("\0") || importer.startsWith("vize:"));
}

function shouldCompileVueSfcRequest(
  request: ReturnType<typeof classifyVitePluginRequest>,
): boolean {
  if (
    !request.isVueSfcPath ||
    request.isVueStyleQuery ||
    request.hasMacroQuery ||
    request.hasDefinePageQuery
  ) {
    return false;
  }

  if (!request.querySuffix) {
    return true;
  }

  const params = new URLSearchParams(request.querySuffix.slice(1));
  if (
    params.has("raw") ||
    params.has("url") ||
    params.has("worker") ||
    params.has("sharedworker")
  ) {
    return false;
  }

  return params.has("nuxt_component");
}

function hasNuxtComponentQuery(request: ReturnType<typeof classifyVitePluginRequest>): boolean {
  if (!request.querySuffix) {
    return false;
  }

  return new URLSearchParams(request.querySuffix.slice(1)).has("nuxt_component");
}

async function resolveAliasedVueImport(
  ctx: ResolveContext,
  state: VizePluginState,
  id: string,
  importer: string | undefined,
  isSsrRequest: boolean,
  handleNodeModules: boolean,
  querySuffix: string,
  preserveQueryAsPath: boolean,
): Promise<string | null> {
  if (path.isAbsolute(id)) {
    return null;
  }

  const viteImporter = normalizeViteRequireBase(importer) ?? importer;
  const viteResolved = await ctx.resolve(id, viteImporter, { skipSelf: true });
  const realPath = viteResolved ? normalizeResolvedVuePath(viteResolved.id) : null;
  if (!realPath) {
    return null;
  }

  const isResolvedNodeModules = realPath.includes("node_modules");
  if (!handleNodeModules && isResolvedNodeModules) {
    state.logger.log(`resolveId: skipping resolved node_modules path ${realPath}`);
    return null;
  }

  if (!isResolvedNodeModules && state.filter && !state.filter(realPath)) {
    state.logger.log(`resolveId: skipping filtered resolved path ${realPath}`);
    return null;
  }

  if (state.cache.has(realPath) || fs.existsSync(realPath)) {
    state.logger.log(`resolveId: resolved via Vite fallback ${id} to ${realPath}`);
    return preserveQueryAsPath
      ? `${realPath}${querySuffix}`
      : `${toVirtualId(realPath, isSsrRequest)}${querySuffix}`;
  }

  return null;
}

export async function resolveIdHook(
  ctx: ResolveContext,
  state: VizePluginState,
  id: string,
  importer?: string,
  options?: { ssr?: boolean },
): Promise<string | { id: string; external?: boolean } | null | undefined> {
  if (!isPotentialVizeResolveId(id) && !isPotentialVizeImporter(importer)) {
    return null;
  }

  const isBuild = state.server === null;
  const importerRequest = importer ? classifyVitePluginRequest(importer) : null;
  const isSsrRequest = !!options?.ssr || (importerRequest?.isVizeSsrVirtual ?? false);
  const request = classifyVitePluginRequest(id);

  // Skip all virtual module IDs
  if (id.startsWith("\0")) {
    // This is one of our .vue.ts virtual modules. Return the ID so Rolldown/Rollup
    // treats imports of Vize virtual modules from other virtual modules as resolved.
    if (request.isVizeVirtual) {
      if (isSsrRequest && !request.isVizeSsrVirtual && request.vizeVirtualPath) {
        return `${toVirtualId(request.vizeVirtualPath, true)}${request.querySuffix}`;
      }
      return id;
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
      const { request: pathPart, querySuffix } = splitViteIdQuery(cleanPath);
      state.logger.log(
        `resolveId: redirecting \0-prefixed non-vue ID to ${pathPart}${querySuffix}`,
      );
      const redirected = pathPart + querySuffix;
      return isBuild
        ? (classifyVitePluginRequest(redirected).normalizedFsId ?? redirected)
        : redirected;
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
    const normalizedFsId = resolved ? classifyVitePluginRequest(resolved.id).normalizedFsId : null;
    if (resolved && isBuild && normalizedFsId) {
      return { ...resolved, id: normalizedFsId };
    }
    return resolved;
  }

  // Handle virtual CSS module for production extraction
  if (id === VIRTUAL_CSS_MODULE) {
    return RESOLVED_CSS_MODULE;
  }

  if (isBuild && request.normalizedFsId) {
    return request.normalizedFsId;
  }

  // Handle route macro queries.
  // - ?macro=true is used by Nuxt page macros.
  // - ?definePage is used by Vue Router file-based routing.
  // Nuxt's router generates `import { default } from "page.vue?macro=true"` to extract
  // route metadata. Without @vitejs/plugin-vue, Vize must resolve this query so the
  // load hook can return compile-time macro artifact modules.
  if ((request.hasMacroQuery || request.hasDefinePageQuery) && request.isVueSfcPath) {
    const resolved = resolveVuePath(state, request.path, importer);
    if (resolved && fs.existsSync(resolved)) {
      return `\0${resolved}${request.querySuffix}`;
    }
  }

  // Handle virtual style imports:
  //   Component.vue?vue&type=style&index=0&lang=scss
  //   Component.vue?vue&type=style&index=0&lang=scss&module
  if (request.isVueStyleQuery && request.styleVirtualSuffix) {
    if (id.includes("vitepress-plugin-llms")) {
      state.logger.log(`resolveId: skipping vitepress-plugin-llms style import ${id}`);
      return null;
    }
    const handleNodeModules = state.mergedOptions.handleNodeModulesVue ?? true;
    if (!handleNodeModules && request.path.includes("node_modules")) {
      state.logger.log(`resolveId: skipping node_modules style import ${id}`);
      return null;
    }
    return `\0${id}${request.styleVirtualSuffix}`;
  }

  // If importer is a vize virtual module or macro module, resolve imports against the real path
  const isMacroImporter = importerRequest?.isMacroVirtualId ?? false;
  const isVizeVirtualImporter = importerRequest?.isVizeVirtual ?? false;
  if (importer && (isVizeVirtualImporter || isMacroImporter)) {
    const cleanImporter = isMacroImporter
      ? (importerRequest?.strippedVirtualPath ?? "")
      : (importerRequest?.vizeVirtualPath ?? "");

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
      const ssrExternalVueRequest = isSsrRequest ? resolveSsrExternalVueRequest(id) : null;
      if (ssrExternalVueRequest) {
        return { id: ssrExternalVueRequest, external: true };
      }

      // For bare module specifiers (not relative, not absolute),
      // resolve them from the real importer path so that Vite can find
      // packages in the correct node_modules directory.
      if (!id.startsWith("./") && !id.startsWith("../") && !id.startsWith("/")) {
        const isVueRuntime = isVueRuntimeRequest(id);
        const aliasRequest = resolveAliasRequest(state, id);
        if (!isVueRuntime && aliasRequest && isViteBareSpecifier(aliasRequest)) {
          const nodeResolved = resolveBareImportCandidatesWithNode(state, id, cleanImporter);
          if (nodeResolved) {
            state.logger.log(
              `resolveId: resolved aliased bare ${id} to ${nodeResolved} via Node fallback`,
            );
            return nodeResolved;
          }
        }

        try {
          const resolved = await ctx.resolve(id, cleanImporter, { skipSelf: true });
          if (resolved) {
            state.logger.log(`resolveId: resolved bare ${id} to ${resolved.id} via Vite resolver`);
            const normalizedFsId = classifyVitePluginRequest(resolved.id).normalizedFsId;
            if (isBuild && normalizedFsId) {
              return { ...resolved, id: normalizedFsId };
            }

            if (isVueRuntime && state.server !== null && !isOptimizedVueDependency(resolved.id)) {
              state.logger.log(`resolveId: deferring Vue runtime ${resolved.id} to Vite optimizer`);
              return null;
            }

            if (isVueRuntime && isVuePackageEntry(resolved.id)) {
              const vueBundlerEntry = resolveVueBundlerEntryWithNode(state, id, cleanImporter);
              if (vueBundlerEntry) {
                state.logger.log(`resolveId: resolved Vue runtime to ${vueBundlerEntry}`);
                return vueBundlerEntry;
              }
              return null;
            }

            if (isViteBareSpecifier(resolved.id)) {
              if (isVueRuntime) {
                const vueBundlerEntry = isBuild
                  ? resolveVueBundlerEntryWithNode(state, id, cleanImporter)
                  : null;
                if (vueBundlerEntry) {
                  state.logger.log(`resolveId: resolved Vue runtime to ${vueBundlerEntry}`);
                  return vueBundlerEntry;
                }
                state.logger.log(`resolveId: deferring bare Vue runtime ${id} to Vite`);
                return null;
              }

              const nodeResolved = resolveBareImportCandidatesWithNode(
                state,
                id,
                cleanImporter,
                resolved.id,
              );
              if (nodeResolved) {
                state.logger.log(
                  `resolveId: normalized bare ${id} to ${nodeResolved} via Node fallback`,
                );
                return nodeResolved;
              }
              return null;
            }
            return resolved;
          }
        } catch {
          // Fall through
        }

        if (isVueRuntime) {
          const vueBundlerEntry = isBuild
            ? resolveVueBundlerEntryWithNode(state, id, cleanImporter)
            : null;
          if (vueBundlerEntry) {
            state.logger.log(`resolveId: resolved Vue runtime to ${vueBundlerEntry}`);
            return vueBundlerEntry;
          }
          state.logger.log(`resolveId: deferring Vue runtime ${id} to Vite`);
          return null;
        }

        const nodeResolved = resolveBareImportCandidatesWithNode(state, id, cleanImporter);
        if (nodeResolved) {
          state.logger.log(`resolveId: resolved bare ${id} to ${nodeResolved} via Node fallback`);
          return nodeResolved;
        }

        if (aliasRequest && aliasRequest !== id && !isViteBareSpecifier(aliasRequest)) {
          try {
            const resolved = await ctx.resolve(aliasRequest, cleanImporter, { skipSelf: true });
            if (resolved) {
              state.logger.log(
                `resolveId: resolved aliased bare ${id} to ${resolved.id} via Vite resolver`,
              );
              const normalizedFsId = classifyVitePluginRequest(resolved.id).normalizedFsId;
              if (isBuild && normalizedFsId) {
                return { ...resolved, id: normalizedFsId };
              }

              if (isViteBareSpecifier(resolved.id)) {
                const nodeResolved = resolveBareImportCandidatesWithNode(
                  state,
                  id,
                  cleanImporter,
                  resolved.id,
                );
                if (nodeResolved) {
                  state.logger.log(
                    `resolveId: normalized aliased bare ${id} to ${nodeResolved} via Node fallback`,
                  );
                  return nodeResolved;
                }
                return null;
              }
              return resolved;
            }
          } catch {
            // Fall through
          }

          const nodeResolved = resolveBareImportCandidatesWithNode(
            state,
            aliasRequest,
            cleanImporter,
          );
          if (nodeResolved) {
            state.logger.log(
              `resolveId: resolved aliased bare ${id} to ${nodeResolved} via Node fallback`,
            );
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
          const normalizedFsId = classifyVitePluginRequest(resolved.id).normalizedFsId;
          if (isBuild && normalizedFsId) {
            return { ...resolved, id: normalizedFsId };
          }
          return resolved;
        }
      } catch {
        // Fall through to manual resolution
      }

      // Fallback: manual resolution for relative imports
      if (id.startsWith("./") || id.startsWith("../")) {
        const resolved = resolveViteRelativeImport(id, cleanImporter);
        if (resolved) {
          state.logger.log(`resolveId: resolved relative ${id} to ${resolved}`);
          return resolved;
        }
      }

      return null;
    }
  }

  // Handle Vue SFC component imports, including Nuxt component-loader queries.
  if (shouldCompileVueSfcRequest(request)) {
    const handleNodeModules = state.mergedOptions.handleNodeModulesVue ?? true;
    const preserveQueryAsPath = hasNuxtComponentQuery(request);

    const vueRequestPath = request.path;

    if (!handleNodeModules && vueRequestPath.includes("node_modules")) {
      state.logger.log(`resolveId: skipping node_modules import ${id}`);
      return null;
    }

    const resolved = resolveVuePath(state, vueRequestPath, importer);
    const fileExists = fs.existsSync(resolved);
    if (!fileExists) {
      const aliased = await resolveAliasedVueImport(
        ctx,
        state,
        id,
        importer,
        isSsrRequest,
        handleNodeModules,
        request.querySuffix,
        preserveQueryAsPath,
      );
      if (aliased) {
        return aliased;
      }
    }

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
    state.logger.log(
      `resolveId: id=${id}, resolved=${resolved}, hasCache=${hasCache}, fileExists=${fileExists}, importer=${importer ?? "none"}`,
    );

    // Return virtual module ID: \0/path/to/Component.vue.ts
    if (hasCache || fileExists) {
      if (preserveQueryAsPath) {
        return `${resolved}${request.querySuffix}`;
      }
      return `${toVirtualId(resolved, isSsrRequest)}${request.querySuffix}`;
    }
  }

  return null;
}
