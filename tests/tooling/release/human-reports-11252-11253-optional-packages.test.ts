import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { writeTarGz } from "../../../tools/support/archive/tar.ts";
import { installedGraph } from "../../../tools/support/release/human_reports_11252_11253_public/package-graph.ts";
import { lockPackageIntegrity } from "../../../tools/support/release/human_reports_11252_11253_public/lock-integrity.ts";
import {
  absentPackage,
  managerFiles,
  platformExclusion,
  physicalPathWitness,
  validateOptionalAbsence,
  type HostFacts,
} from "../../../tools/support/release/human_reports_11252_11253_public/optional-residency.ts";
import type { PublicMetadata } from "../../../tools/support/release/human_reports_11252_11253_public/network.ts";
import { sha256 } from "../../../tools/support/release/human_reports_11252_11253_public/value.ts";

const host: HostFacts = {
  os: "linux",
  cpu: "x64",
  libc: "glibc",
  libcVersion: "2.39",
  libcSource: "Node process.report.header.glibcVersionRuntime; absence never implies musl",
};
const child = "@vendor/foreign";
const version = "1.0.0";
const sri = (body: Buffer) => "sha512-" + createHash("sha512").update(body).digest("base64");
function published(
  name: string,
  body: Buffer,
  conditions: Record<string, unknown> = {},
): PublicMetadata {
  const resolved = `https://registry.npmjs.org/${name}/-/${name.split("/").at(-1)}-${version}.tgz`;
  const metadata = {
    name,
    version,
    ...conditions,
    dist: { tarball: resolved, integrity: sri(body) },
  };
  const raw = Buffer.from(JSON.stringify(metadata) + "\n");
  return {
    name,
    version,
    resolved,
    integrity: sri(body),
    metadata,
    rawBase64: raw.toString("base64"),
    rawSha256: sha256(raw),
    url: `https://registry.npmjs.org/${encodeURIComponent(name)}/${version}`,
    status: 200,
    date: null,
  };
}
function fixture() {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "human-optional-")));
  mkdirSync(join(root, "node_modules/.pnpm"), { recursive: true });
  const body = Buffer.from("synthetic unused child integrity only; never executed");
  const metadata = published(child, body, { os: ["darwin"], cpu: ["arm64"] });
  const lock = Buffer.from(
    `---\nlockfileVersion: '9.0'\npackages:\n  '${child}@${version}':\n    resolution: {integrity: ${metadata.integrity}}\n    cpu: [arm64]\n    os: [darwin]\n\nsnapshots:\n  '${child}@${version}':\n    optional: true\n`,
  );
  const path = join(
    root,
    "node_modules/.pnpm",
    child.replaceAll("/", "+") + "@" + version,
    "node_modules",
    child,
  );
  const archives = new Map<string, { body: Buffer; metadata: PublicMetadata }>();
  const archiveCalls: string[] = [],
    metadataCalls: string[] = [],
    observations: unknown[] = [];
  const parent = (name: string, optional = true) => {
    const directory = join(root, "node_modules/.pnpm", name + "@" + version, "node_modules", name);
    mkdirSync(directory, { recursive: true });
    const raw = Buffer.from(
      JSON.stringify({
        name,
        version,
        [optional ? "optionalDependencies" : "dependencies"]: { [child]: version },
      }) + "\n",
    );
    writeFileSync(join(directory, "package.json"), raw);
    const archive = writeTarGz([{ name: "package/package.json", data: raw }]);
    const parentMetadata = published(name, archive);
    archives.set(name, { body: archive, metadata: parentMetadata });
    return {
      from: name,
      version,
      path: directory,
      resolved: parentMetadata.resolved,
      dependencies: { [child]: { from: child, version, path, resolved: metadata.resolved } },
    };
  };
  const run = (
    parents: ReturnType<typeof parent>[],
    required = new Map<string, string>(),
    actualHost = host,
  ) =>
    installedGraph(
      root,
      [{ devDependencies: Object.fromEntries(parents.map((row) => [row.from, row])) }],
      lock,
      actualHost,
      required,
      new Map(),
      (name, value) => observations.push({ name, value }),
      {
        archive: async (name) => {
          archiveCalls.push(name);
          const row = archives.get(name);
          assert(row, `unexpected archive download: ${name}`);
          return { ...row.metadata, body: row.body };
        },
        metadata: async (name) => {
          metadataCalls.push(name);
          assert.equal(name, child);
          return metadata;
        },
      },
    );
  return {
    root,
    path,
    metadata,
    lock,
    parent,
    run,
    archives,
    archiveCalls,
    metadataCalls,
    observations,
    close: () => rmSync(root, { recursive: true }),
  };
}
test("flattened dependencies require a genuine packed optional parent, lock and foreign metadata; no child tarball", async () => {
  const f = fixture();
  try {
    const result = await f.run([f.parent("parent")]);
    assert.equal(result.packages.length, 1);
    assert.equal(result.absent.length, 1);
    assert.equal(result.absent[0]?.state, "DECLARED_OPTIONAL_NOT_INSTALLED");
    assert.equal(result.absent[0]?.nativeLoadCredit, false);
    assert.deepEqual(f.archiveCalls, ["parent"]);
    assert.deepEqual(f.metadataCalls, [child]);
    validateOptionalAbsence(result.absent[0]!, f.root, f.lock, host, result.packages);
    assert.equal(
      JSON.stringify(f.observations).includes('"body":{"type":"Buffer"'),
      false,
      "archive bodies never enter proof events",
    );
    const mutation = structuredClone(result.absent[0]!);
    mutation.metadata.integrity = sri(Buffer.from("wrong"));
    assert.throws(() => validateOptionalAbsence(mutation, f.root, f.lock, host, result.packages));
    mutation.metadata = structuredClone(result.absent[0]!.metadata);
    mutation.physical.alias += "-forged";
    assert.throws(() => validateOptionalAbsence(mutation, f.root, f.lock, host, result.packages));
  } finally {
    f.close();
  }
});
test("list optional labels and an earlier optional edge cannot waive a required parent's edge", async () => {
  const f = fixture();
  try {
    const optional = f.parent("first");
    const required = f.parent("second", false);
    await assert.rejects(f.run([optional, required]));
    assert.deepEqual(f.archiveCalls, ["first", "second"]);
  } finally {
    f.close();
  }
});
test("a role-required provider cannot become a foreign absence", async () => {
  const f = fixture();
  try {
    await assert.rejects(f.run([f.parent("parent")], new Map([[child, version]])));
    assert.deepEqual(f.metadataCalls, []);
  } finally {
    f.close();
  }
});
test("host-compatible or unconstrained absence refuses; libc needs actual evidence", () => {
  assert.throws(() => platformExclusion({}, host));
  assert.throws(() => platformExclusion({ os: ["linux"], cpu: ["x64"] }, host));
  assert.deepEqual(platformExclusion({ os: ["linux"], cpu: ["x64"], libc: ["musl"] }, host), [
    "libc",
  ]);
  assert.throws(() =>
    platformExclusion({ libc: ["musl"] }, { ...host, libc: "UNKNOWN", libcVersion: null }),
  );
  assert.throws(() => platformExclusion({ os: "darwin" }, host));
  assert.throws(() => platformExclusion({ os: ["darwin"], cpu: [42] }, host));
});
test("forged registry identity, resolution, or locked integrity refuses", async () => {
  for (const field of ["name", "version", "resolved", "integrity"] as const) {
    const f = fixture();
    try {
      const parent = f.parent("parent");
      f.metadata[field] = "forged";
      await assert.rejects(f.run([parent]));
    } finally {
      f.close();
    }
  }
});
test("present dangling links, partial stores and existing aliases never become absence", () => {
  for (const kind of ["dangling", "partial", "alias"] as const) {
    const f = fixture();
    try {
      const parent = f.parent("parent");
      const store = join(f.root, "node_modules/.pnpm", child.replaceAll("/", "+") + "@" + version);
      if (kind === "dangling") symlinkSync(join(f.root, "missing"), store);
      else if (kind === "partial") mkdirSync(store);
      else {
        const alias = join(
          parent.path.slice(0, parent.path.lastIndexOf("/node_modules/") + "/node_modules".length),
          child,
        );
        mkdirSync(join(alias, ".."), { recursive: true });
        symlinkSync(join(f.root, "missing"), alias);
      }
      assert.throws(() => absentPackage(f.root, f.path, parent.path, child, version));
    } finally {
      f.close();
    }
  }
});
test("present payload mismatch refuses before its children or metadata can be waived", async () => {
  const f = fixture();
  try {
    const parent = f.parent("parent");
    writeFileSync(join(parent.path, "unexpected"), "different installed payload");
    await assert.rejects(f.run([parent]));
    assert.deepEqual(f.metadataCalls, []);
  } finally {
    f.close();
  }
});
test("a present foreign package is fully authenticated and counted resident, never an absence", async () => {
  const f = fixture();
  try {
    const parent = f.parent("parent");
    mkdirSync(f.path, { recursive: true });
    const raw = Buffer.from(
      JSON.stringify({ name: child, version, os: ["darwin"], cpu: ["arm64"] }) + "\n",
    );
    writeFileSync(join(f.path, "package.json"), raw);
    const archive = writeTarGz([{ name: "package/package.json", data: raw }]);
    const metadata = published(child, archive, { os: ["darwin"], cpu: ["arm64"] });
    f.archives.set(child, { body: archive, metadata });
    const result = await f.run([parent]);
    assert.equal(result.packages.length, 2);
    assert.equal(result.absent.length, 0);
    assert.deepEqual(f.archiveCalls, ["parent", child]);
    assert.deepEqual(f.metadataCalls, []);
  } finally {
    f.close();
  }
});
test("a broken alias is retained as an actual link witness before strict refusal", () => {
  const f = fixture();
  try {
    const parent = f.parent("parent");
    const alias = join(
      parent.path.slice(0, parent.path.lastIndexOf("/node_modules/") + "/node_modules".length),
      child,
    );
    mkdirSync(join(alias, ".."), { recursive: true });
    symlinkSync(join(f.root, "missing"), alias);
    const witness = physicalPathWitness(f.root, f.path, parent.path, child);
    assert(
      witness.some(
        (row) =>
          row.path === alias &&
          row.state === "PRESENT" &&
          row.kind === "symlink" &&
          row.link === join(f.root, "missing"),
      ),
    );
    assert.throws(() => absentPackage(f.root, f.path, parent.path, child, version));
  } finally {
    f.close();
  }
});
test("early root files preserve actual generated workspace and lock bytes", () => {
  const f = fixture();
  try {
    writeFileSync(join(f.root, "package.json"), '{"private":true}\n');
    writeFileSync(join(f.root, "pnpm-lock.yaml"), f.lock);
    writeFileSync(
      join(f.root, "pnpm-workspace.yaml"),
      "minimumReleaseAgeExclude:\n  - synthetic@1.0.0\n",
    );
    const files = managerFiles(f.root);
    assert.equal(files[3]?.state, "ABSENT");
    assert(
      Buffer.from(files[2]!.rawBase64!, "base64").equals(
        readFileSync(join(f.root, "pnpm-workspace.yaml")),
      ),
    );
  } finally {
    f.close();
  }
});
test("lock proof is package-scoped and refuses duplicate declarations or documents", () => {
  const f = fixture();
  try {
    assert.equal(lockPackageIntegrity(f.lock, child, version).integrity, f.metadata.integrity);
    assert.throws(() => lockPackageIntegrity(Buffer.concat([f.lock, f.lock]), child, version));
    for (const noncanonical of [
      '"packages": {}',
      "packages : {}",
      "'packages': {}",
      "\"lockfileVersion\": '9.0'",
    ])
      assert.throws(() =>
        lockPackageIntegrity(
          Buffer.from(f.lock.toString() + "\n" + noncanonical + "\n"),
          child,
          version,
        ),
      );
    assert.throws(() =>
      lockPackageIntegrity(
        Buffer.from(
          f.lock
            .toString()
            .replace(
              `resolution: {integrity: ${f.metadata.integrity}}`,
              `resolution:\n      tarball: wrong\n    other:\n      integrity: ${f.metadata.integrity}`,
            ),
        ),
        child,
        version,
      ),
    );
    assert.throws(() =>
      lockPackageIntegrity(
        Buffer.from(f.lock.toString().replace("packages:", "lockfileVersion: '9.0'\npackages:")),
        child,
        version,
      ),
    );
    assert.throws(() =>
      lockPackageIntegrity(
        Buffer.from(
          f.lock.toString() +
            "\npackages:\n  other@1.0.0:\n    resolution: {integrity: " +
            f.metadata.integrity +
            "}\n",
        ),
        child,
        version,
      ),
    );
    assert.throws(() =>
      lockPackageIntegrity(
        Buffer.from(f.lock.toString() + "\n---\nother: value\n"),
        child,
        version,
      ),
    );
    assert.throws(() =>
      lockPackageIntegrity(
        Buffer.from(f.lock.toString().replace("packages:", "notPackages:")),
        child,
        version,
      ),
    );
    assert.throws(() => lockPackageIntegrity(f.lock, child, "2.0.0"));
    assert.throws(() =>
      lockPackageIntegrity(
        Buffer.from(f.lock.toString().replace("'9.0'", "'10.0'")),
        child,
        version,
      ),
    );
    assert.throws(() =>
      lockPackageIntegrity(
        Buffer.from(f.lock.toString().replace("resolution:", "other:")),
        child,
        version,
      ),
    );
    assert.throws(() =>
      lockPackageIntegrity(
        Buffer.from(f.lock.toString().replace("resolution:", "<<: *alias\n    resolution:")),
        child,
        version,
      ),
    );
  } finally {
    f.close();
  }
});
