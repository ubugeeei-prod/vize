// Corpus-wide `fmt(fmt(x)) == fmt(x)` and `--check`/`--write` agreement.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { loadFormatterCheckEvidenceOrRecord } from "../../legacy-tools/fixtures/glyph-formatter-evidence.mjs";
import { createFormatterChangeEvidence } from "../../legacy-tools/fixtures/tool-matrix-formatter.mjs";
import {
  assertFormatterCheckWriteAgreement,
  collectFormatterWriteEvidence,
  collectProjectVueFiles,
  createKnownViolationConsumption,
  diffExcerpt,
  loadFormatterCheckEvidence,
  loadGlyphCorpusProjects,
  loadKnownViolations,
  renderViolations,
  resolveGlyphLaunch,
  snapshotWorkspaceFiles,
  withFormattedWorkspace,
  writeGlyphCorpusPropertyEvidence,
} from "../../legacy-tools/fixtures/glyph-corpus.mjs";

type CorpusProject = {
  id: string;
  fixtureDir: string;
  hydrated: boolean;
  vueGlobs: string[];
};

type Violation = { project: string; file: string; detail: string };

const indentedPug = "\n    main\n      //- keep\n      | pipe  text\n      pre.\n        a  b\n";
const property = "idempotence";
const projects = loadGlyphCorpusProjects() as CorpusProject[];
const knownViolations = loadKnownViolations(property);
const waiverConsumption = createKnownViolationConsumption(knownViolations);

function sweepProject(
  project: CorpusProject,
  launch: { command: string; prefix: string[] },
  violations: Violation[],
  counters: { files: number; skipped: number },
  checkEvidence: FormatterEvidence | null = null,
  waivedViolations: Array<Violation & { waiver: object }> = [],
): void {
  const files = collectProjectVueFiles(project) as string[];
  if (files.length === 0) return;
  withFormattedWorkspace(
    project,
    files,
    launch,
    (workspace: { workspaceDir: string; reformat: () => void }) => {
      if (checkEvidence != null) {
        assertFormatterCheckWriteAgreement(
          project.id,
          checkEvidence,
          collectFormatterWriteEvidence(project, files, workspace.workspaceDir),
        );
      }
      const firstPass = snapshotWorkspaceFiles(workspace.workspaceDir, files) as Map<
        string,
        Buffer
      >;
      workspace.reformat();
      const secondPass = snapshotWorkspaceFiles(workspace.workspaceDir, files) as Map<
        string,
        Buffer
      >;
      for (const file of files) {
        const before = firstPass.get(file) as Buffer;
        const after = secondPass.get(file) as Buffer;
        if (before.equals(after)) {
          counters.files += 1;
          continue;
        }
        const detail = diffExcerpt(before.toString("utf8"), after.toString("utf8"), "fmt1", "fmt2");
        const waiver = waiverConsumption.consume(project.id, file, null, "semantic-diff");
        if (waiver) {
          waivedViolations.push({ project: project.id, file, detail, waiver });
          counters.skipped += 1;
          continue;
        }
        violations.push({
          project: project.id,
          file,
          detail,
        });
      }
    },
  );
}

test("glyph corpus idempotence holds for every hydrated fixture", () => {
  const hydrated = projects.filter((project) => project.hydrated);
  if (hydrated.length === 0) {
    // Per-PR lanes run without hydrated fixtures; the machinery subtests below
    // still exercise the property end-to-end on synthetic projects.
    return;
  }
  const launch = resolveGlyphLaunch();
  const violations: Violation[] = [];
  const baselineUnusable: Violation[] = [];
  const waivedViolations: Array<Violation & { waiver: object }> = [];
  const counters = { files: 0, skipped: 0 };
  for (const project of hydrated) {
    const checkEvidence = loadFormatterCheckEvidenceOrRecord(project, baselineUnusable);
    sweepProject(project, launch, violations, counters, checkEvidence, waivedViolations);
  }
  let waiverValidationError: string | null = null;
  try {
    waiverConsumption.assertAllConsumed(new Set(hydrated.map((project) => project.id)));
  } catch (error) {
    waiverValidationError = error instanceof Error ? error.message : String(error);
  }
  writeGlyphCorpusPropertyEvidence(property, {
    projectIds: hydrated.map((project) => project.id),
    counters,
    violations,
    baselineUnusable,
    waivedViolations,
    waiverValidationError,
  });
  assert.equal(waiverValidationError, null, waiverValidationError ?? undefined);
  process.stderr.write(
    `glyph ${property}: ${counters.files} file(s) across ${hydrated.length} project(s), ` +
      `${projects.length - hydrated.length} project(s) not hydrated, ` +
      `${counters.skipped} known violation(s) skipped, ` +
      `${baselineUnusable.length} baseline-unusable check(s), ${violations.length} violation(s)\n`,
  );
  assert.equal(violations.length, 0, renderViolations(property, violations));
});

