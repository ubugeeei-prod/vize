import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

import { defineNativeBuildProfile, type NativeBuildProfile } from "./native-build.js";
import { parseNativeBuildProfile, validateNativeBuildProfile } from "./native-build-validate.js";
import type { NativeBuildProfileDiagnostic } from "./native-build-model.js";

const fixture = (name: string) =>
  new URL(`../../../tests/fixtures/marquette/${name}`, import.meta.url);

async function read<T>(name: string): Promise<T> {
  return JSON.parse(await readFile(fixture(name), "utf8")) as T;
}

void test("accepts the shared production profile without mutation", async () => {
  const profile = await read<NativeBuildProfile>("native-build-profile.valid.json");
  const before = structuredClone(profile);

  assert.equal(parseNativeBuildProfile(profile), profile);
  assert.deepEqual(validateNativeBuildProfile(profile), []);
  assert.deepEqual(profile, before);
  assert.equal(defineNativeBuildProfile(profile), profile);
});

void test("matches shared semantic diagnostics in stable path order", async () => {
  const profile = await read<NativeBuildProfile>("native-build-profile.invalid.json");
  const expected = await read<readonly NativeBuildProfileDiagnostic[]>(
    "native-build-profile.invalid.expected.json",
  );

  assert.deepEqual(validateNativeBuildProfile(profile), expected);
});

void test("requires at least one native build target", () => {
  const profile = {
    format: "vize.native-build-profile",
    formatVersion: 1,
    profile: "development",
    configuration: "development",
    environment: "native",
    targets: [],
  } as const satisfies NativeBuildProfile;

  assert.deepEqual(validateNativeBuildProfile(profile), [
    {
      code: "missing-target",
      path: "targets",
      message: "native build profile must declare a target",
    },
  ]);
});

void test("strict parsing rejects unknown fields and unsupported enum values", () => {
  assert.throws(
    () =>
      parseNativeBuildProfile({
        format: "vize.native-build-profile",
        formatVersion: 1,
        profile: "development",
        configuration: "development",
        environment: "native",
        targets: [],
        executor: "hosted",
      }),
    /profile has unknown field executor/,
  );
  assert.throws(
    () =>
      parseNativeBuildProfile({
        format: "vize.native-build-profile",
        formatVersion: 1,
        profile: "development",
        configuration: "debug",
        environment: "native",
        targets: [],
      }),
    /configuration must be a supported string value/,
  );
  assert.throws(
    () =>
      parseNativeBuildProfile({
        format: "vize.native-build-profile",
        formatVersion: 1,
        profile: "development",
        configuration: "development",
        environment: "native",
        targets: [{ platform: "ios", destination: "simulator", secret: "value" }],
      }),
    /targets\.0 has unknown field secret/,
  );
});
