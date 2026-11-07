# `@vizejs/router`

> Experimental: the route table, SSR handoff shape, and generated metadata contract may change
> during the Vize alpha line.

The package is released on npm as `@vizejs/router`; the route and navigation contracts remain
experimental throughout the Vize alpha line.

`@vizejs/router` provides the first typed route-table kernel for Vize applications. It is
intentionally SSR-safe: matching and serialization do not touch browser globals, timers, or reactive
scope state.

```ts
import { createRouterMatcher, defineRoutes } from "@vizejs/router";

const routes = defineRoutes([
  { name: "home", path: "/", component: "./routes/home.vue" },
  { name: "user", path: "/users/:id/:tab?", component: "./routes/user.vue" },
] as const);

const router = createRouterMatcher(routes);
const match = router.match("/users/42/settings?from=ssr");

router.resolve("user", { id: "42" });
```

Route components are declared as `.vue` specifiers. JavaScript render-function component sources are
not part of this package surface.

SSR integrations can pass `serializeRouteMatch(match, data)` through their own safe serialization
channel, then call `hydrateRouteState(routes, state, currentUrl)` during client activation. Hydration
rematches the pathname against the current table and checks route identity, decoded params, query,
and hash. Malformed or stale handoffs return `undefined`; match and load the current URL afresh in
that case. The optional third argument checks the activation URL without reading browser globals.
Omitting it reconciles the serialized pathname and params only; it cannot detect navigation since
the server render. Query key order is ignored, while repeated-value order is preserved.

## Navigation lifecycle

`createRouter(routes, adapters)` adds cancellable guards, loaders, redirects, stale-result custody,
history commits, and explicit focus/scroll/announcement effects without reading browser globals.
Integrations provide the browser or native adapters they support; missing adapters are deliberate
no-ops rather than implicit global access.

```ts
import { createRouter } from "@vizejs/router";

const router = createRouter(routes, {
  history: {
    push: (url, leaving) => history.pushState(leaving, "", url),
    replace: (url, leaving) => history.replaceState(leaving, "", url),
  },
  announce: (title) => announceRoute(title),
});

router.beforeEach(({ to }) => (to.meta.requiresAuth && !session.value ? "/login" : undefined));
await router.navigate({ name: "user", params: { id: "42" } });
```

Starting a second navigation aborts the first one. A late loader result is reported as `stale` and
cannot replace current state. Explicit cancellation reports `cancelled`; loader/guard failures
report `error` and retain the route's optional `.vue` error boundary. History traversal passes saved
scroll/focus values through `navigate(..., { history: "pop", scroll, focus })`.

## File routes and evidence

`createFileRoutes` maps `index.vue`, `[id].vue`, `[[tab]].vue`, and `[...slug].vue` into explicit
route records. Each source can override its generated name/path while retaining the same manifest,
SSR, and navigation contracts. `vp run --filter './npm/compose/router' bench` records cold table
construction, warm matching, warm navigation, and a 1,000-route table in one machine-readable row.
