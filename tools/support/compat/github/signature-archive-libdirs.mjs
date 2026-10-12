import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { selectedBinary, sha256 } from "./signature-archive-capture.mjs";

// nextest 0.9.146 archive loading supplies LibdirMapper::Path/NotInArchive;
// manual metadata reuse uses NotRequested. Preserve that exact boundary in a
// separate RUNNER_TEMP copy. The original metadata and binary stay untouched.
export function archivedLibdirs(original, archiveList, workspace) {
  const derived = structuredClone(original);
  const before = original["rust-build-meta"].platforms;
  const mapped = archiveList["rust-build-meta"].platforms;
  assert.ok(before && mapped, "current platform schema required");
  assert.ok(Array.isArray(before.targets) && before.targets.length <= 1);
  assert.equal(mapped.targets.length, before.targets.length);
  const edits = [];
  for (const [selector, expected] of [
    ["host", join(workspace, "target/nextest/libdirs/host")],
    ...(before.targets.length
      ? [["target", join(workspace, "target/nextest/libdirs/target/0")]]
      : []),
  ]) {
    const oldPlatform = selector === "host" ? before.host : before.targets[0];
    const archivePlatform = selector === "host" ? mapped.host : mapped.targets[0];
    const newPlatform =
      selector === "host"
        ? derived["rust-build-meta"].platforms.host
        : derived["rust-build-meta"].platforms.targets[0];
    const withoutLibdir = (platform) => {
      const value = structuredClone(platform);
      delete value.libdir;
      return value;
    };
    assert.deepEqual(withoutLibdir(archivePlatform), withoutLibdir(oldPlatform));
    assert.ok(oldPlatform.libdir && archivePlatform.libdir);
    if (archivePlatform.libdir.status === "available") {
      assert.deepEqual(archivePlatform.libdir, { status: "available", path: expected });
    } else {
      assert.deepEqual(archivePlatform.libdir, { status: "unavailable", reason: "not-in-archive" });
    }
    edits.push({ selector, before: oldPlatform.libdir, after: archivePlatform.libdir });
    newPlatform.libdir = structuredClone(archivePlatform.libdir);
  }
  const inverse = structuredClone(derived);
  for (const edit of edits) {
    const platform =
      edit.selector === "host"
        ? inverse["rust-build-meta"].platforms.host
        : inverse["rust-build-meta"].platforms.targets[0];
    platform.libdir = edit.before;
  }
  assert.deepEqual(inverse, original, "whole original metadata inverse must be exact");
  return { derived, edits };
}

export function libraryPaths(meta, exists = existsSync, inherited = []) {
  const target = meta["target-directory"];
  const build = meta["build-directory"] ?? target;
  const sort = (values) =>
    [...values].sort((a, b) => Buffer.compare(Buffer.from(a), Buffer.from(b)));
  assert.ok(Array.isArray(meta["linked-paths"]) && Array.isArray(meta["base-output-directories"]));
  const linked = sort(meta["linked-paths"]).map((path) => join(build, path));
  const paths = [
    ...linked.filter(exists),
    ...sort(meta["base-output-directories"]).flatMap((path) => [
      join(target, path),
      join(build, path, "deps"),
    ]),
    ...[meta.platforms.host, ...meta.platforms.targets]
      .filter((platform) => platform.libdir.status === "available")
      .map((platform) => platform.libdir.path),
  ];
  return {
    ordered: [...new Set(paths), ...inherited],
    linked: linked.map((path) => ({ path, exists: exists(path) })),
  };
}

export function loaderDependencyPaths(output, interpreter) {
  const paths = [interpreter];
  for (const line of output.trimEnd().split("\n")) {
    const value = line.trim();
    if (/^linux-vdso\.so\.1 \(0x[0-9a-f]+\)$/.test(value)) continue;
    const match = /^(?:[\w.+-]+\s+=>\s+)?(\/\S+)\s+\(0x[0-9a-f]+\)$/.exec(value);
    assert.ok(match, `unresolved/unsupported ELF loader line: ${value}`);
    paths.push(match[1]);
  }
  return [...new Set(paths)];
}

export function requireDependencyCustody(dependencies) {
  for (const dependency of dependencies) {
    assert.equal(
      realpathSync(dependency.loaderReportedPath),
      dependency.path,
      "ELF dependency alias retargeted",
    );
    assert.equal(sha256(readFileSync(dependency.path)), dependency.sha256);
  }
}

