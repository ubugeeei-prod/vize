import assert from "node:assert/strict";
import { test } from "node:test";

import { defineApplicationMarquette } from "./author.js";
import { admitNativeBuildProfile } from "./native-build-admission.js";
import {
  defineNativeApplicationBuildProfile,
  type NativeApplicationBuildProfile,
  type NativeApplicationEnvironmentId,
  type NativeBuildProfile,
} from "./native-build.js";

const marquette = defineApplicationMarquette({
  application: "native-admission",
  targets: ["native", "web"],
  capabilities: {
    "native.render": { id: "native.render", description: "Render native views", version: 2 },
    "native.camera": { id: "native.camera", description: "Capture images" },
    "native.notifications": { id: "native.notifications", description: "Deliver notifications" },
    "web.storage": { id: "web.storage", description: "Persist browser preferences" },
  },
  environments: [
    {
      id: "device",
      target: "native",
      consumer: "client",
      runtime: "native",
      capabilities: ["native.render"],
    },
    { id: "browser", target: "web", consumer: "client", runtime: "browser" },
    { id: "service", target: "native", consumer: "server", runtime: "javascript" },
  ],
  routes: [
    {
      id: "camera",
      path: "/camera",
      environment: "device",
      rendering: "native",
      capabilities: ["native.camera"],
    },
    {
      id: "web",
      path: "/",
      environment: "browser",
      rendering: "client",
      capabilities: ["web.storage"],
    },
  ],
} as const);

const profile = defineNativeApplicationBuildProfile(marquette, {
  format: "vize.native-build-profile",
  formatVersion: 1,
  profile: "production",
  configuration: "production",
  environment: "device",
  targets: [{ platform: "ios", destination: "store", signing: "automatic" }],
  capabilities: ["native.notifications", "native.render"],
});

const adapter = {
  adapter: "native.fixture",
  capabilities: [
    { id: "native.render", minVersion: 1, maxVersion: 2 },
    { id: "native.camera", minVersion: 1, maxVersion: 1 },
    { id: "native.notifications", minVersion: 1, maxVersion: 1 },
  ],
};

void test("admits native builds only after profile, application, and adapter contracts pass", () => {
  assert.deepEqual(admitNativeBuildProfile(marquette, profile, adapter), {
    allowed: true,
    profile: "production",
    environment: "device",
    adapter: "native.fixture",
    capabilities: ["native.camera", "native.notifications", "native.render"],
    diagnostics: [],
  });
});

void test("enforces capabilities required only by routes in the selected environment", () => {
  const result = admitNativeBuildProfile(marquette, profile, {
    ...adapter,
    capabilities: adapter.capabilities.filter(({ id }) => id !== "native.camera"),
  });
  assert.equal(result.allowed, false);
  assert.deepEqual(result.capabilities, ["native.camera", "native.notifications", "native.render"]);
  assert.deepEqual(result.diagnostics, [
    {
      source: "capability",
      severity: "error",
      code: "missing-capability",
      capability: "native.camera",
      path: "capabilities.native.camera",
      requiredVersion: 1,
      message: "adapter does not support the required capability",
    },
  ]);
});

void test("refuses unknown, web, and server environment bindings", () => {
  for (const [environment, code] of [
    ["missing", "unknown-environment"],
    ["browser", "environment-not-native"],
    ["service", "environment-not-client"],
  ]) {
    const result = admitNativeBuildProfile(
      marquette,
      { ...profile, environment: environment! },
      adapter,
    );
    assert.equal(result.allowed, false);
    assert.ok(
      result.diagnostics.some((finding) => finding.source === "binding" && finding.code === code),
    );
  }
});

void test("retains standalone signing failures and application failures without scheduling a build", () => {
  const result = admitNativeBuildProfile(
    { ...marquette, application: "" },
    {
      ...profile,
      targets: [{ platform: "ios", destination: "store", signing: "none" }],
    },
    adapter,
  );
  assert.equal(result.allowed, false);
  assert.deepEqual(
    result.diagnostics.map(({ source, code }) => ({ source, code })),
    [
      { source: "application", code: "VIZE_MARQUETTE_002" },
      { source: "profile", code: "store-requires-signing" },
    ],
  );
});

void test("fails closed on invalid adapter manifests and unsupported versions", () => {
  const invalid = admitNativeBuildProfile(marquette, profile, {
    ...adapter,
    capabilities: [{ id: "native.render", minVersion: 3, maxVersion: 1 }],
  });
  assert.equal(invalid.allowed, false);
  assert.ok(
    invalid.diagnostics.some(
      ({ source, code }) => source === "adapter" && code === "invalid-version-range",
    ),
  );

  const incompatible = admitNativeBuildProfile(marquette, profile, {
    ...adapter,
    capabilities: adapter.capabilities.map((support) => ({ ...support, maxVersion: 1 })),
  });
  assert.equal(incompatible.allowed, false);
  assert.ok(
    incompatible.diagnostics.some(
      ({ source, code }) => source === "capability" && code === "version-above-maximum",
    ),
  );
});

