import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  emptyVueStarBodyPairs,
  emptyVueStarMetadataPairs,
} from "../performance/support/warm-type-backed-empty-vue-star-host.ts";
import { ensureHostedLiteralSource } from "../performance/support/warm-type-backed-git-reader-literal-source.ts";
import {
  loadComponentValueManifest,
  componentValueRetainedGuardPairs,
} from "../performance/support/warm-type-backed-component-value-host.ts";

import { retainedHostBeforeEmptyVueStars } from "./support/empty-vue-star-host-custody.ts";
import { retainedPhysicalHostBeforeComponentValues } from "./support/component-value-host-custody.ts";
import {
  authenticateNoMutatingComponentJoinPhysical,
  authenticateNoMutatingComponentJoinCommitted,
} from "./support/no-mutating-component-join.mjs";
import { authenticateJoinLiteralFrame } from "./support/no-mutating-component-join-frame.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const host = "tests/performance/support/warm-type-backed-host-qualifications.ts";
const adapter = "tests/tooling/support/empty-vue-star-host-custody.ts";
const originalSHA = "b19f6783e50429d2779efeb7d2d21c64dcd98a67810fc4f6a1ccc698febae337";
const source = "a93e78147b0990c813e965a7868b32dda341eb77";
const sha = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const git = (cwd: string, args: string[]) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd });

// These are literal source fixtures, never compiled/native workload receipts.
function fixture() {
  ensureHostedLiteralSource(root, source);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-empty-star-custody-"));
  const copied = new Set<string>();
  const copy = (file: string) => {
    if (copied.has(file)) return;
    copied.add(file);
    const target = path.join(directory, file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(root, file), target);
    fs.chmodSync(target, 0o644);
    if (file.endsWith(".ts")) {
      const body = fs.readFileSync(target, "utf8");
      for (const match of body.matchAll(/^import[^;]*?from\s+["'](\.[^"']+)["'];/gmu))
        copy(path.relative(directory, path.resolve(path.dirname(target), match[1])));
    }
  };
  for (const [file] of [...emptyVueStarBodyPairs, ...emptyVueStarMetadataPairs]) copy(file);
  for (const file of [
    host,
    adapter,
    "tests/performance/support/warm-type-backed-empty-vue-star-host.ts",
    "tests/tooling/warm-type-backed-empty-vue-star-host.test.ts",
  ])
    copy(file);
  const component = loadComponentValueManifest((file) => fs.readFileSync(path.join(root, file)));
  for (const [file] of [
    ...component.files,
    ...component.supplementalBodies,
    ...component.preservedBodies,
    ...componentValueRetainedGuardPairs,
  ])
    copy(file);
  const capsules = "tests/_fixtures/differential/linter/component-import-value-host-before";
  for (const name of fs.readdirSync(path.join(root, capsules))) copy(capsules + "/" + name);
  copy("tests/tooling/support/component-value-host-custody.ts");
  for (const name of fs.readdirSync(path.join(root, "tests/performance/support")))
    if (name.startsWith("warm-type-backed-git-reader-") && name.endsWith(".json"))
      copy("tests/performance/support/" + name);
  git(directory, ["init", "--quiet"]);
  const common = fs.realpathSync(
    path.resolve(root, git(root, ["rev-parse", "--git-common-dir"]).toString().trim()),
  );
  fs.writeFileSync(path.join(directory, ".git/objects/info/alternates"), common + "/objects\n");
  return { directory, target: (file: string) => path.join(directory, file) };
}
function observe(directory: string, supplied: string = "current") {
  const program = `
    import fs from 'node:fs';
    import { createHash } from 'node:crypto';
    import { execFileSync } from 'node:child_process';
    import { retainedHostBeforeEmptyVueStars } from './${adapter}';
    const supplied = ${JSON.stringify(supplied)};
    const original = supplied === 'original'
      ? execFileSync('git', ['--no-replace-objects', 'show', '${source}:${host}']) : null;
    const read = file => file !== '${host}' || supplied === 'current'
      ? fs.readFileSync(file) : supplied === 'original' ? original : Buffer.from(supplied);
    try {
      const body = retainedHostBeforeEmptyVueStars(read);
      process.stdout.write(createHash('sha256').update(body).digest('hex'));
    } catch (error) { process.stderr.write(String(error)); process.exitCode = 1; }
  `;
  return spawnSync(process.execPath, ["--input-type=module", "-e", program], {
    cwd: directory,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
    encoding: "utf8",
  });
}
function withFixture(run: (value: ReturnType<typeof fixture>) => void) {
  const value = fixture();
  try {
    run(value);
  } finally {
    fs.rmSync(value.directory, { recursive: true, force: true });
  }
}
function refuses(directory: string, supplied?: string) {
  const result = observe(directory, supplied);
  assert.equal(result.status, 1, result.stderr);
  assert.equal(result.stdout, "");
  assert.match(result.stderr, /AssertionError|ENOENT|ERR_MODULE_NOT_FOUND/u);
}

