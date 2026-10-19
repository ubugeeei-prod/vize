// Keep the original webpack/SSR fixture intact and separately exercise lint setup.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { asArray, asRecord, asString, at, parseJson, readJson } from "../../json-value.ts";
import { importChecked, loadNuxt2 } from "./host-types.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const corpusDir = path.join(root, "npm/framework/nuxt-lint-config/test/nuxt-version-compat");
const corpus = asRecord(readJson(path.join(corpusDir, "corpus.json")), "lint corpus");

interface NuxtLintApi {
  collectNuxtLintDirs(state: unknown): unknown;
  toNuxtLintProjectState(options: unknown): unknown;
  resolveNuxtLintFeatures(options: undefined, hasModule: () => boolean): unknown;
  buildNuxtLintPlan(features: unknown, dirs: unknown, nuxtVersion: unknown): unknown;
  renderNuxtOxlintConfig(plan: unknown, specifier: unknown, paths: Record<string, string>): string;
}

function distHashes(directory: string): Record<string, string> {
  return Object.fromEntries(
    fs
      .readdirSync(directory, { recursive: true, encoding: "utf8" })
      .sort((left, right) => left.localeCompare(right))
      .flatMap((file) => {
        const entry = path.join(directory, file);
        return fs.statSync(entry).isFile()
          ? [[file, createHash("sha256").update(fs.readFileSync(entry)).digest("hex")]]
          : [];
      }),
  );
}

export function verifyNuxt2LintConfig(fixture: string, artifacts: string): void {
  // The module resolves itself from cwd, as in the original Nuxt CLI build.
  const run = spawnSync(process.execPath, [fileURLToPath(import.meta.url), fixture, artifacts], {
    cwd: fixture,
    stdio: "inherit",
    timeout: 60_000,
  });
  assert.equal(run.error, undefined);
  assert.equal(run.signal, null);
  assert.equal(run.status, 0);
}

