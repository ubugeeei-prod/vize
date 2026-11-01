/** Closed, frozen source custody for the two separate candidate cohorts. */
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { verifySourceSdkPair, type SdkScanner } from "./cold-typechecker-scale-strict-pair.ts";

export const componentChecksPath =
  "crates/vize_canon/src/virtual_ts/helpers/declarations/component-checks.d.ts.txt";
export const historicalSourceFix = "0eb2f8db16f6541cd422ee610ab3e554bf041e0d";
const productPaths = {
  "type-work": [componentChecksPath],
  "route-move": [
    "crates/vize/src/commands/check/runner.rs",
    "crates/vize/src/commands/check/runner/execution.rs",
  ],
} as const;
const typeWorkControls = [
  "crates/vize_canon/src/batch/virtual_project/snapshots/vize_canon__batch__virtual_project__tests__pruned_helpers.snap",
  "crates/vize_canon/tests/empty_vue_star_exports/membership/constructor_without_args/payload_update.rs",
  "crates/vize_canon/tests/empty_vue_star_exports/membership/constructor_without_args/payload_update/instance_props.rs",
  "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/instance-prop-metadata-update.json",
  "tests/tooling/canon-instance-prop-ambient.test.ts",
  "tests/tooling/canon-instance-prop-metadata.test.ts",
  "tests/tooling/fixtures/canon-instance-prop-metadata-before.json",
];
type Kind = keyof typeof productPaths;
type Role = "base" | "head";
export interface BodyPin {
  bytes: number;
  sha256: string;
  gitBlob: string;
}
export interface CandidateSources {
  kind: Kind;
  base: { sha: string; tree: string };
  head: { sha: string; tree: string };
  /** Genuine actual-main commits, not proposed PR heads or source test merges. */
  containing: { optionsFix: string; historicalSourceFix: string };
  changed: { path: string; base: BodyPin | null; head: BodyPin | null }[];
  cargo: { file: "Cargo.toml" | "Cargo.lock"; pin: BodyPin }[];
}
type SourceInput = {
  base: { sha: string; tree: string };
  head: { sha: string; tree: string };
  changedPaths: string[];
  bodies: { base: (Buffer | null)[]; head: (Buffer | null)[] };
  cargo: { base: Buffer[]; head: Buffer[] };
  ancestry: { base: boolean[]; head: boolean[] };
};
const digest = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const oid = (value: string) => assert.match(value, /^[a-f0-9]{40}$/u);
const byteOrder = (left: string, right: string) =>
  Buffer.compare(Buffer.from(left), Buffer.from(right));
const pinBody = (body: Buffer | null, pin: BodyPin | null) => {
  if (pin === null) return assert.equal(body, null, "unbound source addition/deletion");
  assert(Buffer.isBuffer(body), "required whole source body missing");
  assert(Number.isSafeInteger(pin.bytes) && pin.bytes >= 0);
  assert.equal(body.length, pin.bytes);
  assert.match(pin.sha256, /^[a-f0-9]{64}$/u);
  assert.equal(digest(body), pin.sha256);
  oid(pin.gitBlob);
  assert.equal(
    createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
    pin.gitBlob,
    "whole source Git blob mismatch",
  );
};

