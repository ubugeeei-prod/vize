import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";
import { test } from "node:test";

import { createRouter, createRouterMatcher, defineRoutes } from "./index.ts";

void test("records cold match, warm navigation and large-table evidence", async (context) => {
  const routes = defineRoutes(
    Array.from({ length: 1_000 }, (_, index) => ({
      name: `route-${index}`,
      path: `/groups/${index}/items/:id`,
      component: `./routes/route-${index}.vue` as `${string}.vue`,
    })),
  );

  const coldStart = performance.now();
  const matcher = createRouterMatcher(routes);
  const coldMilliseconds = performance.now() - coldStart;

  const warmStart = performance.now();
  let checksum = 0;
  for (let index = 0; index < 10_000; index += 1) {
    const route = index % routes.length;
    checksum += matcher.match(`/groups/${route}/items/${index}`)?.pathname.length ?? 0;
  }
  const warmMatchMilliseconds = performance.now() - warmStart;

  const router = createRouter(routes);
  const navigationStart = performance.now();
  for (let index = 0; index < 100; index += 1) {
    const result = await router.navigate(`/groups/${index}/items/${index}`, { history: "none" });
    assert.equal(result.status, "success");
  }
  const warmNavigationMilliseconds = performance.now() - navigationStart;

  assert.ok(checksum > 0);
  context.diagnostic(
    JSON.stringify({
      schemaVersion: 1,
      routes: routes.length,
      coldMilliseconds,
      warmMatches: 10_000,
      warmMatchMilliseconds,
      warmNavigations: 100,
      warmNavigationMilliseconds,
    }),
  );
});
