/** Whole historical Git source custody with an explicit finite compiler-role partition. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

// These exact reviewed tool sources are not primary compiler source inputs.
// Any role change requires a new finite source review, not a rehashed packet.
export const SOURCE_ONLY_RUST_NAMES = [
  "tools/commands/ci/github/release-preflight.rs",
  "tools/commands/ci/source-file-lengths.rs",
  "tools/commands/fixtures/lint-divergence-report.rs",
  "tools/commands/release/pr.rs",
  "tools/support/lint_divergence_surface.rs",
  "tools/support/release/pr_ci.rs",
  "tools/support/release/pr_pin.rs",
  "tools/support/release/pr_pin_metadata.rs",
  "tools/support/release/pr_pin_retire.rs",
  "tools/support/release/pr_pin_retire_archive.rs",
  "tools/support/release/pr_pin_retire_evidence.rs",
  "tools/support/release/pr_pin_retire_guard.rs",
  "tools/support/release/pr_pin_retire_ledger.rs",
  "tools/support/release/pr_pin_retire_lifecycle_tests.rs",
  "tools/support/release/pr_pin_retire_prequalification_archive_tests.rs",
  "tools/support/release/pr_pin_retire_source.rs",
  "tools/support/release/pr_pin_runs.rs",
  "tools/support/release/pr_pin_target_tests.rs",
  "tools/support/release/pr_pin_tests.rs",
  "tools/support/release/pr_watch.rs",
  "tools/support/release/preflight_pinned_evidence_tests.rs",
] as const;

export interface Extension {
  schema: string;
  beforeSha256: string;
  afterSha256: string;
  hunks: { before: string; after: string }[];
  producer: {
    head: string;
    files: Record<string, string | null>;
    absentFiles: string[];
    retiredRust: { path: string; sha256: string };
    sourceOnlyRust: { path: string; sha256: string };
    composedBase: string;
    historicalRenames: { from: string }[];
  };
}
interface Retirement {
  schema: string;
  source: string;
  retiredAt: string;
  files: Record<string, string>;
}
interface SourceIdentity {
  mode: string;
  blob: string;
  sha256: string;
}
export interface SourceOnlyRust {
  schema: string;
  producer: string;
  priorSource: string;
  main: string;
  files: Record<
    string,
    {
      compilerBound: false;
      prior: SourceIdentity | null;
      current: SourceIdentity | null;
      main: SourceIdentity | null;
      retiredAt: string | null;
    }
  >;
}
export function readPackedProducerRetirement(
  repository: string,
  producer: Extension["producer"],
): Retirement {
  const bytes = fs.readFileSync(path.join(repository, producer.retiredRust.path));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), producer.retiredRust.sha256);
  const retirement = JSON.parse(bytes.toString()) as Retirement;
  assert.equal(retirement.schema, "vize.packed-config-retired-rust.v1");
  return retirement;
}

/** Exact finite tooling roles are source custody, never compiler-source claims. */
export function verifySourceOnlyRust(
  repository: string,
  producer: Extension["producer"],
  selected: string,
): {
  sourceOnlyFiles: Record<string, string | null>;
  compilerFiles: Record<string, string | null>;
} {
  const bytes = fs.readFileSync(path.join(repository, producer.sourceOnlyRust.path));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), producer.sourceOnlyRust.sha256);
  const roles = JSON.parse(bytes.toString()) as SourceOnlyRust;
  assert.equal(roles.schema, "vize.packed-config-source-only-rust.v1");
  assert.equal(roles.producer, producer.head);
  for (const ref of [roles.producer, roles.priorSource, roles.main, selected])
    assert.match(ref, /^[0-9a-f]{40}$/u);
  const names = Object.keys(roles.files);
  assert.deepEqual(
    [...names].sort(),
    [...SOURCE_ONLY_RUST_NAMES],
    "exact independently reviewed source-only roles",
  );
  const git = (...args: string[]) => execFileSync("git", args, { cwd: repository });
  for (const ancestor of [roles.priorSource, roles.main])
    git("merge-base", "--is-ancestor", ancestor, producer.head);
  const trees = new Map<string, Map<string, { mode: string; blob: string }>>();
  const hashes = new Map<string, string>();
  const identity = (ref: string, file: string): SourceIdentity | null => {
    if (!trees.has(ref)) {
      const tree = new Map<string, { mode: string; blob: string }>();
      for (const entry of git("ls-tree", "-rz", ref, "--", ...names)
        .toString()
        .split("\0")
        .filter(Boolean)) {
        const match = /^(\d+) blob ([0-9a-f]+)\t([\s\S]+)$/u.exec(entry);
        assert.ok(match);
        tree.set(match[3], { mode: match[1], blob: match[2] });
      }
      trees.set(ref, tree);
    }
    const entry = trees.get(ref)?.get(file);
    if (!entry) return null;
    if (!hashes.has(entry.blob))
      hashes.set(
        entry.blob,
        createHash("sha256")
          .update(git("cat-file", "blob", entry.blob))
          .digest("hex"),
      );
    const sha256 = hashes.get(entry.blob);
    assert.ok(sha256);
    return { ...entry, sha256 };
  };
  for (const [file, row] of Object.entries(roles.files)) {
    assert.equal(row.compilerBound, false, file);
    assert.deepEqual(identity(roles.priorSource, file), row.prior, file);
    assert.deepEqual(identity(roles.main, file), row.main, file);
    assert.deepEqual(identity(producer.head, file), row.current, file);
    assert.deepEqual(identity(selected, file), row.current, file);
    const physical = fs.lstatSync(path.join(repository, file), { throwIfNoEntry: false });
    if (row.current === null) assert.equal(physical, undefined, file);
    else {
      assert.ok(physical?.isFile(), file);
      assert.equal(
        createHash("sha256")
          .update(fs.readFileSync(path.join(repository, file)))
          .digest("hex"),
        row.current.sha256,
        file,
      );
    }
    if (row.retiredAt !== null) {
      assert.match(row.retiredAt, /^[0-9a-f]{40}$/u);
      git("merge-base", "--is-ancestor", row.retiredAt, roles.main);
      assert.equal(identity(row.retiredAt, file), null, file);
      assert.notEqual(row.prior, null, file);
      assert.equal(row.current, null, file);
    }
  }
  const sourceOnlyFiles = Object.fromEntries(
    names.map((file) => [file, roles.files[file].current?.sha256 ?? null]),
  );
  const compilerFiles = Object.fromEntries(
    Object.entries(producer.files).filter(([file]) => !names.includes(file)),
  );
  const renamed = producer.historicalRenames.map((row) => row.from);
  const compiledRust = Object.keys(compilerFiles).filter(
    (file) => file.endsWith(".rs") && !renamed.includes(file),
  );
  const changedRust = git(
    "diff",
    "--name-only",
    "--no-renames",
    producer.composedBase,
    producer.head,
    "--",
    "*.rs",
  )
    .toString()
    .trim()
    .split("\n")
    .filter(Boolean);
  assert.deepEqual(
    [...compiledRust, ...names].sort(),
    changedRust.sort(),
    "every Rust delta has exactly one finite role",
  );
  git("diff", "--exit-code", producer.head, selected, "--", "*.rs");
  return { sourceOnlyFiles, compilerFiles };
}

