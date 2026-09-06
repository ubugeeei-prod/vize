import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { assertParsesAsModule } from "../../_helpers/assertions.ts";
import { withPinnedFixtureWorkspace } from "../../_helpers/realworld-patch.ts";
import { resolveVizeCommand } from "../../_helpers/realworld-typecheck.ts";

const appPath = "packages/playground/src/AppLink.vue";
const sourceSha256 = "f5c888da7bae9c61151c7fbfde578ccdb768e75fbe2e69637d8298ca4f702c96";
const formattedSourceSha256 = "28cf96bec91ca56fff12447fa4fb135c28a40586421729ff274e9c5f9764f712";
const formattedOutputSha256 = "3713f2ddd52bd7722925b7458c64e89cf48b5670c0a611374430c3137f0e9a48";
const formattedCodeSha256 = "c68549e117eb5d7669c9591ed8299b1c8fc37b3e26772ffd60163723fbfbbd66";
const cleanAttrs = "const attrs = useAttrs();";
const brokenAttrs = "const  attrs = useAttrs();";

type CommandResult = {
  status: number | null;
  stderr: string;
  stdout: string;
};

type CompilerOutput = {
  code: string;
  css: string | null;
  errors: string[];
  filename: string;
  macro_artifacts: unknown[];
  script_lang: string;
  warnings: string[];
};

type BuildResult = CommandResult & {
  files: string[];
  output: CompilerOutput;
  outputText: string;
};

test("Vue Router formatter converges without moving object v-bind spreads", async () => {
  await withPinnedFixtureWorkspace(
    { fixtureId: "vue-router", includePaths: [appPath] },
    async (fixture) => {
      const pinnedSource = fixture.read(appPath);
      const sourceMode = fs.statSync(fixture.resolve(appPath)).mode & 0o777;
      assert.equal(sha256(pinnedSource), sourceSha256, "pinned Vue Router source changed");

      const initialCheck = runFmt(fixture.workspaceDir, "--check");
      assertFmtResult(initialCheck, 1, wouldReformatOutput);
      assert.equal(fixture.read(appPath), pinnedSource, "--check must not mutate pinned source");
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);

      const initialWrite = runFmt(fixture.workspaceDir, "--write");
      assertFmtResult(initialWrite, 0, reformattedOutput);
      const formattedSource = fixture.read(appPath);
      assert.notEqual(formattedSource, pinnedSource);
      assert.equal(sha256(formattedSource), formattedSourceSha256);
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);
      assert.equal(formattedSource.includes("\r"), false);
      assert.equal(formattedSource.includes("  \n"), false, "canonical source has trailing spaces");
      assert.equal(count(formattedSource, cleanAttrs), 1);
      assert.equal(
        count(formattedSource, 'v-bind="attrs"\n    class="router-link"'),
        2,
        formattedSource,
      );

      const compiled = runBuild(fixture.workspaceDir);
      assert.equal(compiled.status, 0, compiled.stderr || compiled.stdout);
      assert.deepEqual(compiled.files, ["AppLink.json"]);
      assert.equal(sha256(compiled.outputText), formattedOutputSha256);
      assert.deepEqual(
        {
          css: compiled.output.css,
          errors: compiled.output.errors,
          filename: compiled.output.filename,
          macro_artifacts: compiled.output.macro_artifacts,
          script_lang: compiled.output.script_lang,
          warnings: compiled.output.warnings,
        },
        {
          css: null,
          errors: [],
          filename: "AppLink.vue",
          macro_artifacts: [],
          script_lang: "ts",
          warnings: [],
        },
      );
      assert.equal(sha256(compiled.output.code), formattedCodeSha256, compiled.output.code);
      assertParsesAsModule(compiled.output.code, "formatted vue-router AppLink.json#code");
      for (const branchKey of [0, 1]) {
        assert.equal(
          count(
            compiled.output.code,
            `_createElementBlock("a", _mergeProps({ key: ${branchKey} }, _unref(attrs), {`,
          ),
          1,
          compiled.output.code,
        );
      }
      assert.equal(compiled.output.code.includes("_mergeProps(_unref(attrs), {"), false);
      assert.equal(compiled.output.code.includes("_normalizeClass"), false);
      const externalBranchStart = compiled.output.code.indexOf("_mergeProps({ key: 0 }");
      const internalBranchStart = compiled.output.code.indexOf("_mergeProps({ key: 1 }");
      assert.ok(externalBranchStart >= 0 && internalBranchStart > externalBranchStart);
      assertOrdered(compiled.output.code.slice(externalBranchStart, internalBranchStart), [
        "class:",
        "href:",
        "tabindex:",
        '"aria-disabled":',
      ]);
      assertOrdered(compiled.output.code.slice(internalBranchStart), [
        "class:",
        "href:",
        "tabindex:",
        '"aria-disabled":',
        "onClick:",
      ]);

      const cleanFirst = runFmt(fixture.workspaceDir, "--check");
      const cleanSecond = runFmt(fixture.workspaceDir, "--check");
      assertFmtResult(cleanFirst, 0, alreadyFormattedOutput);
      assertFmtResult(cleanSecond, 0, alreadyFormattedOutput);
      assert.deepEqual(cleanSecond, cleanFirst, "clean checks must be deterministic");

      const brokenSource = fixture.applyExactPatch(appPath, cleanAttrs, brokenAttrs);
      const brokenFirst = runFmt(fixture.workspaceDir, "--check");
      const brokenSecond = runFmt(fixture.workspaceDir, "--check");
      assertFmtResult(brokenFirst, 1, wouldReformatOutput);
      assertFmtResult(brokenSecond, 1, wouldReformatOutput);
      assert.deepEqual(brokenSecond, brokenFirst, "broken checks must be deterministic");
      assert.equal(fixture.read(appPath), brokenSource, "--check must preserve broken source");
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);

      const repaired = runFmt(fixture.workspaceDir, "--write");
      assertFmtResult(repaired, 0, reformattedOutput);
      assert.equal(fixture.read(appPath), formattedSource, "--write must restore canonical source");

      const repairedCheck = runFmt(fixture.workspaceDir, "--check");
      const idempotentWrite = runFmt(fixture.workspaceDir, "--write");
      assertFmtResult(repairedCheck, 0, alreadyFormattedOutput);
      assertFmtResult(idempotentWrite, 0, unchangedOutput);
      assert.equal(fixture.read(appPath), formattedSource);
      assert.equal(fs.statSync(fixture.resolve(appPath)).mode & 0o777, sourceMode);
    },
  );
});

