// Corpus-wide formatter semantic preservation. Native HTML uses the existing
// Vue 3 structural comparator unless a project declares an exact dialect
// partition; Pug remains owned by its independent pinned semantic oracle.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  collectProjectVueFiles,
  createKnownViolationConsumption,
  loadGlyphCorpusProjects,
  loadKnownViolations,
  renderViolations,
  resolveGlyphLaunch,
  runVize,
  writeGlyphCorpusPropertyEvidence,
  writeGlyphPugSemanticEvidence,
} from "../../tools/fixtures/glyph-corpus.mjs";
import {
  evidenceSourceCommit,
  formatterEvidence,
  writeGlyphSfcEquivalenceEvidence,
} from "../../tools/fixtures/glyph-sfc-evidence.mjs";
import {
  makeSyntheticProject,
  sweepProject,
  violationCategory,
} from "./support/glyph-corpus-sweep.ts";
import type {
  CorpusProject,
  PugEvidence,
  SfcDialectEvidence,
  Violation,
} from "./support/glyph-corpus-sweep.ts";
import { isPugSfc, PUG_ORACLE_BASELINE } from "./support/pug-template-equivalence.ts";
import { resolveSfcDialectPartition, sfcDialects } from "./support/sfc-baseline-routes.ts";
import { getSfcBaselineProvenance } from "./support/sfc-baselines.ts";

const property = "parse-preservation";
const projects = loadGlyphCorpusProjects() as CorpusProject[];
const knownViolations = loadKnownViolations(property);
const waiverConsumption = createKnownViolationConsumption(knownViolations);

test("glyph corpus classifies crashed reference oracles precisely", () => {
  assert.equal(violationCategory(["comparison failed: parser crashed"]), "baseline-unusable");
  assert.equal(violationCategory(["block disappeared"]), "semantic-diff");
});

test("glyph corpus parse-preservation holds for every hydrated fixture", () => {
  const hydrated = projects.filter((project) => project.hydrated);
  const launch = resolveGlyphLaunch();
  const violations: Violation[] = [];
  const baselineUnusable: Violation[] = [];
  const waivedViolations: Array<Violation & { waiver: object }> = [];
  const pugEvidence: PugEvidence[] = [];
  const sfcDialectEvidence: SfcDialectEvidence[] = [];
  const counters = { files: 0, skipped: 0 };
  for (const project of hydrated) {
    sweepProject(project, launch, {
      waiverConsumption,
      violations,
      baselineUnusable,
      counters,
      waivedViolations,
      pugEvidence,
      sfcDialectEvidence,
    });
  }

  let waiverValidationError: string | null = null;
  try {
    waiverConsumption.assertAllConsumed(new Set(hydrated.map((project) => project.id)));
  } catch (error) {
    waiverValidationError = error instanceof Error ? error.message : String(error);
  }
  const expectedEvidenceFiles = hydrated.flatMap((project) => {
    if (project.sfcDialectRoutes == null) return [];
    const files = collectProjectVueFiles(project) as string[];
    const partition = resolveSfcDialectPartition(project, files);
    return files.flatMap((file) => {
      const filename = path.join(project.fixtureDir, file);
      if (isPugSfc(fs.readFileSync(filename, "utf8"), filename)) return [];
      const selected = partition.get(file);
      assert.ok(selected, `${project.id}:${file} must have a dialect route`);
      return [
        {
          project: project.id,
          revision: project.revision,
          path: file,
          routeId: selected.routeId,
          dialect: selected.dialect,
          baselineId: getSfcBaselineProvenance(selected.dialect).id,
        },
      ];
    });
  });
  const reportEnabled =
    process.env.FIXTURE_REPORT_DIR != null && process.env.FIXTURE_REPORT_DIR !== "";
  const version = reportEnabled ? runVize(launch, process.cwd(), ["--version"], 30_000) : null;
  if (version != null) assert.equal(version.status, 0, version.stderr);
  writeGlyphSfcEquivalenceEvidence({
    sourceCommit: evidenceSourceCommit(),
    formatter:
      version == null
        ? null
        : formatterEvidence(launch.evidenceCommand, `${version.stdout}\n${version.stderr}`.trim()),
    files: sfcDialectEvidence,
    availableBaselines: sfcDialects.map(getSfcBaselineProvenance),
    expectedFiles: expectedEvidenceFiles,
    waiverValidationError,
  });
  writeGlyphCorpusPropertyEvidence(property, {
    projectIds: hydrated.map((project) => project.id),
    counters,
    violations,
    baselineUnusable,
    waivedViolations,
    waiverValidationError,
  });
  writeGlyphPugSemanticEvidence({
    projectIds: hydrated.map((project) => project.id),
    baseline: PUG_ORACLE_BASELINE,
    files: pugEvidence,
  });

  assert.equal(waiverValidationError, null, waiverValidationError ?? undefined);
  process.stderr.write(
    `glyph ${property}: ${counters.files} file(s) across ${hydrated.length} project(s), ` +
      `${projects.length - hydrated.length} project(s) not hydrated, ` +
      `${counters.skipped} known violation(s) skipped, ` +
      `${baselineUnusable.length} baseline-unusable file(s), ` +
      `${violations.length} violation(s)\n`,
  );
  assert.equal(violations.length, 0, renderViolations(property, violations));
});

