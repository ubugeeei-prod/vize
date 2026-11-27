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
        effects.push(`push:${url}:${String(leaving.scroll)}:${String(leaving.focus)}`);
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

function recordingRouter() {
  const written: string[] = [];
  const router = createRouter(routes, {
    history: {
      push: (url) => {
        written.push(`push:${url}`);
      },
      replace: (url) => {
        written.push(`replace:${url}`);
      },
    },
  });
  return { router, written };
}

void test("a string location on another origin does not match and writes no history", async () => {
  for (const location of [
    "//other.example/users/1",
    "\\\\other.example/users/1",
    "/\t/other.example/users/1",
    "/\\other.example/users/1",
    "https://other.example/users/1",
    "javascript:/users/1",
    "/users/1/../..//other.example/users/1",
  ]) {
    const { router, written } = recordingRouter();
    const result = await router.navigate(location);
    assert.equal(result.status, "error", location);
    assert.equal(router.state().current, undefined, location);
    assert.deepEqual(written, [], location);
  }

  const { router, written } = recordingRouter();
  router.beforeEach(() => "//other.example/users/1");
  const redirected = await router.navigate("/users/1");
  assert.deepEqual(
    redirected.status === "error" ? redirected.error : undefined,
    new Error("Redirect does not match a route: //other.example/users/1"),
  );
  assert.deepEqual(written, []);
});

void test("history receives the normalised path, query and hash of a location", async () => {
  const { router, written } = recordingRouter();
  for (const location of [
    "/users/1?tab=a&tab=b#top",
    "users/2",
    "/users/skipped/../3",
    new URL("https://app.example/users/4?from=url#end"),
  ]) {
    assert.equal((await router.navigate(location)).status, "success", String(location));
  }
  assert.equal((await router.navigate("/users/5", { history: "replace" })).status, "success");
  assert.deepEqual(written, [
    "push:/users/1?tab=a&tab=b#top",
    "push:/users/2",
    "push:/users/3",
    "push:/users/4?from=url#end",
    "replace:/users/5",
  ]);
});

void test("a location that cannot be parsed or decoded is reported, not thrown", async () => {
  for (const location of ["/users/%E0%A4%A", "//[", "http://["]) {
    const { router, written } = recordingRouter();
    const result = await router.navigate(location);
    assert.equal(result.status, "error", location);
    assert.deepEqual(
      result.status === "error" ? result.error : undefined,
      new Error(`No route matches ${location}`),
    );
    assert.equal(router.state().status, "error", location);
    assert.deepEqual(written, [], location);

    assert.equal(createRouter(routes, { initial: location }).state().current, undefined, location);
  }
});

void test("a named location whose params cannot form a path is reported, not thrown", async () => {
  const invalid = '[VIZE_ROUTER_INVALID_PARAM] Route param id cannot contain a "." or ".." segment';
  for (const [id, message] of [
    ["", "[VIZE_ROUTER_MISSING_PARAM] Missing route param id"],
    [".", invalid],
    ["..", invalid],
  ] as const) {
    const { router, written } = recordingRouter();
    const result = await router.navigate({ name: "user", params: { id } });
    assert.deepEqual(result, { status: "error", error: new Error(message) });
    assert.deepEqual(router.state().error, new Error(message));
    assert.deepEqual(written, []);
  }
});
