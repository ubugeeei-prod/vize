import type { ApplicationMarquette, EnvironmentId, MarquetteRoute, RouteId } from "./model.js";
import type { NavigationParameters, ResolvedNavigationTarget } from "./navigation.js";

/** Explicit host policy for incoming application links. */
export interface NavigationLinkPolicy<Environment extends string = string> {
  /** Only routes owned by this application environment may be resolved. */
  readonly environment: Environment;
  /** Non-empty HTTPS or custom-scheme origins, including their explicit host. */
  readonly origins: readonly string[];
}

/** An incoming link target whose decoded parameters remain tied to its route. */
export type ResolvedNavigationLink<
  Marquette extends ApplicationMarquette,
  Environment extends EnvironmentId<Marquette> = EnvironmentId<Marquette>,
  Id extends RouteId<Marquette> = RouteId<Marquette>,
> =
  Id extends RouteId<Marquette>
    ? Extract<ResolvedNavigationTarget<Marquette, Id>, { readonly environment: Environment }> & {
        /** Decoded strings inferred from this route's authored parameter segments. */
        readonly parameters: {
          readonly [Name in keyof NavigationParameters<Marquette, Id>]: string;
        };
      }
    : never;

/** Stable refusal reason emitted before an incoming link reaches a host. */
export type NavigationLinkRefusal =
  | "invalid-link"
  | "untrusted-origin"
  | "invalid-path"
  | "unknown-route"
  | "ambiguous-route";

/** Target-neutral admission result for an untrusted application link. */
export type NavigationLinkResolution<
  Marquette extends ApplicationMarquette,
  Environment extends EnvironmentId<Marquette> = EnvironmentId<Marquette>,
> =
  | {
      /** Whether the link resolved to exactly one route. */
      readonly matched: true;
      /** Inferred route, environment, canonical path, and decoded parameters. */
      readonly target: ResolvedNavigationLink<Marquette, Environment>;
    }
  | {
      /** Whether the link resolved to exactly one route. */
      readonly matched: false;
      /** Machine-readable reason why the host must not navigate. */
      readonly code: NavigationLinkRefusal;
    };

/** Resolves incoming links against a fixed environment and origin policy. */
export interface MarquetteLinkResolver<
  Marquette extends ApplicationMarquette,
  Environment extends EnvironmentId<Marquette> = EnvironmentId<Marquette>,
> {
  /** Admit an absolute application link or return a deterministic refusal. */
  readonly resolve: (link: string) => NavigationLinkResolution<Marquette, Environment>;
}

type CompiledSegment =
  | { readonly kind: "static"; readonly authored: string; readonly decoded: string }
  | { readonly kind: "parameter" | "rest"; readonly name: string };

interface CompiledRoute {
  readonly route: MarquetteRoute;
  readonly segments: readonly CompiledSegment[];
  readonly priority: readonly number[];
}

interface RouteMatch {
  readonly route: CompiledRoute;
  readonly path: `/${string}`;
  readonly parameters: Readonly<Record<string, string>>;
}

/**
 * Creates an inferred deep-link boundary for web, native, and desktop hosts.
 *
 * Only explicitly admitted HTTPS or custom-scheme origins and routes owned by
 * the selected environment can match. Credentials, malformed escapes, control
 * characters, backslashes, and decoded dot segments are refused before URL
 * normalization can conceal them. Static segments outrank parameters, which
 * outrank a terminal `*rest` segment; equally specific matches are refused.
 *
 * Parameters are decoded exactly once and paths are encoded like programmatic
 * navigation. Query strings and fragments never become route parameters and
 * are not forwarded to the host. Invalid policies, duplicate route identifiers,
 * and unsupported route templates throw during construction. Resolution scans
 * the selected environment's precompiled routes without mutating the marquette.
 * This experimental boundary does not launch a device or register OS links.
 */
export function createMarquetteLinkResolver<
  const Marquette extends ApplicationMarquette,
  const Environment extends EnvironmentId<Marquette>,
>(
  marquette: Marquette,
  policy: NavigationLinkPolicy<Environment>,
): MarquetteLinkResolver<Marquette, Environment> {
  if (!marquette.environments?.some(({ id }) => id === policy.environment)) {
    throw new TypeError(`Unknown link environment "${policy.environment}".`);
  }
  if (policy.origins.length === 0) throw new TypeError("Link origins must not be empty.");
  const origins = new Set(policy.origins.map(compileOrigin));
  const routes = (marquette.routes ?? [])
    .filter(({ environment }) => environment === policy.environment)
    .map(compileRoute);
  if (new Set(routes.map(({ route }) => route.id)).size !== routes.length) {
    throw new TypeError("Link route identifiers must be unique within the environment.");
  }
  routes.sort((left, right) => comparePriority(right.priority, left.priority));

  return {
    resolve(link) {
      const parsed = parseLink(link);
      if (parsed === undefined) return { matched: false, code: "invalid-link" };
      if (!origins.has(originKey(parsed.url))) {
        return { matched: false, code: "untrusted-origin" };
      }
      const segments = decodePath(parsed.path);
      if (segments === undefined) return { matched: false, code: "invalid-path" };
      let best: RouteMatch | undefined;
      for (const route of routes) {
        if (best && comparePriority(best.route.priority, route.priority) > 0) break;
        const match = matchRoute(route, segments);
        if (!match) continue;
        if (best) return { matched: false, code: "ambiguous-route" };
        best = match;
      }
      if (!best) return { matched: false, code: "unknown-route" };
      return {
        matched: true,
        target: {
          route: best.route.route.id,
          environment: best.route.route.environment,
          path: best.path,
          parameters: best.parameters,
        } as ResolvedNavigationLink<Marquette, Environment>,
      };
    },
  };
}

