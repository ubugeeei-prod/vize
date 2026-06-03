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
  fromPluginVisibleVirtualId,
  isPluginVisibleSsrVirtualId,
  toPluginVisibleVirtualId,
  toVirtualId,
} from "../virtual.ts";
import { toNativeCssAliasRule } from "../utils/css.ts";

export function resolveVuePath(state: VizePluginState, id: string, importer?: string): string {
  return resolveViteVuePath(state.root, id, importer);
}

const EMPTY_NATIVE_ALIAS_RULES: ReturnType<typeof toNativeCssAliasRule>[] = [];
const VUE_PEER_RUNTIME_PACKAGES = new Set(["vue-router"]);
const VUE_PEER_RUNTIME_ESM_ENTRIES = new Map<string, string[]>([
  ["vue-router", ["dist/vue-router.mjs", "dist/vue-router.js", "index.mjs", "index.js"]],
]);

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

function getBarePackageName(id: string): string | null {
  if (!isViteBareSpecifier(id)) {
    return null;
  }

  const segments = id.split("/");
  if (id.startsWith("@")) {
    return segments.length >= 2 ? `${segments[0]}/${segments[1]}` : null;
  }
  return segments[0] || null;
}

function resolveBareImportFromPnpmHoist(request: string, base: string): string | null {
  const packageName = getBarePackageName(request);
  if (!packageName) {
    return null;
  }

  let current = path.dirname(base);
  while (current !== path.dirname(current)) {
    const directPackage = path.join(current, "node_modules", packageName);
    if (fs.existsSync(directPackage)) {
      return null;
    }

    const hoistRoot = path.join(current, "node_modules", ".pnpm", "node_modules");
    const hoistedPackage = path.join(hoistRoot, packageName);
    if (fs.existsSync(hoistedPackage)) {
      try {
        return createRequire(path.join(hoistRoot, "__vize_probe__.js")).resolve(request);
      } catch {
        // Continue looking from parent directories.
      }
    }
    current = path.dirname(current);
  }

  return null;
}

