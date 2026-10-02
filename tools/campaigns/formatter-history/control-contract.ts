import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";

export const movedTargets = [
  "sfc_block_attributes",
  "sfc_empty_script_blocks",
  "sfc_opaque_template_language",
  "sfc_template_literal_attribute_idempotence",
  "sfc_v_pre_idempotence",
  "template_runtime_semantics",
  "template_shorthand_spread_order",
  "template_suppression_line",
  "template_whitespace_significant",
];
export const checks: Record<string, string[]> = {
  "panic-free-workspace": ["strict-clippy", "selected-rust", "metadata"],
  "level-import-renaming": ["stage-source-tests", "metadata", "selected-rust"],
  "style-spec-reference-pairs": ["style-spec-tests"],
  "allocation-alias": ["selected-rust", "api-replays", "benchmark-smoke"],
  "arena-comment-correction": [],
  "benchmark-black-box-migration": ["benchmark-smoke"],
  "formatter-throughput": ["selected-rust", "api-replays", "benchmark-smoke"],
  "oxc-formatter-provider": ["selected-rust", "api-replays", "strict-clippy"],
  "crate-logo-docs": [],
  "json-module-move": ["selected-rust", "api-replays", "glyph-module-source"],
  "expression-arena-reuse": ["selected-rust", "api-replays", "benchmark-smoke"],
  "template-render-once": ["selected-rust", "api-replays", "benchmark-smoke"],
  "workspace-rust-version": ["msrv-check", "metadata"],
  "template-module-relocation": ["selected-rust", "glyph-module-source"],
  "UTF8-safety-comments": ["strict-clippy", "selected-rust", "api-replays"],
  "toolchain-source-format": ["rustfmt"],
  "readme-refinement": [],
  "vite-plus-docs": [],
  "initial-template-module-split": ["selected-rust", "glyph-module-source"],
  "unpublished-package": ["metadata"],
  "brand-asset-removal": [],
  "docs-rs-metadata": ["metadata"],
  "initial-public-APIs-and-benchmarks": ["selected-rust", "api-replays", "benchmark-smoke"],
  "initial-todo-scaffold": [],
};
export const budgetControls = new Set([
  "panic-free-workspace",
  "allocation-alias",
  "formatter-throughput",
  "expression-arena-reuse",
  "template-render-once",
  "initial-public-APIs-and-benchmarks",
]);
export const benchmarkNames = [
  "format_sfc/simple_sfc",
  "format_sfc_with_allocator/simple_sfc_reuse_allocator",
  "format_script/large_script",
  "format_template/complex_template",
];
export const commands = {
  metadata: ["cargo", "metadata", "--locked", "--no-deps", "--format-version=1"],
  "strict-clippy": [
    "cargo",
    "clippy",
    "--locked",
    "--profile",
    "ci",
    "-p",
    "vize_glyph",
    "--all-targets",
    "--message-format=json-render-diagnostics",
    "--",
    "-D",
    "warnings",
  ],
  "style-spec-tests": [
    "vp",
    "node",
    "--test",
    "--test-reporter=tap",
    "tests/tooling/davinci-style-spec.test.ts",
  ],
  "stage-source-tests": [
    "vp",
    "node",
    "--test",
    "--test-reporter=tap",
    "tests/tooling/davinci-glyph-stage-dependencies.test.ts",
    "tests/tooling/davinci/davinci-glyph-stage-alias.test.ts",
    "tests/tooling/davinci-level-dependencies.test.mjs",
  ],
  "workspace-module-support": [
    "vp",
    "node",
    "--test",
    "--test-reporter=tap",
    "tests/tooling/davinci-module-layout.test.ts",
  ],
  "glyph-module-source": [
    "vp",
    "node",
    "tools/campaigns/formatter-history/glyph-module-proof.ts",
    ".",
  ],
  rustfmt: ["cargo", "fmt", "--all", "--", "--check"],
  "benchmark-build": [
    "cargo",
    "bench",
    "--locked",
    "--profile",
    "ci",
    "-p",
    "vize_glyph",
    "--bench",
    "formatter",
    "--no-run",
    "--message-format=json-render-diagnostics",
  ],
};
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const objectHash = (kind: string, bytes: Buffer) =>
  createHash("sha1").update(`${kind} ${bytes.length}\0`).update(bytes).digest("hex");
