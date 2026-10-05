// Preserve the original Nuxt project and run its real Vite+ lint configuration.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const inputDir = path.join(root, "npm/framework/nuxt/src/lint/fixtures/auto-init-vite-lint");
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

function distHashes(directory) {
  return Object.fromEntries(
    fs
      .readdirSync(directory, { recursive: true })
      .sort((left, right) => left.localeCompare(right))
      .filter((file) => fs.statSync(path.join(directory, file)).isFile())
      .map((file) => [file, hash(fs.readFileSync(path.join(directory, file)))]),
  );
}

export function verifyNuxtAutoInitConfig(fixture, artifacts) {
  // Vite+ 0.1.24 selects workspace-root lint settings. A nested reproduction
  // otherwise runs this repository's rules instead of the issue's Vite config.
  const project = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-auto-init-"));
  try {
    const tracked = spawnSync("git", ["ls-files", "-z", "--", path.relative(root, fixture)], {
      cwd: root,
      encoding: "utf8",
    });
    assert.equal(tracked.error, undefined);
    assert.equal(tracked.status, 0);
    const original = {};
    for (const file of tracked.stdout.split("\0").filter(Boolean)) {
      const relative = path.relative(fixture, path.join(root, file));
      const bytes = fs.readFileSync(path.join(root, file));
      const target = path.join(project, relative);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, bytes, { flag: "wx" });
      original[relative] = hash(bytes);
    }
    assert.equal(Object.keys(original).length, 5);
    const modules = path.join(project, "node_modules");
    fs.mkdirSync(modules);
    for (const name of fs.readdirSync(path.join(fixture, "node_modules"))) {
      fs.symlinkSync(path.join(fixture, "node_modules", name), path.join(modules, name));
    }
    assert.equal(fs.existsSync(path.join(modules, "vite-plus")), false);
    fs.symlinkSync(path.join(root, "node_modules/vite-plus"), path.join(modules, "vite-plus"));
    const run = spawnSync(process.execPath, [fileURLToPath(import.meta.url), project, artifacts], {
      cwd: project,
      stdio: "inherit",
      timeout: 60_000,
    });
    assert.equal(run.error, undefined);
    assert.equal(run.signal, null);
    assert.equal(run.status, 0);
    for (const [file, digest] of Object.entries(original)) {
      assert.equal(hash(fs.readFileSync(path.join(project, file))), digest);
      assert.equal(hash(fs.readFileSync(path.join(fixture, file))), digest);
    }
    fs.writeFileSync(
      path.join(artifacts, "issue-7829-original-project-inventory.json"),
      JSON.stringify(original, null, 2) + "\n",
    );
  } finally {
    fs.rmSync(project, { recursive: true });
  }
}

