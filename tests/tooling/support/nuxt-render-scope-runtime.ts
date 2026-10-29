import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export function sha256(value: string | Uint8Array): string {
  return createHash("sha256").update(value).digest("hex");
}
export type RuntimePackage = {
  name: string;
  version: string;
  importer: string;
  manifest: string;
  root: string;
  sha256: string;
  bytes: number;
  entry: { path: string; sha256: string };
  body: Record<string, unknown>;
};
export function readManifest(manifest: string, importer: string, entry?: string): RuntimePackage {
  const physical = fs.realpathSync(manifest);
  const bytes = fs.readFileSync(physical);
  const body = JSON.parse(bytes.toString("utf8")) as Record<string, unknown>;
  assert.equal(typeof body.name, "string");
  assert.equal(typeof body.version, "string");
  return {
    name: body.name as string,
    version: body.version as string,
    importer,
    manifest: physical,
    root: path.dirname(physical),
    sha256: sha256(bytes),
    bytes: bytes.length,
    entry: {
      path: fs.realpathSync(entry ?? path.join(path.dirname(physical), "dist/index.mjs")),
      sha256: sha256(fs.readFileSync(entry ?? path.join(path.dirname(physical), "dist/index.mjs"))),
    },
    body,
  };
}
function resolveManifest(importer: string, name: string): RuntimePackage {
  assert(
    process.execArgv.includes("--experimental-import-meta-resolve"),
    "The oracle must use Node's real parent-aware ESM import-condition resolution",
  );
  const parent = pathToFileURL(importer).href;
  const entry = fs.realpathSync(fileURLToPath(import.meta.resolve(name, parent)));
  if (name !== "rc9") {
    const manifest = fileURLToPath(import.meta.resolve(name + "/package.json", parent));
    return readManifest(manifest, importer, entry);
  }
  // rc9 does not export package.json. Resolve its actual ESM public entry,
  // then identify its package without choosing an independent hoisted copy.
  let directory = path.dirname(entry);
  for (let depth = 0; depth < 64; depth++) {
    const manifest = path.join(directory, "package.json");
    if (fs.existsSync(manifest)) {
      const result = readManifest(manifest, importer, entry);
      assert.equal(result.name, "rc9");
      return result;
    }
    const parent = path.dirname(directory);
    assert.notEqual(parent, directory);
    directory = parent;
  }
  throw new Error("rc9 manifest resolution exceeded the bounded oracle");
}

export function observeRuntime(project: string): RuntimePackage[] {
  const projectManifest = path.join(project, "package.json");
  const nuxt = resolveManifest(projectManifest, "nuxt");
  const kit = resolveManifest(nuxt.entry.path, "@nuxt/kit");
  const schema = resolveManifest(kit.entry.path, "@nuxt/schema");
  const rc9 = resolveManifest(kit.entry.path, "rc9");
  const coreSchema = resolveManifest(nuxt.entry.path, "@nuxt/schema");
  return [nuxt, kit, schema, rc9, coreSchema];
}
export function requireSupported(packages: RuntimePackage[]): void {
  assert.deepEqual(
    packages.map(({ name, version }) => [name, version]),
    [
      ["nuxt", "4.6.3"],
      ["@nuxt/kit", "4.6.3"],
      ["@nuxt/schema", "4.6.3"],
      ["rc9", "3.1.0"],
      ["@nuxt/schema", "4.6.3"],
    ],
  );
  assert.deepEqual(packages[1].body.inlinedDependencies, {
    c12: "4.0.0-rc.3",
    "pkg-types": "2.4.0",
    untyped: "2.0.0",
  });
}
export function writeInputs(root: string, files: Record<string, string>): void {
  for (const [name, source] of Object.entries(files)) {
    assert(!path.isAbsolute(name) && !name.split("/").includes(".."));
    const target = path.join(root, name);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, source);
  }
}
export function copyGenuineNuxtWithNestedKit(
  project: string,
  supported: RuntimePackage[],
  unsupportedKit: string,
  location: "root" | "dist-unsupported" | "dist-supported" = "root",
): void {
  const destination = path.join(project, "node_modules/nuxt");
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.cpSync(supported[0].root, destination, {
    recursive: true,
    filter: (entry) =>
      !path.relative(supported[0].root, entry).split(path.sep).includes("node_modules"),
  });
  const nested = path.join(
    destination,
    location === "root" ? "node_modules/@nuxt/kit" : "dist/node_modules/@nuxt/kit",
  );
  fs.mkdirSync(path.dirname(nested), { recursive: true });
  fs.symlinkSync(location === "dist-supported" ? supported[1].root : unsupportedKit, nested, "dir");
  if (location === "dist-supported") {
    const decoy = path.join(destination, "node_modules/@nuxt/kit");
    fs.mkdirSync(path.dirname(decoy), { recursive: true });
    fs.symlinkSync(unsupportedKit, decoy, "dir");
  }
  // These are unmodified genuine package bytes in a deliberately unsupported
  // dependency context, not an invented version or a supported runtime proof.
  assert.equal(
    sha256(fs.readFileSync(path.join(destination, "package.json"))),
    supported[0].sha256,
  );
  assert.equal(
    sha256(fs.readFileSync(path.join(destination, "dist/index.mjs"))),
    supported[0].entry.sha256,
  );
}

