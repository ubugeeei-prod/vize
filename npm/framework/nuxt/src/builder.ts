export type NuxtBuilderKind = "unsupported" | "vite" | "webpack";

export function getDetectedNuxtMajor(nuxt: unknown): 2 | 3 | 4 | null {
  const nuxtLike = nuxt as
    | {
        _version?: string;
        version?: string;
        options?: { _nuxtVersion?: string };
        constructor?: { version?: string };
      }
    | undefined;
  const version =
    nuxtLike?._version ??
    nuxtLike?.version ??
    nuxtLike?.options?._nuxtVersion ??
    nuxtLike?.constructor?.version;
  if (!version) return null;
  // Nuxt 2 exposes its public version as Nuxt.version, including a leading v.
  const major = Number.parseInt(version.replace(/^v/, "").split(".")[0] ?? "", 10);
  return major === 2 || major === 3 || major === 4 ? major : null;
}

export function isViteNuxtBuilder(builder: unknown): boolean {
  if (typeof builder !== "string") {
    return false;
  }
  return (
    builder === "vite" ||
    builder.includes("vite-builder") ||
    builder === "rolldown-vite" ||
    builder.includes("rolldown-vite-builder")
  );
}

export function isWebpackNuxtBuilder(builder: unknown): boolean {
  if (typeof builder !== "string") {
    return false;
  }
  return builder === "webpack" || builder.includes("webpack-builder");
}

export function hasNuxtViteCompilerSupport(nuxt: {
  options: { builder?: unknown; vite?: unknown; _nuxtVersion?: string };
  _version?: string;
  version?: string;
}): boolean {
  if (isWebpackNuxtBuilder(nuxt.options.builder)) return false;
  if (isViteNuxtBuilder(nuxt.options.builder)) return true;
  if (nuxt.options.vite) return true;
  return getDetectedNuxtMajor(nuxt) !== 2;
}

export function getNuxtBuilderKind(builder: unknown): NuxtBuilderKind {
  if (isViteNuxtBuilder(builder)) {
    return "vite";
  }
  if (isWebpackNuxtBuilder(builder)) {
    return "webpack";
  }
  return "unsupported";
}