test("glyph corpus idempotence machinery accepts the real formatter", () => {
  const project = makeSyntheticProject([
    ["src/App.vue", '<template>\n  <div   :class="foo">hi</div>\n</template>\n'],
    ["src/Card.vue", "<script setup>\nconst x=1\n</script>\n"],
    ["src/Pug.vue", '<template lang="pug">\nmain\n  //- keep\n  | pipe  text\n</template>\n'],
    // A non-canonical outer indent must rebase to a fixed point, or pass two moves bytes.
    ["src/PugIndented.vue", `<template lang="pug">${indentedPug}</template>\n`],
  ]);
  try {
    const launch = resolveGlyphLaunch();
    const files = collectProjectVueFiles(project) as string[];
    assert.deepEqual(files, ["src/App.vue", "src/Card.vue", "src/Pug.vue", "src/PugIndented.vue"]);
    const violations: Violation[] = [];
    const counters = { files: 0, skipped: 0 };
    sweepProject(project, launch, violations, counters);
    assert.deepEqual(violations, []);
    assert.equal(counters.files, 4);
  } finally {
    fs.rmSync(project.fixtureDir, { recursive: true, force: true });
  }
});

test("glyph corpus idempotence machinery flags a drifting formatter", () => {
  const project = makeSyntheticProject([["src/App.vue", "<template><p>x</p></template>\n"]]);
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-fake-"));
  const fakeFormatter = path.join(fakeDir, "fake-vize.mjs");
  // Appends one comment line per run: pass two never matches pass one.
  fs.writeFileSync(
    fakeFormatter,
    [
      "#!/usr/bin/env node",
      'import fs from "node:fs";',
      'const file = "src/App.vue";',
      'fs.appendFileSync(file, "<!-- drift -->\\n");',
      'process.stderr.write("Found 1 file(s)\\nReformatted: src/App.vue\\n\\nFormatted 1 file(s)\\n  1 file(s) reformatted\\n");',
      "",
    ].join("\n"),
  );
  fs.chmodSync(fakeFormatter, 0o755);
  try {
    const violations: Violation[] = [];
    const counters = { files: 0, skipped: 0 };
    sweepProject(project, { command: fakeFormatter, prefix: [] }, violations, counters);
    assert.equal(violations.length, 1);
    assert.equal(violations[0].project, project.id);
    assert.equal(violations[0].file, "src/App.vue");
    assert.match(violations[0].detail, /first divergence at line 3/);
    assert.match(violations[0].detail, /fmt2 > <!-- drift -->/);
  } finally {
    fs.rmSync(project.fixtureDir, { recursive: true, force: true });
    fs.rmSync(fakeDir, { recursive: true, force: true });
  }
});

for (const fixtureCase of [
  {
    name: "glyph corpus accepts an exact formatter check/write prediction",
    write: "change",
    checkPaths: ["src/App.vue"],
    agrees: true,
  },
  {
    name: "glyph corpus rejects a formatter check false positive",
    write: "clean",
    checkPaths: ["src/App.vue"],
    agrees: false,
  },
  {
    name: "glyph corpus rejects a formatter check false negative",
    write: "change",
    checkPaths: [],
    agrees: false,
  },
] as const) {
  test(fixtureCase.name, () => {
    const fixture = makeAgreementFixture(fixtureCase.write);
    const violations: Violation[] = [];
    const counters = { files: 0, skipped: 0 };
    const sweep = () =>
      sweepProject(
        fixture.project,
        fixture.launch,
        violations,
        counters,
        formatterEvidence(1, [...fixtureCase.checkPaths]),
      );
    try {
      if (fixtureCase.agrees) {
        assert.doesNotThrow(sweep);
        assert.deepEqual(violations, []);
        assert.deepEqual(counters, { files: 1, skipped: 0 });
      } else {
        assert.throws(sweep, /--check prediction disagrees with actual --write mutation/);
      }
    } finally {
      fixture.cleanup();
    }
  });
}

