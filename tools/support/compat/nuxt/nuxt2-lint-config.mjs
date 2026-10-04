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
    const dirs = api.collectNuxtLintDirs(api.toNuxtLintProjectState(nuxt.options));
    const features = api.resolveNuxtLintFeatures(undefined, () => false);
    const source = fs.readFileSync(path.join(corpusDir, corpus.source));
    const input = path.join(artifacts, corpus.source);
    fs.writeFileSync(input, source);
    const requireRoot = createRequire(path.join(root, "package.json"));
    const oxlintRoot = path.dirname(requireRoot.resolve("oxlint/package.json"));
    const rows = [];
    for (const entry of corpus.cases) {
      const configFile =
        entry.nuxtVersion === 2
          ? generated
          : path.join(path.dirname(generated), `${entry.id}.json`);
      if (entry.nuxtVersion !== 2) {
        fs.writeFileSync(
          configFile,
          api.renderNuxtOxlintConfig(
            api.buildNuxtLintPlan(features, dirs, entry.nuxtVersion),
            artifact.jsPlugins[0].specifier,
            { rootDir: fixture, configDir: path.dirname(configFile) },
          ),
        );
      }
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
      assert.equal(run.status, entry.nuxtVersion === 2 ? 0 : 1, run.stderr);
      const report = JSON.parse(run.stdout);
      assert.deepEqual(
        report.diagnostics.map(({ code }) => code),
        entry.expectedDiagnosticCodes,
      );
      if (entry.nuxtVersion !== 2) {
        assert.deepEqual(
          report.diagnostics.map(({ message }) => message),
          [
            "Replace `process.client` with `import.meta.client`.",
            "Replace `process.server` with `import.meta.server`.",
          ],
        );
      }
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
    await nuxt.close();
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await runProbe(path.resolve(process.argv[2]), path.resolve(process.argv[3]));
}