export function verifyRegistryAuthority(
  runtime: string,
  registry: string,
  packages: RuntimePackage[],
): unknown[] {
  const lock = JSON.parse(fs.readFileSync(path.join(runtime, "package-lock.json"), "utf8")) as {
    packages: Record<string, { version?: string; integrity?: string; resolved?: string }>;
  };
  const names: Record<string, string> = {
    "nuxt@4.6.3": "nuxt-registry.json",
    "@nuxt/kit@4.6.3": "kit-registry.json",
    "@nuxt/schema@4.6.3": "schema-registry.json",
    "rc9@3.1.0": "rc9-registry.json",
    "@nuxt/kit@4.6.2": "unsupported-kit-registry.json",
  };
  const unique = new Map(packages.map((pkg) => [pkg.root, pkg]));
  return [...unique.values()].map((pkg) => {
    const metadataPath = path.join(registry, names[pkg.name + "@" + pkg.version]);
    const bytes = fs.readFileSync(metadataPath);
    const metadata = JSON.parse(bytes.toString("utf8")) as {
      name: string;
      version: string;
      dist: { integrity: string; tarball: string };
      exports: unknown;
      imports?: unknown;
    };
    assert.equal(metadata.name, pkg.name);
    assert.equal(metadata.version, pkg.version);
    assert.deepEqual(pkg.body.exports, metadata.exports);
    assert.deepEqual(pkg.body.imports, metadata.imports);
    const relative = path.relative(runtime, pkg.root).split(path.sep).join("/");
    assert(!relative.startsWith("../") && !path.isAbsolute(relative));
    const installed = lock.packages[relative];
    assert(installed, relative + "/package-lock entry");
    assert.equal(installed.version, pkg.version);
    assert.equal(installed.integrity, metadata.dist.integrity);
    assert.equal(installed.resolved, metadata.dist.tarball);
    return {
      name: pkg.name,
      version: pkg.version,
      package: pkg.root,
      manifestSha256: pkg.sha256,
      entry: pkg.entry,
      metadataSha256: sha256(bytes),
      integrity: installed.integrity,
      resolved: installed.resolved,
    };
  });
}

if (
  process.argv[2] === "--probe-effective-config" ||
  process.argv[2] === "--probe-effective-config-unknown"
) {
  const requested = process.argv[3];
  assert(requested, "Missing runtime project");
  const cwd = fs.realpathSync(requested);
  const packages = observeRuntime(cwd);
  requireSupported(packages);
  const kit = (await import(pathToFileURL(packages[1].entry.path).href)) as {
    loadNuxtConfig: (options: { cwd: string }) => Promise<Record<string, unknown>>;
  };
  const config = await kit.loadNuxtConfig({ cwd });
  const app = config.app as Record<string, unknown>;
  const dir = config.dir as Record<string, unknown>;
  assert.equal(config.rootDir, cwd);
  assert.equal(config.srcDir, path.join(cwd, "src"));
  assert.equal(dir.pages, "pages");
  assert.equal(dir.layouts, "layouts");
  assert.notEqual(config.pages, false);
  if (process.argv[2] === "--probe-effective-config") {
    assert.equal(app.pageTransition, false);
    assert.equal(app.layoutTransition, false);
    assert.equal(app.keepalive, false);
  }
  assert.deepEqual(config.modules, []);
  console.log(
    JSON.stringify({
      schema: "vize.nuxt-render.genuine-effective-config.v1",
      packages,
      configFile: config._nuxtConfigFile,
      rootDir: config.rootDir,
      srcDir: config.srcDir,
      dir,
      pages: config.pages,
      modules: config.modules,
      app: {
        pageTransition: app.pageTransition,
        layoutTransition: app.layoutTransition,
        keepalive: app.keepalive,
      },
    }),
  );
}
