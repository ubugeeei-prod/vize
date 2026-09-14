# `@vizejs/router`

> Experimental: the route table, SSR handoff shape, and generated metadata contract may change
> during the Vize alpha line.

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
