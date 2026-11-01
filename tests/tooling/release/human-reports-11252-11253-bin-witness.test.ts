import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { writeTarGz } from "../../../tools/support/archive/tar.ts";
import { fileImage } from "../../../tools/support/release/computed_event_custody.ts";
import { witnessKnownBins } from "../../../tools/support/release/human_reports_11252_11253_public/bin-witness.ts";
import { observedArchiveComparison } from "../../../tools/support/release/human_reports_11252_11253_public/package-inventory.ts";
import { installedGraph } from "../../../tools/support/release/human_reports_11252_11253_public/package-graph.ts";
import {
  object,
  sha256,
} from "../../../tools/support/release/human_reports_11252_11253_public/value.ts";

function fixture(binary = false) {
  const base = realpathSync(mkdtempSync(join(tmpdir(), "human-bin-witness-")));
  const root = join(base, "consumer"),
    directory = join(root, "node_modules/vue");
  const owner = join(root, "node_modules/typescript"),
    managerRoot = join(base, "manager");
  mkdirSync(join(directory, "node_modules/.bin"), { recursive: true });
  mkdirSync(join(owner, "bin"), { recursive: true });
  mkdirSync(join(managerRoot, "bin"), { recursive: true });
  const vueManifest = Buffer.from(
    '{"name":"vue","version":"3.5.44","peerDependencies":{"typescript":"*"}}\n',
  );
  writeFileSync(join(directory, "package.json"), vueManifest);
  const ownerManifest = Buffer.from(
    JSON.stringify({
      name: "typescript",
      version: "1.0.0",
      exports: {},
      bin: { tsc: "./bin/tsc", tsserver: "./bin/tsserver" },
    }) + "\n",
  );
  writeFileSync(join(owner, "package.json"), ownerManifest);
  const target = binary
    ? Buffer.from([0x7f, 0x45, 0x4c, 0x46, 0, 0xff, 0x80])
    : Buffer.from("throw Error('this target must never execute');\n");
  for (const bin of ["tsc", "tsserver"]) writeFileSync(join(owner, "bin", bin), target);
  const body = writeTarGz([
    { name: "package/package.json", data: ownerManifest },
    { name: "package/bin/tsc", data: target },
    { name: "package/bin/tsserver", data: target },
  ]);
  const integrity = "sha512-" + createHash("sha512").update(body).digest("base64");
  const lock = Buffer.from(
    `lockfileVersion: '9.0'\npackages:\n  typescript@1.0.0:\n    resolution: {integrity: ${integrity}}\n`,
  );
  const shim = Buffer.from("#!/bin/sh\n# synthetic capture fixture; never executed\nexit 9\n");
  for (const bin of ["tsc", "tsserver"])
    writeFileSync(join(directory, "node_modules/.bin", bin), shim);
  const manager = join(managerRoot, "bin/pnpm");
  writeFileSync(manager, "throw Error('manager must never execute');\n");
  writeFileSync(
    join(managerRoot, "package.json"),
    '{"name":"pnpm","version":"12.16.1","bin":{"pnpm":"bin/pnpm"}}\n',
  );
  const contextPath = join(base, "install-context.json");
  writeFileSync(
    contextPath,
    JSON.stringify({
      root,
      node: fileImage(manager),
      pnpm: fileImage(manager),
      originalArgv: ["install"],
      managerFiles: [],
    }) + "\n",
  );
  const events: Array<{ kind: string; value: unknown }> = [];
  const sink = (kind: string, value: unknown) => events.push({ kind, value });
  const metadata = {
    name: "typescript",
    version: "1.0.0",
    dist: { integrity, tarball: "https://registry.npmjs.org/typescript/-/typescript-1.0.0.tgz" },
  };
  const raw = Buffer.from(JSON.stringify(metadata));
  const context = {
    root,
    path: contextPath,
    managerVersion: "12.16.1",
    lock,
    archive: async (name: string, version: string) => {
      assert.equal(name, "typescript");
      assert.equal(version, "1.0.0");
      return {
        name,
        version,
        body,
        integrity,
        metadata,
        rawBase64: raw.toString("base64"),
        rawSha256: sha256(raw),
        url: "https://registry.npmjs.org/typescript/1.0.0",
        resolved: metadata.dist.tarball,
        status: 200 as const,
        date: null,
      };
    },
  };
  return {
    base,
    root,
    directory,
    owner,
    target,
    shim,
    manager,
    context,
    sink,
    events,
    vueManifest,
  };
}