async function runProbe(fixture, artifacts) {
  const configBytes = fs.readFileSync(path.join(fixture, "nuxt.config.js"));
  const viteBytes = fs.readFileSync(path.join(inputDir, "vite.config.ts"));
  const source = fs.readFileSync(path.join(inputDir, "NoKey.vue.txt"));
  const viteFile = path.join(fixture, "vite.config.ts");
  const input = path.join(fixture, "issue-7829-no-key.vue");
  const wrapper = path.join(fixture, "oxlint.config.mts");
  const files = [];
  let nuxt;
  try {
    assert.equal(fs.existsSync(wrapper), false);
    for (const [file, bytes] of [
      [viteFile, viteBytes],
      [input, source],
    ]) {
      fs.writeFileSync(file, bytes, { flag: "wx" });
      files.push(file);
    }
    const before = fs.statSync(viteFile);
    fs.writeFileSync(path.join(artifacts, "issue-7829-vite.config.ts"), viteBytes);
    fs.writeFileSync(path.join(artifacts, "issue-7829-no-key.vue"), source);
    const installedDist = path.join(fixture, "node_modules/@vizejs/nuxt/dist");
    const candidate = distHashes(installedDist);
    assert.deepEqual(candidate, distHashes(path.join(root, "npm/framework/nuxt/dist")));
    const vpRoot = path.join(root, "node_modules/vite-plus");
    const runLint = (phase) => {
      const run = spawnSync(
        process.execPath,
        [path.join(vpRoot, "bin/vp"), "lint", input, "--format", "json"],
        { cwd: fixture, encoding: "utf8", timeout: 30_000, env: { ...process.env, NO_COLOR: "1" } },
      );
      fs.writeFileSync(path.join(artifacts, `issue-7829-${phase}-stdout.json`), run.stdout ?? "");
      fs.writeFileSync(path.join(artifacts, `issue-7829-${phase}-stderr.log`), run.stderr ?? "");
      assert.equal(run.error, undefined);
      assert.equal(run.signal, null);
      assert.equal(run.status, 1, run.stderr || run.stdout);
      const report = JSON.parse(run.stdout);
      assert.equal(report.number_of_files, 1);
      assert.equal(report.diagnostics.length, 1);
      assert.equal(report.diagnostics[0].code, "vize(vue/require-v-for-key)");
      assert.equal(report.diagnostics[0].severity, "error");
      assert.equal(report.diagnostics[0].filename, "issue-7829-no-key.vue");
      return report.diagnostics;
    };
    const initial = runLint("before");
    const requireFixture = createRequire(path.join(fixture, "package.json"));
    nuxt = await requireFixture("nuxt").loadNuxt({
      rootDir: fixture,
      for: "build",
      configOverrides: {
        buildDir: path.join(artifacts, "auto-init-build"),
        vize: { compiler: false, lint: true, musea: false },
      },
    });
    assert.equal(nuxt.constructor.version, "v2.17.3");
    const generated = path.join(nuxt.options.buildDir, "oxlint.config.json");
    assert.equal(
      JSON.parse(fs.readFileSync(generated, "utf8")).settings.vize.preset,
      "incremental",
    );
    const wrapperAdded = fs.existsSync(wrapper);
    if (wrapperAdded) {
      const bytes = fs.readFileSync(wrapper);
      fs.writeFileSync(path.join(artifacts, "issue-7829-added-oxlint.config.mts"), bytes);
      if (bytes.toString().startsWith("// Generated by @vizejs/nuxt.\n")) files.push(wrapper);
    }
    const current = runLint("after");
    assert.deepEqual(current, initial);
    assert.deepEqual(fs.readFileSync(viteFile), viteBytes);
    const after = fs.statSync(viteFile);
    assert.equal(after.mtimeMs, before.mtimeMs);
    assert.equal(after.ino, before.ino);
    fs.writeFileSync(
      path.join(artifacts, "issue-7829-auto-init-proof.json"),
      JSON.stringify(
        {
          executionHead: process.env.GITHUB_SHA ?? null,
          nuxt: requireFixture("nuxt/package.json").version,
          vitePlus: JSON.parse(fs.readFileSync(path.join(vpRoot, "package.json"), "utf8")).version,
          plugin: JSON.parse(
            fs.readFileSync(
              path.join(fixture, "node_modules/oxlint-plugin-vize/package.json"),
              "utf8",
            ),
          ).version,
          native: JSON.parse(
            fs.readFileSync(path.join(fixture, "node_modules/@vizejs/native/package.json"), "utf8"),
          ).version,
          candidate,
          originalConfigSha256: hash(configBytes),
          viteConfigSha256: hash(viteBytes),
          sourceSha256: hash(source),
          wrapperAdded,
          diagnostics: current,
        },
        null,
        2,
      ) + "\n",
    );
    assert.deepEqual(fs.readFileSync(path.join(fixture, "nuxt.config.js")), configBytes);
    assert.equal(wrapperAdded, false, "auto-init created a competing root config");
    console.log(
      "Genuine Nuxt auto-init preserves the original Vite+ config and whole CLI witnesses",
    );
  } finally {
    try {
      if (nuxt) await nuxt.close();
    } finally {
      for (const file of files) fs.rmSync(file);
    }
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await runProbe(path.resolve(process.argv[2]), path.resolve(process.argv[3]));
}