function runtimeBoundary() {
  const variables = Object.fromEntries(
    Object.entries(process.env).filter(([name]) => name.startsWith("LD_")),
  );
  assert.deepEqual(variables, {}, "original Ubuntu runtime has no inherited LD overrides");
  const locate = (name) => execFileSync("which", [name], { encoding: "utf8" }).trim();
  const cargoCommand = locate("cargo");
  const cargo = realpathSync(cargoCommand);
  const rustup = realpathSync(locate("rustup"));
  const rustupVersion = execFileSync("rustup", ["--version"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "ignore"],
  }).trim();
  const policies = {
    "1.28.2": "c9b6dade786b07ac971188aeab90c77725e1e6e9e12711c85d74c35a1628e062",
    "1.29.0": "a3e6269af72a260fabd16c9b6d49f02b2db8c803d9e908a355c80d2a845e2925",
    "1.29.1": "66f684bd5e9e920004cfddb2e76dba98fe77354cb2425f90a7695f1384254fae",
  };
  const rustupPolicy = policies[/^rustup (\d+\.\d+\.\d+)\s/.exec(rustupVersion)?.[1]];
  assert.ok(rustupPolicy, "unreviewed Rustup Linux environment policy");
  const toolchain = execFileSync("rustup", ["show", "active-toolchain"], {
    encoding: "utf8",
  }).trim();
  assert.match(toolchain, /^1\.99\.0-x86_64-unknown-linux-gnu(?:\s|$)/);
  const toolchainCargoCommand = execFileSync("rustup", ["which", "cargo"], {
    encoding: "utf8",
  }).trim();
  const toolchainCargo = realpathSync(toolchainCargoCommand);
  const hash = (path) => sha256(readFileSync(path));
  const cargoSha256 = hash(cargo);
  const rustupSha256 = hash(rustup);
  const shim = cargoSha256 === rustupSha256;
  assert.ok(shim || cargo === toolchainCargo, "unsupported Cargo command boundary");
  const cargoVersion = execFileSync("cargo", ["--version"], { encoding: "utf8" }).trim();
  assert.match(cargoVersion, /^cargo 1\.99\.0(?:\s|$)/);
  const inherited = shim ? [join(dirname(dirname(toolchainCargoCommand)), "lib")] : [];
  const nextestCommand = execFileSync("which", ["cargo-nextest"], { encoding: "utf8" }).trim();
  const nextest = realpathSync(nextestCommand);
  const version = execFileSync("cargo", ["nextest", "--version"], { encoding: "utf8" }).trim();
  assert.match(version, /cargo-nextest 0\.9\.146(?:\s|$)/);
  return {
    cwd: process.cwd(),
    variables,
    cargoCommand,
    cargo,
    cargoSha256,
    cargoVersion,
    rustup,
    rustupSha256,
    rustupVersion,
    rustupPolicy,
    toolchain,
    toolchainCargoCommand,
    toolchainCargo,
    toolchainCargoSha256: hash(toolchainCargo),
    cargoBoundary: shim ? "reviewed-rustup-shim" : "direct-selected-cargo",
    inherited,
    nextest,
    nextestCommand,
    nextestSha256: sha256(readFileSync(nextest)),
    version,
  };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [operation, directory] = process.argv.slice(2);
  const json = (path) => JSON.parse(readFileSync(path, "utf8"));
  const save = (name, value) =>
    writeFileSync(join(directory, name), `${JSON.stringify(value, null, 2)}\n`);
  assert.ok(directory && ["environment", "derive", "verify", "finish"].includes(operation));
  if (operation === "environment") {
    save("library-runtime.json", runtimeBoundary());
  } else if (operation === "derive") {
    const original = readFileSync("target/nextest/binaries-metadata.json");
    const archiveList = readFileSync(join(directory, "selected-tests.json"));
    const { derived, edits } = archivedLibdirs(
      JSON.parse(original),
      JSON.parse(archiveList),
      process.cwd(),
    );
    save("derived-binaries-metadata.json", derived);
    save("library-mapping.json", {
      originalSha256: sha256(original),
      archiveListSha256: sha256(archiveList),
      derivedSha256: sha256(readFileSync(join(directory, "derived-binaries-metadata.json"))),
      edits,
      originalUnchanged: true,
      inverseWholeObjectExact: true,
      libraryEnvironmentEqual: false,
    });
  } else if (operation === "verify") {
    // Both full lists are produced by pinned nextest: one via the original
    // archive loader, one via the separate derived manual metadata. Identical
    // RustBuildMeta + identical inherited env gives the same ordered dylib
    // path through RustBuildMeta::dylib_paths/TestList::create_dylib_path.
    assert.deepEqual(
      json(join(directory, "derived-selected-tests.json")),
      json(join(directory, "selected-tests.json")),
    );
    const proof = json(join(directory, "library-mapping.json"));
    assert.equal(
      proof.originalSha256,
      sha256(readFileSync("target/nextest/binaries-metadata.json")),
    );
    assert.equal(
      proof.derivedSha256,
      sha256(readFileSync(join(directory, "derived-binaries-metadata.json"))),
    );
    proof.libraryEnvironmentEqual = true;
    proof.derivedListSha256 = sha256(readFileSync(join(directory, "derived-selected-tests.json")));
    assert.deepEqual(runtimeBoundary(), json(join(directory, "library-runtime.json")));
    proof.paths = libraryPaths(
      json(join(directory, "selected-tests.json"))["rust-build-meta"],
      existsSync,
      json(join(directory, "library-runtime.json")).inherited,
    );
    const binary = selectedBinary(json(join(directory, "selected-tests.json")));
    const readelf = execFileSync("readelf", ["--program-headers", binary], { encoding: "utf8" });
    writeFileSync(join(directory, "readelf.stdout"), readelf);
    const interpreter = /Requesting program interpreter: ([^\]]+)\]/.exec(readelf)?.[1];
    assert.ok(interpreter, "GNU Linux archived test ELF interpreter required");
    const output = execFileSync(interpreter, ["--list", binary], {
      encoding: "utf8",
      env: { ...process.env, LD_LIBRARY_PATH: proof.paths.ordered.join(":") },
    });
    writeFileSync(join(directory, "elf-loader.stdout"), output);
    proof.dependencies = loaderDependencyPaths(output, interpreter).map((loaderReportedPath) => {
      const path = realpathSync(loaderReportedPath);
      return { loaderReportedPath, path, sha256: sha256(readFileSync(path)) };
    });
    assert.ok(proof.dependencies.length > 1, "complete resolved ELF dependencies required");
    save("library-mapping.json", proof);
  } else {
    assert.deepEqual(runtimeBoundary(), json(join(directory, "library-runtime.json")));
    const proof = json(join(directory, "library-mapping.json"));
    assert.equal(proof.libraryEnvironmentEqual, true);
    assert.deepEqual(
      libraryPaths(
        json(join(directory, "selected-tests.json"))["rust-build-meta"],
        existsSync,
        json(join(directory, "library-runtime.json")).inherited,
      ),
      proof.paths,
    );
    requireDependencyCustody(proof.dependencies);
  }
}
