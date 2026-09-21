import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  check,
  compareIdentity,
  diagnosticIdentity,
  fixtureRoot,
  workspace,
} from "./support/upstream/vue-language-tools.ts";

import { prepareGeneralFixtures } from "./support/upstream/general-fixtures.ts";

type Manifest = {
  revision: string;
  projects: string[];
  expectedDiagnostics: Array<{ file: string; line: number; column: number; code: number }>;
  files: Array<{ path: string; sha256: string }>;
};
const manifest = JSON.parse(
  fs.readFileSync(path.join(fixtureRoot, "manifest.json"), "utf8"),
) as Manifest;

// Diagnostics Vize reports where the upstream oracle reports none, each owned
// by a reviewed reason. They are part of the exact expectation of their
// project: one that moves, disappears or gains a neighbour fails the corpus.
type DocumentedDifferences = {
  schema: string;
  version: number;
  revision: string;
  differences: Array<{
    id: string;
    issue: number;
    reason: string;
    vizeOnly: Manifest["expectedDiagnostics"];
  }>;
};
const documented = JSON.parse(
  fs.readFileSync(path.join(fixtureRoot, "documented-differences.json"), "utf8"),
) as DocumentedDifferences;
const vizeOnly = documented.differences.flatMap((difference) => difference.vizeOnly);
const identity = (d: Manifest["expectedDiagnostics"][number]) =>
  `${d.file}:${d.line}:${d.column}:${d.code}`;

test("the pinned Vue Language Tools corpus preserves every source and expected error", () => {
  assert.equal(manifest.revision, "88e8500c1e5f1b29d80f42c8ca065cc9cbd56899");
  assert.match(fs.readFileSync(path.join(fixtureRoot, "LICENSE"), "utf8"), /MIT License/);
  for (const file of manifest.files) {
    const bytes = fs.readFileSync(path.join(fixtureRoot, "upstream", file.path));
    assert.equal(createHash("sha256").update(bytes).digest("hex"), file.sha256, file.path);
  }
  const projects = fs
    .readdirSync(path.join(fixtureRoot, "upstream/test-workspace/tsc"))
    .filter((name) =>
      fs.existsSync(path.join(fixtureRoot, "upstream/test-workspace/tsc", name, "tsconfig.json")),
    )
    .sort();
  assert.deepEqual(projects, [...manifest.projects].sort());
  assert.equal(projects.length, 230);
  assert.equal(manifest.expectedDiagnostics.length, 39);
});

test("every documented difference names an authored line the oracle keeps clean", () => {
  assert.deepEqual(
    { schema: documented.schema, version: documented.version, revision: documented.revision },
    {
      schema: "vize.vueLanguageToolsDocumentedDifferences",
      version: 1,
      revision: manifest.revision,
    },
  );
  assert.deepEqual(
    documented.differences.map((difference) => [difference.id, difference.issue]),
    [["component-default-export-identity", 6253]],
  );
  for (const difference of documented.differences) {
    assert.ok(difference.reason.length >= 200, `${difference.id} must explain itself`);
  }
  assert.deepEqual(vizeOnly.map(identity), [
    "components/main.vue:70:11:2345",
    "components/main.vue:71:11:2345",
    "components/main.vue:72:11:2345",
    "components/main.vue:73:11:2345",
    "components/main.vue:74:11:2345",
    "defineModel/main.vue:25:11:2345",
  ]);
  const oracle = new Set(manifest.expectedDiagnostics.map(identity));
  const corpus = new Set(manifest.files.map((file) => file.path));
  for (const diagnostic of vizeOnly) {
    assert.equal(oracle.has(identity(diagnostic)), false, identity(diagnostic));
    assert.equal(
      corpus.has(`test-workspace/tsc/${diagnostic.file}`),
      true,
      `${diagnostic.file} is not a corpus source`,
    );
  }
});

test(
  "the upstream shared type assertion rejects a deliberately wrong type",
  { timeout: 60_000 },
  async () => {
    const directory = workspace("upstream-assertion-oracle-");
    prepareGeneralFixtures(directory);
    const project = path.join(directory, "test-workspace/tsc/oracle-mutation");
    fs.mkdirSync(project);
    fs.writeFileSync(
      path.join(project, "tsconfig.json"),
      '{"extends":"../../tsconfig.base.json","include":["**/*"]}',
    );
    const source =
      '<script setup lang="ts">\nimport { exactType } from "../shared";\nexactType(1 as number, {} as number);\n</script>';
    const file = path.join(project, "main.vue");
    try {
      fs.writeFileSync(file, source);
      assert.deepEqual(await check(project), []);
      fs.writeFileSync(file, source.replace("{} as number", "{} as string"));
      const diagnostics = await check(project);
      assert.ok(diagnostics.length > 0, "a lost shared declaration must not disable exactType");
      assert.ok(
        diagnostics.every((d) => d.code === 2345 && d.file === "main.vue" && d.line === 3),
        JSON.stringify(diagnostics),
      );
      fs.rmSync(path.join(project, "../shared.d.ts"));
      const missing = await check(project);
      assert.deepEqual(
        missing.map((d) => d.code),
        [2307],
      );
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);

test(
  "Canon matches every upstream project's authored error oracle",
  { timeout: 600_000, concurrency: 4 },
  async (t) => {
    const directory = workspace("upstream-vue-tsc-");
    prepareGeneralFixtures(directory);
    try {
      const queue = [...manifest.projects];
      await Promise.all(
        Array.from({ length: 4 }, async () => {
          for (let project = queue.shift(); project != null; project = queue.shift()) {
            await t.test(project, async () => {
              const projectRoot = path.join(directory, "test-workspace/tsc", project);
              const actual = (await check(projectRoot)).filter((d) => d.severity === "error");
              const expected = [...manifest.expectedDiagnostics, ...vizeOnly]
                .filter((d) => d.file.startsWith(`${project}/`))
                .map((d) => ({ ...d, file: d.file.slice(project.length + 1) }));
              assert.deepEqual(
                actual.map(diagnosticIdentity).sort(compareIdentity),
                expected.sort(compareIdentity),
                actual
                  .map((d) => `${d.file}:${d.line}:${d.column} TS${d.code}: ${d.message}`)
                  .join("\n"),
              );
            });
          }
        }),
      );
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);
