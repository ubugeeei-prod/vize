import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  assertAxeArchive,
  assertActorIdentity,
  assertActorAuthority,
  assertAxeVendor,
  axePolicy,
  installedAxe,
  retainInstalledInputs,
} from "../../tools/support/release/public_acceptance/a11y.ts";

const hash = (value: Buffer) => createHash("sha256").update(value).digest("hex");
const write = (file: string, value: unknown) => {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, typeof value === "string" ? value : JSON.stringify(value));
};
function fixture(t: { after(callback: () => void): void }) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-a11y-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const consumer = path.join(root, "fresh consumer Ω");
  const directory = path.join(consumer, "node_modules", axePolicy.name);
  const manifest = path.join(consumer, "package.json");
  const lockFile = path.join(consumer, "package-lock.json");
  const output = path.join(root, "evidence");
  fs.mkdirSync(output);
  write(manifest, { private: true, dependencies: { [axePolicy.name]: axePolicy.version } });
  const lock = {
    lockfileVersion: 3,
    packages: {
      "": { dependencies: { [axePolicy.name]: String(axePolicy.version) } },
      "node_modules/axe-core": { ...axePolicy },
    },
  };
  write(lockFile, lock);
  write(path.join(directory, "package.json"), {
    name: axePolicy.name,
    version: axePolicy.version,
    main: "axe.js",
  });
  // Controlled discovery fixture, not a public-archive or browser acceptance claim.
  write(path.join(directory, "axe.js"), `module.exports = { version: '${axePolicy.version}' };`);
  write(path.join(directory, "axe.min.js"), "controlled vendor bytes\n");
  return { root, consumer, directory, manifest, lock, lockFile, output };
}

test("actual installed A11y peer resolves only inside its owned consumer and retains original bytes", (t) => {
  const f = fixture(t);
  const files = retainInstalledInputs(f.consumer, f.output);
  const observation = installedAxe(f.consumer);
  assert.equal(observation.loadedVersion, axePolicy.version);
  assert.equal(observation.directory, f.directory);
  assert.equal(observation.successfulReturnObserved, true);
  assert.equal(observation.lockSha256, hash(fs.readFileSync(f.lockFile)));
  for (const item of files) {
    assert.equal(hash(fs.readFileSync(path.join(f.output, item.file))), item.sha256);
  }
  const vendor = fs.readFileSync(path.join(f.directory, "axe.min.js"));
  assertAxeVendor(vendor, observation);
  assert.throws(() => assertAxeVendor(Buffer.from("substituted vendor"), observation));
});

test("original omitted optional-peer contract fails early while actual manifest and lock survive", (t) => {
  const f = fixture(t);
  write(f.manifest, { private: true, dependencies: { playwright: "1.62.1" } });
  const files = retainInstalledInputs(f.consumer, f.output);
  assert.throws(() => installedAxe(f.consumer));
  assert.equal(files.length, 2);
  assert.deepEqual(
    fs.readFileSync(path.join(f.output, "installed-package.json")),
    fs.readFileSync(f.manifest),
  );
  assert.deepEqual(
    fs.readFileSync(path.join(f.output, "package-lock.json")),
    fs.readFileSync(f.lockFile),
  );
});

const mutations: Record<string, (f: ReturnType<typeof fixture>) => void> = {
  "missing package cannot borrow an ancestor installation": (f) => {
    fs.renameSync(f.directory, path.join(f.root, "borrowed-axe"));
  },
  "borrowed package symlink is refused": (f) => {
    const borrowed = path.join(f.root, "borrowed-axe");
    fs.renameSync(f.directory, borrowed);
    fs.symlinkSync(borrowed, f.directory, "dir");
  },
  "borrowed script symlink is refused": (f) => {
    const file = path.join(f.directory, "axe.min.js");
    const borrowed = path.join(f.root, "borrowed.js");
    fs.renameSync(file, borrowed);
    fs.symlinkSync(borrowed, file);
  },
  "changed actual package version is refused": (f) => {
    write(path.join(f.directory, "package.json"), {
      name: axePolicy.name,
      version: "4.12.0",
      main: "axe.js",
    });
  },
  "loaded Node module version must match the manifest": (f) => {
    write(path.join(f.directory, "axe.js"), "module.exports = { version: '4.12.0' };");
  },
  "root lock declaration cannot use a range": (f) => {
    f.lock.packages[""].dependencies[axePolicy.name] = "^4.13.0";
    write(f.lockFile, f.lock);
  },
  "non-public tarball is refused": (f) => {
    Object.assign(f.lock.packages["node_modules/axe-core"], { resolved: "file:borrowed.tgz" });
    write(f.lockFile, f.lock);
  },
  "different archive SRI is refused": (f) => {
    Object.assign(f.lock.packages["node_modules/axe-core"], {
      integrity: `sha512-${Buffer.alloc(64, 1).toString("base64")}`,
    });
    write(f.lockFile, f.lock);
  },
  "linked lock installation is refused": (f) => {
    Object.assign(f.lock.packages["node_modules/axe-core"], { link: true });
    write(f.lockFile, f.lock);
  },
};