test("whole known shim bodies, manager context and declared owner targets are witnessed without execution or waiver", async () => {
  const f = fixture();
  try {
    await witnessKnownBins(f.directory, f.context, f.sink);
    const bodies = f.events.filter((event) => event.kind === "nested-bin-body-witness");
    assert.equal(bodies.length, 2);
    for (const event of bodies) {
      const row = object(event.value),
        shim = object(row.shim);
      assert(Buffer.from(String(shim.rawBase64), "base64").equals(f.shim));
      assert.equal(shim.sha256, sha256(f.shim));
      assert.equal(row.waiver, false);
      assert.equal(row.interpretation, "UNASSESSED");
    }
    const targets = f.events.filter((event) => event.kind === "nested-bin-target-payload-binding");
    assert.equal(targets.length, 2);
    assert(targets.every((event) => object(event.value).targetBytesEqualPublicMember === true));
    assert(
      targets.every(
        (event) =>
          object(event.value).shimBodyTargetBinding === "UNASSESSED" &&
          object(event.value).executed === false &&
          object(event.value).waiver === false,
      ),
    );
    const manager = f.events.find((event) => event.kind === "nested-bin-manager-context");
    assert(manager);
    assert.equal(object(manager.value).reportedVersion, "12.16.1");
    assert.equal(object(manager.value).managerImageUnchanged, true);
    assert(f.events.some((event) => event.kind === "nested-bin-target-owner-manifest"));
    assert(f.events.some((event) => event.kind === "nested-bin-manager-owner-manifest"));
    assert(!f.events.some((event) => event.kind === "nested-bin-witness-refusal"));
    const publicVue = writeTarGz([{ name: "package/package.json", data: f.vueManifest }]);
    assert.throws(
      () =>
        observedArchiveComparison(
          f.directory,
          publicVue,
          "sha512-" + createHash("sha512").update(publicVue).digest("base64"),
          f.sink,
        ),
      /file sets differ/u,
    );
  } finally {
    rmSync(f.base, { recursive: true, force: true });
  }
});

test("a native target is joined by public member bytes/hash while its body is excluded from receipts", async () => {
  const f = fixture(true);
  try {
    await witnessKnownBins(f.directory, f.context, f.sink);
    assert.equal(
      f.events.filter((event) => event.kind === "nested-bin-target-payload-binding").length,
      2,
    );
    const inspect = (value: unknown): void => {
      assert(!Buffer.isBuffer(value));
      if (Array.isArray(value)) value.forEach(inspect);
      else if (value && typeof value === "object") Object.values(value).forEach(inspect);
    };
    inspect(f.events);
    assert(!JSON.stringify(f.events).includes(f.target.toString("base64")));
  } finally {
    rmSync(f.base, { recursive: true, force: true });
  }
});

test("manager loader references are hashed without execution or native body export", async () => {
  const f = fixture();
  try {
    const owner = dirname(dirname(f.manager)),
      target = join(owner, "dist/implementation.cjs");
    mkdirSync(dirname(target));
    const body = Buffer.alloc(8400, 0xff);
    writeFileSync(target, body);
    writeFileSync(f.manager, '#!/usr/bin/env node\nrequire("../dist/implementation.cjs");\n');
    await witnessKnownBins(f.directory, f.context, f.sink);
    const entries = f.events.filter(
      (event) => event.kind === "nested-bin-manager-static-reference",
    );
    assert.equal(entries.length, 2);
    for (const event of entries) {
      const row = object(event.value);
      assert.equal(object(row.image).sha256, sha256(body));
      assert.equal(row.bodyExcluded, true);
      assert.equal(row.runtimeLoadObserved, false);
    }
    assert(!JSON.stringify(f.events).includes(body.toString("base64")));
    assert.equal(
      object(f.events.find((event) => event.kind === "nested-bin-manager-context")!.value)
        .managerImageUnchanged,
      false,
    );
  } finally {
    rmSync(f.base, { recursive: true, force: true });
  }
});

