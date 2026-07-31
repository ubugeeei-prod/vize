import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { withPinnedFixtureWorkspace } from "../../_helpers/realworld-patch.ts";
import {
  resolveTsgoBinary,
  runVizeCheck,
  symlinkVueTypes,
} from "../../_helpers/realworld-typecheck.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";

const SNAPSHOT_DIR = path.join(path.dirname(fileURLToPath(import.meta.url)), "__snapshots__");

// First suite to exercise the pinned vue2-elm fixture (#2971 audit item 7).
// The snapshot pins the exact `vize check` surface over the untouched Vue 2
// (2.1-era) app: real legacy findings (stale bindings, template consts
// reassigned in handlers) alongside current gaps such as TS2307 for the
// webpack-style extensionless `.vue` imports (`from 'src/components/...'`)
// and for vendor modules that are intentionally not installed. Behavior
// changes on legacy input must land as reviewed snapshot updates
// (UPDATE_SNAPSHOTS=1), never as silent drift.
test("vue2-elm vize check surface over the pinned Vue 2 app stays exact", async () => {
  const corsaPath = resolveTsgoBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "vue2-elm", includePaths: ["src"] },
    async (fixture) => {
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
      fixture.write(
        "vize.config.json",
        json({
          compiler: { compatibility: { vueVersion: "2" } },
          typeChecker: { corsaPath, legacyVue2: true },
        }),
      );

      const first = runVizeCheck(fixture.workspaceDir, corsaPath, ["src/**/*.vue"]);
      const second = runVizeCheck(fixture.workspaceDir, corsaPath, ["src/**/*.vue"]);

      assert.equal(first.status, 1, first.stderr || first.stdout);
      assert.equal(first.stderr, "");
      assert.equal(second.stdout, first.stdout, "check JSON must be byte-stable");
      assert.equal(second.status, first.status);
      assert.equal(first.report.fileCount, 55, "every pinned vue2-elm SFC must be checked");
      assert.equal(first.report.warningCount, 0);
      assert.ok(first.report.errorCount > 0, "the legacy surface is intentionally not clean");

      assertSnapshot(SNAPSHOT_DIR, "vue2-elm-check", `${JSON.stringify(first.report, null, 2)}\n`);
    },
  );
});

function json(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}
