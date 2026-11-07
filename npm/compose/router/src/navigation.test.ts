import assert from "node:assert/strict";
import { test } from "node:test";

import { createRouter, defineRoute, defineRoutes } from "./index.ts";

const routes = defineRoutes([
  defineRoute({
    name: "home",
    path: "/",
    component: "./routes/home.vue",
    meta: { title: "Home" },
  }),
  defineRoute({
    name: "user",
    path: "/users/:id",
    component: "./routes/user.vue",
    errorComponent: "./routes/user-error.vue",
    meta: { title: "User" },
    loader: async ({ params, signal }) => {
      if (params.id === "error") throw new Error("user failed");
      if (params.id === "cancel") {
        await new Promise<void>((_resolve, reject) => {
          signal?.addEventListener("abort", () => reject(signal.reason), { once: true });
        });
      }
      return { id: params.id };
    },
  }),
  {
    name: "legacy",
    path: "/legacy",
    component: "./routes/home.vue",
    redirect: "/",
  },
] as const);

void test("runs loaders and commits history, focus, scroll and announcements", async () => {
  const effects: string[] = [];
  const router = createRouter(routes, {
    initial: "/",
    history: {
      push: (url, leaving) => {
        effects.push(`push:${url}:${leaving.scroll}:${leaving.focus}`);
      },
      replace: (url) => {
        effects.push(`replace:${url}`);
      },
    },
    scroll: {
      capture: () => "scroll-before",
      top: (match) => {
        effects.push(`top:${match.name}`);
      },
    },
    focus: {
      capture: () => "focus-before",
      focusMain: (match) => {
        effects.push(`focus:${match.name}`);
      },
    },
    announce: (message) => {
      effects.push(`announce:${message}`);
    },
  });
  const states: string[] = [];
  router.subscribe((state) => states.push(state.status));

  const result = await router.navigate({ name: "user", params: { id: "42" } });
  assert.equal(result.status, "success");
  assert.deepEqual(result.status === "success" ? result.data : undefined, { id: "42" });
  assert.equal(router.state().current?.name, "user");
  assert.deepEqual(states, ["pending", "success"]);
  assert.deepEqual(effects, [
    "push:/users/42:scroll-before:focus-before",
    "top:user",
    "focus:user",
    "announce:User",
  ]);
});

void test("guards cancel or redirect before a loader commits", async () => {
  const router = createRouter(routes);
  const stop = router.beforeEach(({ to }) => (to.name === "user" ? false : undefined));
  assert.equal((await router.navigate("/users/1")).status, "cancelled");
  stop();

  router.beforeEach(({ to }) => (to.name === "legacy" ? "/users/2" : undefined));
  const guarded = await router.navigate("/legacy");
  assert.equal(guarded.status, "success");
  assert.equal(guarded.status === "success" ? guarded.match.name : undefined, "user");
  assert.equal(guarded.status === "success" && guarded.redirected, true);

  const redirected = await createRouter(routes).navigate("/legacy");
  assert.equal(redirected.status, "success");
  assert.equal(redirected.status === "success" ? redirected.match.name : undefined, "home");
});

void test("superseded loader results become stale and cannot replace current state", async () => {
  let release: (() => void) | undefined;
  const delayed = defineRoutes([
    {
      name: "slow",
      path: "/slow",
      component: "./routes/home.vue",
      loader: () => new Promise<void>((resolve) => (release = resolve)),
    },
    { name: "fast", path: "/fast", component: "./routes/home.vue" },
  ] as const);
  const router = createRouter(delayed);
  const slow = router.navigate("/slow");
  await Promise.resolve();
  assert.equal((await router.navigate("/fast")).status, "success");
  release?.();
  assert.equal((await slow).status, "stale");
  assert.equal(router.state().current?.name, "fast");
});

void test("explicit cancellation and error boundaries have distinct results", async () => {
  const router = createRouter(routes);
  const cancelled = router.navigate("/users/cancel");
  await Promise.resolve();
  router.cancel("test cancellation");
  assert.equal((await cancelled).status, "cancelled");

  const failed = await router.navigate("/users/error");
  assert.equal(failed.status, "error");
  assert.equal(
    failed.status === "error" ? failed.errorComponent : undefined,
    "./routes/user-error.vue",
  );
});

void test("history traversal restores supplied scroll and focus snapshots", async () => {
  const effects: string[] = [];
  const router = createRouter(routes, {
    scroll: {
      restore: (value) => {
        effects.push(`scroll:${String(value)}`);
      },
    },
    focus: {
      restore: (value) => {
        effects.push(`focus:${String(value)}`);
      },
    },
  });
  const result = await router.navigate("/", {
    history: "pop",
    scroll: { x: 0, y: 120 },
    focus: "search",
  });
  assert.equal(result.status, "success");
  assert.deepEqual(effects, ["scroll:[object Object]", "focus:search"]);
});
