import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  type PinnedFixtureWorkspace,
  withPinnedFixtureWorkspace,
} from "../../_helpers/realworld-patch.ts";
import {
  omitProgramEvidence,
  resolveTsgoBinary,
  runVizeCheck,
  symlinkVueTypes,
} from "../../_helpers/realworld-typecheck.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";

const SNAPSHOT_DIR = path.join(path.dirname(fileURLToPath(import.meta.url)), "__snapshots__");
const APP_PATH = "src/App.vue";
const CLEAN_IMPORT = "\timport svgIcon from './components/common/svg';\n";
const BROKEN_IMPORT = `${CLEAN_IMPORT}\tconst vizeLegacyProbe = missingVizeLegacyProbe;\n`;
const BROKEN_DIAGNOSTIC =
  "error:17:26 [TS2552] Cannot find name 'missingVizeLegacyProbe'. Did you mean 'vizeLegacyProbe'?";
const PROFILE_PATH = "src/page/profile/profile.vue";
const DEFINED_MIXIN_CALL = "path = this.getImgPath(this.avatar)";
const MISSPELLED_MIXIN_CALL = "path = this.getImgPat(this.avatar)";
const MISSPELLED_MIXIN_DIAGNOSTIC =
  "error:179:29 [TS2551] Property 'getImgPat' does not exist on type '$This'. Did you mean 'getImgPath'?";

// First suite to exercise the pinned vue2-elm fixture (#2971 audit item 7).
// The snapshot pins the exact `vize check` surface over the untouched Vue 2
// (2.1-era) app: real legacy findings (stale bindings, template consts
// reassigned in handlers) alongside current gaps such as TS2307 for the
// webpack-style extensionless `.vue` imports (`from 'src/components/...'`)
// and for vendor modules that are intentionally not installed. Behavior
// changes on legacy input must land as reviewed snapshot updates
// (UPDATE_SNAPSHOTS=1), never as silent drift.
async function verifyVue2ElmSnapshot(): Promise<void> {
  const corsaPath = resolveTsgoBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "vue2-elm", includePaths: ["src"] },
    async (fixture) => {
      configureVue2Workspace(fixture, corsaPath);

      const first = runVizeCheck(fixture.workspaceDir, corsaPath, ["src/**/*.vue"]);
      const second = runVizeCheck(fixture.workspaceDir, corsaPath, ["src/**/*.vue"]);

      assert.equal(first.status, 1, first.stderr || first.stdout);
      assert.equal(first.stderr, "");
      assert.equal(second.stdout, first.stdout, "check JSON must be byte-stable");
      assert.equal(second.status, first.status);
      assert.equal(
        first.report.files.filter((file) => file.file.endsWith(".vue")).length,
        55,
        "every pinned vue2-elm SFC must be checked",
      );
      // The authored JavaScript the SFCs import is now reported alongside them,
      // so a regression that only surfaces in a dependency cannot hide (#3996).
      assert.equal(
        first.report.fileCount,
        61,
        JSON.stringify(first.report.files.map((file) => file.file)),
      );
      assert.equal(first.report.warningCount, 0);
      assert.ok(first.report.errorCount > 0, "the legacy surface is intentionally not clean");
      assertDiagnosticsStayInAuthoredBlocks(fixture.workspaceDir, first.report.files);

      // The two historical packets remain frozen. The additive display
      // successor changes the 170 reviewed `__VizeThis` names. Its sole later
      // removal is the real inherited getImgPath method's false positive (#11780).
      assertSnapshot(
        SNAPSHOT_DIR,
        "vue2-elm-display-check",
        `${JSON.stringify(omitProgramEvidence(first.report), null, 2)}\n`,
      );
    },
  );
}

test("vue2-elm vize check surface over the pinned Vue 2 app stays exact", verifyVue2ElmSnapshot);

test("vue2-elm accepts the defined JS mixin method and rejects a misspelled call", async () => {
  const corsaPath = resolveTsgoBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "vue2-elm", includePaths: ["src"] },
    async (fixture) => {
      configureVue2Workspace(fixture, corsaPath);
      const pinnedSource = fixture.read(PROFILE_PATH);
      const clean = runVizeCheck(fixture.workspaceDir, corsaPath, [PROFILE_PATH]);
      assert.equal(clean.status, 1, clean.stderr || clean.stdout);
      assert.equal(clean.stderr, "");
      const cleanDiagnostics = diagnosticsFor(clean.report.files, PROFILE_PATH);
      assert.deepEqual(
        cleanDiagnostics.filter((diagnostic) => diagnostic.startsWith("error:179:29 ")),
        [],
        "the imported mixin provides getImgPath",
      );

      fixture.applyExactPatch(PROFILE_PATH, DEFINED_MIXIN_CALL, MISSPELLED_MIXIN_CALL);
      const broken = runVizeCheck(fixture.workspaceDir, corsaPath, [PROFILE_PATH]);
      assert.equal(broken.status, 1, broken.stderr || broken.stdout);
      assert.equal(broken.stderr, "");
      const brokenDiagnostics = diagnosticsFor(broken.report.files, PROFILE_PATH);
      assert.deepEqual(
        brokenDiagnostics.filter((diagnostic) => !cleanDiagnostics.includes(diagnostic)),
        [MISSPELLED_MIXIN_DIAGNOSTIC],
      );
      assert.deepEqual(
        brokenDiagnostics.filter((diagnostic) => diagnostic !== MISSPELLED_MIXIN_DIAGNOSTIC),
        cleanDiagnostics,
        "the typo must preserve every other diagnostic",
      );
      const repairedSource = fixture.applyExactPatch(
        PROFILE_PATH,
        MISSPELLED_MIXIN_CALL,
        DEFINED_MIXIN_CALL,
      );
      assert.equal(repairedSource, pinnedSource);
      const repaired = runVizeCheck(fixture.workspaceDir, corsaPath, [PROFILE_PATH]);
      assert.equal(repaired.status, 1, repaired.stderr || repaired.stdout);
      assert.equal(repaired.stderr, "");
      assert.deepEqual(repaired.report, clean.report);
      assert.equal(repaired.stdout, clean.stdout);
    },
  );
});

