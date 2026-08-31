import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { collectVueInputPaths } from "../../legacy-tools/fixtures/tool-matrix-inputs.mjs";
import { resolveSfcDialectPartition, validateRouteShapes } from "./support/sfc-baseline-routes.ts";
import type { SfcDialectRoute } from "./support/sfc-baseline-routes.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const registry = JSON.parse(
  fs.readFileSync(path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
) as {
  projects: Array<{
    id: string;
    fixturePath: string;
    vueGlobs: string[];
    sfcDialectRoutes?: SfcDialectRoute[];
  }>;
};

test("GoGoCode's mixed Vue corpus is an exact dialect partition", () => {
  const record = registry.projects.find((project) => project.id === "gogocode");
  assert.ok(record);
  const fixtureDir = path.join(root, record.fixturePath);
  if (!fs.existsSync(fixtureDir) || fs.readdirSync(fixtureDir).length === 0) return;
  const files = collectVueInputPaths(fixtureDir, record.vueGlobs);
  const partition = resolveSfcDialectPartition({ ...record, fixtureDir }, files);
  const counts = { "2": 0, "3": 0 };
  for (const selected of partition.values()) counts[selected.dialect] += 1;

  assert.equal(files.length, 186);
  assert.equal(partition.size, 186);
  assert.deepEqual(counts, { "2": 97, "3": 89 });
  assert.equal(
    [...partition.keys()].filter((file) => file.startsWith("packages/gogocode-plugin-vue/test/"))
      .length,
    22,
  );
  for (const path of [
    "packages/gogocode-plugin-vue/test/key-attribute/Comp.vue",
    "packages/gogocode-plugin-vue/test/listeners-removed/Comp.vue",
  ]) {
    assert.deepEqual(partition.get(path), { routeId: "vue2", dialect: "2" });
  }
});

test("dialect routes reject missing, overlapping, and out-of-scope files", () => {
  withFixture(["src/vue2/A.vue", "src/vue3/B.vue", "outside/C.vue"], (fixtureDir) => {
    const base = {
      id: "synthetic",
      fixtureDir,
      vueGlobs: ["src/**/*.vue"],
    };
    const files = collectVueInputPaths(fixtureDir, base.vueGlobs);
    assert.equal(
      resolveSfcDialectPartition(
        {
          ...base,
          sfcDialectRoutes: [
            {
              id: "source",
              dialect: "2",
              globs: ["src/**/*.vue", "src/vue2/**/*.vue"],
            },
          ],
        },
        files,
      ).size,
      2,
      "overlapping globs inside one route must not look like two dialect routes",
    );
    assert.throws(
      () =>
        resolveSfcDialectPartition(
          {
            ...base,
            sfcDialectRoutes: [{ id: "vue2", dialect: "2", globs: ["src/vue2/**/*.vue"] }],
          },
          files,
        ),
      /has no dialect route: src\/vue3\/B\.vue/,
    );
    assert.throws(
      () =>
        resolveSfcDialectPartition(
          {
            ...base,
            sfcDialectRoutes: [
              { id: "all", dialect: "2", globs: ["src/**/*.vue"] },
              { id: "vue3", dialect: "3", globs: ["src/vue3/**/*.vue"] },
            ],
          },
          files,
        ),
      /overlapping dialect routes/,
    );
    assert.throws(
      () =>
        resolveSfcDialectPartition(
          {
            ...base,
            sfcDialectRoutes: [
              { id: "source", dialect: "2", globs: ["src/**/*.vue"] },
              { id: "outside", dialect: "3", globs: ["outside/**/*.vue"] },
            ],
          },
          files,
        ),
      /routed file is outside vueGlobs: outside\/C\.vue/,
    );
    assert.throws(
      () =>
        resolveSfcDialectPartition(
          {
            ...base,
            sfcDialectRoutes: [
              { id: "source", dialect: "2", globs: ["src/**/*.vue"] },
              { id: "empty", dialect: "3", globs: ["missing/**/*.vue"] },
            ],
          },
          files,
        ),
      /glob matched no files/,
    );
  });
});

test("dialect route schema is closed and fail-closed", () => {
  const valid: SfcDialectRoute[] = [{ id: "legacy", dialect: "0.10", globs: ["src/**/*.vue"] }];
  assert.doesNotThrow(() => validateRouteShapes(valid));
  for (const [routes, expected] of [
    [[], /sfcDialectRoutes must be a non-empty array/],
    [[null], /dialect routes must be objects/],
    [[{ id: "legacy", dialect: "4", globs: ["src/**/*.vue"] }], /invalid SFC dialect: 4/],
    [[{ id: "Legacy", dialect: "2", globs: ["src/**/*.vue"] }], /invalid dialect route id/],
    [[{ id: "legacy", dialect: "2", globs: [] }], /must declare globs/],
    [[{ id: "legacy", dialect: "2", globs: ["../src/**/*.vue"] }], /invalid SFC dialect glob/],
    [[{ id: "legacy", dialect: "2", globs: ["/src/**/*.vue"] }], /invalid SFC dialect glob/],
    [[{ id: "legacy", dialect: "2", globs: ["C:/src/**/*.vue"] }], /invalid SFC dialect glob/],
    [[{ id: "legacy", dialect: "2", globs: ["src/**/*.ts"] }], /invalid SFC dialect glob/],
    [[{ id: "legacy", dialect: "2", globs: ["src\\**\\*.vue"] }], /invalid SFC dialect glob/],
    [
      [{ id: "legacy", dialect: "2", globs: ["src/**/*.vue", "src/**/*.vue"] }],
      /duplicate SFC dialect glob/,
    ],
    [
      [
        { id: "legacy", dialect: "2", globs: ["src/**/*.vue"] },
        { id: "legacy", dialect: "3", globs: ["next/**/*.vue"] },
      ],
      /duplicate dialect route id: legacy/,
    ],
  ] as const) {
    assert.throws(() => validateRouteShapes(routes as unknown as SfcDialectRoute[]), expected);
  }
});

function withFixture(files: string[], run: (fixtureDir: string) => void): void {
  const fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-sfc-routes-"));
  try {
    for (const file of files) {
      fs.mkdirSync(path.dirname(path.join(fixtureDir, file)), { recursive: true });
      fs.writeFileSync(path.join(fixtureDir, file), "<template><p /></template>\n");
    }
    run(fixtureDir);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
  }
}