test("new actor recipe authenticates the exact main workflow rather than frozen product H", () => {
  const head = "a".repeat(40);
  const environment = {
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_REF: "refs/heads/main",
    GITHUB_SHA: head,
    GITHUB_WORKFLOW_SHA: head,
    GITHUB_WORKFLOW_REF:
      "ubugeeei-prod/vize/.github/workflows/release-public-acceptance.yml@refs/heads/main",
  };
  assert.equal(assertActorIdentity(head, environment).commit, head);
  for (const [key, changed] of Object.entries({
    GITHUB_REPOSITORY: "other/vize",
    GITHUB_REF: "refs/pull/1/merge",
    GITHUB_SHA: "b".repeat(40),
    GITHUB_WORKFLOW_SHA: "b".repeat(40),
    GITHUB_WORKFLOW_REF:
      "ubugeeei-prod/vize/.github/workflows/release-public-acceptance.yml@refs/heads/borrowed",
  })) {
    assert.throws(() => assertActorIdentity(head, { ...environment, [key]: changed }));
    assert.throws(() => assertActorIdentity(head, { ...environment, [key]: undefined }));
  }
});

test("A11y dependency contract is the exact peer already resolved in the reviewed workspace lock", () => {
  const lock = fs.readFileSync(new URL("../../pnpm-lock.yaml", import.meta.url), "utf8");
  const importer = lock.split("  npm/builder/vite-musea:\n")[1].split(/\n  \S/)[0];
  assert.ok(
    importer.includes(
      `      axe-core:\n        specifier: ^4.7.0\n        version: ${axePolicy.version}`,
    ),
  );
  assert.ok(
    lock.includes(
      `  axe-core@${axePolicy.version}:\n    resolution: {integrity: ${axePolicy.integrity}}`,
    ),
  );
});

test("working actor mutations cannot hide behind a matching main commit name", () => {
  const files = new Map([["a11y.ts", Buffer.from("reviewed actor bytes")]]);
  const read = (name: string) => {
    const bytes = files.get(name);
    assert.ok(bytes, "normal policy blob required");
    return bytes;
  };
  const entry = { file: "a11y.ts", sha256: hash(read("a11y.ts")) };
  assertActorAuthority([entry], read);
  assert.throws(() =>
    assertActorAuthority([{ ...entry, sha256: hash(Buffer.from("altered")) }], read),
  );
  assert.throws(() => assertActorAuthority([{ ...entry, file: "unreviewed.ts" }], read));
  assert.throws(() => assertActorAuthority([{ ...entry, file: "../a11y.ts" }], read));
  assert.throws(() => assertActorAuthority([], read));
});
for (const [name, mutate] of Object.entries(mutations)) {
  test(name, (t) => {
    const f = fixture(t);
    mutate(f);
    assert.throws(() => installedAxe(f.consumer));
  });
}

test("independent inspected archive entries bind manifest, loaded entry and emitted script", (t) => {
  const f = fixture(t);
  const observation = installedAxe(f.consumer);
  const entries = [observation.manifest, observation.entry, observation.script].map((item) => ({
    path: path.relative(f.directory, item.file).split(path.sep).join("/"),
    sha256: item.sha256,
    bytes: item.bytes,
  }));
  const archive = {
    archive: { sha256: "a".repeat(64), sha512: axePolicy.integrity.slice(7), bytes: 1 },
    archiveManifest: {
      name: axePolicy.name,
      version: axePolicy.version,
      sha256: observation.manifest.sha256,
      bytes: observation.manifest.bytes,
      content: fs.readFileSync(observation.manifest.file, "utf8"),
    },
    archiveEntries: entries,
  };
  assertAxeArchive(archive, observation);
  for (const item of [observation.manifest, observation.entry, observation.script]) {
    const changed = structuredClone(observation);
    changed.manifest = { ...observation.manifest };
    changed.entry = { ...observation.entry };
    changed.script = { ...observation.script };
    for (const candidate of [changed.manifest, changed.entry, changed.script]) {
      if (candidate.file === item.file) candidate.sha256 = "b".repeat(64);
    }
    assert.throws(() => assertAxeArchive(archive, changed));
  }
  assert.throws(() => assertAxeArchive({ ...archive, archiveEntries: [] }, observation));
});
