import assert from "node:assert/strict";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative } from "node:path";
import { asRecord, asString } from "../../../support/json-value.ts";
import {
  assertCustody,
  captureDeclarationGraph,
  fileImage,
  inside,
  type FileImage,
} from "../../../support/release/computed_event_custody.ts";
import {
  packageMetadata,
  read,
  readBytes,
} from "../../../support/release/public_install/registry.ts";
import { assertPackedManifest } from "../../../support/release/slot_instance_public/packed-manifest.ts";
import {
  requireNativeLoadPrerequisite,
  recheckNativeLoadPrerequisite,
} from "../../../support/release/human_reports_11252_11253_public/native-loader-prerequisite.ts";
import type { PackageBase } from "../../../support/release/human_reports_11252_11253_public/model.ts";
import {
  managerImages,
  assertManager,
  shippedImages,
} from "../public-typechecker-11317/install.ts";
import { authenticateProvenance, type authenticate } from "./authority.ts";
import { profileFor, requireEnvironment, limitations, type SourceFile } from "./contract.ts";
import { capture, healthy, inputImages, type Recorder } from "./execute.ts";
import { sourceBin, tsrsRoute, type CommandBin } from "./routes.ts";
import { e649MetadataUrl, readE649Metadata, usesE649Packument } from "./e649-metadata.ts";
import { publishedTscRoute } from "./generic-arrow-route.ts";
import {
  usesP73FullPackument,
  readP73FullMetadata,
  p73FullMetadataUrl,
} from "./global-listener-data.ts";

