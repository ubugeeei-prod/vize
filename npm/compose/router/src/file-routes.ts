import { defineRoutes } from "./runtime.ts";
import type { FileRouteSource, RouteDefinition } from "./types.ts";

/** Generate route records from source-owned `.vue` route modules. */
export function createFileRoutes(sources: readonly FileRouteSource[]): readonly RouteDefinition[] {
  return defineRoutes(
    sources.map((source) => ({
      name: source.name ?? routeNameFromFile(source.file),
      path: source.path ?? routePathFromFile(source.file),
      component: source.file,
      ...(source.meta === undefined ? {} : { meta: source.meta }),
      ...(source.loader === undefined ? {} : { loader: source.loader }),
      ...(source.redirect === undefined ? {} : { redirect: source.redirect }),
      ...(source.errorComponent === undefined ? {} : { errorComponent: source.errorComponent }),
    })),
  );
}

/** Convert `routes/users/[id].vue` into `/users/:id`. */
export function routePathFromFile(file: `${string}.vue`): string {
  const segments = routeSegments(file).flatMap((segment) => {
    if (segment === "index") return [];
    const rest = segment.match(/^\[\.\.\.([^\]]+)\]$/u);
    if (rest?.[1]) return [`*${rest[1]}`];
    const optional = segment.match(/^\[\[([^\]]+)\]\]$/u);
    if (optional?.[1]) return [`:${optional[1]}?`];
    const dynamic = segment.match(/^\[([^\]]+)\]$/u);
    if (dynamic?.[1]) return [`:${dynamic[1]}`];
    return [segment];
  });
  return segments.length === 0 ? "/" : `/${segments.join("/")}`;
}

/** Produce a stable manifest name from a file route. */
export function routeNameFromFile(file: `${string}.vue`): string {
  const segments = routeSegments(file)
    .filter((segment) => segment !== "index")
    .map((segment) =>
      segment
        .replace(/^\[+\.\.\./u, "")
        .replace(/^\[+/u, "")
        .replace(/\]+$/u, ""),
    );
  return segments.length === 0 ? "index" : segments.join("-");
}

function routeSegments(file: `${string}.vue`): string[] {
  const normalized = file.replaceAll("\\", "/").replace(/^\.\//u, "");
  const routeRoot = normalized.lastIndexOf("/routes/");
  const relative =
    routeRoot === -1 ? normalized.replace(/^routes\//u, "") : normalized.slice(routeRoot + 8);
  return relative.slice(0, -4).split("/").filter(Boolean);
}
