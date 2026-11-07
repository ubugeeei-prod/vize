import { createRouterMatcher } from "./runtime.ts";
import type {
  NamedRouteLocation,
  NavigationContext,
  NavigationGuard,
  NavigationGuardResult,
  NavigationOptions,
  NavigationResult,
  RouteDefinition,
  RouteLoaderContext,
  RouteLocation,
  RouteMatch,
  Router,
  RouterNavigationOptions,
  RouterNavigationState,
} from "./types.ts";

const MAX_REDIRECTS = 16;
type ResolvedLocation<Routes extends readonly RouteDefinition[]> = {
  match: RouteMatch<Routes[number]>;
  url: string;
};

/**
 * Create an SSR-safe navigation lifecycle around a literal route table.
 *
 * Browser history, focus, scrolling, and announcements are explicit adapters;
 * importing or constructing a router never reads browser globals.
 */
export function createRouter<const Routes extends readonly RouteDefinition[]>(
  routes: Routes,
  options: RouterNavigationOptions<Routes> = {},
): Router<Routes> {
  const matcher = createRouterMatcher(routes);
  const guards = new Set<NavigationGuard<Routes>>();
  const listeners = new Set<(state: RouterNavigationState<Routes>) => void>();
  let revision = 0;
  let active: AbortController | undefined;
  let state: RouterNavigationState<Routes> = {
    status: "idle",
    current: options.initial === undefined ? undefined : matcher.match(options.initial),
    pending: undefined,
    data: undefined,
    error: undefined,
  };

  const publish = (next: RouterNavigationState<Routes>): void => {
    state = next;
    for (const listener of listeners) listener(state);
  };

  return {
    matcher,
    state: () => state,
    beforeEach(guard) {
      guards.add(guard);
      return () => guards.delete(guard);
    },
    subscribe(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    cancel(reason) {
      active?.abort(reason ?? new Error("Navigation cancelled"));
    },
    async navigate(location, navigationOptions = {}) {
      const navigationRevision = ++revision;
      active?.abort(new Error("Navigation superseded"));
      const controller = new AbortController();
      active = controller;

      const initial = resolveLocation(matcher, location);
      if (!initial) {
        const result: NavigationResult<Routes> = {
          status: "error",
          error: new Error(`No route matches ${locationLabel(location)}`),
        };
        publish({ ...state, status: "error", pending: undefined, error: result.error });
        return result;
      }
      let resolved: ResolvedLocation<Routes> = initial;

      publish({ ...state, status: "pending", pending: resolved.match, error: undefined });
      let redirected = false;
      try {
        for (
          let redirectCount = 0;
          redirectCount <= (options.maxRedirects ?? MAX_REDIRECTS);
          redirectCount += 1
        ) {
          if (isStale(navigationRevision, revision, controller.signal)) {
            return finishInterrupted(navigationRevision, revision, state, publish);
          }

          const context: NavigationContext<Routes> = navigationContext(
            state.current,
            resolved.match,
            controller.signal,
          );
          const guarded: NavigationGuardResult<Routes> = await runGuards(guards, context);
          if (guarded === false) {
            publish({ ...state, status: "cancelled", pending: undefined, error: undefined });
            return { status: "cancelled", match: resolved.match };
          }
          if (isRouteLocation<Routes>(guarded)) {
            const next: ResolvedLocation<Routes> | undefined = resolveLocation(matcher, guarded);
            if (!next)
              throw new Error(`Redirect does not match a route: ${locationLabel(guarded)}`);
            resolved = next;
            redirected = true;
            publish({ ...state, status: "pending", pending: resolved.match, error: undefined });
            continue;
          }

          const redirect = resolved.match.route.redirect;
          if (redirect !== undefined) {
            const nextLocation =
              typeof redirect === "function"
                ? await redirect(loaderContext(resolved.match, controller.signal))
                : redirect;
            const next: ResolvedLocation<Routes> | undefined = resolveLocation(
              matcher,
              nextLocation,
            );
            if (!next) throw new Error(`Redirect does not match a route: ${String(nextLocation)}`);
            resolved = next;
            redirected = true;
            publish({ ...state, status: "pending", pending: resolved.match, error: undefined });
            continue;
          }

          const data =
            resolved.match.route.loader === undefined
              ? undefined
              : await resolved.match.route.loader(
                  loaderContext(resolved.match, controller.signal) as never,
                );
          if (isStale(navigationRevision, revision, controller.signal)) {
            return finishInterrupted(navigationRevision, revision, state, publish);
          }

          await commitNavigation(options, resolved.url, resolved.match, navigationOptions);
          publish({
            status: "success",
            current: resolved.match,
            pending: undefined,
            data,
            error: undefined,
          });
          return { status: "success", match: resolved.match, data, redirected };
        }
        throw new Error(`Redirect limit exceeded (${options.maxRedirects ?? MAX_REDIRECTS})`);
      } catch (error) {
        if (isStale(navigationRevision, revision, controller.signal)) {
          return finishInterrupted(navigationRevision, revision, state, publish);
        }
        publish({ ...state, status: "error", pending: undefined, error });
        return {
          status: "error",
          match: resolved.match,
          error,
          ...(resolved.match.route.errorComponent === undefined
            ? {}
            : { errorComponent: resolved.match.route.errorComponent }),
        };
      } finally {
        if (active === controller) active = undefined;
      }
    },
  };
}

function resolveLocation<const Routes extends readonly RouteDefinition[]>(
  matcher: Router<Routes>["matcher"],
  location: RouteLocation<Routes>,
): ResolvedLocation<Routes> | undefined {
  const url = isNamedLocation(location)
    ? matcher.resolve(location.name, location.params, {
        ...(location.query === undefined ? {} : { query: location.query }),
        ...(location.hash === undefined ? {} : { hash: location.hash }),
      })
    : location instanceof URL
      ? `${location.pathname}${location.search}${location.hash}`
      : location;
  const match = matcher.match(url);
  return match === undefined ? undefined : { match, url };
}

function isNamedLocation<Routes extends readonly RouteDefinition[]>(
  location: RouteLocation<Routes>,
): location is NamedRouteLocation<Routes> {
  return typeof location === "object" && !(location instanceof URL) && "name" in location;
}

function locationLabel<Routes extends readonly RouteDefinition[]>(
  location: RouteLocation<Routes>,
): string {
  if (typeof location === "string") return location;
  if (location instanceof URL) return location.href;
  return location.name;
}

function isRouteLocation<Routes extends readonly RouteDefinition[]>(
  value: unknown,
): value is RouteLocation<Routes> {
  return (
    typeof value === "string" ||
    value instanceof URL ||
    (value !== null && typeof value === "object" && "name" in value)
  );
}

function navigationContext<Routes extends readonly RouteDefinition[]>(
  from: RouteMatch<Routes[number]> | undefined,
  to: RouteMatch<Routes[number]>,
  signal: AbortSignal,
) {
  return { from, to, signal } as const;
}

function loaderContext(match: RouteMatch, signal: AbortSignal): RouteLoaderContext {
  return {
    pathname: match.pathname,
    params: match.params,
    query: match.query,
    hash: match.hash,
    signal,
  };
}

async function runGuards<Routes extends readonly RouteDefinition[]>(
  guards: ReadonlySet<NavigationGuard<Routes>>,
  context: Parameters<NavigationGuard<Routes>>[0],
) {
  for (const guard of guards) {
    const result = await guard(context);
    if (result === false || isRouteLocation<Routes>(result)) return result;
  }
  return undefined;
}

function isStale(current: number, revision: number, signal: AbortSignal): boolean {
  return current !== revision || signal.aborted;
}

function finishInterrupted<Routes extends readonly RouteDefinition[]>(
  current: number,
  revision: number,
  state: RouterNavigationState<Routes>,
  publish: (state: RouterNavigationState<Routes>) => void,
): NavigationResult<Routes> {
  const status = current === revision ? "cancelled" : "stale";
  if (current === revision) publish({ ...state, status, pending: undefined, error: undefined });
  return { status, ...(state.pending === undefined ? {} : { match: state.pending }) };
}

async function commitNavigation<Routes extends readonly RouteDefinition[]>(
  options: RouterNavigationOptions<Routes>,
  url: string,
  match: RouteMatch<Routes[number]>,
  navigation: NavigationOptions,
): Promise<void> {
  const history = navigation.history ?? "push";
  const leaving = {
    ...(options.scroll?.capture === undefined ? {} : { scroll: options.scroll.capture() }),
    ...(options.focus?.capture === undefined ? {} : { focus: options.focus.capture() }),
  };
  if (history === "push") await options.history?.push(url, leaving);
  if (history === "replace") await options.history?.replace(url, leaving);

  if (history === "pop") {
    if (navigation.scroll !== undefined) await options.scroll?.restore?.(navigation.scroll);
    if (navigation.focus !== undefined) await options.focus?.restore?.(navigation.focus);
  } else {
    await options.scroll?.top?.(match);
    await options.focus?.focusMain?.(match);
  }

  const title = typeof match.meta?.title === "string" ? match.meta.title : match.name;
  await options.announce?.(title, match);
}