void test("adapter-only support cannot authorize undeclared profile capabilities", () => {
  const result = admitNativeBuildProfile(
    marquette,
    {
      ...profile,
      capabilities: ["native.private"],
    },
    {
      ...adapter,
      capabilities: [
        ...adapter.capabilities,
        { id: "native.private", minVersion: 1, maxVersion: 1 },
      ],
    },
  );
  assert.equal(result.allowed, false);
  assert.ok(
    result.diagnostics.some(
      ({ source, code }) => source === "capability" && code === "unknown-requirement",
    ),
  );
});

void test("inherited object members cannot authorize a profile capability", () => {
  const result = admitNativeBuildProfile(
    { application: "inherited", capabilities: {} },
    {
      ...profile,
      capabilities: ["constructor"],
    },
    {
      adapter: "native.fixture",
      capabilities: [{ id: "constructor", minVersion: 1, maxVersion: 1 }],
    },
  );
  assert.equal(result.allowed, false);
  assert.ok(
    result.diagnostics.some(
      ({ source, code }) => source === "capability" && code === "unknown-requirement",
    ),
  );
});

void test("strict admission parsing refuses unsafe numeric bounds and hidden executor fields", () => {
  for (const bound of [Number.NaN, Number.POSITIVE_INFINITY, 1.5, Number.MAX_SAFE_INTEGER + 1]) {
    assert.throws(
      () =>
        admitNativeBuildProfile(marquette, profile, {
          adapter: "native.fixture",
          capabilities: [{ id: "native.render", minVersion: bound, maxVersion: bound }],
        }),
      TypeError,
    );
  }
  for (const field of ["secret", "executor"]) {
    assert.throws(
      () => admitNativeBuildProfile(marquette, { ...profile, [field]: "private" }, adapter),
      TypeError,
    );
  }
  assert.throws(
    () =>
      admitNativeBuildProfile(
        marquette,
        {
          ...profile,
          targets: [{ platform: "ios", destination: "store", signing: "bogus" }],
        } as unknown as NativeBuildProfile,
        adapter,
      ),
    TypeError,
  );
});

void test("decisions remain deterministic when independent contract collections are reordered", () => {
  const refusedAdapter = { ...adapter, capabilities: [] };
  const expected = admitNativeBuildProfile(marquette, profile, refusedAdapter);
  const actual = admitNativeBuildProfile(
    {
      ...marquette,
      environments: [...marquette.environments].reverse(),
      routes: [...marquette.routes].reverse(),
    },
    {
      ...profile,
      capabilities: [...profile.capabilities].reverse(),
    },
    refusedAdapter,
  );
  assert.deepEqual(actual, expected);
});

void test("the authoring helper preserves exact literals and returns the original profile", () => {
  assert.equal(defineNativeApplicationBuildProfile(marquette, profile), profile);
  const environment: "device" = profile.environment;
  const platform: "ios" = profile.targets[0].platform;
  void environment;
  void platform;
});

void test("admission leaves deeply frozen input contracts unchanged", () => {
  function freeze<Value>(value: Value): Value {
    if (value && typeof value === "object") {
      for (const nested of Object.values(value)) freeze(nested);
      Object.freeze(value);
    }
    return value;
  }
  const frozen = freeze(structuredClone({ marquette, profile, adapter }));
  assert.equal(
    admitNativeBuildProfile(frozen.marquette, frozen.profile, frozen.adapter).allowed,
    true,
  );
  assert.deepEqual(frozen, { marquette, profile, adapter });
});

const environment: NativeApplicationEnvironmentId<typeof marquette> = "device";
const applicationProfile: NativeApplicationBuildProfile<typeof marquette> = profile;
const portableProfile: NativeBuildProfile = applicationProfile;
void environment;
void portableProfile;

function assertBuildTypes(): void {
  // @ts-expect-error Browser environments cannot own native application builds.
  const webEnvironment: NativeApplicationEnvironmentId<typeof marquette> = "browser";
  // @ts-expect-error Server environments cannot own native application builds.
  const serverEnvironment: NativeApplicationEnvironmentId<typeof marquette> = "service";
  void webEnvironment;
  void serverEnvironment;
  defineNativeApplicationBuildProfile(marquette, {
    ...profile,
    // @ts-expect-error Capabilities must be declared by the application.
    capabilities: ["native.missing"],
  });
  defineNativeApplicationBuildProfile(marquette, {
    ...profile,
    // @ts-expect-error Build environments must be native client environments.
    environment: "browser",
  });
}
void assertBuildTypes;
