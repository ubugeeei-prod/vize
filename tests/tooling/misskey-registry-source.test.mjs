import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {
  authenticateMisskeyRegistrySource,
  misskeyRegistryCounterpart,
} from "../../tools/support/compat/github/misskey-registry-source.mjs";

const root = fs.realpathSync(new URL("../../", import.meta.url));
const metadataPath = "tools/support/compat/github/misskey-registry-source.json";
const metadata = JSON.parse(fs.readFileSync(path.join(root, metadataPath)));
const rows = metadata.parts.flatMap(([file]) => JSON.parse(fs.readFileSync(path.join(root, file))));
const files = [
  ...rows.map(([file]) => file),
  metadataPath,
  ...metadata.parts.map(([file]) => file),
  ...metadata.executing,
];
const registry = "tests/_fixtures/vue-ecosystem-fixtures.json";
const law = "tests/tooling/empty-vue-star-javascript-source.test.mjs";
const git = (owner, args, input) =>
  execFileSync("git", ["--no-replace-objects", ...args], {
    cwd: owner,
    input,
    maxBuffer: 64 * 1024 * 1024,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_ALLOW_PROTOCOL: "" },
  });
function fixture(run) {
  const owner = fs.mkdtempSync(path.join(os.tmpdir(), "vize-misskey-registry-refusals-"));
  try {
    git(owner, ["init", "--quiet"]);
    const common = git(root, ["rev-parse", "--git-common-dir"]).toString().trim();
    fs.writeFileSync(
      path.join(owner, ".git/objects/info/alternates"),
      fs.realpathSync(path.resolve(root, common, "objects")) + "\n",
    );
    const head = git(root, ["rev-parse", "HEAD"]).toString().trim();
    git(owner, ["read-tree", head]);
    git(owner, ["update-ref", "refs/heads/current", head]);
    git(owner, ["symbolic-ref", "HEAD", "refs/heads/current"]);
    for (const file of files) {
      const target = path.join(owner, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.copyFileSync(path.join(root, file), target);
      fs.chmodSync(target, 0o644);
    }
    run(owner, head);
  } finally {
    fs.rmSync(owner, { recursive: true, force: true });
  }
}
function changed(owner, head, file, body, mode = "100644") {
  git(owner, ["read-tree", head]);
  const blob = git(owner, ["hash-object", "-w", "--stdin"], body).toString().trim();
  git(owner, ["update-index", "--cacheinfo", mode + "," + blob + "," + file]);
  const tree = git(owner, ["write-tree"]).toString().trim();
  const next = git(
    owner,
    [
      "-c",
      "user.name=Registry refusal",
      "-c",
      "user.email=source@invalid.example",
      "commit-tree",
      tree,
      "-p",
      head,
    ],
    "private refused registry source\n",
  )
    .toString()
    .trim();
  git(owner, ["update-ref", "refs/heads/current", next]);
  fs.writeFileSync(path.join(owner, file), body);
  fs.chmodSync(path.join(owner, file), Number.parseInt(mode, 8) & 0o777);
  return next;
}

await test("complete actual registry source precedes unchanged literal Star322/290 and JS93 laws", () => {
  const packet = authenticateMisskeyRegistrySource(root);
  assert.equal(packet.originalStarBodies, 322);
  assert.equal(packet.originalJSBodies, 93);
  assert.equal(packet.runtimeCredit, false);
  assert.equal(packet.original400Credit, false);
  assert.equal(packet.performanceCredit, false);
  assert.equal(rows.length, 412);
  assert.deepEqual(
    rows
      .filter((row) => row[1] !== row[4])
      .map(([file]) => file)
      .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0)),
    [...metadata.changed].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0)),
  );
  const counterpart = misskeyRegistryCounterpart(packet, law);
  try {
    for (const file of [
      law,
      registry,
      "tools/support/compat/github/empty-vue-star-javascript-source.mjs",
      "tools/support/compat/github/empty-vue-star-custody-runner.mjs",
    ])
      assert.deepEqual(
        fs.readFileSync(path.join(counterpart.root, file)),
        git(root, ["show", metadata.base + ":" + file]),
        "whole original source: " + file,
      );
    assert.equal(git(counterpart.root, ["rev-parse", "HEAD"]).toString().trim(), metadata.base);
  } finally {
    counterpart.cleanup();
  }
});