function resolveBareImportWithNode(
  state: Pick<VizePluginState, "root">,
  id: string,
  importer?: string,
): string | null {
  const { request, querySuffix } = splitViteIdQuery(id);
  for (const candidate of createViteBareImportBases(state.root, importer)) {
    const hoisted = resolveBareImportFromPnpmHoist(request, candidate);
    if (hoisted) {
      return `${hoisted}${querySuffix}`;
    }

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

function resolveBareImportFromPnpmHoistWithNode(
  state: Pick<VizePluginState, "root">,
  id: string,
  importer?: string,
): string | null {
  const { request, querySuffix } = splitViteIdQuery(id);
  for (const candidate of createViteBareImportBases(state.root, importer)) {
    const hoisted = resolveBareImportFromPnpmHoist(request, candidate);
    if (hoisted) {
      return `${hoisted}${querySuffix}`;
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

function resolvePackageRootWithNode(
  state: Pick<VizePluginState, "root">,
  packageName: string,
  importer?: string,
): string | null {
  const packageJson = resolveBareImportWithNode(state, `${packageName}/package.json`, importer);
  if (packageJson) {
    return path.dirname(packageJson);
  }

  const resolvedEntry = resolveBareImportWithNode(state, packageName, importer);
  return resolvedEntry ? findPackageRoot(resolvedEntry) : null;
}

function resolveVueBundlerEntryWithNode(
  state: Pick<VizePluginState, "root">,
  id: string,
  importer?: string,
): string | null {
  const { request, querySuffix } = splitViteIdQuery(id);
  let relativeEntries: string[];
  if (request === "vue") {
    relativeEntries = ["dist/vue.runtime.esm-bundler.js", "dist/vue.esm-bundler.js", "index.mjs"];
  } else if (request.startsWith("vue/dist/") && request.endsWith(".js")) {
    relativeEntries = [request.slice("vue/".length)];
  } else {
    return null;
  }

  const packageRoot = resolvePackageRootWithNode(state, "vue", importer);
  if (!packageRoot) {
    return null;
  }

  for (const relativeEntry of relativeEntries) {
    const entry = path.join(packageRoot, relativeEntry);
    if (fs.existsSync(entry)) {
      return `${entry}${querySuffix}`;
    }
  }

  return null;
}

function resolveVueBundlerEntryFromPnpmHoist(
  state: Pick<VizePluginState, "root">,
  id: string,
  importer?: string,
): string | null {
  const { request, querySuffix } = splitViteIdQuery(id);
  let relativeEntries: string[];
  if (request === "vue") {
    relativeEntries = ["dist/vue.runtime.esm-bundler.js", "dist/vue.esm-bundler.js", "index.mjs"];
  } else if (request.startsWith("vue/dist/") && request.endsWith(".js")) {
    relativeEntries = [request.slice("vue/".length)];
  } else {
    return null;
  }

  const packageJson = resolveBareImportFromPnpmHoistWithNode(state, "vue/package.json", importer);
  if (!packageJson) {
    return null;
  }

  const packageRoot = path.dirname(splitViteIdQuery(packageJson).request);
  for (const relativeEntry of relativeEntries) {
    const entry = path.join(packageRoot, relativeEntry);
    if (fs.existsSync(entry)) {
      return `${entry}${querySuffix}`;
    }
  }

  return null;
}

function isVueRuntimeRequest(id: string): boolean {
  const request = splitViteIdQuery(id).request;
  return request === "vue" || (request.startsWith("vue/dist/") && request.endsWith(".js"));
}

function isVuePeerRuntimeRequest(id: string): boolean {
  const request = splitViteIdQuery(id).request;
  const packageName = getBarePackageName(request);
  return packageName ? VUE_PEER_RUNTIME_PACKAGES.has(packageName) : false;
}

function isProjectVueRuntimeRequest(id: string): boolean {
  return isVueRuntimeRequest(id) || isVuePeerRuntimeRequest(id);
}

function isVueServerRendererRequest(request: string): boolean {
  return (
    request === "@vue/server-renderer" ||
    request === "vue/server-renderer" ||
    (request.startsWith("vue/dist/") && request.endsWith("/server-renderer"))
  );
}

function resolveVueServerRendererPackageRootWithNode(
  state: Pick<VizePluginState, "root">,
  importer?: string,
): string | null {
  const packageRoot = resolvePackageRootWithNode(state, "@vue/server-renderer", importer);
  if (packageRoot) {
    return packageRoot;
  }

  const vuePackageRoot = resolvePackageRootWithNode(state, "vue", importer);
  return vuePackageRoot
    ? resolvePackageRootWithNode(
        state,
        "@vue/server-renderer",
        path.join(vuePackageRoot, "package.json"),
      )
    : null;
}

function resolveVueServerRendererBundlerEntryWithNode(
  state: Pick<VizePluginState, "root">,
  id: string,
  importer?: string,
): string | null {
  const { request, querySuffix } = splitViteIdQuery(id);
  if (querySuffix || !isVueServerRendererRequest(request)) {
    return null;
  }

  const packageRoot = resolveVueServerRendererPackageRootWithNode(state, importer);
  if (!packageRoot) {
    return null;
  }

  for (const relativeEntry of ["dist/server-renderer.esm-bundler.js", "index.mjs", "index.js"]) {
    const entry = path.join(packageRoot, relativeEntry);
    if (fs.existsSync(entry)) {
      return entry;
    }
  }

  return null;
}

function resolveSsrExternalVueRequest(id: string): string | null {
  const { request, querySuffix } = splitViteIdQuery(id);
  if (querySuffix) {
    return null;
  }

  if (isVueServerRendererRequest(request)) {
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

function isInsidePath(parent: string, child: string): boolean {
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

function isProjectLocalImporter(state: Pick<VizePluginState, "root">, importer?: string): boolean {
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

function resolveProjectLocalPnpmVueRuntime(
  state: Pick<VizePluginState, "root">,
  resolvedId: string,
): string | null {
  const normalizedResolvedId = normalizeResolvedVuePath(resolvedId) ?? resolvedId;
  if (!isVuePackageEntry(normalizedResolvedId)) {
    return null;
  }

  const { request, querySuffix } = splitViteIdQuery(normalizedResolvedId);
  const normalizedRequest = request.split(path.sep).join("/");
  if (!normalizedRequest.includes("/node_modules/.pnpm/")) {
    return null;
  }

  let realPath = request;
  try {
    realPath = fs.realpathSync(request);
  } catch {
    // The resolver tests can use synthetic resolved IDs. Keep the normalized
    // path if the file is not present on disk.
  }

  return isInsidePath(state.root, realPath) ? `${realPath}${querySuffix}` : null;
}

function resolveProjectLocalResolvedPath(
  state: Pick<VizePluginState, "root">,
  resolvedId: string,
): string | null {
  const { request, querySuffix } = splitViteIdQuery(resolvedId);
  if (!path.isAbsolute(request)) {
    return null;
  }

  let realPath = request;
  try {
    realPath = fs.realpathSync(request);
  } catch {
    // Synthetic test paths may not exist. Keep the original path for checks.
  }

  return isInsidePath(state.root, realPath) ? `${realPath}${querySuffix}` : null;
}

function resolveVuePeerRuntimeEntryWithNode(
  state: Pick<VizePluginState, "root">,
  id: string,
  importer?: string,
): string | null {
  const { request, querySuffix } = splitViteIdQuery(id);
  const packageName = getBarePackageName(request);
  if (!packageName || !VUE_PEER_RUNTIME_PACKAGES.has(packageName)) {
    return null;
  }

  const packageRoot = resolvePackageRootWithNode(state, packageName, importer);
  if (!packageRoot || !resolveProjectLocalResolvedPath(state, packageRoot)) {
    return null;
  }

  if (request === packageName) {
    for (const relativeEntry of VUE_PEER_RUNTIME_ESM_ENTRIES.get(packageName) ?? ["index.mjs"]) {
      const entry = path.join(packageRoot, relativeEntry);
      if (fs.existsSync(entry)) {
        return `${entry}${querySuffix}`;
      }
    }
  }

  const nodeResolved = resolveBareImportWithNode(state, id, importer);
  return nodeResolved ? resolveProjectLocalResolvedPath(state, nodeResolved) : null;
}

function resolveVuePeerRuntimeEntryFromBaseWithNode(
  state: Pick<VizePluginState, "root">,
  id: string,
  base: string,
): string | null {
  const { request, querySuffix } = splitViteIdQuery(id);
  const packageName = getBarePackageName(request);
  if (!packageName || !VUE_PEER_RUNTIME_PACKAGES.has(packageName)) {
    return null;
  }

  let packageJson: string;
  try {
    packageJson = createRequire(base).resolve(`${packageName}/package.json`);
  } catch {
    return null;
  }

  const packageRoot = path.dirname(packageJson);
  if (!resolveProjectLocalResolvedPath(state, packageRoot)) {
    return null;
  }

  if (request === packageName) {
    for (const relativeEntry of VUE_PEER_RUNTIME_ESM_ENTRIES.get(packageName) ?? ["index.mjs"]) {
      const entry = path.join(packageRoot, relativeEntry);
      if (fs.existsSync(entry)) {
        return `${entry}${querySuffix}`;
      }
    }
  }

  try {
    const resolved = createRequire(base).resolve(request);
    return resolveProjectLocalResolvedPath(state, `${resolved}${querySuffix}`);
  } catch {
    return null;
  }
}

function resolveProjectNuxtVuePeerRuntimeEntryWithNode(
  state: Pick<VizePluginState, "root">,
  id: string,
): string | null {
  const { request } = splitViteIdQuery(id);
  if (getBarePackageName(request) !== "vue-router") {
    return null;
  }

  const nuxtPackageJson = resolveBareImportWithNode(
    state,
    "nuxt/package.json",
    path.join(state.root, "package.json"),
  );
  if (!nuxtPackageJson) {
    return null;
  }

  const nuxtPackageRoot = path.dirname(splitViteIdQuery(nuxtPackageJson).request);
  if (!resolveProjectLocalResolvedPath(state, nuxtPackageRoot)) {
    return null;
  }

  return resolveVuePeerRuntimeEntryFromBaseWithNode(
    state,
    id,
    path.join(nuxtPackageRoot, "package.json"),
  );
}

function isOptimizedVueDependency(id: string): boolean {
  const { request } = splitViteIdQuery(id);
  const normalized = request.split(path.sep).join("/");
  return normalized.includes("/node_modules/.vite/deps/vue.");
}

// Cache per project root: does `vue` resolve from that root via Node?
//
// When vize defers Vue runtime in dev (returns null), Vite re-runs resolveId
// with the \0-prefixed virtual module ID as importer. With pnpm-isolated
// installs the project root has no hoisted `node_modules/vue`, so that
// secondary lookup fails with `Failed to resolve import "vue"`. The deferral
// only makes sense when the project root or one of its parent directories can
// serve as a fallback base for vite:resolve.
const vueResolvableFromRootCache = new Map<string, boolean>();
function isVueResolvableFromRoot(root: string): boolean {
  let cached = vueResolvableFromRootCache.get(root);
  if (cached === undefined) {
    const rootNodeModules = path.join(root, "node_modules");
    const directPackageJson = path.join(rootNodeModules, "vue", "package.json");
    cached = fs.existsSync(directPackageJson);
    if (!cached) {
      try {
        createRequire(path.join(root, "__vize_probe__.js")).resolve("vue/package.json");
        cached = true;
      } catch {
        // Not resolvable from root.
      }
    }
    vueResolvableFromRootCache.set(root, cached);
  }
  return cached;
}

function normalizeResolvedVuePath(id: string): string | null {
  return normalizeViteResolvedVuePath(id);
}

async function resolveProjectVueRuntime(
  ctx: ResolveContext,
  state: VizePluginState,
  id: string,
  importer: string | undefined,
  isSsrRequest: boolean,
): Promise<string | null> {
  if (isSsrRequest || !isProjectVueRuntimeRequest(id) || !isProjectLocalImporter(state, importer)) {
    return null;
  }

  const viteImporter = normalizeViteRequireBase(importer) ?? importer;
  if (isVuePeerRuntimeRequest(id)) {
    const nuxtPeerEntry = resolveProjectNuxtVuePeerRuntimeEntryWithNode(state, id);
    if (nuxtPeerEntry) {
      state.logger.log(`resolveId: resolved Nuxt Vue peer runtime ${id} to ${nuxtPeerEntry}`);
      return nuxtPeerEntry;
    }

    const projectLocalEntry = resolveVuePeerRuntimeEntryWithNode(state, id, viteImporter);
    if (projectLocalEntry) {
      state.logger.log(
        `resolveId: resolved project-local Vue peer runtime ${id} to ${projectLocalEntry}`,
      );
      return projectLocalEntry;
    }
    return null;
  }

  const pnpmHoistedEntry = resolveVueBundlerEntryFromPnpmHoist(state, id, viteImporter);
  if (pnpmHoistedEntry) {
    state.logger.log(`resolveId: resolved project pnpm-hoisted Vue runtime to ${pnpmHoistedEntry}`);
    return pnpmHoistedEntry;
  }

  try {
    const resolved = await ctx.resolve(id, viteImporter, { skipSelf: true });
    if (resolved && !isOptimizedVueDependency(resolved.id)) {
      const projectLocalEntry = resolveProjectLocalPnpmVueRuntime(state, resolved.id);
      if (projectLocalEntry) {
        state.logger.log(`resolveId: resolved project-local Vue runtime to ${projectLocalEntry}`);
        return projectLocalEntry;
      }
    }
  } catch {
    // Fall back to Node resolution below.
  }

  const importerLocalEntry = resolveVueBundlerEntryWithNode(state, id, viteImporter);
  const projectLocalEntry = importerLocalEntry
    ? resolveProjectLocalPnpmVueRuntime(state, importerLocalEntry)
    : null;
  if (projectLocalEntry) {
    state.logger.log(`resolveId: resolved importer-local Vue runtime to ${projectLocalEntry}`);
    return projectLocalEntry;
  }

  return null;
}

function nativeCssAliasRules(
  state: Pick<VizePluginState, "cssAliasRules">,
): ReturnType<typeof toNativeCssAliasRule>[] {
  return state.cssAliasRules.length === 0
    ? EMPTY_NATIVE_ALIAS_RULES
    : state.cssAliasRules.map(toNativeCssAliasRule);
}

function isPotentialVizeResolveId(id: string): boolean {
  // `resolveId` is called for every dependency in a Vite graph. Most bare
  // package imports cannot be Vize-owned, so this cheap string gate keeps regular
  // dependencies off the heavier classifier/alias/Node-resolution path.
  return (
    id.startsWith("\0") ||
    id.startsWith("vize:") ||
    id.startsWith("/@fs") ||
    isProjectVueRuntimeRequest(id) ||
    id === VIRTUAL_CSS_MODULE ||
    id.endsWith(".vue") ||
    id.includes(".vue?") ||
    id.includes(".vue.ts?") ||
    id.includes("?macro=true") ||
    id.includes("?definePage")
  );
}

function isPotentialVizeImporter(importer: string | undefined): boolean {
  // Imports from Vize virtual modules still need custom resolution even when the
  // requested id itself is a regular-looking relative or bare specifier.
  if (importer === undefined) {
    return false;
  }
  if (importer.startsWith("\0") || importer.startsWith("vize:")) {
    return true;
  }

  const request = classifyVitePluginRequest(importer);
  return request.isVueSfcPath;
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

function cleanVueSfcImporter(
  importer: string,
  request: ReturnType<typeof classifyVitePluginRequest> | null,
): string {
  let cleanImporter = request?.normalizedFsId ?? request?.normalizedVuePath ?? importer;

  if (cleanImporter.startsWith("/@id/__x00__")) {
    cleanImporter = cleanImporter.slice("/@id/__x00__".length);
  } else if (cleanImporter.startsWith("__x00__")) {
    cleanImporter = cleanImporter.slice("__x00__".length);
  }

  return cleanImporter.endsWith(".vue.ts") ? cleanImporter.slice(0, -3) : cleanImporter;
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
  isDependencyScan: boolean,
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
      : isDependencyScan
        ? toVirtualId(realPath, isSsrRequest)
        : toPluginVisibleVirtualId(realPath, isSsrRequest, querySuffix);
  }

  return null;
}

export async function resolveIdHook(
  ctx: ResolveContext,
  state: VizePluginState,
  id: string,
  importer?: string,
  options?: { ssr?: boolean; scan?: boolean },
): Promise<string | { id: string; external?: boolean } | null | undefined> {
  // Fast-return before request classification for the common case where neither
  // the id nor importer can involve a Vue SFC or Vize virtual module. This was
  // added after profiles showed ordinary dependency graph edges dominating the
  // plugin hook cost in dev servers.
  if (!isPotentialVizeResolveId(id) && !isPotentialVizeImporter(importer)) {
    return null;
  }

  const isBuild = state.server === null;
  const isDependencyScan = !!options?.scan;
  const importerRequest = importer ? classifyVitePluginRequest(importer) : null;
  const isSsrRequest =
    !!options?.ssr ||
    (importerRequest?.isVizeSsrVirtual ?? false) ||
    (importer ? isPluginVisibleSsrVirtualId(importer) : false);
  const request = classifyVitePluginRequest(id);
  const pluginVisibleVirtualPath = fromPluginVisibleVirtualId(id);

  const projectVueRuntime = await resolveProjectVueRuntime(ctx, state, id, importer, isSsrRequest);
  if (projectVueRuntime) {
    return projectVueRuntime;
  }

  if (pluginVisibleVirtualPath) {
    if (isDependencyScan) {
      return toVirtualId(pluginVisibleVirtualPath, isSsrRequest);
    }
    return isSsrRequest
      ? toPluginVisibleVirtualId(pluginVisibleVirtualPath, true, request.querySuffix)
      : id;
  }

  // Skip all virtual module IDs
  if (id.startsWith("\0")) {
    // This is one of our .vue.ts virtual modules. Return the ID so Rolldown/Rollup
    // treats imports of Vize virtual modules from other virtual modules as resolved.
    if (request.isVizeVirtual) {
      if (isSsrRequest && !request.isVizeSsrVirtual && request.vizeVirtualPath) {
        return isDependencyScan
          ? toVirtualId(request.vizeVirtualPath, true)
          : toPluginVisibleVirtualId(request.vizeVirtualPath, true, request.querySuffix);
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
    return `${request.normalizedFsId ?? id}${request.styleVirtualSuffix}`;
  }

  if (isBuild && request.normalizedFsId) {
    return request.normalizedFsId;
  }

  // If importer is a vize virtual module or macro module, resolve imports against the real path
  const isMacroImporter = importerRequest?.isMacroVirtualId ?? false;
  const isVizeVirtualImporter = importerRequest?.isVizeVirtual ?? false;
  const isVueSfcImporter = importerRequest?.isVueSfcPath ?? false;
  if (importer && (isVizeVirtualImporter || isMacroImporter || isVueSfcImporter)) {
    const cleanImporter = isMacroImporter
      ? (importerRequest?.strippedVirtualPath ?? "")
      : isVizeVirtualImporter
        ? (importerRequest?.vizeVirtualPath ?? "")
        : cleanVueSfcImporter(importer, importerRequest);

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
        if (!isBuild) {
          const devSsrVueEntry =
            ssrExternalVueRequest === "vue/server-renderer"
              ? resolveVueServerRendererBundlerEntryWithNode(state, id, cleanImporter)
              : resolveVueBundlerEntryWithNode(state, id, cleanImporter);
          if (devSsrVueEntry) {
            state.logger.log(`resolveId: resolved SSR Vue request ${id} to ${devSsrVueEntry}`);
            return devSsrVueEntry;
          }
        }
        return { id: ssrExternalVueRequest, external: true };
      }

      // For bare module specifiers (not relative, not absolute),
      // resolve them from the real importer path so that Vite can find
      // packages in the correct node_modules directory.
      if (!id.startsWith("./") && !id.startsWith("../") && !id.startsWith("/")) {
        const isVueRuntime = isVueRuntimeRequest(id);
        if (isVueRuntime && isBuild) {
          const vueBundlerEntry = resolveVueBundlerEntryWithNode(state, id, cleanImporter);
          if (vueBundlerEntry) {
            state.logger.log(`resolveId: resolved Vue runtime to ${vueBundlerEntry}`);
            return vueBundlerEntry;
          }
        }

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
              const pnpmHoistedEntry = resolveVueBundlerEntryFromPnpmHoist(
                state,
                id,
                cleanImporter,
              );
              if (pnpmHoistedEntry) {
                state.logger.log(
                  `resolveId: resolved pnpm-hoisted Vue runtime to ${pnpmHoistedEntry}`,
                );
                return pnpmHoistedEntry;
              }

              const projectLocalEntry = resolveProjectLocalPnpmVueRuntime(state, resolved.id);
              if (projectLocalEntry) {
                state.logger.log(
                  `resolveId: resolved project-local Vue runtime to ${projectLocalEntry}`,
                );
                return projectLocalEntry;
              }

              if (isVueResolvableFromRoot(state.root)) {
                state.logger.log(
                  `resolveId: deferring Vue runtime ${resolved.id} to Vite optimizer`,
                );
                return null;
              }
              const isolatedEntry =
                resolveVueBundlerEntryWithNode(state, id, cleanImporter) ?? resolved.id;
              state.logger.log(
                `resolveId: isolated install — resolved Vue runtime to ${isolatedEntry}`,
              );
              return isolatedEntry;
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
                const vueBundlerEntry =
                  isBuild || !isVueResolvableFromRoot(state.root)
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
          const importerLocalEntry = resolveVueBundlerEntryWithNode(state, id, cleanImporter);
          const projectLocalEntry = importerLocalEntry
            ? resolveProjectLocalPnpmVueRuntime(state, importerLocalEntry)
            : null;
          if (projectLocalEntry) {
            state.logger.log(
              `resolveId: resolved project-local Vue runtime to ${projectLocalEntry}`,
            );
            return projectLocalEntry;
          }

          const vueBundlerEntry =
            isBuild || !isVueResolvableFromRoot(state.root) ? importerLocalEntry : null;
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
        isDependencyScan,
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
      return isDependencyScan
        ? toVirtualId(resolved, isSsrRequest)
        : toPluginVisibleVirtualId(resolved, isSsrRequest, request.querySuffix);
    }
  }

  return null;
}
