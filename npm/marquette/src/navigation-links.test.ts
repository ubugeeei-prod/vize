import assert from "node:assert/strict";
import { test } from "node:test";

import { defineApplicationMarquette } from "./author.js";
import { createMarquetteNavigator } from "./navigation.js";
import { createMarquetteLinkResolver } from "./navigation-links.js";

const marquette = defineApplicationMarquette({
  application: "application-links",
  targets: ["native", "web"],
  environments: [
    { id: "device", target: "native", consumer: "client", runtime: "native" },
    { id: "browser", target: "web", consumer: "client", runtime: "browser" },
  ],
  routes: [
    { id: "home", path: "/", environment: "device", rendering: "native" },
    { id: "new", path: "/users/new", environment: "device", rendering: "native" },
    { id: "user", path: "/users/:userId", environment: "device", rendering: "native" },
    { id: "guide", path: "/guides/*path", environment: "device", rendering: "native" },
    { id: "web", path: "/web-only", environment: "browser", rendering: "client" },
  ],
} as const);

const resolver = createMarquetteLinkResolver(marquette, {
  environment: "device",
  origins: ["https://app.example", "example://app"],
});

void test("resolves HTTPS and native links to inferred canonical navigation targets", () => {
  assert.deepEqual(resolver.resolve("https://app.example"), {
    matched: true,
    target: { route: "home", environment: "device", path: "/", parameters: {} },
  });
  assert.deepEqual(resolver.resolve("example://app/users/a%20user?ignored=1#details"), {
    matched: true,
    target: {
      route: "user",
      environment: "device",
      path: "/users/a%20user",
      parameters: { userId: "a user" },
    },
  });
  assert.deepEqual(resolver.resolve("https://APP.example:443/users/%e6%97%a5%e6%9c%ac"), {
    matched: true,
    target: {
      route: "user",
      environment: "device",
      path: "/users/%E6%97%A5%E6%9C%AC",
      parameters: { userId: "日本" },
    },
  });
});

void test("round-trips path parameters and catch-all paths through programmatic navigation", () => {
  const navigator = createMarquetteNavigator(marquette, (target) => target);
  for (const userId of ["a b", "日本語", "a/b", "%2e%2e", "...", "100%", "?query#hash"]) {
    const target = navigator.navigate("user", { userId });
    assert.deepEqual(resolver.resolve(`example://app${target.path}`), {
      matched: true,
      target: { ...target, parameters: { userId } },
    });
  }
  for (const path of ["intro/start", "日本語/詳細", "a//b/", "%2F/start", "a b"]) {
    const target = navigator.navigate("guide", { path });
    assert.deepEqual(resolver.resolve(`https://app.example${target.path}`), {
      matched: true,
      target: { ...target, parameters: { path } },
    });
  }
});

void test("matches static segments before parameters independently of route declaration order", () => {
  const reversed = createMarquetteLinkResolver(
    { ...marquette, routes: [...marquette.routes].reverse() },
    {
      environment: "device",
      origins: ["https://app.example"],
    },
  );
  for (const candidate of [resolver, reversed]) {
    assert.deepEqual(candidate.resolve("https://app.example/users/new"), {
      matched: true,
      target: { route: "new", environment: "device", path: "/users/new", parameters: {} },
    });
  }
});

void test("only the selected environment and complete origin can admit a link", () => {
  for (const link of [
    "https://other.example/users/one",
    "https://app.example.evil/users/one",
    "https://app.example:444/users/one",
    "http://app.example/users/one",
    "example://other/users/one",
  ]) {
    assert.deepEqual(resolver.resolve(link), { matched: false, code: "untrusted-origin" }, link);
  }
  for (const link of [
    "https://app.example/web-only",
    "https://app.example/missing",
    "example://app/users/",
  ]) {
    assert.deepEqual(resolver.resolve(link), { matched: false, code: "unknown-route" }, link);
  }
});

void test("rejects credentials, relative links, URL normalization tricks, and malformed authorities", () => {
  for (const link of [
    "/users/one",
    "//app.example/users/one",
    "https:/app.example/users/one",
    " https://app.example/users/one",
    "https://app.example/users/one ",
    "https://user@app.example/users/one",
    "https://@app.example/users/one",
    "https://user:secret@app.example/users/one",
    "https://app.example\\@evil.example/users/one",
    "https://app.example/\tusers/one",
    "https://app.example/\nusers/one",
    "https://app.example:bad/users/one",
    "example:///users/one",
  ]) {
    assert.deepEqual(resolver.resolve(link), { matched: false, code: "invalid-link" }, link);
  }
});

