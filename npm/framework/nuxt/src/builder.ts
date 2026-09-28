export type NuxtBuilderKind = "unsupported" | "vite" | "webpack";

export function getDetectedNuxtMajor(nuxt: unknown): 2 | 3 | 4 | null {
  const nuxtLike = nuxt as
    | {
        _version?: string;
        version?: string;
        options?: { _nuxtVersion?: string };
      }
    | undefined;
  const version = nuxtLike?._version ?? nuxtLike?.version ?? nuxtLike?.options?._nuxtVersion;
  if (!version) return null;
  const major = Number.parseInt(version.split(".")[0] ?? "", 10);
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

export function getNuxtBuilderKind(builder: unknown): NuxtBuilderKind {
  if (isViteNuxtBuilder(builder)) {
    return "vite";
  }
  if (isWebpackNuxtBuilder(builder)) {
    return "webpack";
  }
  return "unsupported";
}