function compileOrigin(origin: string): string {
  const parsed = parseLink(origin);
  if (
    parsed === undefined ||
    parsed.path !== "/" ||
    parsed.url.search !== "" ||
    parsed.url.hash !== "" ||
    /[?#]/.test(origin) ||
    /^(?:http|ws|wss|ftp|file|data|javascript|blob|about|mailto|tel):$/.test(parsed.url.protocol)
  ) {
    throw new TypeError(`Invalid application link origin "${origin}".`);
  }
  return originKey(parsed.url);
}

function originKey(url: URL): string {
  return `${url.protocol}//${url.host.toLowerCase()}`;
}

function parseLink(link: string): { readonly url: URL; readonly path: string } | undefined {
  if (typeof link !== "string" || /[\p{Cc}\s\\]/u.test(link)) return undefined;
  const match = /^[a-z][a-z0-9+.-]*:\/\/([^/?#]+)([^?#]*)/i.exec(link);
  if (!match || match[1]!.includes("@")) return undefined;
  try {
    const url = new URL(link);
    if (!url.host || url.username || url.password) return undefined;
    return { url, path: match[2] || "/" };
  } catch {
    return undefined;
  }
}

function decodePath(path: string): string[] | undefined {
  if (!path.startsWith("/") || path.startsWith("//")) return undefined;
  try {
    return path
      .slice(1)
      .split("/")
      .map((segment) => {
        const decoded = decodeURIComponent(segment);
        if (decoded === "." || decoded === ".." || /[\p{Cc}\\]/u.test(decoded)) {
          throw new TypeError("Invalid path segment.");
        }
        // Also reject malformed UTF-16 that a URL parser would silently replace.
        encodeURIComponent(decoded);
        return decoded;
      });
  } catch {
    return undefined;
  }
}

function compileRoute(route: MarquetteRoute): CompiledRoute {
  const invalid = (): never => {
    throw new TypeError(`Unsupported application link route "${route.id}".`);
  };
  const decoded = decodePath(route.path);
  if (decoded === undefined || /[?#]/.test(route.path)) invalid();
  const names = new Set<string>();
  const authored = route.path.slice(1).split("/");
  const segments = authored.map((segment, index): CompiledSegment => {
    if (segment.startsWith(":") || segment.startsWith("*")) {
      const name = segment.slice(1);
      const rest = segment.startsWith("*");
      if (!name || names.has(name) || (rest && index !== authored.length - 1)) invalid();
      names.add(name);
      return { kind: rest ? "rest" : "parameter", name };
    }
    return {
      kind: "static",
      authored: encodeURIComponent(decoded![index]!),
      decoded: decoded![index]!,
    };
  });
  return {
    // Snapshot metadata so later caller mutations cannot change the admitted route.
    route: { ...route },
    segments,
    priority: segments.map(({ kind }) => (kind === "static" ? 2 : kind === "parameter" ? 1 : 0)),
  };
}

function comparePriority(left: readonly number[], right: readonly number[]): number {
  for (let index = 0; index < Math.max(left.length, right.length); index += 1) {
    const difference = (left[index] ?? 3) - (right[index] ?? 3);
    if (difference !== 0) return difference;
  }
  return 0;
}

function matchRoute(route: CompiledRoute, input: readonly string[]): RouteMatch | undefined {
  const parameters: [string, string][] = [];
  const encoded: string[] = [];
  for (const [index, segment] of route.segments.entries()) {
    const value = input[index];
    if (value === undefined) return undefined;
    if (segment.kind === "static") {
      if (value !== segment.decoded) return undefined;
      encoded.push(segment.authored);
    } else if (segment.kind === "parameter") {
      if (value === "") return undefined;
      parameters.push([segment.name, value]);
      encoded.push(encodeURIComponent(value));
    } else {
      const remaining = input.slice(index);
      // Catch-all separators must have come from the path itself. An encoded
      // separator would change meaning when passed back to the navigator.
      if (!remaining[0] || remaining.some((value) => value.includes("/"))) return undefined;
      parameters.push([segment.name, remaining.join("/")]);
      encoded.push(...remaining.map(encodeURIComponent));
      return { route, path: `/${encoded.join("/")}`, parameters: Object.fromEntries(parameters) };
    }
  }
  if (route.segments.length !== input.length) return undefined;
  return { route, path: `/${encoded.join("/")}`, parameters: Object.fromEntries(parameters) };
}