void test("rejects traversal and malformed path escaping before URL normalization", () => {
  for (const path of [
    "/users/../users/one",
    "/users/./one",
    "/users/%2e%2e/users/one",
    "/users/.%2E/users/one",
    "/users/%2E",
    "/users/%",
    "/users/%C0%AE",
    "/users/%ED%A0%80",
    "/users/%00",
    "/users/%7F",
    "/users/%C2%85",
    "/users/%5C",
    "/users/\ud800",
    "//users/one",
  ]) {
    assert.deepEqual(
      resolver.resolve(`https://app.example${path}`),
      {
        matched: false,
        code: "invalid-path",
      },
      path,
    );
  }
});

void test("catch-all parameters cannot reinterpret encoded separators or leading empty segments", () => {
  for (const path of ["/guides/%2Fintro", "/guides/intro%2Fstart", "/guides//intro"]) {
    assert.deepEqual(resolver.resolve(`example://app${path}`), {
      matched: false,
      code: "unknown-route",
    });
  }
});

void test("refuses equally specific routes instead of choosing by declaration order", () => {
  const ambiguous = createMarquetteLinkResolver(
    {
      ...marquette,
      routes: [
        { id: "first", path: "/:first", environment: "device", rendering: "native" },
        { id: "second", path: "/:second", environment: "device", rendering: "native" },
      ],
    } as const,
    { environment: "device", origins: ["example://app"] },
  );
  assert.deepEqual(ambiguous.resolve("example://app/one"), {
    matched: false,
    code: "ambiguous-route",
  });
});

void test("validates origin policy and unsupported route templates at construction", () => {
  assert.throws(
    () => createMarquetteLinkResolver(marquette, { environment: "device", origins: [] }),
    TypeError,
  );
  for (const origin of [
    "http://app.example",
    "https://app.example/base",
    "https://app.example?",
    "https://app.example#",
    "https://app.example/?search=1",
    "https://app.example/#fragment",
    "javascript://app",
    "file://app",
    "https://user@app.example",
    "app:///",
    "app:",
  ]) {
    assert.throws(
      () => createMarquetteLinkResolver(marquette, { environment: "device", origins: [origin] }),
      TypeError,
      origin,
    );
  }
  for (const path of ["/:", "/*", "/:id/:id", "/a/*rest/end", "/a/../b", "/a?b", "/a#b", "/%zz"]) {
    assert.throws(
      () =>
        createMarquetteLinkResolver(
          {
            ...marquette,
            routes: [
              {
                id: "invalid",
                path: path as `/${string}`,
                environment: "device",
                rendering: "native",
              },
            ],
          } as const,
          { environment: "device", origins: ["example://app"] },
        ),
      TypeError,
      path,
    );
  }
});

void test("snapshots origin policy and route metadata without mutating caller-owned records", () => {
  const routes = [
    { id: "one", path: "/:id" as const, environment: "device", rendering: "native" as const },
  ];
  const origins = ["example://app"];
  const local = createMarquetteLinkResolver(
    { ...marquette, routes },
    { environment: "device", origins },
  );
  routes[0]!.id = "changed";
  origins[0] = "example://other";
  assert.deepEqual(local.resolve("example://app/value"), {
    matched: true,
    target: { route: "one", environment: "device", path: "/value", parameters: { id: "value" } },
  });
  assert.equal(routes[0]!.id, "changed");
  assert.equal(origins[0], "example://other");
});

void test("route identifiers remain globally unique across application environments", () => {
  assert.throws(
    () =>
      createMarquetteLinkResolver(
        {
          ...marquette,
          routes: [
            ...marquette.routes,
            { id: "user", path: "/browser-user", environment: "browser", rendering: "client" },
          ],
        } as const,
        { environment: "device", origins: ["example://app"] },
      ),
    /unique within the application/,
  );
});

void test("large untrusted catch-all paths resolve without overflowing argument limits", () => {
  const path = Array.from({ length: 150_000 }, () => "x").join("/");
  const result = resolver.resolve(`example://app/guides/${path}`);
  assert.equal(result.matched, true);
  if (result.matched && result.target.route === "guide") {
    assert.equal(result.target.parameters.path, path);
    assert.equal(result.target.path, `/guides/${path}`);
  } else {
    assert.fail("the catch-all route must remain selected");
  }
});

function assertLinkTypes(): void {
  const result = resolver.resolve("example://app/users/one");
  if (result.matched) {
    const environment: "device" = result.target.environment;
    void environment;
    if (result.target.route === "user") {
      const userId: string = result.target.parameters.userId;
      void userId;
      // @ts-expect-error Parameters remain tied to the selected route.
      void result.target.parameters.path;
    }
    // @ts-expect-error The selected environment excludes browser-only routes.
    const route: "web" = result.target.route;
    void route;
  }
  // @ts-expect-error The environment comes from the authored application contract.
  createMarquetteLinkResolver(marquette, { environment: "missing", origins: [] });
}
void assertLinkTypes;