/** Pure checks; the caller supplies the separately frozen, reviewed literal source manifest. */
export function verifyCandidateSourceInputs(frozen: CandidateSources, actual: SourceInput) {
  assert(frozen.kind === "type-work" || frozen.kind === "route-move");
  for (const role of ["base", "head"] as const) {
    oid(frozen[role].sha);
    oid(frozen[role].tree);
    assert.deepEqual(actual[role], frozen[role]);
    assert.deepEqual(actual.ancestry[role], [true, true], "both actual fixes must be ancestors");
    assert.equal(actual.bodies[role].length, frozen.changed.length);
    frozen.changed.forEach((row, index) => pinBody(actual.bodies[role][index], row[role]));
  }
  assert.notEqual(frozen.base.sha, frozen.head.sha);
  oid(frozen.containing.optionsFix);
  oid(frozen.containing.historicalSourceFix);
  assert.equal(frozen.containing.historicalSourceFix, historicalSourceFix);
  assert.notEqual(frozen.containing.optionsFix, frozen.containing.historicalSourceFix);
  assert(frozen.changed.length > 0);
  const names = frozen.changed.map((row) => {
    assert(!path.posix.isAbsolute(row.path) && !row.path.includes("\\"));
    assert.equal(path.posix.normalize(row.path), row.path);
    assert(!row.path.startsWith("../") && row.path !== ".." && !row.path.endsWith("/"));
    assert(row.base !== null || row.head !== null);
    assert.notDeepEqual(row.base, row.head, "unchanged paths are not an owned source delta");
    return row.path;
  });
  assert.deepEqual(names, [...new Set(names)].sort(byteOrder), "closed ordered unique path roster");
  assert.deepEqual(actual.changedPaths, names, "unrelated or omitted source change");
  assert.deepEqual(
    names,
    [...productPaths[frozen.kind], ...(frozen.kind === "type-work" ? typeWorkControls : [])].sort(
      byteOrder,
    ),
    "candidate effect and source controls must stay in the closed owned roster",
  );
  for (const name of productPaths[frozen.kind]) {
    const row = frozen.changed.find((entry) => entry.path === name)!;
    assert(
      row.base !== null && row.head !== null,
      "the candidate must modify existing product bodies",
    );
  }
  assert.deepEqual(
    frozen.cargo.map((row) => row.file),
    ["Cargo.toml", "Cargo.lock"],
  );
  for (const role of ["base", "head"] as const) {
    assert.equal(actual.cargo[role].length, 2);
    frozen.cargo.forEach((row, index) => pinBody(actual.cargo[role][index], row.pin));
  }
  assert(actual.cargo.base.every((body, index) => body.equals(actual.cargo.head[index])));
  return {
    sources: { base: frozen.base, head: frozen.head },
    containing: frozen.containing,
    changedPaths: names,
    cargo: frozen.cargo,
    scope:
      "whole frozen source bodies and Cargo identity; not loaded provider files or runtime qualification",
  };
}

/** Hosted source check, before building either normal image or any observation overlay. */
export function verifyCandidateSources(
  roots: Record<Role, string>,
  provider: string,
  frozen: CandidateSources,
) {
  const git = (root: string, args: string[]) =>
    execFileSync("git", args, { cwd: root, maxBuffer: 8 * 1024 * 1024 })
      .toString()
      .trim();
  const body = (root: string, name: string, pin: BodyPin | null) => {
    const file = path.join(root, name);
    if (pin === null) {
      assert(!fs.existsSync(file));
      return null;
    }
    assert(fs.lstatSync(file).isFile() && !fs.lstatSync(file).isSymbolicLink());
    return fs.readFileSync(file);
  };
  const actual = {} as Pick<SourceInput, "base" | "head">;
  const bodies = { base: [], head: [] } as SourceInput["bodies"];
  const cargo = { base: [], head: [] } as SourceInput["cargo"];
  const ancestry = { base: [], head: [] } as SourceInput["ancestry"];
  for (const role of ["base", "head"] as const) {
    const root = roots[role];
    assert(path.isAbsolute(root));
    assert.equal(git(root, ["status", "--porcelain", "--untracked-files=all"]), "");
    actual[role] = {
      sha: git(root, ["rev-parse", "HEAD"]),
      tree: git(root, ["rev-parse", "HEAD^{tree}"]),
    };
    bodies[role] = frozen.changed.map((row) => body(root, row.path, row[role]));
    cargo[role] = frozen.cargo.map((row) => fs.readFileSync(path.join(root, row.file)));
    ancestry[role] = Object.values(frozen.containing).map((commit) => {
      oid(commit);
      // Missing history is a refusal, not permission to infer ancestry or fetch in a unit test.
      git(root, ["merge-base", "--is-ancestor", commit, actual[role].sha]);
      return true;
    });
  }
  const changedPaths = git(roots.base, [
    "diff",
    "--name-only",
    frozen.base.sha,
    frozen.head.sha,
  ]).split("\n");
  const source = verifyCandidateSourceInputs(frozen, {
    ...actual,
    changedPaths,
    bodies,
    cargo,
    ancestry,
  });
  const require = createRequire(path.join(provider, "package.json"));
  const sdk = verifySourceSdkPair(roots.base, roots.head, require("typescript") as SdkScanner);
  return {
    ...source,
    sdk,
    assets:
      frozen.kind === "type-work"
        ? {
            before:
              bodies.base[frozen.changed.findIndex((row) => row.path === componentChecksPath)]!,
            after:
              bodies.head[frozen.changed.findIndex((row) => row.path === componentChecksPath)]!,
          }
        : null,
  };
}
