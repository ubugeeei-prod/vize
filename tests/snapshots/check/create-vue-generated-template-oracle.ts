import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";

import { withPinnedFixtureWorkspace } from "../../_helpers/realworld-patch.ts";
import {
  type CommandResult,
  omitProgramEvidence,
  resolveTsgoBinary,
  resolveVueTscBinary,
  runVizeCheck,
  runVueTsc,
  symlinkVueTypes,
  type VizeCheckResult,
} from "../../_helpers/realworld-typecheck.ts";

const templateRoot = "template/code/typescript-default/src";
const helloWorldPath = `${templateRoot}/components/HelloWorld.vue`;
const generatedTemplateFiles = [
  `${templateRoot}/App.vue`,
  `${templateRoot}/components/HelloWorld.vue`,
  `${templateRoot}/components/TheWelcome.vue`,
  `${templateRoot}/components/WelcomeItem.vue`,
  `${templateRoot}/components/icons/IconCommunity.vue`,
  `${templateRoot}/components/icons/IconDocumentation.vue`,
  `${templateRoot}/components/icons/IconEcosystem.vue`,
  `${templateRoot}/components/icons/IconSupport.vue`,
  `${templateRoot}/components/icons/IconTooling.vue`,
];
const helloWorldSha256 = "c4daba95e3842514356ca909968bfe243a803d31b0303d3cce23bd45c130ceec";
const cleanHeading = '    <h1 class="green">{{ msg }}</h1>';
const brokenHeading = '    <h1 class="green">{{ msg.lenght }}</h1>';
const brokenDiagnostic =
  "error:9:30 [TS2551] Property 'lenght' does not exist on type 'string'. Did you mean 'length'?";
const brokenBaselineOutput = `${helloWorldPath}(9,30): error TS2551: Property 'lenght' does not exist on type 'string'. Did you mean 'length'?\n`;

const tsconfig = {
  compilerOptions: {
    lib: ["ES2022", "DOM", "DOM.Iterable"],
    module: "ESNext",
    moduleResolution: "bundler",
    noEmit: true,
    skipLibCheck: true,
    strict: true,
    target: "ES2022",
  },
  include: [`${templateRoot}/**/*.vue`],
};

test("create-vue generated template repairs one exact broken prop usage in batch check", async () => {
  const corsaPath = resolveTsgoBinary();
  const vueTscPath = resolveVueTscBinary();

  await withPinnedFixtureWorkspace(
    { fixtureId: "create-vue", includePaths: generatedTemplateFiles },
    async (fixture) => {
      symlinkVueTypes(fixture.workspaceDir);
      fixture.write("tsconfig.json", `${JSON.stringify(tsconfig, null, 2)}\n`);

      const pinnedSource = fixture.read(helloWorldPath);
      const sourceMode = fs.statSync(fixture.resolve(helloWorldPath)).mode & 0o777;
      assert.equal(sha256(pinnedSource), helloWorldSha256, "pinned create-vue source changed");

      const cleanFirst = runVizeCheck(fixture.workspaceDir, corsaPath, [templateRoot]);
      const cleanSecond = runVizeCheck(fixture.workspaceDir, corsaPath, [templateRoot]);
      assertCleanParity(cleanFirst, runVueTsc(fixture.workspaceDir, vueTscPath));
      assert.equal(cleanSecond.stdout, cleanFirst.stdout, "clean check JSON must be byte-stable");
      assert.equal(fixture.read(helloWorldPath), pinnedSource, "check must preserve the source");
      assert.equal(fs.statSync(fixture.resolve(helloWorldPath)).mode & 0o777, sourceMode);

      const brokenSource = fixture.applyExactPatch(helloWorldPath, cleanHeading, brokenHeading);
      const brokenFirst = runVizeCheck(fixture.workspaceDir, corsaPath, [templateRoot]);
      const brokenSecond = runVizeCheck(fixture.workspaceDir, corsaPath, [templateRoot]);
      assertBrokenParity(brokenFirst, runVueTsc(fixture.workspaceDir, vueTscPath));
      assert.equal(
        brokenSecond.stdout,
        brokenFirst.stdout,
        "broken check JSON must be byte-stable",
      );
      assert.equal(fixture.read(helloWorldPath), brokenSource, "check must preserve the edit");
      assert.equal(fs.statSync(fixture.resolve(helloWorldPath)).mode & 0o777, sourceMode);

      const repairedSource = fixture.applyExactPatch(helloWorldPath, brokenHeading, cleanHeading);
      assert.equal(repairedSource, pinnedSource, "repair must restore the exact pinned source");
      const repairedFirst = runVizeCheck(fixture.workspaceDir, corsaPath, [templateRoot]);
      assertCleanParity(repairedFirst, runVueTsc(fixture.workspaceDir, vueTscPath));
      assert.equal(repairedFirst.stdout, cleanFirst.stdout, "repair must restore clean output");
      assert.equal(fixture.read(helloWorldPath), pinnedSource);
      assert.equal(fs.statSync(fixture.resolve(helloWorldPath)).mode & 0o777, sourceMode);
    },
  );
});

function expectedReport(diagnosticsByFile: Record<string, string[]>): unknown {
  const files = generatedTemplateFiles.map((file) => ({
    file,
    diagnostics: diagnosticsByFile[file] ?? [],
  }));
  const errorCount = files.reduce((total, entry) => total + entry.diagnostics.length, 0);
  return {
    files,
    errorCount,
    warningCount: 0,
    fileCount: generatedTemplateFiles.length,
  };
}

function assertCleanParity(vize: VizeCheckResult, vueTsc: CommandResult): void {
  assert.equal(vize.status, 0, vize.stderr || vize.stdout);
  assert.deepEqual(
    omitProgramEvidence(vize.report),
    expectedReport({}),
    JSON.stringify(vize.report),
  );
  assert.equal(vueTsc.status, 0, vueTsc.stderr || vueTsc.stdout);
  assert.equal(vueTsc.stdout, "", vueTsc.stdout);
}

function assertBrokenParity(vize: VizeCheckResult, vueTsc: CommandResult): void {
  assert.equal(vize.status, 1, vize.stderr || vize.stdout);
  assert.deepEqual(
    omitProgramEvidence(vize.report),
    expectedReport({ [helloWorldPath]: [brokenDiagnostic] }),
    JSON.stringify(vize.report),
  );
  assert.equal(vueTsc.status, 2, vueTsc.stderr || vueTsc.stdout);
  assert.equal(vueTsc.stdout, brokenBaselineOutput, vueTsc.stdout);
}

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}
