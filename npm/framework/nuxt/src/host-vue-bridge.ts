import type { VizeNuxtCompilerOptions, VizeNuxtPattern } from "./compiler-options.ts";

type ViteHook = (this: unknown, ...args: unknown[]) => unknown;
type ViteHookObject = { handler?: ViteHook };

const VIZE_NUXT_HOST_VUE_EXCLUDE_PATCHED = "__vizeNuxtHostVueExcludePatched";

export type NuxtHostVueVitePlugin = {
  name?: string;
  load?: unknown;
  resolveId?: unknown;
  transform?: unknown;
  handleHotUpdate?: unknown;
  [VIZE_NUXT_HOST_VUE_EXCLUDE_PATCHED]?: boolean;
};

/**
 * Keep Nuxt's host Vue compiler active only for SFCs that Vize has explicitly
 * excluded. This preserves custom renderer pipelines such as `.takumi.vue`
 * while preventing @vitejs/plugin-vue from touching Vize virtual module IDs.
 */
export function patchNuxtHostVuePluginForCompilerExcludes(
  plugin: NuxtHostVueVitePlugin | undefined,
  compilerOptions: VizeNuxtCompilerOptions,
): boolean {
  if (!plugin || plugin.name !== "vite:vue") {
    return false;
  }
  if (plugin[VIZE_NUXT_HOST_VUE_EXCLUDE_PATCHED]) {
    return true;
  }
  if (!compilerOptions.exclude || !hasHostVueCompilerHooks(plugin)) {
    return false;
  }

  const shouldHandleRequest = createCompilerExcludeRequestMatcher(compilerOptions.exclude);
  const delegatedVuePaths = new Set<string>();
  const shouldHandleId = (id: unknown) => {
    if (typeof id !== "string") {
      return false;
    }
    if (isPluginVueInternalRequest(id)) {
      return delegatedVuePaths.size > 0;
    }
    if (shouldHandleRequest(id)) {
      const pathOnly = normalizeVueRequestPath(id);
      if (pathOnly) {
        delegatedVuePaths.add(pathOnly);
      }
      return true;
    }

    const pathOnly = normalizeVueRequestPath(id);
    return !!pathOnly && isPluginVueSubrequest(id) && delegatedVuePaths.has(pathOnly);
  };
  const shouldHandleIdOrImporter = (id: unknown, importer: unknown) =>
    shouldHandleId(id) || shouldHandleId(importer);

  let patched = false;
  patched =
    patchGuardedHook(
      plugin,
      "resolveId",
      (args) => shouldHandleIdOrImporter(args[0], args[1]),
      null,
    ) || patched;
  patched = patchGuardedHook(plugin, "load", (args) => shouldHandleId(args[0]), null) || patched;
  patched =
    patchGuardedHook(plugin, "transform", (args) => shouldHandleId(args[1]), null) || patched;
  patched =
    patchGuardedHook(
      plugin,
      "handleHotUpdate",
      (args) => {
        const context = args[0] as { file?: unknown } | undefined;
        return shouldHandleId(context?.file);
      },
      undefined,
    ) || patched;

  if (!patched) {
    return false;
  }

  plugin[VIZE_NUXT_HOST_VUE_EXCLUDE_PATCHED] = true;
  return true;
}

function hasHostVueCompilerHooks(plugin: NuxtHostVueVitePlugin): boolean {
  return (
    hasHookHandler(plugin.resolveId) ||
    hasHookHandler(plugin.load) ||
    hasHookHandler(plugin.transform) ||
    hasHookHandler(plugin.handleHotUpdate)
  );
}

function hasHookHandler(hook: unknown): boolean {
  return (
    typeof hook === "function" || !!(hook && typeof (hook as ViteHookObject).handler === "function")
  );
}

function patchGuardedHook(
  plugin: NuxtHostVueVitePlugin,
  hookName: "resolveId" | "load" | "transform" | "handleHotUpdate",
  shouldRun: (args: unknown[]) => boolean,
  skippedResult: unknown,
): boolean {
  const hook = plugin[hookName];
  if (typeof hook === "function") {
    const original = hook;
    plugin[hookName] = function (this: unknown, ...args: unknown[]) {
      return shouldRun(args) ? original.call(this, ...args) : skippedResult;
    };
    return true;
  }

  const hookObject = hook as ViteHookObject | undefined;
  if (!hookObject || typeof hookObject.handler !== "function") {
    return false;
  }

  const original = hookObject.handler;
  hookObject.handler = function (this: unknown, ...args: unknown[]) {
    return shouldRun(args) ? original.call(this, ...args) : skippedResult;
  };
  return true;
}

