import type {
  CompiledRoute,
  RouteDefinition,
  RouteManifest,
  RouteMatch,
  RouteNames,
  RouteParams,
  RouteQuery,
  RouteResolveOptions,
  RouterMatcher,
  SerializedRouteState,
} from "./types.ts";

const ROUTER_ERROR = {
  duplicateName: "VIZE_ROUTER_DUPLICATE_NAME",
  duplicatePath: "VIZE_ROUTER_DUPLICATE_PATH",
  invalidComponent: "VIZE_ROUTER_COMPONENT_NOT_VUE",
  invalidPath: "VIZE_ROUTER_INVALID_PATH",
  missingParam: "VIZE_ROUTER_MISSING_PARAM",
  unknownRoute: "VIZE_ROUTER_UNKNOWN_ROUTE",
} as const;

/** Define a literal route table and fail closed on duplicate or unsupported rows. */
export function defineRoutes<const Routes extends readonly RouteDefinition[]>(
  routes: Routes,
): Routes {
  validateRoutes(routes);
  return routes;
}

/** Define one route record while preserving loader param inference from its path. */
export function defineRoute<
  const Name extends string,
  const Path extends string,
  const Meta extends Readonly<Record<string, unknown>> = Readonly<Record<string, unknown>>,
  Data = unknown,
>(route: RouteDefinition<Name, Path, Meta, Data>): RouteDefinition<Name, Path, Meta, Data> {
  validateRoutes([route]);
  return route;
}

/** Create an SSR-safe matcher and typed URL resolver for a route table. */
export function createRouterMatcher<const Routes extends readonly RouteDefinition[]>(
  routes: Routes,
): RouterMatcher<Routes> {
  validateRoutes(routes);
  const compiled = routes.map((route) => ({ route, ...compilePath(route.path) }));

  return {
    routes,
    match(input) {
      const url = toUrl(input);
      for (const candidate of compiled) {
        const matched = candidate.pattern.exec(url.pathname);
        if (!matched) continue;
        const params = Object.fromEntries(
          candidate.params.flatMap((name, index) => {
            const value = matched[index + 1];
            return value == null ? [] : [[name, decodeURIComponent(value)]];
          }),
        );
        return {
          route: candidate.route,
          name: candidate.route.name,
          pathname: url.pathname,
          params,
          query: parseQuery(url.searchParams),
          hash: url.hash.startsWith("#") ? url.hash.slice(1) : url.hash,
          meta: candidate.route.meta ?? {},
        } as RouteMatch<Routes[number]>;
      }
      return undefined;
    },
    manifest: () => createRouteManifest(routes),
    resolve(name, params, options) {
      const route = routes.find((candidate) => candidate.name === name);
      if (!route) throw new Error(`[${ROUTER_ERROR.unknownRoute}] Unknown route ${String(name)}`);
      return `${buildPath(route.path, params)}${stringifyQuery(options?.query)}${stringifyHash(options?.hash)}`;
    },
  };
}

/** Emit serializable metadata for generators, docs, and devtools. */
export function createRouteManifest<const Routes extends readonly RouteDefinition[]>(
  routes: Routes,
): RouteManifest {
  validateRoutes(routes);
  return {
    schemaVersion: 1,
    routes: routes.map((route) => ({
      name: route.name,
      path: route.path,
      component: route.component,
      meta: route.meta ?? {},
    })),
  };
}

/** Serialize a route match for SSR handoff. */
export function serializeRouteMatch<Route extends RouteDefinition>(
  match: RouteMatch<Route>,
  data?: unknown,
): SerializedRouteState<Route["name"]> {
  return {
    schemaVersion: 1,
    name: match.name,
    pathname: match.pathname,
    params: match.params as RouteParams,
    query: match.query,
    hash: match.hash,
    ...(data === undefined ? {} : { data }),
  };
}

