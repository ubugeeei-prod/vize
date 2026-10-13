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
  const lockBytes = fs.readFileSync(path.join(consumer, "package-lock.json"));
  fs.writeFileSync(path.join(output, "package-lock.json"), lockBytes);
  const lock = JSON.parse(lockBytes);
  assert.equal(lock.lockfileVersion, 3);
  const records = [];
  fs.mkdirSync(path.join(output, "packages"));
  for (const [location, entry] of Object.entries(lock.packages)) {
    if (!location) continue;
    assert.equal(entry.link, undefined, "no linked source dependencies");
    const directory = path.join(consumer, location);
    if (!fs.existsSync(directory)) {
      assert.equal(entry.optional, true, "missing nonoptional public package");
      continue;
    }
    assert(fs.lstatSync(directory).isDirectory() && !fs.lstatSync(directory).isSymbolicLink());
    assert(fs.realpathSync(directory).startsWith(path.join(consumer, "node_modules") + path.sep));
    const manifest = JSON.parse(fs.readFileSync(path.join(directory, "package.json"), "utf8"));
    assert.equal(manifest.name, location.split("node_modules/").at(-1));
    assert.equal(manifest.version, entry.version);
    assert.equal(new URL(entry.resolved).origin, "https://registry.npmjs.org");
    if (!select(manifest.name) && !Object.hasOwn(dependencies, manifest.name)) continue;
    const metadataResponse = await fetch(
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
    const response = await fetch(entry.resolved, { signal: AbortSignal.timeout(120_000) });
    assert(response.ok);
    const chunks = [];
    let count = 0;
    for await (const chunk of response.body) {
      count += chunk.length;
      assert(count <= 128 * 1024 * 1024);
      chunks.push(Buffer.from(chunk));
    }
    const archive = Buffer.concat(chunks);
    assert.equal(
      `sha512-${createHash("sha512").update(archive).digest("base64")}`,
      entry.integrity,
    );
    const filename = path.join(output, "packages", `${records.length}.tgz`);
    fs.writeFileSync(filename, archive, { flag: "wx" });
    const comparison = spawnSync(
      "python3",
      [fileURLToPath(new URL("./package_members.py", import.meta.url)), filename, directory],
      { encoding: "utf8", timeout: 60_000, maxBuffer: 8 * 1024 * 1024 },
    );
    assert.equal(comparison.error, undefined);
    assert.equal(comparison.status, 0, comparison.stderr);
    const record = {
      location,
      directory,
      metadata,
      archiveSha256: sha256(archive),
      members: JSON.parse(comparison.stdout),
    };
    records.push(record);
    fs.writeFileSync(
      path.join(output, "dependencies.json"),
      JSON.stringify(records, null, 2) + "\n",
    );
  }
  for (const [name, version] of Object.entries(dependencies)) {
    const row = records.find((r) => r.location === `node_modules/${name}`);
    assert.equal(row?.metadata.version, version);
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
