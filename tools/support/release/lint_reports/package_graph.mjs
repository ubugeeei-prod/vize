import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sha256 } from "../typed_defaults/custody.mjs";

export async function installPackageGraph(consumer, output, dependencies, select = () => true) {
  fs.writeFileSync(
    path.join(consumer, "package.json"),
    JSON.stringify({ private: true, dependencies }, null, 2) + "\n",
    { flag: "wx" },
  );
  const install = spawnSync(
    "npm",
    [
      "install",
      "--ignore-scripts",
      "--no-audit",
      "--no-fund",
      "--registry=https://registry.npmjs.org",
    ],
    { cwd: consumer, timeout: 180_000, maxBuffer: 8 * 1024 * 1024 },
  );
  fs.writeFileSync(
    path.join(output, "install.json"),
    JSON.stringify(
      {
        status: install.status,
        signal: install.signal,
        error: install.error?.message ?? null,
        stdoutBase64: install.stdout?.toString("base64"),
        stderrBase64: install.stderr?.toString("base64"),
      },
      null,
      2,
    ) + "\n",
  );
  assert.equal(install.error, undefined);
  assert.equal(install.signal, null);
  assert.equal(install.status, 0);
  return verifyPackageGraph(consumer, output, dependencies, select);
}

export async function verifyPackageGraph(
  consumer,
  output,
  dependencies,
  select = () => true,
  read = fetch,
) {
  const lockBytes = fs.readFileSync(path.join(consumer, "package-lock.json"));
  fs.writeFileSync(path.join(output, "package-lock.json"), lockBytes);
  const lock = JSON.parse(lockBytes);
  assert.equal(lock.lockfileVersion, 3);
  assert.deepEqual(lock.packages[""].dependencies, dependencies);
  const records = [];
  const installed = new Map();
  fs.mkdirSync(path.join(output, "packages"));
  for (const [location, entry] of Object.entries(lock.packages)) {
    if (!location) continue;
    assert.equal(entry.link, undefined, "no linked source dependencies");
    assert.match(location, /^node_modules\//);
    assert(!location.split("/").some((part) => part === "." || part === ".."));
    const directory = path.join(consumer, location);
    if (!fs.existsSync(directory)) {
      assert.equal(entry.optional, true, "missing nonoptional public package");
      continue;
    }
    assert(fs.lstatSync(directory).isDirectory() && !fs.lstatSync(directory).isSymbolicLink());
    assert(fs.realpathSync(directory).startsWith(path.join(consumer, "node_modules") + path.sep));
    const manifest = JSON.parse(fs.readFileSync(path.join(directory, "package.json"), "utf8"));
    const key = location.split("node_modules/").at(-1);
    assert.equal(manifest.version, entry.version);
    if (manifest.name !== key) {
      assert.equal(entry.name, manifest.name, "alias target must match the locked package name");
      assert(
        hasAliasDeclaration(lock.packages, location, key, manifest.name, consumer),
        "package name mismatch requires a resolving locked npm alias declaration",
      );
    }
    if (entry.resolved) assert.equal(new URL(entry.resolved).origin, "https://registry.npmjs.org");
    else assert.equal(entry.inBundle, true, "only bundled packages may lack their own tarball");
    installed.set(location, { entry, directory, manifest, key });
  }
  const authenticated = new Map();
  const authenticate = async (location) => {
    if (authenticated.has(location)) return authenticated.get(location);
    const current = installed.get(location);
    assert(current, "bundled package requires its installed enclosing package");
    const { entry, directory, manifest, key } = current;
    let record;
    if (!entry.resolved) {
      const split = location.lastIndexOf("/node_modules/");
      assert(split > 0, "bundled package requires an enclosing registry archive");
      const parentLocation = location.slice(0, split);
      const parent = installed.get(parentLocation);
      const bundles = parent?.manifest.bundleDependencies ?? parent?.manifest.bundledDependencies;
      assert(
        Array.isArray(bundles) && bundles.includes(key),
        "enclosing package must declare the bundled dependency",
      );
      const enclosing = await authenticate(parentLocation);
      const prefix = location.slice(parentLocation.length + 1) + "/";
      const members = enclosing.members
        .filter((member) => member.path.startsWith(prefix))
        .map((member) => ({ ...member, path: member.path.slice(prefix.length) }));
      assert(
        members.some((member) => member.path === "package.json"),
        "bundled package must occur in the authenticated enclosing archive",
      );
      record = {
        location,
        directory,
        metadata: { name: manifest.name, version: manifest.version },
        archiveSha256: enclosing.archiveSha256,
        members,
        bundledIn: { location: parentLocation, prefix, archiveSha256: enclosing.archiveSha256 },
      };
    } else {
      record = await registryPackage(location, current, output, records.length, read);
    }
    authenticated.set(location, record);
    records.push(record);
    fs.writeFileSync(
      path.join(output, "dependencies.json"),
      JSON.stringify(records, null, 2) + "\n",
    );
    return record;
  };
  for (const [location, { manifest }] of installed) {
    // Unselected transitive bundles carry lock metadata only. A selected/direct
    // bundle always authenticates its members through its enclosing npm archive.
    if (
      select(manifest.name) ||
      Object.hasOwn(dependencies, location.slice("node_modules/".length))
    )
      await authenticate(location);
  }
  for (const [name, version] of Object.entries(dependencies)) {
    const row = records.find((r) => r.location === `node_modules/${name}`);
    const target = aliasTarget(version);
    if (target) {
      assert.equal(row?.metadata.name, target.name);
      assert.equal(
        row?.metadata.version,
        target.version,
        "direct aliases must pin an exact version",
      );
    } else assert.equal(row?.metadata.version, version);
  }
  const assertDependencies = () => {
    for (const record of records)
      for (const member of record.members) {
        assert(
          fs
            .realpathSync(record.directory)
            .startsWith(path.join(consumer, "node_modules") + path.sep),
        );
        const filename = path.join(record.directory, member.path);
        assert(fs.lstatSync(filename).isFile() && !fs.lstatSync(filename).isSymbolicLink());
        assert.equal(fs.statSync(filename).size, member.bytes);
        assert.equal(
          sha256(fs.readFileSync(filename)),
          member.sha256,
          "installed public dependency changed",
        );
      }
  };
  assertDependencies();
  return { records, assertDependencies };
}

function aliasTarget(spec) {
  const match = /^npm:((?:@[^/]+\/)?[^@/]+)@(.+)$/.exec(spec);
  return match && { name: match[1], version: match[2] };
}

function hasAliasDeclaration(packages, location, key, target, consumer) {
  return Object.entries(packages).some(([owner, entry]) => {
    const spec = entry.dependencies?.[key] ?? entry.optionalDependencies?.[key];
    if (aliasTarget(spec)?.name !== target) return false;
    if (owner) {
      const manifestFile = path.join(consumer, owner, "package.json");
      if (!fs.existsSync(manifestFile)) return false;
      const manifest = JSON.parse(fs.readFileSync(manifestFile, "utf8"));
      if ((manifest.dependencies?.[key] ?? manifest.optionalDependencies?.[key]) !== spec)
        return false;
    }
    // Check the first locked Node resolution candidate, so an alias in an
    // unrelated nested dependency cannot authorize this installation location.
    for (let current = owner; ;) {
      const candidate = `${current ? current + "/" : ""}node_modules/${key}`;
      if (Object.hasOwn(packages, candidate)) return candidate === location;
      if (!current) return false;
      const parent = current.lastIndexOf("/node_modules/");
      current = parent < 0 ? "" : current.slice(0, parent);
    }
  });
}

async function registryPackage(location, { entry, directory, manifest }, output, sequence, read) {
  const metadataResponse = await read(
    `https://registry.npmjs.org/${encodeURIComponent(manifest.name)}/${entry.version}`,
    { signal: AbortSignal.timeout(60_000) },
  );
  assert(metadataResponse.ok);
  const metadata = await metadataResponse.json();
  assert.equal(metadata.name, manifest.name);
  assert.equal(metadata.version, entry.version);
  assert.equal(metadata.dist.tarball, entry.resolved);
  assert.equal(metadata.dist.integrity, entry.integrity);
  assert.match(entry.integrity, /^sha512-[A-Za-z0-9+/]{86}==$/);
  const response = await read(entry.resolved, { signal: AbortSignal.timeout(120_000) });
  assert(response.ok);
  const chunks = [];
  let count = 0;
  for await (const chunk of response.body) {
    count += chunk.length;
    assert(count <= 128 * 1024 * 1024);
    chunks.push(Buffer.from(chunk));
  }
  const archive = Buffer.concat(chunks);
  assert.equal(`sha512-${createHash("sha512").update(archive).digest("base64")}`, entry.integrity);
  const filename = path.join(output, "packages", `${sequence}.tgz`);
  fs.writeFileSync(filename, archive, { flag: "wx" });
  const comparison = spawnSync(
    "python3",
    [fileURLToPath(new URL("./package_members.py", import.meta.url)), filename, directory],
    { encoding: "utf8", timeout: 60_000, maxBuffer: 8 * 1024 * 1024 },
  );
  assert.equal(comparison.error, undefined);
  assert.equal(comparison.status, 0, comparison.stderr);
  return {
    location,
    directory,
    metadata,
    archiveSha256: sha256(archive),
    members: JSON.parse(comparison.stdout),
  };
}