/** Historical custody survives later product changes; current equality belongs to the selected preflight. */
export function verifyHistoricalProducerFiles(
  repository: string,
  producer: Extension["producer"],
): void {
  const git = (...args: string[]) => execFileSync("git", args, { cwd: repository });
  const retirement = readPackedProducerRetirement(repository, producer);
  const absentFiles = [...producer.absentFiles, ...Object.keys(retirement.files)];
  assert.equal(new Set(absentFiles).size, absentFiles.length);
  assert.deepEqual(
    [...absentFiles].sort(),
    Object.keys(producer.files)
      .filter((file) => producer.files[file] === null)
      .sort(),
  );
  git("merge-base", "--is-ancestor", retirement.source, "HEAD");
  git("merge-base", "--is-ancestor", retirement.retiredAt, producer.head);
  for (const [file, expected] of Object.entries(retirement.files)) {
    assert.equal(
      createHash("sha256")
        .update(git("show", retirement.source + ":" + file))
        .digest("hex"),
      expected,
      file,
    );
    assert.equal(git("ls-tree", "--name-only", retirement.retiredAt, "--", file).length, 0, file);
  }
  for (const [file, expected] of Object.entries(producer.files)) {
    if (absentFiles.includes(file)) {
      assert.equal(expected, null, file);
      assert.equal(git("ls-tree", "--name-only", producer.head, "--", file).length, 0, file);
    } else {
      assert.match(expected ?? "", /^[0-9a-f]{64}$/u);
      assert.equal(
        createHash("sha256")
          .update(git("show", producer.head + ":" + file))
          .digest("hex"),
        expected,
        file,
      );
    }
  }
}
