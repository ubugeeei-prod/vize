import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

import type { ApplicationMarquette } from "./model.js";
import { validateApplicationMarquette } from "./validate.js";

const fixture = (name: string) =>
  new URL(`../../../tests/fixtures/marquette/${name}`, import.meta.url);

void test("accepts the shared cross-language marquette fixture", async () => {
  const marquette = JSON.parse(
    await readFile(fixture("valid.json"), "utf8"),
  ) as ApplicationMarquette;

  assert.deepEqual(validateApplicationMarquette(marquette), []);
});

void test("matches shared invalid diagnostic-code fixtures", async () => {
  const marquette = JSON.parse(
    await readFile(fixture("invalid.json"), "utf8"),
  ) as ApplicationMarquette;
  const expected = JSON.parse(await readFile(fixture("invalid.expected.json"), "utf8")) as string[];

  assert.deepEqual(
    validateApplicationMarquette(marquette).map((diagnostic) => diagnostic.code),
    expected,
  );
});

void test("reports empty descriptions and identifies the previous duplicate route", () => {
  const diagnostics = validateApplicationMarquette({
    application: "shop",
    targets: ["web"],
    capabilities: {
      "auth.session": {
        id: "auth.session",
        description: "   ",
      },
    },
    environments: [
      {
        id: "browser",
        target: "web",
        consumer: "client",
        runtime: "browser",
      },
    ],
    routes: [
      {
        id: "home",
        path: "/",
        environment: "browser",
        rendering: "client",
      },
      {
        id: "landing",
        path: "/",
        environment: "browser",
        rendering: "client",
      },
    ],
  });

  assert.ok(
    diagnostics.some(
      (diagnostic) =>
        diagnostic.code === "VIZE_MARQUETTE_024" &&
        diagnostic.message === "capability description must not be empty",
    ),
  );
  assert.ok(
    diagnostics.some(
      (diagnostic) =>
        diagnostic.code === "VIZE_MARQUETTE_019" &&
        diagnostic.message === 'route path is already used by route "home"',
    ),
  );
});

void test("reports cycles, incompatible rendering, capabilities, and suspicious runtimes", () => {
  const diagnostics = validateApplicationMarquette({
    application: "workbench",
    targets: ["web", "native"],
    environments: [
      {
        id: "first",
        target: "web",
        consumer: "client",
        runtime: "rust",
        dependsOn: ["second"],
        capabilities: ["missing"],
      },
      {
        id: "second",
        target: "web",
        consumer: "server",
        runtime: "javascript",
        dependsOn: ["first"],
      },
    ],
    routes: [
      {
        id: "home",
        path: "/",
        environment: "first",
        rendering: "native",
      },
    ],
  });
  const codes = new Set(diagnostics.map((diagnostic) => diagnostic.code));

  assert.ok(codes.has("VIZE_MARQUETTE_010"));
  assert.ok(codes.has("VIZE_MARQUETTE_020"));
  assert.ok(codes.has("VIZE_MARQUETTE_021"));
  assert.ok(codes.has("VIZE_MARQUETTE_022"));
  assert.ok(codes.has("VIZE_MARQUETTE_023"));
});

void test("returns diagnostics in stable path, code, and message order", () => {
  const marquette: ApplicationMarquette = {
    application: "Broken App",
    targets: ["web"],
    environments: [
      {
        id: "client",
        target: "native",
        consumer: "client",
        runtime: "browser",
        dependsOn: ["missing"],
      },
    ],
  };

  assert.deepEqual(
    validateApplicationMarquette(marquette),
    validateApplicationMarquette(structuredClone(marquette)),
  );
  assert.deepEqual(
    validateApplicationMarquette(marquette).map(({ path, code }) => [path, code]),
    [
      ["application", "VIZE_MARQUETTE_002"],
      ["environments.client", "VIZE_MARQUETTE_007"],
      ["environments.client", "VIZE_MARQUETTE_009"],
      ["targets", "VIZE_MARQUETTE_020"],
    ],
  );
});

void test("normalizes set-backed references before producing diagnostics", () => {
  const diagnostics = validateApplicationMarquette({
    application: "deduplicated",
    targets: ["web"],
    environments: [
      {
        id: "server",
        target: "web",
        consumer: "server",
        runtime: "javascript",
        dependsOn: ["missing", "missing"],
        capabilities: ["missing", "missing"],
      },
    ],
  });

  assert.equal(diagnostics.filter(({ code }) => code === "VIZE_MARQUETTE_009").length, 1);
  assert.equal(diagnostics.filter(({ code }) => code === "VIZE_MARQUETTE_021").length, 1);
});

void test("covers every stable validation diagnostic", () => {
  const marquette = {
    formatVersion: 2,
    application: "Broken App",
    targets: ["native"],
    capabilities: {
      "Bad Key": {
        id: "different",
        description: "",
        version: 0,
      },
    },
    environments: [
      {
        id: "client",
        target: "web",
        consumer: "client",
        runtime: "rust",
        dependsOn: ["server", "missing"],
        capabilities: ["missing"],
      },
      {
        id: "server",
        target: "web",
        consumer: "server",
        runtime: "javascript",
        dependsOn: ["client"],
      },
      {
        id: "self",
        target: "web",
        consumer: "server",
        runtime: "javascript",
        dependsOn: ["self"],
      },
    ],
    backends: [
      {
        id: "missing-owner",
        family: "external",
        environment: "missing",
        capabilities: ["missing"],
      },
      { id: "client-owner", family: "external", environment: "client" },
      { id: "api-a", family: "external" },
      { id: "api-b", family: "external" },
    ],
    protocols: [
      { id: "orphan", family: "schema-query", backend: "missing" },
      { id: "api.query", family: "schema-query", backend: "api-a" },
    ],
    routes: [
      {
        id: "broken",
        path: "broken",
        environment: "missing",
        rendering: "native",
        backend: "missing",
        protocol: "missing",
        capabilities: ["missing"],
      },
      {
        id: "mismatch",
        path: "/",
        environment: "client",
        rendering: "native",
        backend: "api-b",
        protocol: "api.query",
      },
      {
        id: "mismatch",
        path: "/",
        environment: "client",
        rendering: "client",
      },
    ],
  } as unknown as ApplicationMarquette;

  assert.deepEqual(
    [...new Set(validateApplicationMarquette(marquette).map(({ code }) => code))].sort(),
    Array.from(
      { length: 24 },
      (_, index) => `VIZE_MARQUETTE_${String(index + 1).padStart(3, "0")}`,
    ),
  );
});
