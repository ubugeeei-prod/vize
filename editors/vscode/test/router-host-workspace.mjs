import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { prepareExperimentalRouterProject } from "../../../tests/tooling/support/router-experimental-provider.ts";

export function readRouterEditorFixture(repository) {
  const directory = path.join(repository, "tests/fixtures/editor/experimental-router-5.4");
  const authority = JSON.parse(fs.readFileSync(path.join(directory, "provider.json"), "utf8"));
  assert.equal(authority.schema, "vize.router-editor-artifact-provider");
  assert.equal(authority.version, 1);
  const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
  assert.equal(
    digest(fs.readFileSync(path.join(repository, authority.parentProvider))),
    authority.parentProviderSha256,
  );
  const files = new Map();
  for (const entry of authority.files) {
    assert.equal(path.isAbsolute(entry.path), false);
    assert.equal(entry.path.split(/[\\/]/u).includes(".."), false);
    assert.equal(files.has(entry.path), false);
    const bytes = fs.readFileSync(path.join(directory, `${entry.path}.txt`));
    assert.equal(bytes.length, entry.bytes);
    assert.equal(digest(bytes), entry.sha256);
    files.set(entry.path, bytes);
  }
  return { authority, files };
}

export async function prepareRouterHostWorkspace({
  directory,
  output,
  repository,
  sourceExtension,
  corsa,
}) {
  const fixture = readRouterEditorFixture(repository);
  const provider = await prepareExperimentalRouterProject(path.join(output, "provider"));
  const proxy = path.join(sourceExtension, "test/fixtures/passive-vize-server.cjs");
  // Use the same passive forwarder for the compiler side as well. Its raw
  // chunks preserve in-memory Art projections that have no on-disk mirror.
  const observedCorsa = path.join(directory, "typescript-observed");
  const compilerLauncher = `#!${process.execPath}\nprocess.env.VIZE_TEST_OBSERVED_SERVER_PATH=${JSON.stringify(corsa)};\nprocess.env.VIZE_TEST_CONFIG_PROTOCOL_PATH=${JSON.stringify(path.join(output, "compiler-protocol.jsonl"))};\nrequire(${JSON.stringify(proxy)});\n`;
  fs.writeFileSync(observedCorsa, compilerLauncher);
  fs.chmodSync(observedCorsa, 0o755);
  fs.writeFileSync(path.join(output, "compiler-launcher.js.txt"), compilerLauncher);
  const projects = [];
  for (const name of ["enabled", "disabled"]) {
    const root = path.join(directory, name);
    fs.cpSync(provider.root, root, {
      recursive: true,
      filter: (file) => !file.split(path.sep).includes("node_modules"),
    });
    fs.symlinkSync(
      fs.realpathSync(path.join(provider.root, "node_modules")),
      path.join(root, "node_modules"),
      "junction",
    );
    for (const [relative, bytes] of fixture.files) {
      const file = path.join(root, relative);
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, bytes);
    }
    run("official-expanded-generation", process.execPath, ["generate.mjs"], root, output, name);
    assert.deepEqual(
      fs.readFileSync(path.join(root, "typed-router.d.ts")),
      fixture.files.get("typed-router.d.ts"),
    );
    // These full scanner outputs are retained outside authored tsconfig roots.
    for (const file of ["routes.generated.ts", "resolver.generated.ts"]) {
      fs.copyFileSync(path.join(root, file), path.join(output, `${name}-${file}.txt`));
      fs.rmSync(path.join(root, file));
    }
    run(
      "official-reference-check",
      process.execPath,
      ["node_modules/vue-tsc/bin/vue-tsc.js", "--noEmit", "--project", "tsconfig.json"],
      root,
      output,
      name,
    );
    const config = path.join(root, "tsconfig.json");
    const configValue = JSON.parse(fs.readFileSync(config, "utf8"));
    if (name === "disabled") configValue.vueCompilerOptions.plugins = [];
    fs.writeFileSync(config, `${JSON.stringify(configValue, null, 2)}\n`);
    fs.writeFileSync(
      path.join(root, "vite.config.mjs"),
      `export default ${JSON.stringify({ vue: { typeChecker: { corsaPath: observedCorsa }, linter: { enabled: false } } })};\n`,
    );
    projects.push({
      name,
      root,
      config,
      enabled: name === "enabled",
      pages: [...fixture.files.keys()]
        .filter((file) => file.endsWith(".vue"))
        .map((file) => path.join(root, file)),
    });
  }
  const launcher = path.join(directory, "vize-observed");
  fs.writeFileSync(launcher, `#!${process.execPath}\nrequire(${JSON.stringify(proxy)});\n`);
  fs.chmodSync(launcher, 0o755);
  const workspace = path.join(directory, "router.code-workspace");
  fs.writeFileSync(
    workspace,
    `${JSON.stringify({ folders: projects.map(({ root, name }) => ({ path: root, name })), settings: { "vize.serverPath": launcher, "vize.lint.enable": false, "editor.wordBasedSuggestions": "off", "chat.disableAIFeatures": true } }, null, 2)}\n`,
  );
  const manifest = {
    schema: "vize.router-installed-editor-campaign",
    version: 1,
    launcher,
    projects,
    workspace,
    authority: fixture.authority,
    providerDependencies: provider.dependencies,
  };
  const manifestPath = path.join(output, "campaign.json");
  fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
  return { workspace, manifestPath, provider, projects };
}

function run(label, executable, args, cwd, output, name) {
  const result = spawnSync(executable, args, {
    cwd,
    encoding: "utf8",
    timeout: 120_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  fs.writeFileSync(
    path.join(output, `${name}-${label}.json`),
    `${JSON.stringify({ executable, args, status: result.status, signal: result.signal, error: result.error?.message ?? null, stdout: result.stdout, stderr: result.stderr }, null, 2)}\n`,
  );
  assert.ifError(result.error);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, `${label}: ${result.stderr}\n${result.stdout}`);
}
