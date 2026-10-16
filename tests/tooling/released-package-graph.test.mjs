import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { verifyPackageGraph } from "../../tools/support/release/lint_reports/package_graph.mjs";

function fixture(t, dependencies = { "@parcel/watcher-wasm": "2.6.0", "@isaacs/cliui": "8.0.2" }) {
  const root = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-published-package-graph-")),
  );
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const consumer = path.join(root, "consumer");
  fs.mkdirSync(consumer);
  const packages = { "": { dependencies } };
  const responses = new Map();
  const requests = [];
  function add(location, name, version, extras = {}, bundled = {}) {
    const directory = path.join(consumer, location);
    fs.mkdirSync(directory, { recursive: true });
    const manifest = { name, version, ...extras };
    const files = {
      "package.json": JSON.stringify(manifest) + "\n",
      "index.js": "export default 1;\n",
      ...bundled,
    };
    for (const [file, bytes] of Object.entries(files)) {
      fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
      fs.writeFileSync(path.join(directory, file), bytes);
    }
    const archive = spawnSync(
      "python3",
      [
        "-c",
        "import io,json,sys,tarfile; f=json.load(sys.stdin); b=io.BytesIO(); t=tarfile.open(fileobj=b,mode='w:gz');\nfor n,s in f.items():\n d=s.encode(); m=tarfile.TarInfo('package/'+n); m.size=len(d); t.addfile(m,io.BytesIO(d))\nt.close(); sys.stdout.buffer.write(b.getvalue())",
      ],
      { input: JSON.stringify(files) },
    );
    assert.equal(archive.status, 0, archive.stderr.toString());
    const resolved = `https://registry.npmjs.org/${name}/-/fixture-${version}.tgz`;
    const integrity = `sha512-${createHash("sha512").update(archive.stdout).digest("base64")}`;
    const metadata = { name, version, dist: { tarball: resolved, integrity } };
    responses.set(`https://registry.npmjs.org/${encodeURIComponent(name)}/${version}`, metadata);
    responses.set(resolved, archive.stdout);
    packages[location] = {
      version,
      resolved,
      integrity,
      ...(extras.dependencies ? { dependencies: extras.dependencies } : {}),
    };
    return { directory, files, metadata, archive: archive.stdout };
  }
  const bundle = add(
    "node_modules/@parcel/watcher-wasm/node_modules/napi-wasm",
    "napi-wasm",
    "1.1.0",
  );
  packages["node_modules/@parcel/watcher-wasm/node_modules/napi-wasm"] = {
    version: "1.1.0",
    inBundle: true,
  };
  const parent = add(
    "node_modules/@parcel/watcher-wasm",
    "@parcel/watcher-wasm",
    "2.6.0",
    {
      bundleDependencies: ["napi-wasm"],
      dependencies: { "napi-wasm": "^1.1.0" },
    },
    Object.fromEntries(
      Object.entries(bundle.files).map(([name, bytes]) => [
        `node_modules/napi-wasm/${name}`,
        bytes,
      ]),
    ),
  );
  add("node_modules/@isaacs/cliui", "@isaacs/cliui", "8.0.2", {
    dependencies: { "string-width-cjs": "npm:string-width@^4.2.0" },
  });
  const alias = add("node_modules/string-width-cjs", "string-width", "4.2.3");
  packages["node_modules/string-width-cjs"].name = "string-width";
  let sequence = 0;
  const read = async (url) => {
    requests.push(url);
    assert(responses.has(url), `unexpected registry request: ${url}`);
    const value = responses.get(url);
    return new Response(Buffer.isBuffer(value) ? value : JSON.stringify(value));
  };
  const verify = async (select = () => true) => {
    fs.writeFileSync(
      path.join(consumer, "package-lock.json"),
      JSON.stringify({ lockfileVersion: 3, packages }),
    );
    const output = path.join(root, `evidence-${sequence++}`);
    fs.mkdirSync(output);
    return verifyPackageGraph(consumer, output, dependencies, select, read);
  };
  return {
    consumer,
    dependencies,
    packages,
    responses,
    requests,
    bundle,
    parent,
    alias,
    add,
    verify,
  };
}

test("normal npm alias and bundled lock shapes retain real archive-member and later byte custody", async (t) => {
  const f = fixture(t);
  const result = await f.verify();
  const bundled = result.records.find((row) => row.location.endsWith("/node_modules/napi-wasm"));
  assert.equal(bundled.metadata.name, "napi-wasm");
  assert.equal(
    bundled.archiveSha256,
    result.records.find((row) => row.metadata.name === "@parcel/watcher-wasm").archiveSha256,
  );
  assert.equal(bundled.bundledIn.location, "node_modules/@parcel/watcher-wasm");
  assert.equal(
    f.requests.some((url) => url.includes("napi-wasm")),
    false,
  );
  assert.equal(
    result.records.find((row) => row.location === "node_modules/string-width-cjs").metadata.version,
    "4.2.3",
  );
  fs.writeFileSync(path.join(f.bundle.directory, "index.js"), "export default 2;\n");
  assert.throws(result.assertDependencies, /installed public dependency changed/);
});