test("glyph corpus rejects different changed paths even when counts agree", () => {
  assert.throws(
    () =>
      assertFormatterCheckWriteAgreement(
        "synthetic-path-mismatch",
        formatterEvidence(2, ["src/App.vue"]),
        formatterEvidence(2, ["src/Card.vue"]),
      ),
    /changedPathsSha256/,
  );
});

test("glyph corpus loads only the matching validated matrix check evidence", () => {
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-check-report-"));
  const project = { id: "synthetic-report" };
  const evidence = formatterEvidence(2, ["src/App.vue"]);
  const reportPath = path.join(reportDir, `${project.id}-formatter.json`);
  try {
    assert.throws(
      () => loadFormatterCheckEvidence(project, reportDir),
      /missing formatter check report/,
    );
    fs.writeFileSync(
      reportPath,
      `${JSON.stringify({
        schema: "vize.fixtureToolRun",
        version: 1,
        project: project.id,
        tool: "formatter",
        formatterCheck: evidence,
      })}\n`,
    );
    assert.deepEqual(loadFormatterCheckEvidence(project, reportDir), evidence);

    fs.writeFileSync(
      reportPath,
      `${JSON.stringify({
        schema: "vize.fixtureToolRun",
        version: 1,
        project: "wrong-project",
        tool: "formatter",
        formatterCheck: evidence,
      })}\n`,
    );
    assert.throws(
      () => loadFormatterCheckEvidence(project, reportDir),
      /invalid formatter check report identity/,
    );
  } finally {
    fs.rmSync(reportDir, { recursive: true, force: true });
  }
});

type FormatterEvidence = {
  checkedFileCount: number;
  changedFileCount: number;
  unchangedFileCount: number;
  changedPathsSha256: string;
};

function formatterEvidence(checkedFileCount: number, changedPaths: string[]): FormatterEvidence {
  return createFormatterChangeEvidence(checkedFileCount, changedPaths) as FormatterEvidence;
}

function makeAgreementFixture(mode: "change" | "clean") {
  const project = makeSyntheticProject([["src/App.vue", "<template><p>x</p></template>\n"]]);
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-check-write-"));
  const executable = path.join(fakeDir, "fake-vize.mjs");
  fs.writeFileSync(
    executable,
    [
      "#!/usr/bin/env node",
      'import fs from "node:fs";',
      ...(mode === "change"
        ? [
            'const file = "src/App.vue";',
            'const source = fs.readFileSync(file, "utf8");',
            'if (source === "<template><p>x</p></template>\\n") {',
            '  fs.writeFileSync(file, "<template>\\n  <p>x</p>\\n</template>\\n");',
            "}",
          ]
        : []),
      "",
    ].join("\n"),
  );
  fs.chmodSync(executable, 0o755);
  return {
    project,
    launch: { command: executable, prefix: [] },
    cleanup: () => {
      fs.rmSync(project.fixtureDir, { recursive: true, force: true });
      fs.rmSync(fakeDir, { recursive: true, force: true });
    },
  };
}

function makeSyntheticProject(
  files: Array<[string, string]>,
  vueGlobs: string[] = ["src/**/*.vue"],
): CorpusProject {
  const fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-corpus-"));
  for (const [file, content] of files) {
    fs.mkdirSync(path.dirname(path.join(fixtureDir, file)), { recursive: true });
    fs.writeFileSync(path.join(fixtureDir, file), content);
  }
  return {
    id: "synthetic-idempotence",
    fixtureDir,
    hydrated: true,
    vueGlobs,
  };
}
