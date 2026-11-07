/** Compile-only assertions for typed router contracts. */

import {
  createRouter,
  createRouterMatcher,
  defineRoutes,
  type RouteNames,
  type RouteParamsForName,
  type RoutePathParams,
} from "./index.js";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const routes = defineRoutes([
  { name: "home", path: "/", component: "./fixtures/routes/home.vue" },
  { name: "user", path: "/users/:id/:tab?", component: "./fixtures/routes/user.vue" },
  { name: "docs", path: "/docs/*slug", component: "./fixtures/routes/docs.vue" },
] as const);

type _RouteNamesStayLiteral = Expect<Equal<RouteNames<typeof routes>, "home" | "user" | "docs">>;
type _UserParamsInferRequiredAndOptional = Expect<
  Equal<RouteParamsForName<typeof routes, "user">, { readonly id: string; readonly tab?: string }>
>;
type _SplatParamsAreNamed = Expect<
  Equal<RoutePathParams<"/docs/*slug">, { readonly slug: string }>
>;

const router = createRouterMatcher(routes);

router.resolve("home", {});
router.resolve("user", { id: "42" });
router.resolve("user", { id: "42", tab: "settings" }, { query: { page: 1 } });
router.resolve("docs", { slug: "guide/router" });

// @ts-expect-error unknown route names are rejected.
router.resolve("missing", {});
// @ts-expect-error required route params are enforced.
router.resolve("user", {});
// @ts-expect-error route params are strings after URL decoding.
router.resolve("user", { id: 42 });
// @ts-expect-error query values must be serializable URL scalars or scalar arrays.
router.resolve("user", { id: "42" }, { query: { filter: { active: true } } });
// @ts-expect-error route components must be source-owned Vue files.
defineRoutes([{ name: "bad", path: "/", component: "./render.ts" }] as const);

const navigation = createRouter(routes);
void navigation.navigate({ name: "home", params: {} });
void navigation.navigate({ name: "user", params: { id: "42" }, query: { tab: "activity" } });
// @ts-expect-error named navigation preserves required path params.
void navigation.navigate({ name: "user", params: {} });
// @ts-expect-error named navigation rejects unknown route names.
void navigation.navigate({ name: "missing", params: {} });
// @ts-expect-error named navigation rejects non-serializable query values.
void navigation.navigate({ name: "user", params: { id: "42" }, query: { since: new Date() } });