test("direct npm aliases retain the exact target version and registry archive", async (t) => {
  const f = fixture(t, { "string-width-cjs": "npm:string-width@4.2.3" });
  const result = await f.verify(() => false);
  assert.equal(result.records.length, 1);
  assert.equal(result.records[0].metadata.name, "string-width");
  f.dependencies["string-width-cjs"] = "npm:string-width@4.2.4";
  await assert.rejects(
    f.verify(() => false),
    /direct aliases must pin an exact version/,
  );
});

test("unselected transitive bundles keep metadata without pretending to have a separate npm archive", async (t) => {
  const f = fixture(t, { consumer: "1.0.0" });
  f.add("node_modules/consumer", "consumer", "1.0.0");
  const result = await f.verify(() => false);
  assert.deepEqual(
    result.records.map((row) => row.metadata.name),
    ["consumer"],
  );
  assert.equal(
    f.requests.some((url) => url.includes("watcher-wasm") || url.includes("napi-wasm")),
    false,
  );
  delete f.packages["node_modules/@parcel/watcher-wasm/node_modules/napi-wasm"].inBundle;
  await assert.rejects(
    f.verify(() => false),
    /only bundled packages/,
  );
});

test("selection and direct-dependency guards never bypass bundled archive authentication", async (t) => {
  const f = fixture(t, { consumer: "1.0.0" });
  f.add("node_modules/consumer", "consumer", "1.0.0");
  const result = await f.verify((name) => name === "napi-wasm");
  assert(result.records.some((row) => row.metadata.name === "@parcel/watcher-wasm"));
  assert(result.records.some((row) => row.metadata.name === "napi-wasm"));
  f.add("node_modules/direct-bundle", "direct-bundle", "1.0.0");
  f.packages["node_modules/direct-bundle"] = { version: "1.0.0", inBundle: true };
  f.dependencies["direct-bundle"] = "1.0.0";
  await assert.rejects(
    f.verify(() => false),
    /enclosing registry archive/,
  );
});

test("an alias target needs an actual resolving npm declaration rather than an arbitrary name override", async (t) => {
  const f = fixture(t);
  delete f.packages["node_modules/@isaacs/cliui"].dependencies;
  await assert.rejects(f.verify(), /resolving locked npm alias/);
  f.packages["node_modules/@isaacs/cliui"].dependencies = {
    "string-width-cjs": "npm:wrong-target@4.2.3",
  };
  await assert.rejects(f.verify(), /resolving locked npm alias/);
  f.packages["node_modules/@isaacs/cliui"].dependencies["string-width-cjs"] =
    "npm:string-width@5.0.0";
  await assert.rejects(f.verify(), /resolving locked npm alias/);
  f.packages["node_modules/@isaacs/cliui"].dependencies["string-width-cjs"] =
    "npm:string-width@^4.2.0";
  f.add("node_modules/@isaacs/cliui/node_modules/string-width-cjs", "string-width-cjs", "1.0.0");
  await assert.rejects(f.verify(), /resolving locked npm alias/);
});

test("alias version, origin, metadata and integrity checks remain strict", async (t) => {
  for (const mutate of [
    (f) => {
      f.packages["node_modules/string-width-cjs"].name = "wrong";
    },
    (f) => {
      f.packages["node_modules/string-width-cjs"].version = "4.2.4";
    },
    (f) => {
      f.packages["node_modules/string-width-cjs"].resolved = "https://example.invalid/package.tgz";
    },
    (f) => {
      f.alias.metadata.version = "4.2.4";
    },
    (f) => {
      f.alias.metadata.dist.integrity = "sha512-wrong";
    },
    (f) => {
      f.responses.set(f.alias.metadata.dist.tarball, Buffer.from("altered archive"));
    },
  ]) {
    const f = fixture(t);
    mutate(f);
    await assert.rejects(f.verify());
  }
});

test("bundled bytes, extra installed members and missing parent archive membership fail closed", async (t) => {
  for (const mutate of [
    (f) => {
      fs.writeFileSync(path.join(f.bundle.directory, "index.js"), "tampered\n");
    },
    (f) => {
      fs.writeFileSync(path.join(f.bundle.directory, "injected.js"), "injected\n");
    },
    (f) => {
      f.packages["node_modules/@parcel/watcher-wasm"].integrity = "sha512-wrong";
    },
    (f) => {
      f.add("node_modules/@parcel/watcher-wasm", "@parcel/watcher-wasm", "2.6.0", {
        bundleDependencies: ["napi-wasm"],
      });
    },
  ]) {
    const f = fixture(t);
    mutate(f);
    await assert.rejects(f.verify());
  }
});
