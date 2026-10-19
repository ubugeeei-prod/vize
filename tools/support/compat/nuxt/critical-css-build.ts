// #6897: actual Nuxt client/SSR/prerender, styles manifest, and browser CSS without JS.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { assertCriticalCss, observeCriticalCss } from "./critical-css-observe.mjs";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const fixture = path.join(root, "tests/_fixtures/_projects/nuxt-critical-css-build");
const artifacts = path.resolve(process.argv[2] ?? path.join(os.tmpdir(), "vize-nuxt-critical-css"));
const dependencies = fs.realpathSync(process.argv[3] ?? path.join(fixture, "node_modules"));
fs.mkdirSync(artifacts, { recursive: true });
const sha256 = (data) => createHash("sha256").update(data).digest("hex");
const write = (name, data) =>
  fs.writeFileSync(path.join(artifacts, name), JSON.stringify(data, null, 2) + "\n");
const versions = {};
for (const [name, version] of [
  ["nuxt", "4.5.2"],
  ["vue", "3.5.43"],
  ["vite", "8.3.1"],
  ["rolldown", "1.2.11"],
  ["@vizejs/nuxt", "0.429.0"],
  ["@vizejs/native", "0.429.0"],
  ["@vizejs/vite-plugin", "0.429.0"],
]) {
  const actual = JSON.parse(fs.readFileSync(path.join(dependencies, name, "package.json"), "utf8"));
  assert.equal(actual.version, version);
  versions[name] = actual.version;
}
const inputs = ["nuxt.config.ts", "app/app.vue", "app/layouts/default.vue", "app/pages/index.vue"];
const inputHashes = Object.fromEntries(
  inputs.map((name) => [name, sha256(fs.readFileSync(path.join(fixture, name)))]),
);
const packageFiles = (dir) =>
  fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const filename = path.join(dir, entry.name);
    return entry.isDirectory() ? packageFiles(filename) : [filename];
  });
const originalPackages = {};
for (const name of ["nuxt", "vite-plugin"]) {
  const dir = path.join(dependencies, "@vizejs", name);
  for (const filename of packageFiles(dir))
    originalPackages[filename] = sha256(fs.readFileSync(filename));
}

function linkDependencies(workspace, candidate) {
  const dir = path.join(workspace, "node_modules");
  fs.mkdirSync(path.join(dir, "@vizejs"), { recursive: true });
  for (const entry of fs.readdirSync(dependencies)) {
    if (["@vizejs", ".cache"].includes(entry)) continue;
    fs.symlinkSync(path.join(dependencies, entry), path.join(dir, entry), "dir");
  }
  fs.mkdirSync(path.join(dir, ".cache"));
  for (const entry of fs.readdirSync(path.join(dependencies, "@vizejs"))) {
    const installed = path.join(dependencies, "@vizejs", entry);
    const target = path.join(dir, "@vizejs", entry);
    if (!["nuxt", "vite-plugin"].includes(entry)) {
      fs.symlinkSync(installed, target, "dir");
      continue;
    }
    fs.cpSync(installed, target, { recursive: true });
    if (candidate) {
      const source = path.join(
        root,
        entry === "nuxt" ? "npm/framework/nuxt/dist" : "npm/builder/vite/dist",
      );
      assert.equal(
        fs.existsSync(path.join(source, "index.mjs")),
        true,
        "candidate JS must be source-built first",
      );
      fs.rmSync(path.join(target, "dist"), { recursive: true });
      fs.cpSync(source, path.join(target, "dist"), { recursive: true });
    }
  }
}

const results = {};
for (const variant of ["stock", "published", "candidate"]) {
  const workspace = path.join(artifacts, variant);
  assert.equal(fs.existsSync(workspace), false, "use a new artifact directory per proof run");
  fs.mkdirSync(workspace);
  for (const name of ["package.json", ...inputs]) {
    const target = path.join(workspace, name);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(fixture, name), target);
  }
  if (variant === "stock") {
    const filename = path.join(workspace, "nuxt.config.ts");
    const config = fs.readFileSync(filename, "utf8");
    assert.equal(config.split('modules: ["@vizejs/nuxt"]').length, 2);
    fs.writeFileSync(filename, config.replace('modules: ["@vizejs/nuxt"]', "modules: []"));
  }
  linkDependencies(workspace, variant === "candidate");
  const log = fs.openSync(path.join(artifacts, `${variant}-build.log`), "w");
  let build;
  try {
    build = spawnSync(
      process.execPath,
      [path.join(workspace, "node_modules/nuxt/bin/nuxt.mjs"), "build"],
      {
        cwd: workspace,
        env: { ...process.env, NO_COLOR: "1" },
        stdio: ["ignore", log, log],
        timeout: 240_000,
      },
    );
  } finally {
    fs.closeSync(log);
  }
  assert.equal(build.error, undefined);
  assert.equal(build.status, 0, `Nuxt ${variant} build must pass; see ${variant}-build.log`);
  for (const name of inputs.filter((name) => name !== "nuxt.config.ts")) {
    assert.equal(sha256(fs.readFileSync(path.join(workspace, name))), inputHashes[name]);
  }
  results[variant] = await observeCriticalCss(workspace, root);
  assertCriticalCss(results[variant], variant === "published");
  fs.copyFileSync(
    path.join(workspace, ".output/public/index.html"),
    path.join(artifacts, `${variant}-prerender.html`),
  );
  write(`${variant}-observation.json`, results[variant]);
  console.log(`${variant}: full style manifest and no-JavaScript computed CSS verified`);
}
for (const [filename, hash] of Object.entries(originalPackages))
  assert.equal(sha256(fs.readFileSync(filename)), hash);
const target =
  process.platform === "darwin"
    ? `native-darwin-${process.arch}`
    : `native-linux-${process.arch}-gnu`;
const addonDir = path.join(dependencies, "@vizejs", target);
const addons = fs.readdirSync(addonDir).filter((name) => name.endsWith(".node"));
assert.equal(addons.length, 1);
const addon = path.join(addonDir, addons[0]);
const candidateFiles = {};
for (const dir of ["npm/builder/vite/dist", "npm/framework/nuxt/dist"]) {
  for (const filename of packageFiles(path.join(root, dir)))
    candidateFiles[path.relative(root, filename)] = sha256(fs.readFileSync(filename));
}
write("proof.json", {
  issue: 6897,
  candidateHead: process.env.GITHUB_SHA ?? null,
  node: process.version,
  versions,
  inputHashes,
  nativeAddon: { path: addon, sha256: sha256(fs.readFileSync(addon)) },
  candidateFiles,
  variants: Object.keys(results),
  publishedPackageBytesPreserved: true,
  nativeAcceptance: 0,
});