function physical(path: string) {
  const stat = lstatSync(path);
  assert(
    stat.isFile() && !stat.isSymbolicLink() && realpathSync(path) === path,
    "regular canonical owned file required",
  );
  return readFileSync(path);
}
function dependencyDirectory(root: string, from: string, name: string) {
  assert(/^(@[a-z0-9_.-]+\/)?[a-z0-9_.-]+$/u.test(name));
  let current = from;
  while (inside(root, current) || current === root) {
    const path = join(current, "node_modules", name);
    if (existsSync(path)) {
      const canonical = realpathSync(path);
      assert(inside(join(root, "node_modules"), canonical));
      return canonical;
    }
    if (current === root) break;
    current = dirname(current);
  }
  return null;
}
export async function install(
  root: string,
  files: SourceFile[],
  npm: string,
  npx: string,
  authority: Awaited<ReturnType<typeof authenticate>>,
  record: Recorder,
  cache: Map<string, Buffer>,
  commandBins: readonly CommandBin[] = ["tsrs", "vue-tsc"],
) {
  const checkEnvironment = () => requireEnvironment(process.env);
  checkEnvironment();
  assert.equal(
    process.version,
    profileFor().subset === "GlobalListener11373" ? "v26.12.1" : "v26.13.0",
  );
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  const manager = managerImages(npm, npx),
    checkManager = () => {
      checkEnvironment();
      assertManager(manager, npm, npx);
    };
  const managerRoot = dirname(dirname(npm)),
    managerLock = asRecord(
      JSON.parse(
        physical(join(dirname(dirname(managerRoot)), "package-lock.json")).toString("utf8"),
      ),
    );
  const managerRow = asRecord(asRecord(managerLock.packages)["node_modules/npm"]);
  await packageMetadata("npm", "11.19.1", managerRow);
  const managerPayload = await readBytes(asString(managerRow.resolved));
  const managerPublic = shippedImages(managerRoot, managerPayload, asString(managerRow.integrity));
  assert(managerPublic.archive.manifest.equals(physical(join(managerRoot, "package.json"))));
  record.json("manager-public-authority.json", {
    integrity: managerRow.integrity,
    resolved: managerRow.resolved,
    tarballSha512: managerPublic.archive.tarballSHA512,
    entries: manager,
    completeExtraFilesetQualified: false,
  });
  mkdirSync(root);
  assert(realpathSync(root) === root);
  for (const file of files) {
    const path = join(root, file.path);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, file.source, { flag: "wx" });
  }
  const initial = inputImages(root, files);
  assert(!existsSync(join(root, "node_modules")) && !existsSync(join(root, "package-lock.json")));
  record.json("initial-input-state.json", {
    files: initial,
    absent: ["node_modules", "package-lock.json"],
    limitations,
  });
  const version = capture(
    record,
    "npm-version",
    [process.execPath, npm, "--version"],
    root,
    checkManager,
  );
  healthy(version, 0);
  assert.equal(version.stdout, "11.19.1\n");
  assert.equal(version.stderr, "");
  // Literal reported installation, with normal scripts; no compiler alias or provider override.
  const attempt = capture(record, "npm-install", [process.execPath, npm, "i"], root, () => {
    checkManager();
    inputImages(root, files);
  });
  healthy(attempt, 0);
  inputImages(root, files);
  const lockRaw = physical(join(root, "package-lock.json"));
  record.raw("installed-package-lock.json", lockRaw);
  const lock = asRecord(JSON.parse(lockRaw.toString("utf8")));
  assert.equal(lock.lockfileVersion, 3);
  const rows = asRecord(lock.packages),
    requested = asRecord(JSON.parse(files.find((file) => file.path === "package.json")!.source));
  assert.deepEqual(asRecord(rows[""]).devDependencies, requested.devDependencies);
  const profile = profileFor(),
    packages: PackageBase[] = [],
    images: FileImage[] = [],
    registry: unknown[] = [];
  const reader = async (url: string) => {
    const value = await read(url);
    registry.push({ url, value });
    record.json("registry-observations.json", registry);
    return value;
  };
  // Authenticate the selected declared dependency closure, not unrelated/extraneous files.
  const queue = Object.keys(asRecord(requested.devDependencies)).map((name) => {
    const directory = dependencyDirectory(root, root, name);
    assert(directory);
    return directory;
  });
  const visited = new Set<string>();
  while (queue.length) {
    const directory = queue.shift()!;
    if (visited.has(directory)) continue;
    visited.add(directory);
    const path = relative(root, directory),
      row = asRecord(rows[path]);
    assert.equal(row.link, undefined);
    const raw = physical(join(directory, "package.json")),
      manifest = asRecord(JSON.parse(new TextDecoder("utf8", { fatal: true }).decode(raw)));
    const name = asString(manifest.name),
      version = asString(manifest.version);
    assert.equal(version, row.version);
    const vize = name === "vize" || name.startsWith("@vizejs/");
    if (vize) {
      const sri = asRecord(asRecord(authority.capsule.npmSRI)[name]);
      assert.equal(version, profile.version);
      assert.equal(row.integrity, sri.integrity);
      assert.equal(row.resolved, sri.tarball);
    }
    let metadata: unknown;
    await packageMetadata(name, version, row, undefined, async (url) => {
      if (usesE649Packument(profile, name, version)) {
        metadata = await readE649Metadata(
          profile,
          row,
          asRecord(asRecord(authority.capsule.npmSRI)[name]),
          record,
          cache,
        );
        registry.push({
          url: e649MetadataUrl,
          selectedVersion: version,
          representation: "EXACT_ABBREVIATED_INSTALL_PACKUMENT_MEMBER",
          value: metadata,
        });
        record.json("registry-observations.json", registry);
      } else if (
        name === "vize" &&
        usesP73FullPackument(
          profile,
          name,
          version,
          asRecord(asRecord(authority.capsule.npmSRI)[name]),
        )
      ) {
        metadata = await readP73FullMetadata(
          profile,
          row,
          asRecord(asRecord(authority.capsule.npmSRI)[name]),
          record,
          cache,
        );
        registry.push({
          url: p73FullMetadataUrl,
          selectedVersion: version,
          representation: "EXACT_FULL_PACKUMENT_MEMBER",
          value: metadata,
        });
        record.json("registry-observations.json", registry);
      } else metadata = await reader(url);
      return metadata;
    });
    const provenance = vize
      ? await authenticateProvenance("after", name, asRecord(asRecord(metadata).dist), reader)
      : null;
    const integrity = asString(row.integrity);
    let payload = cache.get(integrity);
    if (!payload) {
      payload = await readBytes(asString(row.resolved));
      cache.set(integrity, payload);
    }
    const checked = shippedImages(directory, payload, integrity);
    assert(
      checked.archive.manifest.equals(raw),
      "installed package manifest must equal authenticated public bytes",
    );
    if (name === "vize")
      assertPackedManifest(
        raw,
        checked.archive.manifest,
        asRecord(
          JSON.parse(Buffer.from(authority.sourceFiles[0]!.rawBase64, "base64").toString("utf8")),
        ),
      );
    packages.push({
      name,
      version,
      directory,
      integrity,
      resolved: asString(row.resolved),
      archiveSha512: checked.archive.tarballSHA512,
      files: checked.images,
    });
    images.push(...checked.images);
    record.json("package-" + packages.length + ".json", {
      name,
      version,
      path,
      provenance,
      archiveSha512: checked.archive.tarballSHA512,
      manifestBase64: raw.toString("base64"),
      files: checked.archive.files,
      completeInstalledFilesetQualified: false,
    });
    for (const [dependency, specification] of Object.entries({
      ...asRecord(manifest.dependencies ?? {}),
      ...asRecord(manifest.optionalDependencies ?? {}),
    })) {
      assert.equal(typeof specification, "string");
      const resolved = dependencyDirectory(root, directory, dependency);
      if (resolved) queue.push(resolved);
      else
        assert(
          Object.hasOwn(asRecord(manifest.optionalDependencies ?? {}), dependency),
          "required declared dependency missing",
        );
    }
  }
  for (const [name, version] of Object.entries(asRecord(requested.devDependencies))) {
    const direct = packages.find(
      (pkg) => pkg.directory === realpathSync(join(root, "node_modules", name)),
    );
    assert(direct);
    assert.equal(direct.version, version);
  }
  const cliRoot = realpathSync(join(root, "node_modules/vize")),
    graph = captureDeclarationGraph(cliRoot);
  assert.deepEqual(
    graph.files.map((file) => relative(cliRoot, file.path)).sort(),
    authority.sourceFiles
      .slice(1)
      .map((file) => file.file)
      .sort(),
  );
  for (const image of graph.files) {
    const expected = authority.sourceFiles.find(
      (source) => source.file === relative(cliRoot, image.path),
    );
    assert(expected);
    assert.equal(image.bytes, expected.bytes);
    assert.equal(image.sha256, expected.sha256);
  }
  const nativeLoad = requireNativeLoadPrerequisite(
    root,
    packages,
    graph.files,
    {
      version: profile.version,
      candidate: profile.candidate,
      source: profile.source,
      workflowHead: authority.ownership.workflowHead,
      run: profile.run,
      capsule: authority.capsule,
      sdk: graph.files,
      observations: [],
      produced: authority.ownership.produced,
    },
    process.env,
    (kind, value) => record.json("native-prerequisite-" + kind + ".json", value),
  );
  // This is non-executing route/image proof. No companion probe is attributed to the original compiler.
  const selectedRoute = () =>
    profile.subset === "GArrowTsc" ? publishedTscRoute(root, packages) : tsrsRoute(root, packages);
  const route = selectedRoute(),
    bindings = commandBins.map((name) => sourceBin(root, name, images));
  const custody = [
    ...new Map(
      [
        ...images,
        ...graph.files,
        ...managerPublic.images,
        ...manager,
        ...("ldd" in route ? [route.ldd] : []),
        route.node,
        fileImage(join(root, "package-lock.json")),
      ].map((image) => [image.path, image]),
    ).values(),
  ];
  const verify = () => {
    checkManager();
    assertCustody(custody);
    assert.deepEqual(
      commandBins.map((name) => sourceBin(root, name, images)),
      bindings,
    );
    assert.deepEqual(selectedRoute(), route);
    recheckNativeLoadPrerequisite({ root, packages, sdk: graph.files, nativeLoad }, process.env);
  };
  verify();
  record.json("mandatory-installed-route.json", {
    route,
    bindings,
    custody,
    nativeLoad,
    selectedCompilerPid: null,
    completeInstalledFilesetQualified: false,
  });
  return { root, verify, custody };
}
