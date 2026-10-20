import assert from "node:assert/strict";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  entrypoints,
  facadeSpecifier,
  parseChannel,
  preparePackage,
  publishedAt,
  waitForPublishedVersion,
} from "../../tools/support/release/jsr/package.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));

function temporaryDirectory(t: { after(cleanup: () => void): void }, prefix: string): string {
  const directory = mkdtempSync(resolve(tmpdir(), prefix));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  return directory;
}

await test("every JSR entry point re-exports one npm package at the release version", (t) => {
  const output = temporaryDirectory(t, "vize-jsr-package-");
  const version = preparePackage(output);
  const manifest = JSON.parse(readFileSync(resolve(output, "jsr.json"), "utf8"));
  assert.equal(manifest.name, "@vizejs/vize");
  assert.equal(manifest.version, version);
  assert.deepEqual(manifest.exports, {
    ".": "./mod.ts",
    "./config": "./config.ts",
    "./native": "./native.ts",
    "./vite": "./vite.ts",
  });
  for (const [filename, entrypoint] of Object.entries(entrypoints)) {
    const source = readFileSync(resolve(output, filename), "utf8");
    const specifier = JSON.stringify(facadeSpecifier(entrypoint, version));
    assert.ok(
      source.includes(`export * from ${specifier};`),
      `${filename} re-exports ${specifier}`,
    );
    assert.equal(source.includes("export { default }"), entrypoint.hasDefault === true, filename);
    assert.ok(Object.values(manifest.exports).includes(`./${filename}`), `${filename} is exported`);
  }
  assert.equal(
    readFileSync(resolve(output, "config.ts"), "utf8").includes(`"npm:vize@${version}/config"`),
    true,
  );
  assert.deepEqual(
    readFileSync(resolve(output, "LICENSE")),
    readFileSync(resolve(root, "LICENSE")),
  );
});

await test("the JSR package refuses a version other than the checked-out release", (t) => {
  const output = temporaryDirectory(t, "vize-jsr-version-");
  assert.throws(() => preparePackage(output, { version: "0.1.0" }), /match the checked-out/);
  assert.throws(
    () => preparePackage(output, { version: "0.1.0; echo unsafe" }),
    /match the checked-out/,
  );
  assert.throws(() => preparePackage(resolve(root, "jsr/vize")), /separate staging/);
});

await test("a facade is never generated over npm packages with different versions", (t) => {
  const fixture = temporaryDirectory(t, "vize-jsr-mismatch-");
  for (const directory of ["npm/cli", "npm/native", "npm/builder/vite"]) {
    mkdirSync(resolve(fixture, directory), { recursive: true });
    cpSync(resolve(root, directory, "package.json"), resolve(fixture, directory, "package.json"));
  }
  mkdirSync(resolve(fixture, "jsr/vize"), { recursive: true });
  for (const file of ["jsr.json", "README.md"]) {
    cpSync(resolve(root, "jsr/vize", file), resolve(fixture, "jsr/vize", file));
  }
  cpSync(resolve(root, "LICENSE"), resolve(fixture, "LICENSE"));
  preparePackage(resolve(fixture, "output"), { root: fixture });
  for (const directory of ["npm/native", "npm/builder/vite"]) {
    const manifestPath = resolve(fixture, directory, "package.json");
    const original = readFileSync(manifestPath, "utf8");
    writeFileSync(manifestPath, JSON.stringify({ ...JSON.parse(original), version: "0.1.0" }));
    assert.throws(
      () => preparePackage(resolve(fixture, "output"), { root: fixture }),
      new RegExp(`${directory} must match`),
    );
    writeFileSync(manifestPath, original);
  }
});

