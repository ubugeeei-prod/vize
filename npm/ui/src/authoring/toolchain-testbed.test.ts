import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";

import ts from "typescript";
import { test } from "vite-plus/test";

import { uiFamilyCatalog } from "../catalog/family-catalog.ts";

test("package check keeps the UI source corpus on the toolchain gate", async () => {
  const manifest = JSON.parse(await readFile(path.resolve("package.json"), "utf8")) as {
    readonly scripts: Readonly<Record<string, string>>;
  };

  assert.equal(
    manifest.scripts["lint:sfc"],
    "vp exec node scripts/lint-sfc.ts src && vp exec node scripts/check-renderers.ts src",
  );
  assert.equal(
    manifest.scripts["check:story-testbed"],
    "vp exec node scripts/story-testbed.ts check --format json",
  );
  assert.equal(
    manifest.scripts.check,
    "pnpm lint:sfc && pnpm check:static && pnpm check:story-testbed",
  );
  assert.match(manifest.scripts["check:static"], /vue-tsc --noEmit -p tsconfig\.typecheck\.json/);
  assert.equal(manifest.scripts.fmt, "vp fmt --write src scripts vite.config.ts");
});

test("renderer conformance script owns the DOM, SSR, and Vapor lanes", async () => {
  const checkRendererSource = await readFile(path.resolve("scripts/check-renderers.ts"), "utf8");

  assert.ok(checkRendererSource.includes('name: "dom"'));
  assert.ok(checkRendererSource.includes('name: "ssr"'));
  assert.ok(checkRendererSource.includes('name: "vapor"'));
  assert.ok(checkRendererSource.includes("compileSfc"));
  assert.ok(checkRendererSource.includes("sourceFiles.length + inlineFixtures.length"));
});

test("sfc lint script owns the Patina opinionated authoring lane", async () => {
  const packageLintSource = await readFile(path.resolve("scripts/lint-sfc.ts"), "utf8");
  const sharedLintSource = await readFile(
    path.resolve("scripts/source-quality/lint-sfc.ts"),
    "utf8",
  );

  assert.ok(packageLintSource.includes("lintPatinaSfc"));
  assert.ok(packageLintSource.includes("runSfcLintCli"));
  assert.ok(sharedLintSource.includes('preset: "opinionated"'));
  assert.ok(sharedLintSource.includes("typeAware: true"));
  assert.ok(sharedLintSource.includes('helpLevel: "short"'));
  assert.ok(sharedLintSource.includes('entry.name.endsWith(".vue")'));
  assert.ok(sharedLintSource.includes("[...new Set(discovered.flat())].sort()"));
});

test("static typecheck keeps catalogued public contracts on the Canon lane", async () => {
  const tsconfig = JSON.parse(await readFile(path.resolve("tsconfig.typecheck.json"), "utf8")) as {
    readonly include: readonly string[];
    readonly exclude: readonly string[];
    readonly compilerOptions: { readonly noEmit: boolean };
  };

  assert.equal(tsconfig.compilerOptions.noEmit, true);
  assert.deepEqual(tsconfig.include, ["src/**/*.ts", "src/**/*.vue"]);

  const typeTestFiles = uiFamilyCatalog.flatMap((entry) => entry.typeTests ?? []);
  const parsedConfig = parseTypecheckConfig();
  const program = ts.createProgram({
    rootNames: parsedConfig.fileNames,
    options: parsedConfig.options,
  });
  const programFiles = new Set(
    [...program.getRootFileNames(), ...program.getSourceFiles().map((file) => file.fileName)].map(
      (file) => path.resolve(file),
    ),
  );

  assert.ok(typeTestFiles.length >= 70, "catalogued families must publish compile-only contracts");
  for (const file of typeTestFiles) {
    assert.ok(file.endsWith(".types.test-d.ts"), `${file} must be a compile-only type contract`);
    await readFile(path.resolve(file), "utf8");
    assert.ok(
      programFiles.has(path.resolve(file)),
      `${file} must be included in the resolved tsconfig.typecheck.json program`,
    );
  }
});

function parseTypecheckConfig() {
  const configPath = path.resolve("tsconfig.typecheck.json");
  const config = ts.readConfigFile(configPath, (fileName) => ts.sys.readFile(fileName));
  assert.equal(config.error, undefined);

  const parsed = ts.parseJsonConfigFileContent(
    config.config,
    ts.sys,
    path.dirname(configPath),
    undefined,
    configPath,
  );
  assert.deepEqual(parsed.errors, []);
  return parsed;
}
