// #8099: explicit JS configuration uses physical, locked framework and Node types.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { files, save, sha } from "./javascript-workspace-project.mjs";

export function nuxt3TypeProviders(root, project, artifacts, cohort, corpus) {
  if (cohort.id !== "nuxt3") return;
  const selected = createRequire(path.join(project, "apps/nuxt/package.json"));
  const nuxtManifest = fs.realpathSync(selected.resolve("nuxt/package.json"));
  const nuxt = JSON.parse(fs.readFileSync(nuxtManifest, "utf8"));
  assert.equal(nuxt.version, cohort.nuxt);
  const packageRoot = path.dirname(nuxtManifest);
  const configTypes = path.join(packageRoot, nuxt.exports["./config"].types);
  const schemaTypes = path.join(packageRoot, nuxt.exports["./schema"].types);
  const schemaManifest = fs.realpathSync(
    createRequire(schemaTypes).resolve("@nuxt/schema/package.json"),
  );
  const schema = JSON.parse(fs.readFileSync(schemaManifest, "utf8"));
  assert.equal(schema.version, nuxt.dependencies["@nuxt/schema"]);
  assert.equal(schema.version, cohort.nuxt);
  const lock = fs.readFileSync(path.join(root, cohort.path, "package-lock.json"), "utf8");
  const schemaKey = path
    .relative(path.join(project, "node_modules"), schemaManifest)
    .split(path.sep)
    .join("/")
    .replace(/\/package\.json$/, "");
  const lockedSchema = JSON.parse(lock).packages[`node_modules/${schemaKey}`];
  assert.equal(lockedSchema.version, schema.version);
  const graph = [];
  const nodeManifest = createRequire(path.join(root, "tests/package.json")).resolve(
    "@types/node/package.json",
  );
  const node = JSON.parse(fs.readFileSync(nodeManifest, "utf8"));
  assert.equal(node.version, corpus.nodeTypes.node);
  assert.deepEqual(Object.keys(node.dependencies), ["undici-types"]);
  const dependency = createRequire(nodeManifest).resolve("undici-types/package.json");
  for (const [name, manifestPath, version] of [
    ["@types/node", nodeManifest, corpus.nodeTypes.node],
    ["undici-types", dependency, corpus.nodeTypes.undici],
  ]) {
    const physicalManifest = fs.realpathSync(manifestPath);
    const source = path.dirname(physicalManifest);
    const manifestBytes = fs.readFileSync(physicalManifest, "utf8");
    const manifest = JSON.parse(manifestBytes);
    assert.equal(manifest.name, name);
    assert.equal(manifest.version, version);
    const target = path.join(project, "node_modules", name);
    assert.equal(fs.existsSync(target), false, "the cohort has no prior Node type provider");
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.cpSync(source, target, { recursive: true, dereference: true });
    const members = files(source);
    assert.deepEqual(files(target), members);
    graph.push({ name, source, target, manifestBytes, members });
  }
  const driverLock = fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf8");
  const driver = createRequire(path.join(root, "tests/package.json"))("yaml").parse(driverLock);
  assert.equal(driver.importers.tests.devDependencies["@types/node"].version, node.version);
  for (const { name, manifestBytes } of graph)
    assert.ok(driver.packages[`${name}@${JSON.parse(manifestBytes).version}`].resolution.integrity);
  save(artifacts, "nuxt3-type-providers.json", {
    authority: "framework-owned Nuxt config/schema; separate actual driver-locked Node types",
    nuxtManifest,
    nuxt,
    configTypes: { path: configTypes, source: fs.readFileSync(configTypes, "utf8") },
    schemaTypes: { path: schemaTypes, source: fs.readFileSync(schemaTypes, "utf8") },
    schemaManifest,
    schema,
    schemaMembers: files(path.dirname(schemaManifest)),
    lockedSchema,
    cohortLock: { sha256: sha(lock), source: lock },
    driverLock: { sha256: sha(driverLock), source: driverLock },
    graph,
  });
  return () => {
    for (const { source, target, members } of graph) {
      assert.deepEqual(files(source), members);
      assert.deepEqual(files(target), members);
    }
  };
}