test("all actual281 source bodies and nine metadata bodies precede the exact historical hook", () => {
  withFixture(({ directory }) => {
    for (const supplied of ["current", "original"]) {
      const result = observe(directory, supplied);
      assert.equal(result.status, 0, result.stderr);
      assert.equal(result.stdout, originalSHA);
      assert.equal(result.stderr, "");
    }
  });
});

void test("complete current290 Star and current323 NoMutating bodies precede the genuine historical NoUnused source check", () => {
  const currentHead = git(root, ["rev-parse", "HEAD"]).toString().trim();
  const retainedHook = retainedHostBeforeEmptyVueStars((file) =>
    fs.readFileSync(path.join(root, file)),
  );
  assert.equal(sha(retainedHook), originalSHA);
  const current = authenticateNoMutatingComponentJoinPhysical(root);
  assert.equal(current.completeCurrentClosedBodies, 323);
  assert.equal(authenticateNoMutatingComponentJoinCommitted(root), currentHead);
  const declared = current.manifest.literalFrames.filter(
    (row) => row.source === "f20740ef6b0958ddb0bbbd64fd1b38210ea5e207",
  );
  assert.equal(declared.length, 1);
  const historical = authenticateJoinLiteralFrame(root, declared[0]);
  assert.equal(historical.frame.files.length, 683);
  assert.deepEqual(retainedHook, fs.readFileSync(path.join(historical.root, host)));
  const retained = retainedPhysicalHostBeforeComponentValues(historical.root);
  assert.equal(sha(retained), "fc37f5b9a81aa2245a92013adac0cad40aee147d51f66f0c96efc267503cc16f");
  assert.equal(current.sourceOnly, true);
  assert.equal(current.compiled, false);
  assert.equal(current.native, false);
  assert.equal(current.performance, false);
  assert.equal(current.currentOld111Receipt, null);
  assert.equal(current.oldMigrationMeasurement, null);
  assert.equal(current.oldSetupMeasurement, null);
  assert.equal(git(root, ["rev-parse", "HEAD"]).toString().trim(), currentHead);
  assert.equal(authenticateNoMutatingComponentJoinCommitted(root), currentHead);
  assert.deepEqual(
    retainedHostBeforeEmptyVueStars((file) => fs.readFileSync(path.join(root, file))),
    retainedHook,
  );
});

test("foreign and one-clause hybrid supplied hooks refuse before historical projection", () => {
  withFixture(({ directory, target }) => {
    refuses(directory, "foreign source");
    const current = fs.readFileSync(target(host), "utf8");
    refuses(
      directory,
      current.replace(
        'import { qualifyEmptyVueStarHost } from "./warm-type-backed-empty-vue-star-host.ts";\n\n',
        "",
      ),
    );
  });
});

test("physical production, supplemental input and preserved original bodies remain mandatory", () => {
  withFixture(({ directory, target }) => {
    for (const file of [
      "crates/vize_canon/src/batch/virtual_project/star_exports.rs",
      "tests/_fixtures/differential/typechecker/empty-vue-star-exports/supplement.json",
      "tests/_fixtures/differential/typechecker/manifest.json",
    ]) {
      const original = fs.readFileSync(target(file));
      fs.appendFileSync(target(file), "\n");
      refuses(directory);
      fs.writeFileSync(target(file), original);
    }
  });
});

test("missing or changed whole metadata and externally pinned helper bodies refuse", () => {
  withFixture(({ directory, target }) => {
    for (const file of [
      ...emptyVueStarMetadataPairs.map(([file]) => file),
      "tests/performance/support/warm-type-backed-empty-vue-star-host.ts",
      "tests/tooling/warm-type-backed-empty-vue-star-host.test.ts",
    ]) {
      const original = fs.readFileSync(target(file));
      fs.unlinkSync(target(file));
      refuses(directory);
      fs.writeFileSync(target(file), Buffer.concat([original, Buffer.from("\n")]));
      refuses(directory);
      fs.writeFileSync(target(file), original);
    }
  });
});

test("executable and symlink copies cannot stand in for actual regular source bodies", () => {
  withFixture(({ directory, target }) => {
    const file = target("crates/vize_canon/src/batch/virtual_project/star_exports.rs");
    fs.chmodSync(file, 0o755);
    refuses(directory);
    fs.chmodSync(file, 0o644);
    fs.renameSync(file, file + ".copy");
    fs.symlinkSync(file + ".copy", file);
    refuses(directory);
  });
});

test("replacement refs and missing predecessor objects refuse without fetching private history", () => {
  withFixture(({ directory, target }) => {
    git(directory, ["replace", source, source]);
    refuses(directory);
    git(directory, ["replace", "--delete", source]);
    fs.unlinkSync(target(".git/objects/info/alternates"));
    const result = observe(directory);
    assert.equal(result.status, 1, result.stderr);
    assert.equal(result.stdout, "");
    assert.match(result.stderr, /AssertionError/u);
    assert.equal(git(directory, ["remote"]).length, 0);
  });
});

test("a whole old hook substituted into the current source root is a refused hybrid", () => {
  withFixture(({ directory, target }) => {
    const original = git(directory, ["show", source + ":" + host]);
    assert.equal(sha(original), originalSHA);
    fs.writeFileSync(target(host), original);
    refuses(directory);
    refuses(directory, "foreign original hook");
  });
});