async function runProbe(fixture: string, artifacts: string): Promise<void> {
  const packages: Record<string, Record<string, string>> = {};
  for (const name of ["nuxt", "nuxt-lint-config"]) {
    const installed = distHashes(path.join(fixture, "node_modules/@vizejs", name, "dist"));
    packages[name] = installed;
    assert.deepEqual(installed, distHashes(path.join(root, "npm/framework", name, "dist")));
  }
  const configBytes = fs.readFileSync(path.join(fixture, "nuxt.config.js"));
  const nuxt = await loadNuxt2(fixture, {
    rootDir: fixture,
    for: "build",
    configOverrides: {
      buildDir: path.join(artifacts, "lint-build"),
      vize: { compiler: false, lint: { autoInit: false }, musea: false },
    },
  });
  const temporaryFiles: string[] = [];
  try {
    const nuxtVersion = at(nuxt, "constructor", "version");
    assert.equal(nuxtVersion, "v2.17.3");
    const generated = path.join(fixture, ".oxlint.vize.json");
    temporaryFiles.push(generated);
    const artifact = readJson(generated);
    assert.equal(at(artifact, "settings", "vize", "preset"), "incremental");
    assert.equal(at(artifact, "rules", "vize/nuxt/prefer-import-meta"), undefined);
    assert.ok(
      asArray(at(artifact, "overrides"), "overrides").every(
        (override) => at(override, "rules", "vize/nuxt/no-page-meta-runtime-values") === undefined,
      ),
    );

    const lintEntry = path.join(fixture, "node_modules/@vizejs/nuxt/dist/lint/index.mjs");
    const api = await importChecked<NuxtLintApi>(lintEntry, [
      "collectNuxtLintDirs",
      "toNuxtLintProjectState",
      "resolveNuxtLintFeatures",
      "buildNuxtLintPlan",
      "renderNuxtOxlintConfig",
    ]);
    // Execute the actual corrected default; no root-placement workaround.
    const rootConfig = generated;
    const pluginSpecifier = at(readJson(rootConfig), "jsPlugins", 0, "specifier");
    fs.copyFileSync(rootConfig, path.join(artifacts, "nuxt2-root-oxlint.config.json"));
    const dirs = api.collectNuxtLintDirs(api.toNuxtLintProjectState(nuxt.options));
    const features = api.resolveNuxtLintFeatures(undefined, () => false);
    const sourceName = asString(corpus.source, "corpus.source");
    const source = fs.readFileSync(path.join(corpusDir, sourceName));
    const input = path.join(fixture, "issue-7828-process-flags.ts");
    fs.writeFileSync(input, source, { flag: "wx" });
    temporaryFiles.push(input);
    fs.writeFileSync(path.join(artifacts, sourceName), source);
    const requireRoot = createRequire(path.join(root, "package.json"));
    const oxlintRoot = path.dirname(requireRoot.resolve("oxlint/package.json"));
    const rows: unknown[] = [];
    for (const value of asArray(corpus.cases, "corpus.cases")) {
      const entry = asRecord(value, "lint case");
      const id = asString(entry.id, "case.id");
      const configFile = entry.nuxtVersion === 2 ? rootConfig : path.join(fixture, `${id}.json`);
      if (entry.nuxtVersion !== 2) {
        assert.equal(fs.existsSync(configFile), false);
        temporaryFiles.push(configFile);
        fs.writeFileSync(
          configFile,
          api.renderNuxtOxlintConfig(
            api.buildNuxtLintPlan(features, dirs, entry.nuxtVersion),
            pluginSpecifier,
            { rootDir: fixture, configDir: path.dirname(configFile) },
          ),
        );
      }
      fs.copyFileSync(configFile, path.join(artifacts, `${id}-config.json`));
      const run = spawnSync(
        process.execPath,
        [path.join(oxlintRoot, "bin/oxlint"), "-c", configFile, "-f", "json", input],
        {
          cwd: fixture,
          encoding: "utf8",
          timeout: 30_000,
          env: { ...process.env, NO_COLOR: "1" },
        },
      );
      fs.writeFileSync(path.join(artifacts, `${id}-stdout.json`), run.stdout ?? "");
      fs.writeFileSync(path.join(artifacts, `${id}-stderr.log`), run.stderr ?? "");
      assert.equal(run.error, undefined);
      assert.equal(run.signal, null);
      assert.equal(run.status, entry.nuxtVersion === 2 ? 0 : 1, run.stderr || run.stdout);
      const report = asRecord(parseJson(run.stdout), "oxlint report");
      assert.equal(report.number_of_files, 1);
      assert.deepEqual(report.diagnostics, entry.expectedDiagnostics);
      rows.push({ id, configFile, exit: run.status, diagnostics: report.diagnostics });
    }
    fs.writeFileSync(
      path.join(artifacts, "lint-proof.json"),
      JSON.stringify(
        {
          executionHead: process.env.GITHUB_SHA ?? null,
          nuxt: nuxtVersion,
          packages,
          fixtureConfigSha256: createHash("sha256").update(configBytes).digest("hex"),
          sourceSha256: createHash("sha256").update(source).digest("hex"),
          oxlint: at(requireRoot("oxlint/package.json"), "version"),
          plugin: at(
            readJson(path.join(fixture, "node_modules/oxlint-plugin-vize/package.json")),
            "version",
          ),
          native: at(
            readJson(path.join(fixture, "node_modules/@vizejs/native/package.json")),
            "version",
          ),
          rows,
        },
        null,
        2,
      ) + "\n",
    );
    assert.deepEqual(fs.readFileSync(path.join(fixture, "nuxt.config.js")), configBytes);
    console.log("Genuine Nuxt 2 lint generation and authored Nuxt 3/4 plan controls passed");
  } finally {
    try {
      await nuxt.close();
    } finally {
      for (const file of temporaryFiles) fs.rmSync(file, { force: true });
    }
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [fixture, artifacts] = process.argv.slice(2);
  assert.ok(fixture && artifacts, "usage: nuxt2-lint-config.ts FIXTURE ARTIFACTS");
  await runProbe(path.resolve(fixture), path.resolve(artifacts));
}
