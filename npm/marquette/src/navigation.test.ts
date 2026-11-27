import assert from "node:assert/strict";
import { test } from "node:test";

import { defineApplicationMarquette } from "./author.js";
import {
  createMarquetteNavigator,
  type NavigationParameters,
  type ResolvedNavigationTarget,
} from "./navigation.js";

const marquette = defineApplicationMarquette({
  application: "navigation-fixture",
  targets: ["native"],
  environments: [{ id: "device", target: "native", consumer: "client", runtime: "native" }],
  routes: [
    { id: "home", path: "/", environment: "device", rendering: "native" },
    {
      id: "photo",
      path: "/users/:userId/photos/:photoId",
      environment: "device",
      rendering: "native",
    },
    { id: "guide", path: "/guides/*path", environment: "device", rendering: "native" },
  ],
} as const);

void test("infers route parameters and delivers encoded adapter targets", () => {
  const visited: ResolvedNavigationTarget<typeof marquette>[] = [];
  const navigator = createMarquetteNavigator(marquette, (target) => {
    visited.push(target);
    return target.path;
  });

  assert.equal(navigator.navigate("home"), "/");
  assert.equal(
    navigator.navigate("photo", { userId: "A B", photoId: 42 }),
    "/users/A%20B/photos/42",
  );
  assert.equal(
    navigator.navigate("guide", { path: "intro/日本語" }),
    "/guides/intro/%E6%97%A5%E6%9C%AC%E8%AA%9E",
  );
  assert.deepEqual(visited, [
    { route: "home", environment: "device", path: "/" },
    { route: "photo", environment: "device", path: "/users/A%20B/photos/42" },
    { route: "guide", environment: "device", path: "/guides/intro/%E6%97%A5%E6%9C%AC%E8%AA%9E" },
  ]);
});

void test("rejects untrusted missing, extra, and invalid parameters before visiting", () => {
  let visits = 0;
  const navigator = createMarquetteNavigator(marquette, () => {
    visits += 1;
  });
  const navigate = navigator.navigate as (
    route: "home" | "photo" | "guide" | "missing",
    parameters?: Readonly<Record<string, string | number>>,
  ) => void;

  assert.throws(() => navigate("missing"), new Error('Unknown navigation route "missing".'));
  assert.throws(
    () => navigate("photo", { userId: "one" }),
    new Error('Missing navigation parameter "photoId" for route "photo".'),
  );
  assert.throws(
    () => navigate("home", { extra: "value" }),
    new Error('Unexpected navigation parameter "extra" for route "home".'),
  );
  assert.throws(
    () => navigate("guide", { path: "" }),
    new Error('Invalid navigation parameter "path" for route "guide".'),
  );
  assert.throws(
    () => navigate("photo", { userId: Number.NaN, photoId: 1 }),
    new Error('Invalid navigation parameter "userId" for route "photo".'),
  );
  assert.throws(
    () => navigate("photo", { userId: true, photoId: 1 } as unknown as Record<string, string>),
    new Error('Invalid navigation parameter "userId" for route "photo".'),
  );
  assert.equal(visits, 0);
});

void test("a resolved path never starts with two slashes or holds a dot segment", () => {
  const rooted = defineApplicationMarquette({
    application: "navigation-rest-fixture",
    targets: ["native"],
    environments: [{ id: "device", target: "native", consumer: "client", runtime: "native" }],
    routes: [
      { id: "any", path: "/*path", environment: "device", rendering: "native" },
      { id: "item", path: "/items/:itemId", environment: "device", rendering: "native" },
    ],
  } as const);
  let visits = 0;
  const navigator = createMarquetteNavigator(rooted, (target) => {
    visits += 1;
    return target.path;
  });

  assert.equal(navigator.navigate("any", { path: "/other.example/start" }), "/other.example/start");
  assert.equal(navigator.navigate("any", { path: "//other.example//a" }), "/other.example//a");
  assert.equal(navigator.navigate("any", { path: "guide/.../start" }), "/guide/.../start");
  assert.equal(navigator.navigate("item", { itemId: "..." }), "/items/...");
  assert.equal(visits, 4);

  for (const path of ["/", "//", "..", "../items/1", "guide/./start", "guide/.."]) {
    assert.throws(
      () => navigator.navigate("any", { path }),
      new Error('Invalid navigation parameter "path" for route "any".'),
      path,
    );
  }
  for (const itemId of [".", ".."]) {
    assert.throws(
      () => navigator.navigate("item", { itemId }),
      new Error('Invalid navigation parameter "itemId" for route "item".'),
      itemId,
    );
  }
  assert.equal(visits, 4);
});

const photoParameters: NavigationParameters<typeof marquette, "photo"> = {
  userId: "user",
  photoId: 1,
};
void photoParameters;

const typedNavigator = createMarquetteNavigator(marquette, (target) => target);
function assertNavigationTypes(): void {
  typedNavigator.navigate("home");
  typedNavigator.navigate("photo", { userId: "user", photoId: 1 });
  typedNavigator.navigate("guide", { path: "intro/start" });
  // @ts-expect-error Static routes do not accept parameters.
  typedNavigator.navigate("home", {});
  // @ts-expect-error Parameterized routes require their exact parameter object.
  typedNavigator.navigate("photo", { userId: "user" });
  // @ts-expect-error Route identifiers come from the authored marquette.
  typedNavigator.navigate("missing");
}
void assertNavigationTypes;