export function readControlAuthority(root: string, audit: any, id: string) {
  const git = (args: string[]) => {
    const result = spawnSync("git", ["--no-replace-objects", ...args], {
      cwd: root,
      maxBuffer: 32 * 1024 * 1024,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.status, 0, result.stderr.toString());
    return result.stdout;
  };
  const control = audit.controls[id];
  assert(control && Object.hasOwn(checks, id));
  const scope = ["crates/vize_glyph"];
  if (id === "style-spec-reference-pairs")
    scope.push(
      "davinci-road/plan/style-spec.md",
      "docs/davinci/plan/style-spec.md",
      "tests/tooling/davinci-style-spec.test.ts",
    );
  if (id === "panic-free-workspace") scope.push("Cargo.toml");
  const commit = git(["cat-file", "commit", control.commit]);
  assert.equal(objectHash("commit", commit), control.commit);
  const parents = commit
    .toString()
    .split("\n\n")[0]!
    .split("\n")
    .filter((line) => line.startsWith("parent "))
    .map((line) => line.slice(7));
  assert.equal(parents.length, 1, "original engineering control must have its exact first parent");
  const parent = parents[0]!;
  const parentCommit = git(["cat-file", "commit", parent]);
  assert.equal(objectHash("commit", parentCommit), parent);
  const patch = git([
    "show",
    "--format=",
    "--no-ext-diff",
    "--binary",
    "--find-renames",
    control.commit,
    "--",
    ...scope,
  ]);
  const raw = git([
    "diff-tree",
    "--no-commit-id",
    "-r",
    "--raw",
    "--no-abbrev",
    "-z",
    "-M",
    parent,
    control.commit,
    "--",
    ...scope,
  ]);
  assert(patch.length > 0 && raw.length > 0);
  const fields = raw.toString().split("\0");
  assert.equal(fields.pop(), "");
  const changed: any[] = [];
  const objects: { kind: string; oid: string; sha256: string; bytes: Buffer }[] = [];
  const retain = (kind: string, oid: string, bytes: Buffer) => {
    assert.equal(objectHash(kind, bytes), oid);
    objects.push({ kind, oid, sha256: hash(bytes), bytes });
    return { oid, sha256: hash(bytes), bytes: bytes.length };
  };
  retain("commit", control.commit, commit);
  retain("commit", parent, parentCommit);
  while (fields.length) {
    const header = /^:(\d{6}) (\d{6}) ([a-f0-9]{40}) ([a-f0-9]{40}) ([ACDMRTUXB]\d*)$/.exec(
      fields.shift()!,
    );
    assert(header, "invalid original raw tree diff");
    const [, oldMode, newMode, oldOid, newOid, status] = header;
    const oldPath = fields.shift()!;
    const newPath = /^[RC]/.test(status!) ? fields.shift()! : oldPath;
    assert(oldPath && newPath);
    const blob = (oid: string, revision: string, file: string) => {
      if (/^0{40}$/.test(oid)) return null;
      assert.equal(
        git(["rev-parse", `${revision}:${file}`])
          .toString()
          .trim(),
        oid,
      );
      return retain("blob", oid, git(["cat-file", "blob", oid]));
    };
    changed.push({
      status,
      oldMode,
      newMode,
      oldPath,
      newPath,
      before: blob(oldOid!, parent, oldPath),
      after: blob(newOid!, control.commit, newPath),
    });
  }
  const refs = [
    ...new Set<string>([
      ...(control.sourceRefs ?? []),
      ...(control.bijectionSourceRef ? [control.bijectionSourceRef] : []),
      ...(control.pairs ?? []).flatMap((pair: any) => [
        pair.inputSourceRef,
        pair.expectedSourceRef,
      ]),
    ]),
  ];
  const sources = refs.map((ref) => {
    const source = audit.sourceCatalog[ref];
    assert(source);
    return {
      ref,
      ...source,
      blobs: source.revisions.map((revision: string) => {
        const revisionCommit = git(["cat-file", "commit", revision]);
        retain("commit", revision, revisionCommit);
        const bytes = git(["show", `${revision}:${source.path}`]);
        assert.equal(hash(bytes), source.sha256, `original control blob differs: ${ref}`);
        const gitBlob = git(["rev-parse", `${revision}:${source.path}`])
          .toString()
          .trim();
        retain("blob", gitBlob, bytes);
        return { revision, gitBlob, bytes: bytes.length };
      }),
    };
  });
  const pin = {
    id,
    commit: control.commit,
    contract: control.contract,
    kind: control.kind,
    scope,
    firstParent: parent,
    rawCommitSha256: hash(commit),
    rawFirstParentSha256: hash(parentCommit),
    rawPatchSha256: hash(patch),
    rawTreeDiffSha256: hash(raw),
    changed,
    sourcePins: sources,
    requiredChecks: checks[id],
    requiredMovedTargets: id === "panic-free-workspace" ? movedTargets : [],
    protectedInstructionBudgetRequired: budgetControls.has(id),
    outputCredit: false,
    nativeCredit: false,
    acceptance:
      "unexecuted hosted scope/runtime capture; independent review remains; applicable performance metrics are unmeasured",
  };
  return {
    pin,
    rawCommit: commit,
    rawFirstParent: parentCommit,
    rawPatch: patch,
    rawTreeDiff: raw,
    objects,
  };
}
export function originalControlPins(root: string, audit: any) {
  assert.deepEqual(Object.keys(audit.controls).sort(), Object.keys(checks).sort());
  return Object.keys(audit.controls).map((id) => readControlAuthority(root, audit, id).pin);
}
export function currentControlPins(root: string, names: string[]) {
  const files = [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "crates/vize_glyph/Cargo.toml",
    "crates/vize_glyph/benches/formatter.rs",
    "crates/vize_glyph/src/lib.rs",
    "crates/vize_glyph/src/json.rs",
    "crates/vize_glyph/src/template.rs",
    "crates/vize_glyph/src/script/format.rs",
    "crates/vize_glyph/src/error.rs",
    "clippy.toml",
    "docs/davinci/plan/style-spec.md",
    ...names.map((name) => `crates/vize_glyph/tests/${name}.rs`),
    ...Object.values(commands).flatMap((command) =>
      command.filter((arg) => arg.startsWith("tests/tooling/")),
    ),
  ];
  return [...new Set(files)].sort().map((file) => {
    const bytes = fs.readFileSync(path.join(root, file));
    return {
      path: file,
      bytes: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
    };
  });
}
