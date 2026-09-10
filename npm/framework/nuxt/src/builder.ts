export type NuxtBuilderKind = "unsupported" | "vite" | "webpack";

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
