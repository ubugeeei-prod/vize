import assert from "node:assert/strict";
import { test } from "node:test";

import { resolveExperimentalCompilerOptions, resolveExperimentalOptions } from "./experimentals.ts";

void test("experimental Vue RFC flags resolve from shared config", () => {
  const resolved = resolveExperimentalOptions(undefined, {
    intagComment: {},
    patternedTemplate: true,
    selfComponent: {},
    strictSlotChildren: true,
  });

  assert.equal(resolved.inTagComments, true);
  assert.equal(resolved.patternedTemplate, true);
  assert.equal(resolved.selfComponent, true);
  assert.equal(resolved.strictSlotChildren, true);
});

void test("direct experimental values can disable shared config defaults", () => {
  const resolved = resolveExperimentalOptions(
    {
      inTagComment: false,
      patternedTemplate: null,
      selfComponent: false,
      strictSlotChildren: null,
    },
    {
      intagComment: {},
      patternedTemplate: true,
      selfComponent: {},
      strictSlotChildren: true,
    },
  );

  assert.equal(resolved.inTagComments, false);
  assert.equal(resolved.patternedTemplate, false);
  assert.equal(resolved.selfComponent, false);
  assert.equal(resolved.strictSlotChildren, false);
});

void test("experimental compiler options expose the RFC flags", () => {
  const resolved = resolveExperimentalCompilerOptions(
    {
      experimentals: {
        intagComment: true,
        patternedTemplate: true,
        selfComponent: true,
        strictSlotChildren: true,
      },
    },
    {},
    undefined,
  );

  assert.equal(resolved.experimentalInTagComments, true);
  assert.equal(resolved.experimentalPatternedTemplate, true);
  assert.equal(resolved.experimentalSelfComponent, true);
  assert.equal(resolved.experimentalStrictSlotChildren, true);
});
