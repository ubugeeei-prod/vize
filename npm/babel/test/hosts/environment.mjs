import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export const hostSpecs = [
  { major: 7, version: "7.29.7" },
  { major: 8, version: "8.0.9" },
];
export const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const json = (file, value) => {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(
    file,
    JSON.stringify(value, (_, v) => (typeof v === "bigint" ? { bigint: String(v) } : v), 2) + "\n",
  );
};
const directory = path.dirname(fileURLToPath(import.meta.url));
function entryExists(file) {
  try {
    fs.lstatSync(file);
    return true;
  } catch (error) {
    if (error.code === "ENOENT") return false;
    throw error;
  }
}

export function settings() {
  const source = process.env.SOURCE_SHA;
  assert.match(
    source ?? "",
    /^[a-f0-9]{40}$/,
    "SOURCE_SHA must identify the actual source-native workspace",
  );
  assert(
    process.env.VIZE_BABEL_PACKED_PLUGIN,
    "VIZE_BABEL_PACKED_PLUGIN must point to the real built npm tarball",
  );
  const tarball = fs.realpathSync(process.env.VIZE_BABEL_PACKED_PLUGIN);
  assert(fs.statSync(tarball).isFile(), "VIZE_BABEL_PACKED_PLUGIN must be an actual npm tarball");
  const output = path.resolve(
    process.env.VIZE_BABEL_HOST_OUTPUT ?? path.join(directory, ".artifacts", source),
  );
  assert(!entryExists(output), "the output must be fresh so earlier raw observations are retained");
  fs.mkdirSync(output, { recursive: true });
  return {
    source,
    tarball,
    output,
    tarballSha256: hash(fs.readFileSync(tarball)),
    nativeRoot: process.env.VIZE_BABEL_NATIVE_ROOT
      ? fs.realpathSync(process.env.VIZE_BABEL_NATIVE_ROOT)
      : null,
  };
}

export function packageIdentity(requireHost, name) {
  const entry = requireHost.resolve(name);
  let directory = path.dirname(entry);
  for (;;) {
    const manifest = path.join(directory, "package.json");
    if (fs.existsSync(manifest)) {
      const bytes = fs.readFileSync(manifest);
      const pkg = JSON.parse(bytes);
      if (pkg.name === name)
        return {
          name,
          version: pkg.version,
          root: directory,
          entry,
          manifestSha256: hash(bytes),
          entrySha256: hash(fs.readFileSync(entry)),
        };
    }
    const parent = path.dirname(directory);
    assert.notEqual(parent, directory, "package manifest for " + name);
    directory = parent;
  }
}

export function prepareHost(spec, config) {
  const root = path.join(directory, "fixtures", "babel" + spec.major);
  const manifest = JSON.parse(fs.readFileSync(path.join(root, "package.json")));
  assert.equal(manifest.dependencies["@babel/core"], spec.version);
  const requireHost = createRequire(path.join(root, "package.json"));
  const core = packageIdentity(requireHost, "@babel/core");
  assert.equal(core.version, spec.version, "actual host core version");
  const modules = [
    "@babel/core",
    "@babel/plugin-transform-typescript",
    "@babel/plugin-transform-modules-commonjs",
    "@babel/plugin-proposal-decorators",
    "@babel/plugin-transform-class-properties",
    "@babel/plugin-transform-react-jsx",
  ].map((name) => {
    const actual = packageIdentity(requireHost, name);
    assert.equal(
      actual.version,
      manifest.dependencies[name],
      "exact installed version for " + name,
    );
    assert.equal(
      Number(actual.version.split(".")[0]),
      spec.major,
      "matching host major for " + name,
    );
    return actual;
  });
  const members = execFileSync("tar", ["-tzf", config.tarball], { encoding: "utf8" })
    .trim()
    .split("\n");
  assert(members.length > 0);
  for (const member of members) {
    assert(member.startsWith("package/"), "npm package root");
    assert(!member.split("/").includes(".."), "no tar traversal");
    assert(
      !member.startsWith("package/src/"),
      "the consumer must execute packed dist, never source",
    );
  }
  for (const name of ["index.mjs", "index.cjs", "index.d.mts", "index.d.cts"]) {
    assert(members.includes("package/dist/" + name), "packed public entrypoint " + name);
  }
  const scope = path.join(root, "node_modules/@vizejs");
  fs.mkdirSync(scope, { recursive: true });
  const target = path.join(scope, "babel-plugin");
  assert(!entryExists(target), "run npm ci in the isolated host before each packed test run");
  const temporary = fs.mkdtempSync(path.join(root, "node_modules/.vize-babel-unpack-"));
  try {
    execFileSync("tar", ["-xzf", config.tarball, "-C", temporary]);
    const original = path.join(temporary, "package");
    const check = (directory) => {
      for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
        assert(!entry.isSymbolicLink(), "packed runtime files must not be source symlinks");
        if (entry.isDirectory()) check(path.join(directory, entry.name));
      }
    };
    check(original);
    fs.cpSync(original, target, { recursive: true, errorOnExist: true });
  } finally {
    fs.rmSync(temporary, { recursive: true });
  }
  const pkg = JSON.parse(fs.readFileSync(path.join(target, "package.json")));
  assert.equal(pkg.name, "@vizejs/babel-plugin");
  assert.equal(pkg.exports["."].import.default, "./dist/index.mjs");
  assert.equal(pkg.exports["."].require.default, "./dist/index.cjs");
  assert(
    ["workspace:*", pkg.version].includes(pkg.dependencies["@vizejs/native"]),
    "the packed native dependency must match its explicit source workspace",
  );
  if (config.nativeRoot) {
    const native = JSON.parse(fs.readFileSync(path.join(config.nativeRoot, "package.json")));
    assert.equal(native.name, "@vizejs/native");
    assert.equal(native.version, pkg.version, "packed plugin and source native package versions");
    const link = path.join(scope, "native");
    assert(!entryExists(link), "no published native fallback may be installed");
    fs.symlinkSync(config.nativeRoot, link, process.platform === "win32" ? "junction" : "dir");
  }
  const identity = {
    source: config.source,
    root,
    modules,
    tarball: config.tarball,
    tarballSha256: config.tarballSha256,
    lockSha256: hash(fs.readFileSync(path.join(root, "package-lock.json"))),
    plugin: {
      name: pkg.name,
      version: pkg.version,
      root: target,
      esmSha256: hash(fs.readFileSync(path.join(target, "dist/index.mjs"))),
      cjsSha256: hash(fs.readFileSync(path.join(target, "dist/index.cjs"))),
    },
  };
  json(path.join(config.output, "babel" + spec.major + "-host.json"), identity);
  return { ...spec, root, identity };
}

export async function activeNative(config) {
  assert(config.nativeRoot, "VIZE_BABEL_NATIVE_ROOT must be an explicitly prepared source package");
  const helper = path.join(config.nativeRoot, "scripts/test-preparation.mjs");
  const { readActiveNativePreparation } = await import(pathToFileURL(helper).href);
  const preparation = readActiveNativePreparation(config.nativeRoot);
  assert(preparation, "an active source-native preparation owner and unchanged image are required");
  assert.equal(preparation.head, config.source, "actual prepared native source head");
  const image = fs.realpathSync(path.join(config.nativeRoot, preparation.binding));
  assert.equal(hash(fs.readFileSync(image)), preparation.sha256, "actual prepared addon bytes");
  return {
    preparation,
    image,
    receiptSha256: hash(
      fs.readFileSync(path.join(config.nativeRoot, ".artifacts/native/js-test-preparation.json")),
    ),
  };
}