test("glyph corpus records unusable reference baselines without failing the property", () => {
  const project = makeSyntheticProject([
    [
      "src/BrokenPug.vue",
      [
        '<template lang="pug">',
        'template(v-for="item in items")',
        '  div(:key="item") {{ item }}',
        "</template>",
        "",
      ].join("\n"),
    ],
  ]);
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-noop-"));
  const fakeFormatter = path.join(fakeDir, "fake-vize.mjs");
  fs.writeFileSync(
    fakeFormatter,
    [
      "#!/usr/bin/env node",
      'process.stderr.write("Found 1 file(s)\\n\\nFormatted 1 file(s)\\n  1 file(s) already formatted\\n");',
      "",
    ].join("\n"),
  );
  fs.chmodSync(fakeFormatter, 0o755);
  try {
    const violations: Violation[] = [];
    const baselineUnusable: Violation[] = [];
    const counters = { files: 0, skipped: 0 };
    const localWaivers = createKnownViolationConsumption([]);
    sweepProject(
      project,
      { command: fakeFormatter, prefix: [] },
      {
        waiverConsumption: localWaivers,
        violations,
        baselineUnusable,
        counters,
      },
    );

    assert.deepEqual(violations, []);
    assert.equal(baselineUnusable.length, 1);
    assert.equal(baselineUnusable[0].project, project.id);
    assert.equal(baselineUnusable[0].file, "src/BrokenPug.vue");
    assert.match(baselineUnusable[0].detail, /pristine Vue baseline failed/);
    assert.deepEqual(counters, { files: 0, skipped: 0 });
  } finally {
    fs.rmSync(project.fixtureDir, { recursive: true, force: true });
    fs.rmSync(fakeDir, { recursive: true, force: true });
  }
});

test("glyph corpus parse-preservation machinery accepts the real formatter", () => {
  const source = [
    '<script setup lang="ts">',
    "const label = 'hi'",
    "</script>",
    "<template>",
    '  <button v-bind="$attrs" :class="label" data-x>',
    "    {{ label }} <pre>  keep   me </pre>",
    "  </button>",
    "</template>",
    "<style scoped>.a{color:#ffffff}</style>",
    "",
  ].join("\n");
  const project = makeSyntheticProject([["src/App.vue", source]]);
  try {
    const violations: Violation[] = [];
    const counters = { files: 0, skipped: 0 };
    sweepProject(project, resolveGlyphLaunch(), { waiverConsumption, violations, counters });
    assert.deepEqual(violations, []);
    assert.equal(counters.files, 1);
  } finally {
    fs.rmSync(project.fixtureDir, { recursive: true, force: true });
  }
});