/** Hydrate serialized SSR route state against the client route table. */
export function hydrateRouteState<const Routes extends readonly RouteDefinition[]>(
  routes: Routes,
  state: SerializedRouteState<RouteNames<Routes>>,
): RouteMatch<Routes[number]> | undefined {
  if (state.schemaVersion !== 1) return undefined;
  const route = routes.find((candidate) => candidate.name === state.name);
  if (!route) return undefined;
  return {
    route,
    name: route.name,
    pathname: state.pathname,
    params: state.params,
    query: state.query,
    hash: state.hash,
    meta: route.meta ?? {},
  } as RouteMatch<Routes[number]>;
}

function validateRoutes(routes: readonly RouteDefinition[]): void {
  const names = new Set<string>();
  const paths = new Set<string>();
  for (const route of routes) {
    if (!route.path.startsWith("/")) {
      throw new Error(`[${ROUTER_ERROR.invalidPath}] Route ${route.name} path must start with /`);
    }
    if (!route.component.endsWith(".vue")) {
      throw new Error(
        `[${ROUTER_ERROR.invalidComponent}] Route ${route.name} must use a .vue component`,
      );
    }
    if (names.has(route.name)) {
      throw new Error(`[${ROUTER_ERROR.duplicateName}] Duplicate route name ${route.name}`);
    }
    if (paths.has(route.path)) {
      throw new Error(`[${ROUTER_ERROR.duplicatePath}] Duplicate route path ${route.path}`);
    }
    names.add(route.name);
    paths.add(route.path);
  }
}

function compilePath(pathPattern: string): Pick<CompiledRoute, "params" | "pattern"> {
  if (pathPattern === "/") return { pattern: /^\/$/, params: [] };

  const params: string[] = [];
  const source = pathPattern
    .split("/")
    .filter(Boolean)
    .map((segment) => {
      if (segment.startsWith(":")) {
        const optional = segment.endsWith("?");
        const name = segment.slice(1, optional ? -1 : undefined);
        params.push(name);
        return optional ? "(?:/([^/]+))?" : "/([^/]+)";
      }
      if (segment.startsWith("*")) {
        const name = segment.slice(1);
        params.push(name);
        return "/(.+)";
      }
      return `/${escapeRegExp(segment)}`;
    })
    .join("");

  return { pattern: new RegExp(`^${source}$`), params };
}

function buildPath(pathPattern: string, params: RouteParams): string {
  if (pathPattern === "/") return "/";
  const path = pathPattern
    .split("/")
    .filter(Boolean)
    .flatMap((segment) => {
      if (!segment.startsWith(":") && !segment.startsWith("*")) return [segment];
      const optional = segment.endsWith("?");
      const name = segment.slice(segment.startsWith(":") ? 1 : 1, optional ? -1 : undefined);
      const value = params[name];
      if (value == null) {
        if (optional) return [];
        throw new Error(`[${ROUTER_ERROR.missingParam}] Missing route param ${name}`);
      }
      const encoded = segment.startsWith("*")
        ? value.split("/").map(encodeURIComponent).join("/")
        : encodeURIComponent(value);
      return [encoded];
    })
    .join("/");
  return `/${path}`;
}

function toUrl(input: string | URL): URL {
  return input instanceof URL ? input : new URL(input, "https://vize.local");
}

function parseQuery(searchParams: URLSearchParams): RouteQuery {
  const query: Record<string, string | string[]> = {};
  for (const [key, value] of searchParams) {
    const current = query[key];
    if (current == null) {
      query[key] = value;
    } else if (Array.isArray(current)) {
      current.push(value);
    } else {
      query[key] = [current, value];
    }
  }
  return query;
}

function stringifyQuery(query: RouteResolveOptions["query"]): string {
  if (!query) return "";
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(query)) {
    if (value == null) continue;
    const values = Array.isArray(value) ? value : [value];
    for (const item of values) search.append(key, String(item));
  }
  const text = search.toString();
  return text === "" ? "" : `?${text}`;
}

function stringifyHash(hash: string | undefined): string {
  if (!hash) return "";
  return hash.startsWith("#") ? hash : `#${hash}`;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
