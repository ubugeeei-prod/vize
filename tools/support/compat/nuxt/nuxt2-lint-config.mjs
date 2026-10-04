// Keep the original webpack/SSR fixture intact and separately exercise lint setup.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const corpusDir = path.join(root, "npm/framework/nuxt-lint-config/test/nuxt-version-compat");
const corpus = JSON.parse(fs.readFileSync(path.join(corpusDir, "corpus.json"), "utf8"));

function distHashes(directory) {
  return Object.fromEntries(
    fs
      .readdirSync(directory, { recursive: true })
      .sort((left, right) => left.localeCompare(right))
      .flatMap((file) => {
        const entry = path.join(directory, file);
        return fs.statSync(entry).isFile()
          ? [[file, createHash("sha256").update(fs.readFileSync(entry)).digest("hex")]]
          : [];
      }),
  );
}

export function verifyNuxt2LintConfig(fixture, artifacts) {
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

async function runProbe(fixture, artifacts) {
  const requireFixture = createRequire(path.join(fixture, "package.json"));
  const { loadNuxt } = requireFixture("nuxt");
  const packages = {};
  for (const name of ["nuxt", "nuxt-lint-config"]) {
    const installed = path.join(fixture, "node_modules/@vizejs", name, "dist");
    packages[name] = distHashes(installed);
    assert.deepEqual(packages[name], distHashes(path.join(root, "npm/framework", name, "dist")));
  }
  const configBytes = fs.readFileSync(path.join(fixture, "nuxt.config.js"));
  const nuxt = await loadNuxt({
    rootDir: fixture,
    for: "build",
    configOverrides: {
      buildDir: path.join(artifacts, "lint-build"),
      vize: { compiler: false, lint: { autoInit: false }, musea: false },
    },
  });
  const temporaryFiles = [];
  try {
    assert.equal(nuxt.constructor.version, "v2.17.3");
    const generated = path.join(nuxt.options.buildDir, "oxlint.config.json");
    const artifact = JSON.parse(fs.readFileSync(generated, "utf8"));
    assert.equal(artifact.settings.vize.preset, "incremental");
    assert.equal(artifact.rules?.["vize/nuxt/prefer-import-meta"], undefined);
    assert.ok(
      artifact.overrides.every(
        ({ rules }) => rules?.["vize/nuxt/no-page-meta-runtime-values"] === undefined,
      ),
    );

    const lintEntry = path.join(fixture, "node_modules/@vizejs/nuxt/dist/lint/index.mjs");
    const api = await import(pathToFileURL(lintEntry).href);
    // Preserve the default module artifact, then use the public root-config
    // option for CLI execution: oxlint 1.78 rejects parent-relative ignores.
    const rootConfig = path.join(fixture, "issue-7828-oxlint.config.json");
    assert.equal(fs.existsSync(rootConfig), false);
    temporaryFiles.push(rootConfig);
    const generation = await api.setupNuxtLintConfigGeneration(
      { autoInit: false, configFile: rootConfig },
      nuxt,
    );
    assert.equal(generation.configFile, rootConfig);
    const lintArtifact = JSON.parse(fs.readFileSync(rootConfig, "utf8"));
    fs.copyFileSync(rootConfig, path.join(artifacts, "nuxt2-root-oxlint.config.json"));
    const dirs = api.collectNuxtLintDirs(api.toNuxtLintProjectState(nuxt.options));
    const features = api.resolveNuxtLintFeatures(undefined, () => false);
    const source = fs.readFileSync(path.join(corpusDir, corpus.source));
    const input = path.join(fixture, "issue-7828-process-flags.ts");
    fs.writeFileSync(input, source, { flag: "wx" });
    temporaryFiles.push(input);
    fs.writeFileSync(path.join(artifacts, corpus.source), source);
    const requireRoot = createRequire(path.join(root, "package.json"));
    const oxlintRoot = path.dirname(requireRoot.resolve("oxlint/package.json"));
    const rows = [];
    for (const entry of corpus.cases) {
      const configFile =
        entry.nuxtVersion === 2 ? rootConfig : path.join(fixture, `${entry.id}.json`);
      if (entry.nuxtVersion !== 2) {
        assert.equal(fs.existsSync(configFile), false);
        temporaryFiles.push(configFile);
        fs.writeFileSync(
          configFile,
          api.renderNuxtOxlintConfig(
            api.buildNuxtLintPlan(features, dirs, entry.nuxtVersion),
            lintArtifact.jsPlugins[0].specifier,
            { rootDir: fixture, configDir: path.dirname(configFile) },
          ),
        );
      }
      fs.copyFileSync(configFile, path.join(artifacts, `${entry.id}-config.json`));
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
      fs.writeFileSync(path.join(artifacts, `${entry.id}-stdout.json`), run.stdout ?? "");
      fs.writeFileSync(path.join(artifacts, `${entry.id}-stderr.log`), run.stderr ?? "");
      assert.equal(run.error, undefined);
      assert.equal(run.signal, null);
      assert.equal(run.status, entry.nuxtVersion === 2 ? 0 : 1, run.stderr || run.stdout);
      const report = JSON.parse(run.stdout);
      assert.equal(report.number_of_files, 1);
      assert.deepEqual(report.diagnostics, entry.expectedDiagnostics);
      rows.push({ id: entry.id, configFile, exit: run.status, diagnostics: report.diagnostics });
    }
    fs.writeFileSync(
      path.join(artifacts, "lint-proof.json"),
      JSON.stringify(
        {
          executionHead: process.env.GITHUB_SHA ?? null,
          nuxt: nuxt.constructor.version,
          packages,
          fixtureConfigSha256: createHash("sha256").update(configBytes).digest("hex"),
          sourceSha256: createHash("sha256").update(source).digest("hex"),
          oxlint: requireRoot("oxlint/package.json").version,
          plugin: JSON.parse(
            fs.readFileSync(
              path.join(fixture, "node_modules/oxlint-plugin-vize/package.json"),
              "utf8",
            ),
          ).version,
          native: JSON.parse(
            fs.readFileSync(path.join(fixture, "node_modules/@vizejs/native/package.json"), "utf8"),
          ).version,
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
  await runProbe(path.resolve(process.argv[2]), path.resolve(process.argv[3]));
}