test("vue2-elm detects and repairs an exact authored JavaScript type error", async () => {
  const corsaPath = resolveTsgoBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "vue2-elm", includePaths: ["src"] },
    async (fixture) => {
      configureVue2Workspace(fixture, corsaPath);
      const pinnedSource = fixture.read(APP_PATH);
      const clean = runVizeCheck(fixture.workspaceDir, corsaPath, [APP_PATH]);
      const cleanDiagnostics = diagnosticsFor(clean.report.files, APP_PATH);

      const brokenSource = fixture.applyExactPatch(APP_PATH, CLEAN_IMPORT, BROKEN_IMPORT);
      const brokenFirst = runVizeCheck(fixture.workspaceDir, corsaPath, [APP_PATH]);
      const brokenSecond = runVizeCheck(fixture.workspaceDir, corsaPath, [APP_PATH]);
      assert.equal(
        brokenSecond.stdout,
        brokenFirst.stdout,
        "broken check JSON must be byte-stable",
      );
      assert.equal(fixture.read(APP_PATH), brokenSource, "check must preserve the broken edit");
      assert.deepEqual(
        diagnosticsFor(brokenFirst.report.files, APP_PATH).filter(
          (diagnostic) => !cleanDiagnostics.includes(diagnostic),
        ),
        [BROKEN_DIAGNOSTIC],
      );

      const repairedSource = fixture.applyExactPatch(APP_PATH, BROKEN_IMPORT, CLEAN_IMPORT);
      assert.equal(repairedSource, pinnedSource, "repair must restore the exact pinned source");
      const repaired = runVizeCheck(fixture.workspaceDir, corsaPath, [APP_PATH]);
      assert.deepEqual(
        repaired.report,
        clean.report,
        "repair must restore the clean diagnostic set",
      );
      assert.equal(repaired.stdout, clean.stdout, "repair must restore byte-stable check JSON");
    },
  );
});

function configureVue2Workspace(fixture: PinnedFixtureWorkspace, corsaPath: string): void {
  symlinkVueTypes(fixture.workspaceDir);
  fixture.write(
    "tsconfig.json",
    json({
      compilerOptions: {
        allowJs: true,
        // vue2-elm is a plain-JavaScript Vue 2 app: type-checking it at
        // all is the `checkJs` opt-in, exactly as `tsc`/`vue-tsc` require
        // for a `lang="js"` script block (#3322).
        checkJs: true,
        lib: ["ES2022", "DOM"],
        paths: { "src/*": ["./src/*"] },
        skipLibCheck: true,
        strict: false,
      },
      include: ["src"],
    }),
  );
  fixture.writeVueConfig(
    json({
      compiler: { compatibility: { vueVersion: "2" } },
      typeChecker: { corsaPath, legacyVue2: true },
    }),
  );
}

function diagnosticsFor(
  files: Array<{ diagnostics: string[]; file: string }>,
  file: string,
): string[] {
  return files.find((entry) => entry.file === file)?.diagnostics ?? [];
}

function json(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function assertDiagnosticsStayInAuthoredBlocks(
  workspaceDir: string,
  files: Array<{ diagnostics: string[]; file: string }>,
): void {
  for (const file of files) {
    const source = fs.readFileSync(path.join(workspaceDir, file.file), "utf8");
    // Imported authored scripts are reported alongside the SFCs that pull them
    // in (#3996); every one of their lines is authored, so the whole file is the
    // authored range rather than its `<template>`/`<script>` blocks.
    const ranges = file.file.endsWith(".vue")
      ? [...source.matchAll(/<(template|script)\b[^>]*>[\s\S]*?<\/\1>/g)].map((match) => {
          const start = source.slice(0, match.index ?? 0).split("\n").length;
          const end = start + match[0].split("\n").length - 1;
          return { end, start };
        })
      : [{ end: source.split("\n").length, start: 1 }];

    for (const diagnostic of file.diagnostics) {
      const match = /^(?:error|warning|info|hint):(\d+):(\d+) /.exec(diagnostic);
      assert.ok(match, `diagnostic must include an authored location: ${file.file}: ${diagnostic}`);
      const line = Number(match[1]);
      assert.ok(
        ranges.some((range) => line >= range.start && line <= range.end),
        `diagnostic escaped template/script blocks: ${file.file}: ${diagnostic}`,
      );
    }
  }
}