await test("missing, mutated or executable physical bodies and source ancestor symlinks refuse", () => {
  fixture((owner) => {
    for (const file of [
      registry,
      law,
      metadataPath,
      metadata.executing[0],
      metadata.executing[1],
      metadata.parts[0][0],
      rows[0][0],
      "tools/support/compat/github/empty-vue-star-javascript-source.mjs",
    ]) {
      const target = path.join(owner, file),
        body = fs.readFileSync(target);
      fs.unlinkSync(target);
      assert.throws(() => authenticateMisskeyRegistrySource(owner));
      fs.writeFileSync(target, Buffer.concat([body, Buffer.from("foreign source\n")]));
      assert.throws(() => authenticateMisskeyRegistrySource(owner));
      fs.writeFileSync(target, body);
      fs.chmodSync(target, 0o755);
      assert.throws(() => authenticateMisskeyRegistrySource(owner));
      fs.chmodSync(target, 0o644);
    }
    const directory = path.join(owner, "tools/support/compat/github"),
      held = directory + "-held";
    fs.renameSync(directory, held);
    fs.symlinkSync(held, directory);
    assert.throws(() => authenticateMisskeyRegistrySource(owner));
    fs.unlinkSync(directory);
    fs.renameSync(held, directory);
    assert(authenticateMisskeyRegistrySource(owner));
  });
});

await test("raw and physical matching foreign registry, predecessor hybrids and neighbor budget changes refuse", () => {
  fixture((owner, head) => {
    const original = fs.readFileSync(path.join(owner, registry));
    const neighboring = JSON.parse(original);
    neighboring.projects[0].typecheckPerformance.hangTimeoutMs++;
    for (const body of [
      Buffer.concat([original, Buffer.from("\n")]),
      git(root, ["show", metadata.base + ":" + registry]),
      Buffer.from(JSON.stringify(neighboring, null, 2) + "\n"),
    ]) {
      changed(owner, head, registry, body);
      assert.throws(() => authenticateMisskeyRegistrySource(owner));
    }
    changed(owner, head, registry, original, "100755");
    assert.throws(() => authenticateMisskeyRegistrySource(owner));
    git(owner, ["update-ref", "refs/heads/current", head]);
    fs.writeFileSync(path.join(owner, registry), original);
    fs.chmodSync(path.join(owner, registry), 0o644);
    const oldLaw = git(root, ["show", metadata.base + ":" + law]);
    changed(owner, head, law, oldLaw);
    assert.throws(() => authenticateMisskeyRegistrySource(owner));
  });
});

await test("forged, unknown, stale or same-HEAD mutated source packets cannot replay a historical law", () => {
  fixture((owner, head) => {
    const packet = authenticateMisskeyRegistrySource(owner);
    assert.equal(Reflect.set(packet, "runtimeCredit", true), false);
    assert.throws(() => misskeyRegistryCounterpart({ ...packet }, law));
    assert.throws(() => misskeyRegistryCounterpart(packet, "tests/tooling/unreviewed.test.mjs"));
    const target = path.join(owner, registry),
      body = fs.readFileSync(target);
    fs.writeFileSync(target, Buffer.concat([body, Buffer.from("\n")]));
    assert.throws(() => misskeyRegistryCounterpart(packet, law));
    fs.writeFileSync(target, body);
    const next = changed(owner, head, registry, body);
    assert.throws(() => misskeyRegistryCounterpart(packet, law));
    git(owner, ["update-ref", "refs/heads/current", head]);
    git(owner, ["update-ref", "refs/replace/" + head, next]);
    assert.throws(() => authenticateMisskeyRegistrySource(owner));
    assert.throws(() => misskeyRegistryCounterpart(packet, law));
    git(owner, ["update-ref", "-d", "refs/replace/" + head]);
  });
});

await test("ambient private source owners and overlays refuse before historical replay", () => {
  for (const key of ["STAR_JS_SOURCE_ROOT", "JS_STAR_SOURCE_INDEX"]) {
    const old = process.env[key];
    process.env[key] = root;
    try {
      assert.throws(() => authenticateMisskeyRegistrySource(root));
    } finally {
      if (old === undefined) delete process.env[key];
      else process.env[key] = old;
    }
  }
});