function createCompilerExcludeRequestMatcher(
  exclude: NonNullable<VizeNuxtCompilerOptions["exclude"]>,
): (id: string) => boolean {
  const patterns = Array.isArray(exclude) ? exclude : [exclude];
  return (id: string) => {
    const normalized = normalizeViteRequestId(id);
    if (!/\.vue(?:\?|$)/.test(normalized)) {
      return false;
    }

    const pathOnly = normalized.replace(/[?#].*$/, "");
    return (
      patterns.some((pattern) => matchesCompilerPattern(pattern, normalized)) ||
      patterns.some((pattern) => matchesCompilerPattern(pattern, pathOnly))
    );
  };
}

function normalizeVueRequestPath(id: string): string | null {
  const normalized = normalizeViteRequestId(id);
  const pathOnly = normalized.replace(/[?#].*$/, "");
  return pathOnly.endsWith(".vue") ? pathOnly : null;
}

function isPluginVueSubrequest(id: string): boolean {
  const normalized = normalizeViteRequestId(id);
  const query = normalized.slice(normalized.indexOf("?") + 1);
  return normalized.includes("?") && new URLSearchParams(query).has("vue");
}

function isPluginVueInternalRequest(id: string): boolean {
  return normalizeViteRequestId(id).startsWith("\0plugin-vue:");
}

function normalizeViteRequestId(id: string): string {
  let normalized = id;
  if (normalized.startsWith("/@id/__x00__")) {
    normalized = `\0${normalized.slice("/@id/__x00__".length)}`;
  } else if (normalized.startsWith("__x00__")) {
    normalized = `\0${normalized.slice("__x00__".length)}`;
  }
  if (normalized.startsWith("/@fs/")) {
    normalized = normalized.slice("/@fs".length);
  }
  try {
    normalized = decodeURIComponent(normalized);
  } catch {
    // Keep the original request if it is not percent-encoded cleanly.
  }
  return normalized.replace(/\\/g, "/");
}

const GLOB_META = /[*?]/;

function matchesCompilerPattern(pattern: VizeNuxtPattern, id: string): boolean {
  if (typeof pattern !== "string") {
    pattern.lastIndex = 0;
    const matched = pattern.test(id);
    pattern.lastIndex = 0;
    return matched;
  }

  const normalizedPattern = pattern.replace(/\\/g, "/");
  if (!GLOB_META.test(normalizedPattern)) {
    return id.includes(normalizedPattern);
  }
  return stringGlobToRegExp(normalizedPattern).test(id);
}

function stringGlobToRegExp(pattern: string): RegExp {
  const absolute = pattern.startsWith("/");
  let body = absolute ? pattern : stripRelativePrefix(pattern);
  let descendantSuffix = false;

  if (body.endsWith("/**")) {
    body = body.slice(0, -3);
    descendantSuffix = true;
  }

  const prefix = absolute ? "^" : "(?:^|/)";
  const suffix = descendantSuffix ? "(?:/.*)?$" : "$";
  return new RegExp(`${prefix}${globBodyToRegExp(body)}${suffix}`);
}

function stripRelativePrefix(pattern: string): string {
  let rest = pattern;
  while (rest.startsWith("../")) {
    rest = rest.slice(3);
  }
  while (rest.startsWith("./")) {
    rest = rest.slice(2);
  }
  return rest;
}

function globBodyToRegExp(pattern: string): string {
  let source = "";
  for (let i = 0; i < pattern.length;) {
    const char = pattern[i];
    if (char === "*") {
      if (pattern[i + 1] === "*") {
        if (pattern[i + 2] === "/") {
          source += "(?:.*\\/)?";
          i += 3;
        } else {
          source += ".*";
          i += 2;
        }
      } else {
        source += "[^/]*";
        i += 1;
      }
      continue;
    }
    if (char === "?") {
      source += "[^/]";
      i += 1;
      continue;
    }
    source += escapeRegExp(char);
    i += 1;
  }
  return source;
}

function escapeRegExp(char: string): string {
  return "\\^$+?.()|[]{}".includes(char) ? `\\${char}` : char;
}
