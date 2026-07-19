import assert from "node:assert/strict";
import { test } from "node:test";

import {
  defineApplicationMarquette,
  type BackendId,
  type CapabilityId,
  type EnvironmentId,
  type ProtocolId,
  type RouteId,
  type ServerEnvironmentId,
  type TargetId,
} from "./index.js";

const marquette = defineApplicationMarquette({
  application: "shop",
  targets: ["web"],
  capabilities: {
    "auth.session": {
      id: "auth.session",
      description: "Authenticated application session",
    },
  },
  environments: [
    { id: "web", target: "web", consumer: "client", runtime: "browser" },
    { id: "server", target: "web", consumer: "server", runtime: "rust" },
  ],
  backends: [{ id: "api", family: "rust", environment: "server" }],
  protocols: [{ id: "api.query", family: "schema-query", backend: "api" }],
  routes: [
    {
      id: "home",
      path: "/",
      environment: "web",
      rendering: "hybrid",
      backend: "api",
      protocol: "api.query",
      capabilities: ["auth.session"],
    },
  ],
} as const);

void test("preserves exact identifier unions without runtime allocation", () => {
  const environment: EnvironmentId<typeof marquette> = "web";
  const backend: BackendId<typeof marquette> = "api";
  const protocol: ProtocolId<typeof marquette> = "api.query";
  const route: RouteId<typeof marquette> = "home";
  const server: ServerEnvironmentId<typeof marquette> = "server";
  const target: TargetId<typeof marquette> = "web";
  const capability: CapabilityId<typeof marquette> = "auth.session";

  assert.equal(environment, "web");
  assert.equal(backend, "api");
  assert.equal(protocol, "api.query");
  assert.equal(route, "home");
  assert.equal(server, "server");
  assert.equal(target, "web");
  assert.equal(capability, "auth.session");
  assert.equal(marquette.application, "shop");
});

void test("accepts self-contained environment dependency references", () => {
  const dependent = defineApplicationMarquette({
    application: "docs",
    targets: ["web"],
    environments: [
      { id: "server", target: "web", consumer: "server", runtime: "rust" },
      {
        id: "web",
        target: "web",
        consumer: "client",
        runtime: "browser",
        dependsOn: ["server"],
      },
    ],
  } as const);

  assert.deepEqual(dependent.environments[1].dependsOn, ["server"]);
});

defineApplicationMarquette({
  application: "invalid",
  // @ts-expect-error Backend environments must reference an authored environment.
  backends: [{ id: "api", family: "rust", environment: "missing" }],
} as const);

defineApplicationMarquette({
  application: "invalid",
  // @ts-expect-error Protocols must reference an authored backend.
  protocols: [{ id: "api.query", family: "schema-query", backend: "missing" }],
} as const);

defineApplicationMarquette({
  application: "invalid",
  // @ts-expect-error Routes must reference an authored environment.
  routes: [{ id: "home", path: "/", environment: "missing", rendering: "client" }],
} as const);

defineApplicationMarquette({
  application: "invalid",
  targets: ["web"],
  // @ts-expect-error Environment targets must appear in the authored target list.
  environments: [{ id: "native", target: "native", consumer: "client", runtime: "native" }],
} as const);

defineApplicationMarquette({
  application: "invalid",
  targets: ["web"],
  capabilities: {
    session: { id: "session", description: "Session" },
  },
  // @ts-expect-error Required capabilities must be declared by key.
  environments: [
    {
      id: "web",
      target: "web",
      consumer: "client",
      runtime: "browser",
      capabilities: ["missing"],
    },
  ],
} as const);

defineApplicationMarquette({
  application: "invalid",
  targets: ["web"],
  environments: [{ id: "web", target: "web", consumer: "client", runtime: "browser" }],
  // @ts-expect-error Locally owned backends must reference a server environment.
  backends: [{ id: "api", family: "rust", environment: "web" }],
} as const);