function runFmt(workspaceDir: string, mode: "--check" | "--write"): CommandResult {
  const [command, ...prefixArgs] = resolveVizeCommand();
  const result = spawnSync(command, [...prefixArgs, "fmt", mode, appPath], {
    cwd: workspaceDir,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 64 * 1024 * 1024,
    timeout: 120_000,
  });
  if (result.error != null) throw result.error;
  return { status: result.status, stderr: result.stderr, stdout: result.stdout };
}

function assertFmtResult(result: CommandResult, status: number, stderr: string): void {
  assert.deepEqual(result, { status, stdout: "", stderr });
}

function runBuild(workspaceDir: string): BuildResult {
  const outputDirectory = ".vize-formatter-compile";
  const [command, ...prefixArgs] = resolveVizeCommand();
  const result = spawnSync(
    command,
    [...prefixArgs, "build", appPath, "--format", "json", "--output", outputDirectory],
    {
      cwd: workspaceDir,
      encoding: "utf8",
      env: { ...process.env, LANG: "C", LC_ALL: "C" },
      maxBuffer: 64 * 1024 * 1024,
      timeout: 120_000,
    },
  );
  if (result.error != null) throw result.error;
  const outputRoot = path.join(workspaceDir, outputDirectory);
  const files = fs.existsSync(outputRoot)
    ? fs
        .readdirSync(outputRoot)
        .filter((entry) => entry.endsWith(".json"))
        .sort()
    : [];
  const outputText =
    files.length === 1 ? fs.readFileSync(path.join(outputRoot, files[0]), "utf8") : "";
  return {
    files,
    output: outputText === "" ? ({} as CompilerOutput) : (JSON.parse(outputText) as CompilerOutput),
    outputText,
    status: result.status,
    stderr: result.stderr,
    stdout: result.stdout,
  };
}

function sha256(source: string | Buffer): string {
  return createHash("sha256").update(source).digest("hex");
}

function count(source: string, needle: string): number {
  return source.split(needle).length - 1;
}

function assertOrdered(source: string, needles: string[]): void {
  const offsets = needles.map((needle) => source.indexOf(needle));
  assert.equal(
    offsets.every((offset) => offset >= 0),
    true,
    source,
  );
  assert.equal(
    offsets.slice(1).every((offset, index) => offsets[index] < offset),
    true,
    source,
  );
}

const wouldReformatOutput = `Found 1 file(s)
Would reformat: ${appPath}

Checked 1 file(s)
  1 file(s) would be reformatted
`;

const reformattedOutput = `Found 1 file(s)
Reformatted: ${appPath}

Formatted 1 file(s)
  1 file(s) reformatted
`;

const alreadyFormattedOutput = `Found 1 file(s)

Checked 1 file(s)
  1 file(s) already formatted
`;

const unchangedOutput = `Found 1 file(s)

Formatted 1 file(s)
  1 file(s) unchanged
`;
