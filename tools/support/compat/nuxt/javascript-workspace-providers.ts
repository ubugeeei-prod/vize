// #8099: explicit JS configuration uses physical, locked framework and Node types.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { asArray, asRecord, asString, at, parseJson, readJson } from "../../json-value.ts";
import { files, save, sha } from "./javascript-workspace-project.ts";
import type { Cohort, Corpus, FilePins } from "./javascript-workspace-project.ts";

interface ProviderPackage {
  name: string;
  source: string;
  target: string;
  manifestBytes: string;
  members: FilePins;
}

export function nuxt3TypeProviders(
  root: string,
  project: string,
  artifacts: string,
  cohort: Cohort,
  corpus: Corpus,
): (() => void) | undefined {
  if (cohort.id !== "nuxt3") return;
  const selected = createRequire(path.join(project, "apps/nuxt/package.json"));
  const nuxtManifest = fs.realpathSync(selected.resolve("nuxt/package.json"));
  const nuxt = asRecord(readJson(nuxtManifest), "nuxt manifest");
  assert.equal(nuxt.version, cohort.nuxt);
  const packageRoot = path.dirname(nuxtManifest);
  const configTypes = path.join(packageRoot, asString(at(nuxt, "exports", "./config", "types")));
  const schemaTypes = path.join(packageRoot, asString(at(nuxt, "exports", "./schema", "types")));
  const schemaManifest = fs.realpathSync(
    createRequire(schemaTypes).resolve("@nuxt/schema/package.json"),
  );
  const schema = asRecord(readJson(schemaManifest), "schema manifest");
  assert.equal(schema.version, at(nuxt, "dependencies", "@nuxt/schema"));
  assert.equal(schema.version, cohort.nuxt);
  const lock = fs.readFileSync(path.join(root, cohort.path, "package-lock.json"), "utf8");
  const schemaKey = path
    .relative(path.join(project, "node_modules"), schemaManifest)
    .split(path.sep)
    .join("/")
    .replace(/\/package\.json$/, "");
  const lockedSchema = at(parseJson(lock), "packages", `node_modules/${schemaKey}`);
  assert.equal(at(lockedSchema, "version"), schema.version);
  const graph: ProviderPackage[] = [];
  const nodeManifest = createRequire(path.join(root, "tests/package.json")).resolve(
    "@types/node/package.json",
  );
  const node = asRecord(readJson(nodeManifest), "@types/node manifest");
  assert.equal(node.version, corpus.nodeTypes.node);
  assert.deepEqual(Object.keys(asRecord(node.dependencies)), ["undici-types"]);
  const dependency = createRequire(nodeManifest).resolve("undici-types/package.json");
  for (const [name, manifestPath, version] of [
    ["@types/node", nodeManifest, corpus.nodeTypes.node],
    ["undici-types", dependency, corpus.nodeTypes.undici],
  ] as const) {
    const physicalManifest = fs.realpathSync(manifestPath);
    const source = path.dirname(physicalManifest);
    const manifestBytes = fs.readFileSync(physicalManifest, "utf8");
    const manifest = asRecord(parseJson(manifestBytes), name);
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
  const yaml: unknown = createRequire(path.join(root, "tests/package.json"))("yaml");
  const parseAllDocuments = at(yaml, "parseAllDocuments");
  assert.equal(typeof parseAllDocuments, "function");
  const documents = asArray(
    (parseAllDocuments as (source: string) => unknown)(driverLock),
    "lock documents",
  ).map((document) => {
    const toJS = at(document, "toJS");
    assert.equal(typeof toJS, "function");
    assert.deepEqual(at(document, "errors"), []);
    return (toJS as () => unknown).call(document);
  });
  assert.equal(documents.length, 2, "manager and workspace lock documents remain separate");
  const [manager, driver] = documents;
  assert.equal(at(manager, "lockfileVersion"), "9.0");
  assert.deepEqual(Object.keys(asRecord(at(manager, "importers"))), ["."]);
  assert.ok(at(manager, "importers", ".", "packageManagerDependencies", "pnpm"));
  assert.equal(at(driver, "lockfileVersion"), at(manager, "lockfileVersion"));
  assert.equal(
    at(driver, "importers", "tests", "devDependencies", "@types/node", "version"),
    node.version,
  );
  for (const { name, manifestBytes } of graph) {
    const version = String(at(parseJson(manifestBytes), "version"));
    assert.ok(at(driver, "packages", `${name}@${version}`, "resolution", "integrity"));
  }
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
    driverLock: { sha256: sha(driverLock), source: driverLock, documents: 2, workspaceDocument: 1 },
    graph,
  });
  return () => {
    for (const { source, target, members } of graph) {
      assert.deepEqual(files(source), members);
      assert.deepEqual(files(target), members);
    }
  };
}
