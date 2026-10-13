import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { publicNative, rejectOverrides } from "../public_acceptance/installed.ts";

export const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
export function assertSuiteCensus(log: string) {
  for (const [name, count] of Object.entries({
    tests: 3,
    pass: 3,
    fail: 0,
    cancelled: 0,
    skipped: 0,
    todo: 0,
  })) {
    const hits = [...log.matchAll(new RegExp(`^# ${name} ([0-9]+)$`, "gm"))].map((hit) => hit[1]);
    assert.deepEqual(hits, [String(count)], `unique complete ${name} census`);
  }
}
export function rejectSubstitution() {
  rejectOverrides();
  for (const key of ["VIZE_BINDING_PATH", "LD_PRELOAD", "DYLD_INSERT_LIBRARIES"])
    assert.ok(!process.env[key], `runtime override must be absent: ${key}`);
}
export function assertFiles(root: string, records: { file: string; sha256: string }[]) {
  for (const record of records) {
    const file = path.join(root, record.file);
    assert.equal(fs.realpathSync(file), file, record.file);
    assert.equal(digest(fs.readFileSync(file)), record.sha256, record.file);
  }
}

/** Authenticate installed bytes against exact public npm archives, including the actual provider. */
export async function authenticate(version: string, output: string) {
  rejectSubstitution();
  const root = fs.realpathSync(process.cwd()),
    { packages, native, loaded } = publicNative(version);
  assert.equal(typeof native.compileSfc, "function");
  const records = [];
  for (const record of packages.filter((item) => item.installed)) {
    assert.ok(record.installed);
    const prefix = record.name.replaceAll("/", "_");
    const request = async (url: string) => {
      const response = await fetch(url, {
        redirect: "error",
        credentials: "omit",
        signal: AbortSignal.timeout(30_000),
      });
      assert.ok(response.ok, `public HTTP ${response.status}: ${url}`);
      return response;
    };
    const metadata = await (
      await request(`https://registry.npmjs.org/${encodeURIComponent(record.name)}/${version}`)
    ).json();
    assert.equal(metadata.name, record.name);
    assert.equal(metadata.version, version);
    assert.equal(metadata.dist.integrity, record.integrity);
    assert.equal(metadata.dist.tarball, record.resolved);
    const response = await request(record.resolved);
    assert.ok(response.body);
    const chunks = [],
      hashes = createHash("sha512");
    let length = 0;
    for await (const chunk of response.body) {
      length += chunk.byteLength;
      assert.ok(length <= 512 * 1024 * 1024, "public archive size bound");
      hashes.update(chunk);
      chunks.push(chunk);
    }
    assert.ok(length > 0);
    assert.equal(`sha512-${hashes.digest("base64")}`, record.integrity);
    const directory = path.join(output, "registry", prefix);
    fs.mkdirSync(directory, { recursive: true });
    fs.writeFileSync(
      path.join(directory, "metadata.json"),
      JSON.stringify(metadata, null, 2) + "\n",
    );
    const archive = path.join(directory, "archive.tgz");
    fs.writeFileSync(archive, Buffer.concat(chunks));
    const names = execFileSync("tar", ["-tzf", archive], {
      encoding: "utf8",
      maxBuffer: 4 * 1024 * 1024,
    })
      .trim()
      .split("\n");
    assert.ok(
      names.every((name) => name.startsWith("package/") && !name.split("/").includes("..")),
    );
    const unpacked = fs.mkdtempSync(path.join(directory, "unpacked-"));
    execFileSync("tar", ["-xzf", archive, "-C", unpacked], { timeout: 30_000 });
    const archiveRoot = path.join(unpacked, "package"),
      files: { file: string; sha256: string }[] = [];
    const walk = (directory: string) => {
      for (const item of fs.readdirSync(directory, { withFileTypes: true })) {
        const original = path.join(directory, item.name);
        if (item.isDirectory()) walk(original);
        else {
          assert.ok(item.isFile(), "public archive must contain normal files");
          const relative = path.relative(archiveRoot, original),
            installed = path.join(record.directory, relative);
          assert.equal(fs.realpathSync(installed), installed);
          assert.deepEqual(fs.readFileSync(installed), fs.readFileSync(original), installed);
          files.push({
            file: path.relative(root, installed),
            sha256: digest(fs.readFileSync(installed)),
          });
        }
      }
    };
    walk(archiveRoot);
    assert.ok(files.length > 0);
    records.push({ ...record, files });
  }
  return { packages: records, loaded };
}