await test("the JSR channel is an explicit boolean and never defaults to disabled", () => {
  assert.deepEqual(parseChannel('{"enabled": true}'), { enabled: true });
  assert.deepEqual(parseChannel('{"enabled": false}'), { enabled: false });
  for (const invalid of [
    "null",
    "[]",
    "{}",
    '{"enabled":"true"}',
    '{"enabled":true,"skip":true}',
  ]) {
    assert.throws(() => parseChannel(invalid), /must be exactly/);
  }
  parseChannel(readFileSync(resolve(root, "jsr/vize/channel.json"), "utf8"));
});

await test("the JSR public wait survives server processing and proves immutable metadata", async () => {
  const sleeps: number[] = [];
  const metadata: unknown[] = [
    { latest: "0.502.0", versions: {} },
    { latest: "0.502.0", versions: { "0.504.0": { createdAt: "2026-10-20T15:55:00Z" } } },
  ];
  assert.equal(
    await waitForPublishedVersion("0.504.0", {
      attempts: 2,
      intervalMs: 30_000,
      readMetadata: async () => metadata.shift(),
      sleep: async (milliseconds) => {
        sleeps.push(milliseconds);
      },
    }),
    "2026-10-20T15:55:00Z",
  );
  assert.deepEqual(sleeps, [30_000]);
  assert.equal(publishedAt({ versions: {} }, "0.504.0"), undefined);
  assert.throws(
    () => publishedAt({ versions: { "0.504.0": {} } }, "0.504.0"),
    /no valid createdAt/,
  );
});

await test("the JSR public wait fails with an exact bounded timeout", async () => {
  await assert.rejects(
    waitForPublishedVersion("0.504.0", {
      attempts: 3,
      intervalMs: 30_000,
      readMetadata: async () => ({ versions: {} }),
      sleep: async () => {},
    }),
    /JSR 0\.504\.0 was not public after 1 minutes/,
  );
});

await test("Deno exempts exactly the fresh Vize dependency closure and retains the external age policy", () => {
  interface Manifest {
    readonly name: string;
    readonly dependencies?: Readonly<Record<string, string>>;
    readonly optionalDependencies?: Readonly<Record<string, string>>;
    readonly peerDependencies?: Readonly<Record<string, string>>;
  }
  const directories = new Set([
    ...Object.values(entrypoints).map(({ directory }) => directory),
    "npm/builder/unplugin",
  ]);
  const manifests = new Map<string, Manifest>();
  for (const directory of directories) {
    const manifest: Manifest = JSON.parse(
      readFileSync(resolve(root, directory, "package.json"), "utf8"),
    );
    manifests.set(manifest.name, manifest);
  }
  const jsrManifest = JSON.parse(readFileSync(resolve(root, "jsr/vize/jsr.json"), "utf8"));
  const expected = new Set([`jsr:${jsrManifest.name}`]);
  const pending = Object.values(entrypoints).map(({ name }) => name);
  for (const name of pending) {
    if (expected.has(`npm:${name}`)) continue;
    expected.add(`npm:${name}`);
    const manifest = manifests.get(name);
    assert.ok(manifest, `Review the fresh consumer dependency closure for ${name}`);
    for (const [dependency, version] of Object.entries({
      ...manifest.dependencies,
      ...manifest.optionalDependencies,
      ...manifest.peerDependencies,
    })) {
      if (version.startsWith("workspace:")) {
        pending.push(dependency);
      } else if (version === "catalog:native-binaries") {
        expected.add(`npm:${dependency}`);
      } else {
        assert.ok(
          dependency !== "vize" && !dependency.startsWith("@vizejs/"),
          `Review the fresh consumer dependency closure for ${dependency}`,
        );
      }
    }
  }
  const config = JSON.parse(
    readFileSync(resolve(root, "tools/support/release/jsr/consumer-deno.json"), "utf8"),
  );
  config.minimumDependencyAge.exclude.sort((left: string, right: string) =>
    left.localeCompare(right),
  );
  assert.deepEqual(config, {
    nodeModulesDir: "none",
    minimumDependencyAge: {
      age: "P1D",
      exclude: [...expected].sort((left, right) => left.localeCompare(right)),
    },
  });
});
