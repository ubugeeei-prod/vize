import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fixtureDirectory, hash, relativeMember, type FileIdentity } from "./authority.ts";
import { inspectExperimentalExports } from "./semantic-exports.ts";
import { verifyInstalledProvider, type verifyBuiltProvider } from "./provider.ts";
import { processRecorder } from "./processes.ts";
import { readOracles } from "./oracles.ts";

type ProjectAuthority = {
  schema: string;
  version: number;
  files: FileIdentity[];
  strictOracles: string;
};

export function readProjectFixture(directory = fixtureDirectory) {
  const bytes = fs.readFileSync(path.join(directory, "project.json"));
  const authority = JSON.parse(bytes.toString("utf8")) as ProjectAuthority;
  assert.equal(authority.schema, "vize.router-preview-project");
  assert.equal(authority.version, 1);
  const files = new Map<string, Buffer>();
  for (const entry of authority.files) {
    const relative = relativeMember(entry.path);
    assert.equal(files.has(relative), false);
    const bytes = fs.readFileSync(path.join(directory, "project", `${relative}.txt`));
    assert.equal(bytes.length, entry.bytes, relative);
    assert.equal(hash(bytes), entry.sha256, relative);
    files.set(relative, bytes);
  }
  return { authority, files, sha256: hash(bytes) };
}

export function prepareProject(output: string, provider: ReturnType<typeof verifyBuiltProvider>) {
  assert.equal(process.env.GITHUB_ACTIONS, "true", "real preview installs run in Actions");
  const pinned = readProjectFixture();
  const oracles = readOracles();
  assert.equal(pinned.authority.strictOracles, "frozen-independent-controls");
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-router-preview-593d7ad-"));
  for (const [relative, bytes] of pinned.files) {
    const destination = path.join(root, relative);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, bytes);
  }
  fs.mkdirSync(path.join(root, ".provider"));
  fs.copyFileSync(provider.archive, path.join(root, ".provider/vue-router-5.4.0.tgz"));
  const processes = processRecorder(output);
  const install = processes.run(
    "install-real-built-provider",
    process.platform === "win32" ? "npm.cmd" : "npm",
    ["ci", "--ignore-scripts", "--no-audit", "--no-fund", "--registry=https://registry.npmjs.org"],
    root,
    180_000,
  );
  assert.equal(install.error, null);
  assert.equal(install.signal, null);
  assert.equal(install.status, 0, install.stderr);
  const installed = verifyInstalledProvider(root, provider);
  const exports = inspectExperimentalExports(root, installed.root);
  fs.writeFileSync(
    path.join(output, "installed-provider.json"),
    `${JSON.stringify({ receiptSha256: provider.receiptSha256, projectAuthoritySha256: pinned.sha256, identities: installed.identities, semantic: exports }, null, 2)}\n`,
  );
  const routeProps = exports.exports.find((entry) => entry.name === "_RoutePropsForFilePath");
  assert.ok(
    routeProps && routeProps.declarations.length > 0,
    "the actual built provider exports the helper",
  );
  assert.equal(routeProps.targetName, "RoutePropsForFilePath");
  assert.equal(
    routeProps.targetIsTypeAlias,
    true,
    "the exported helper is an actual declaration alias",
  );
  assert.equal(
    exports.exports.some((entry) => entry.name === "useRoute"),
    false,
    "no invented experimental useRoute",
  );
  assert.deepEqual(exports, oracles.json("installed-exports.json"), "all real semantic exports");
  const generation = processes.run(
    "real-preview-route-scanner",
    process.execPath,
    ["generate.mjs"],
    root,
  );
  assert.equal(generation.error, null);
  assert.equal(generation.signal, null);
  assert.equal(generation.status, 0, generation.stderr);
  const generated = fs.readFileSync(path.join(root, "typed-router.d.ts"));
  fs.writeFileSync(path.join(output, "typed-router.d.ts.txt"), generated);
  assert.deepEqual(
    generated,
    oracles.files.get("frozen/typed-router.d.ts.txt"),
    "complete declaration from the actual pinned route scanner",
  );
  for (const name of ["routes.generated.ts", "resolver.generated.ts"]) {
    fs.copyFileSync(path.join(root, name), path.join(output, `${name}.txt`));
    fs.rmSync(path.join(root, name));
  }
  fs.copyFileSync(
    path.join(root, "package-lock.json"),
    path.join(output, "consumer-package-lock.json"),
  );
  assert.deepEqual(
    fs.readFileSync(path.join(root, "package-lock.json")),
    pinned.files.get("package-lock.json"),
    "npm ci cannot rewrite the frozen real-provider consumer lock",
  );
  const dependencies = ["vue", "vue-router", "typescript", "vue-tsc", "vize"].map((name) => {
    const bytes = fs.readFileSync(path.join(root, "node_modules", name, "package.json"));
    const manifest = JSON.parse(bytes.toString("utf8"));
    return { name: manifest.name, version: manifest.version, sha256: hash(bytes) };
  });
  fs.writeFileSync(
    path.join(output, "project-identity.json"),
    `${JSON.stringify({ root, sourceInputs: pinned.authority.files, dependencies, generatedDeclarationSha256: hash(generated) }, null, 2)}\n`,
  );
  assert.deepEqual(dependencies, oracles.json("dependencies.json"), "independent host identities");
  return {
    root,
    pinned,
    processes,
    generated,
    cleanup() {
      fs.rmSync(path.join(root, "node_modules"), { recursive: true, force: true });
    },
  };
}
