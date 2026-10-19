import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  authenticate,
  apiRoot,
  consumerPaths,
  fixedMerge,
  optionsFrom,
  overrides,
  sha256,
} from "../../tools/support/release/typed_defaults/custody.mjs";
import {
  archiveName,
  assertLoadedBinary,
  selectAssets,
  verifyAssets,
} from "../../tools/support/release/typed_defaults/distribution.mjs";
import { officialFixture } from "./support/released-typed-defaults-fixtures.mjs";
import { writeTarGz } from "../../tools/support/archive/tar.ts";

await test("official C/H/source PR/R tag custody accepts inert bindings and refuses every changed authority", async () => {
  await authenticate(officialFixture().options, officialFixture().read);
  const mutations = [
    [
      "pending Release",
      (f) => {
        f.rows.get(`${apiRoot}/actions/runs/${f.options.run}`).status = "in_progress";
      },
    ],
    [
      "red Release",
      (f) => {
        f.rows.get(`${apiRoot}/actions/runs/${f.options.run}`).conclusion = "failure";
      },
    ],
    [
      "wrong public H",
      (f) => {
        f.rows.get(`${apiRoot}/git/tags/${"a".repeat(40)}`).object.sha = f.options.cut;
      },
    ],
    [
      "wrong cut annotation",
      (f) => {
        f.rows.get(`${apiRoot}/git/tags/${"a".repeat(40)}`).message = f.rows
          .get(`${apiRoot}/git/tags/${"a".repeat(40)}`)
          .message.replace(f.options.cut, f.options.head);
      },
    ],
    [
      "wrong source PR",
      (f) => {
        f.rows.get(`${apiRoot}/pulls/${f.options.sourcePr}`).head.sha = f.options.cut;
      },
    ],
    [
      "foreign source",
      (f) => {
        f.rows.get(`${apiRoot}/pulls/${f.options.sourcePr}`).head.repo.full_name = "foreign/vize";
      },
    ],
    [
      "repacked H graph",
      (f) => {
        f.rows.get(`${apiRoot}/git/commits/${f.options.head}`).parents[0].sha = f.options.head;
      },
    ],
    [
      "fix not released",
      (f) => {
        f.rows.get(`${apiRoot}/compare/${fixedMerge}...${f.options.head}`).merge_base_commit.sha =
          f.options.cut;
      },
    ],
    [
      "fake source marker",
      (f) => {
        f.rows.get(`${apiRoot}/pulls/${f.options.sourcePr}`).body +=
          "\n<!-- vize-release-pin-cut: fake -->";
      },
    ],
    [
      "wrong Release source",
      (f) => {
        f.rows.get(`${apiRoot}/actions/runs/${f.options.run}`).head_sha = f.options.cut;
      },
    ],
    [
      "red actual job",
      (f) => {
        f.rows.get(
          `${apiRoot}/actions/runs/${f.options.run}/jobs?filter=latest&per_page=100&page=1`,
        ).jobs[0].conclusion = "failure";
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const fixture = officialFixture();
    mutate(fixture);
    await assert.rejects(() => authenticate(fixture.options, fixture.read), undefined, name);
  }
});

await test("runtime override and source checkout consumer substitutions are refused before execution", (t) => {
  const f = officialFixture();
  const env = {
    PUBLIC_CUT: f.options.cut,
    PUBLIC_HEAD: f.options.head,
    PUBLIC_TAG: f.options.tag,
    PUBLIC_SOURCE_PR: f.options.sourcePr,
    PUBLIC_RELEASE_RUN: f.options.run,
  };
  assert.deepEqual(optionsFrom(env), f.options);
  for (const key of overrides)
    assert.throws(() => optionsFrom({ ...env, [key]: "inert override" }), /runtime override/);
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "typed-public-paths-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const checkout = path.join(root, "checkout"),
    consumer = path.join(root, "consumer"),
    output = path.join(root, "evidence");
  fs.mkdirSync(checkout);
  const paths = { RUNNER_TEMP: root, PUB_CONSUMER: consumer, PUBLIC_OUTPUT: output };
  assert.deepEqual(consumerPaths(paths, checkout), { consumer, output });
  assert.throws(
    () => consumerPaths({ ...paths, PUB_CONSUMER: path.join(checkout, "consumer") }, checkout),
    /outside checkout/,
  );
  const alias = path.join(root, "checkout-alias");
  fs.symlinkSync(checkout, alias);
  assert.throws(
    () => consumerPaths({ ...paths, PUB_CONSUMER: path.join(alias, "consumer") }, checkout),
    /outside checkout/,
  );
  fs.mkdirSync(consumer);
  assert.throws(() => consumerPaths(paths, checkout), /fresh public paths/);
});

await test("the actual observer refuses an untrusted branch or unsupported host before product execution", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "typed-untrusted-observer-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const f = officialFixture();
  const env = {
    ...process.env,
    PUBLIC_CUT: f.options.cut,
    PUBLIC_HEAD: f.options.head,
    PUBLIC_TAG: f.options.tag,
    PUBLIC_SOURCE_PR: f.options.sourcePr,
    PUBLIC_RELEASE_RUN: f.options.run,
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_REF: "refs/heads/untrusted",
    RUNNER_TEMP: root,
    PUB_CONSUMER: path.join(root, "consumer"),
    PUBLIC_OUTPUT: path.join(root, "evidence"),
  };
  for (const key of overrides) delete env[key];
  const run = spawnSync(
    process.execPath,
    [
      fileURLToPath(
        new URL("../../tools/support/release/typed_defaults/main.mjs", import.meta.url),
      ),
    ],
    { env, encoding: "utf8", timeout: 10_000 },
  );
  assert.equal(run.error, undefined);
  assert.equal(run.status, 1);
  assert.match(run.stderr, /reviewed main observer required|actual hosted GNU consumer required/);
  assert.equal(run.stdout, "");
  assert.equal(fs.existsSync(env.PUB_CONSUMER), false);
  assert.equal(fs.existsSync(env.PUBLIC_OUTPUT), false);
});

function archiveFixture(t, kind = "regular") {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "typed-public-archive-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const archive = path.join(root, archiveName);
  const elf = Buffer.alloc(128);
  elf.set([0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01], 0);
  elf.set([0x3e, 0x00], 18);
  const payload =
    kind === "fake"
      ? Buffer.concat([Buffer.from("#!/bin/sh\necho fake\n"), Buffer.alloc(100, "x")])
      : elf;
  const members =
    kind === "symlink"
      ? [{ name: "vize", mode: 0o755, typeflag: "2", linkname: "/outside/foreign" }]
      : [{ name: "vize", mode: 0o755, data: payload }];
  if (kind === "extra") members.push({ name: "extra", data: Buffer.from("x") });
  fs.writeFileSync(archive, writeTarGz(members));
  const bytes = fs.readFileSync(archive),
    checksum = Buffer.from(`${sha256(bytes)}  ${archiveName}\n`);
  const tag = "v0.445.0";
  const assets = [archiveName, `${archiveName}.sha256`].map((name, index) => ({
    name,
    state: "uploaded",
    browser_download_url: `https://github.com/ubugeeei-prod/vize/releases/download/${tag}/${name}`,
    digest: `sha256:${sha256(index ? checksum : bytes)}`,
    size: index ? checksum.length : bytes.length,
  }));
  const extract = (destination) =>
    spawnSync(
      process.execPath,
      [
        fileURLToPath(
          new URL("../../tools/support/release/typed_defaults/archive.ts", import.meta.url),
        ),
        archive,
        destination,
      ],
      { encoding: "utf8" },
    );
  return { root, archive, bytes, checksum, tag, assets, extract };
}

await test("single official GNU archive, both digests and exact sidecar are required; repacks fail", (t) => {
  const f = archiveFixture(t);
  const assets = selectAssets({ assets: f.assets }, f.tag);
  verifyAssets(assets, f.bytes, f.checksum);
  const repack = Buffer.from(f.bytes);
  repack[10] ^= 1;
  assert.throws(() => verifyAssets(assets, repack, f.checksum), /archive digest mismatch/);
  assert.throws(
    () => selectAssets({ assets: [...f.assets, f.assets[0]] }, f.tag),
    /exactly one official/,
  );
  const foreign = structuredClone(f.assets);
  foreign[0].browser_download_url = "https://foreign.invalid/archive";
  assert.throws(() => selectAssets({ assets: foreign }, f.tag));
  const wrong = Buffer.from(f.checksum.toString().replace(archiveName, "foreign-file"));
  const wrongAssets = structuredClone(f.assets);
  wrongAssets[1].size = wrong.length;
  wrongAssets[1].digest = `sha256:${sha256(wrong)}`;
  assert.throws(() => verifyAssets(wrongAssets, f.bytes, wrong), /exact archive checksum/);
});

await test("inert ELF transport controls retain regular bytes; scripts, symlinks, extra members and foreign binary refuse", (t) => {
  const f = archiveFixture(t);
  const consumer = path.join(f.root, "consumer");
  fs.mkdirSync(consumer);
  const result = f.extract(path.join(consumer, "distribution"));
  assert.equal(result.status, 0, result.stderr);
  const member = JSON.parse(result.stdout);
  assertLoadedBinary(member.binary, member, consumer);
  const foreign = path.join(f.root, "foreign");
  fs.copyFileSync(member.binary, foreign);
  assert.throws(() => assertLoadedBinary(foreign, member, consumer), /foreign loaded binary/);
  const changed = fs.readFileSync(member.binary);
  changed[80] ^= 1;
  fs.writeFileSync(member.binary, changed);
  assert.throws(() => assertLoadedBinary(member.binary, member, consumer), /identity changed/);
  fs.unlinkSync(member.binary);
  fs.symlinkSync(foreign, member.binary);
  assert.throws(() => assertLoadedBinary(member.binary, member, consumer), /foreign loaded binary/);
  for (const kind of ["fake", "symlink", "extra"]) {
    const invalid = archiveFixture(t, kind);
    assert.notEqual(invalid.extract(path.join(invalid.root, "rejected")).status, 0, kind);
  }
});
