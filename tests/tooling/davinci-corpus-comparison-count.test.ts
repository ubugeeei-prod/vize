import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import {
  buildArtifact,
  expectedComparisonCount,
  verifyScope,
} from "../../legacy-tools/davinci/lib/corpus-baseline-artifact.mjs";
import { requiredSection } from "./support/davinci-phase2-ledger.ts";

type ManifestProject = {
  id: string;
  fixturePath: string;
};

type Manifest = {
  projects: ManifestProject[];
};

function read(url: URL): string {
  return fs.readFileSync(url, "utf8");
}

function currentManifest(): Manifest {
  return JSON.parse(
    read(new URL("../../tests/_fixtures/vue-ecosystem-fixtures.json", import.meta.url)),
  );
}

function uniqueFixturePathCount(manifest: Manifest): number {
  return new Set(manifest.projects.map((project) => project.fixturePath)).size;
}

function duplicateFixtureGroups(manifest: Manifest): Array<[string, string[]]> {
  const byPath = new Map<string, string[]>();
  for (const project of manifest.projects) {
    const ids = byPath.get(project.fixturePath) ?? [];
    ids.push(project.id);
    byPath.set(project.fixturePath, ids);
  }
  return [...byPath.entries()].filter(([, ids]) => ids.length > 1);
}

function syntheticCompilerArtifact(projects: ManifestProject[]) {
  return buildArtifact(
    projects.map((project, index) => ({
      surface: "compiler",
      project: project.id,
      file_count: index + 1,
      content_hash: `${index}`.repeat(64).slice(0, 64),
    })),
    { projects },
  );
}

test("corpus comparison counts use project rows, not unique fixture paths", () => {
  const projects = [
    { id: "primevue", fixturePath: "tests/_fixtures/_git/primevue" },
    { id: "primevue-volt", fixturePath: "tests/_fixtures/_git/primevue" },
    { id: "primevue-showcase", fixturePath: "tests/_fixtures/_git/primevue" },
  ];
  const artifact = syntheticCompilerArtifact(projects);

  assert.equal(expectedComparisonCount({ projects }, ["compiler"]), 3);
  assert.deepEqual(verifyScope(artifact, { projects }, ["compiler"], "synthetic"), []);

  const stale = {
    ...artifact,
    scope: { ...artifact.scope, row_count: 2 },
    rows: artifact.rows.slice(0, 2),
  };
  assert.match(
    verifyScope(stale, { projects }, ["compiler"], "synthetic stale").join("\n"),
    /synthetic stale: 2 rows != 3 projects x 1 surfaces = 3/u,
  );
});

test("P2-11 DOM differential docs pin the hydrated full-corpus comparison count", () => {
  const manifest = currentManifest();
  const compilerComparisons = expectedComparisonCount(manifest, ["compiler"]);
  const fixturePaths = uniqueFixturePathCount(manifest);
  const duplicateGroups = duplicateFixtureGroups(manifest);

  assert.equal(compilerComparisons, 144, "intentional ratchet for the current manifest");
  assert.equal(fixturePaths, 142, "sanity check: fixture paths are not comparison rows");
  assert.deepEqual(duplicateGroups, [
    ["tests/_fixtures/_git/primevue", ["primevue", "primevue-volt", "primevue-showcase"]],
  ]);

  const phase = read(new URL("../../davinci-road/plan/phase-2.md", import.meta.url));
  const tasks = read(new URL("../../davinci-road/plan/phase-2-tasks.md", import.meta.url));
  const record = read(new URL("../../davinci-road/plan/phase-2-records/p2-11.md", import.meta.url));
  const countSection = requiredSection(
    record,
    /^## Hydrated corpus differential count contract/mu,
    /^## Not series installments/mu,
    "P2-11 hydrated corpus differential count contract",
  );

  for (const source of [phase, tasks, countSection]) {
    assert.match(source, /\b144 DOM-output comparisons\b/u);
  }
  assert.match(tasks, /\b144-project manifest\b/u);
  assert.doesNotMatch(tasks, /\b142-project manifest\b/u);
  assert.match(countSection, /\b142 ecosystem fixture\s+paths\b/u);
  assert.match(countSection, /`primevue`, `primevue-volt`, and `primevue-showcase`/u);
  assert.match(countSection, /\b142 DOM-output comparisons is stale\b/u);
  assert.match(countSection, /#5359/u);
  assert.match(countSection, /Real Project Matrix run `33531193323`/u);
  assert.match(countSection, /files=42,668/u);
  assert.match(countSection, /compared=42,279/u);
  assert.match(countSection, /divergences=0/u);
  assert.match(countSection, /production-lane switch is now closed/u);
  assert.doesNotMatch(countSection, /production-lane switch remains open/u);
  assert.doesNotMatch(countSection, /hydrated zero-divergence corpus run\s+is still a blocker/u);
  assert.doesNotMatch(countSection, /This is still a blocker/u);
});