test("the real package graph persists bin facts before the unchanged strict file-set refusal", async () => {
  const f = fixture();
  try {
    const body = writeTarGz([{ name: "package/package.json", data: f.vueManifest }]);
    const integrity = "sha512-" + createHash("sha512").update(body).digest("base64");
    const resolved = "https://registry.npmjs.org/vue/-/vue-3.5.44.tgz";
    const metadata = { name: "vue", version: "3.5.44", dist: { integrity, tarball: resolved } };
    const raw = Buffer.from(JSON.stringify(metadata));
    await assert.rejects(
      installedGraph(
        f.root,
        [{ dependencies: { vue: { version: "3.5.44", path: f.directory, resolved } } }],
        Buffer.concat([
          f.context.lock,
          Buffer.from(`\n  vue@3.5.44:\n    resolution: {integrity: ${integrity}}\n`),
        ]),
        {
          os: "linux",
          cpu: "x64",
          libc: "glibc",
          libcVersion: "2.39",
          libcSource: "Node process.report.header.glibcVersionRuntime; absence never implies musl",
        },
        new Map(),
        new Map(),
        f.sink,
        {
          archive: async (name, version) =>
            name === "typescript"
              ? f.context.archive(name, version)
              : {
                  name,
                  version,
                  body,
                  integrity,
                  metadata,
                  rawBase64: raw.toString("base64"),
                  rawSha256: sha256(raw),
                  url: "https://registry.npmjs.org/vue/3.5.44",
                  resolved,
                  status: 200 as const,
                  date: null,
                },
          metadata: async () => {
            throw Error("no absent package expected");
          },
        },
        { path: f.context.path, managerVersion: "12.16.1" },
      ),
      /public archive and installed package file sets differ/u,
    );
    assert.equal(f.events.filter((event) => event.kind === "nested-bin-body-witness").length, 2);
    assert.equal(
      f.events.filter((event) => event.kind === "nested-bin-target-payload-binding").length,
      2,
    );
    assert(f.events.some((event) => event.kind === "public-package-file-inventory"));
  } finally {
    rmSync(f.base, { recursive: true, force: true });
  }
});

test("changed targets or forged integrity produce explicit witness refusal, never a binding or changed file-set rule", async () => {
  for (const forge of [false, true]) {
    const f = fixture();
    try {
      if (forge) {
        const read = f.context.archive;
        f.context.archive = async (name, version) => ({
          ...(await read(name, version)),
          integrity: "sha512-" + Buffer.alloc(64).toString("base64"),
        });
      } else
        for (const bin of ["tsc", "tsserver"])
          writeFileSync(join(f.owner, "bin", bin), "modified target");
      await witnessKnownBins(f.directory, f.context, f.sink);
      assert.equal(
        f.events.filter((event) => event.kind === "nested-bin-witness-refusal").length,
        2,
      );
      assert.equal(
        f.events.filter((event) => event.kind === "nested-bin-target-payload-binding").length,
        0,
      );
      assert.equal(f.events.filter((event) => event.kind === "nested-bin-body-witness").length, 2);
    } finally {
      rmSync(f.base, { recursive: true, force: true });
    }
  }
});

test("oversized or nontext shims stay hash-only and cannot create a body or ownership claim", async () => {
  const f = fixture();
  try {
    writeFileSync(join(f.directory, "node_modules/.bin/tsc"), Buffer.alloc(4201, 65));
    writeFileSync(join(f.directory, "node_modules/.bin/tsserver"), Buffer.from([0, 0xff]));
    await witnessKnownBins(f.directory, f.context, f.sink);
    const bodies = f.events
      .filter((event) => event.kind === "nested-bin-body-witness")
      .map((event) => object(object(event.value).shim));
    assert.deepEqual(
      bodies.map((row) => row.rawBase64),
      [null, null],
    );
    assert.deepEqual(
      bodies.map((row) => row.bodyState),
      ["SIZE_EXCEEDS_TEXT_WITNESS", "NOT_UTF8_TEXT"],
    );
  } finally {
    rmSync(f.base, { recursive: true, force: true });
  }
});

test("bin-owner target escapes refuse without reading or exporting the outside body", async () => {
  const f = fixture();
  try {
    const manifest = JSON.parse(readFileSync(join(f.owner, "package.json"), "utf8")) as {
      bin: Record<string, string>;
    };
    manifest.bin = { tsc: "../../../secret.txt", tsserver: "../../../secret.txt" };
    writeFileSync(join(f.owner, "package.json"), JSON.stringify(manifest));
    writeFileSync(join(f.base, "secret.txt"), "unrelated private body must not be read");
    await witnessKnownBins(f.directory, f.context, f.sink);
    assert.equal(f.events.filter((event) => event.kind === "nested-bin-witness-refusal").length, 2);
    assert(!JSON.stringify(f.events).includes("unrelated private body must not be read"));
    assert(!f.events.some((event) => event.kind === "nested-bin-target-public-archive"));
  } finally {
    rmSync(f.base, { recursive: true, force: true });
  }
});
