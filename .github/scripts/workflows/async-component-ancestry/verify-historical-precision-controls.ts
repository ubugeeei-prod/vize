import assert from "node:assert/strict";
import { join } from "node:path";
import { requiredEnv, writeJson } from "../shared/files.ts";

import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
const paths = [
  "crates/vize_croquis/src/script_parser.rs",
  "crates/vize_croquis/src/script_parser/value_binding_origins.rs",
  "crates/vize_croquis/src/script_parser/parse.rs",
  "crates/vize_croquis/src/script_parser/value_binding_origins/resolution.rs",
  "crates/vize_croquis/src/facts/components.rs",
];
const specs = [
  {
    "id": "mixed-demand",
    "parent": "4c7211a42ef27ba8fc3bae4c220fc95cd6ac8278",
    "feature": "748876d5d580aa5c6d0fb0bf32108b8bb0429951",
    "test": "facts::components::tests::missing_setup_capture_cannot_reuse_plain_program_names",
    "marker": "    fn uncaptured_setup(",
    "failure": "LocalComponentBinding::Opaque",
    "changed": [
      "crates/vize_croquis/src/script_parser.rs",
      "crates/vize_croquis/src/script_parser/value_binding_origins.rs",
      "crates/vize_croquis/src/script_parser/parse.rs",
      "crates/vize_croquis/src/script_parser/value_binding_origins/resolution.rs",
      "crates/vize_croquis/src/facts/components.rs",
    ],
  },
  {
    "id": "component-spelling",
    "parent": "a15d99e984886cd7b13e706cc755290d4fc98d8c",
    "feature": "748876d5d580aa5c6d0fb0bf32108b8bb0429951",
    "test": "facts::components::tests::component_binding_spelling_follows_vue_lookup_order",
    "marker": "    #[test]\n    fn component_binding_spelling_follows_vue_lookup_order",
    "failure": "./Camel.vue",
    "changed": [
      "crates/vize_croquis/src/script_parser.rs",
      "crates/vize_croquis/src/script_parser/value_binding_origins.rs",
      "crates/vize_croquis/src/script_parser/value_binding_origins/resolution.rs",
      "crates/vize_croquis/src/facts/components.rs",
    ],
  },
];
const landedFeature = "8a089d97ea3c95a23c588085f47502fa7d288bdf",
  landedMerge = "deda174ca6da0fc54a4f7483acbd3f71c382095c";
const git = (...args: string[]) => execFileSync("git", args, { encoding: "utf8" }).trim();
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const head = git("rev-parse", "HEAD");
assert.equal(head, requiredEnv("SOURCE_SHA"));
git("merge-base", "--is-ancestor", landedMerge, head);
const component = paths.at(-1)!,
  current = readFileSync(component),
  proof = "target/vize-tests/async-component";
for (const spec of specs) {
  const { parent, feature } = spec;
  const delta = JSON.parse(
    execFileSync(
      "node",
      [
        "qualification/tools/support/cross_file/async_component/feature-delta.ts",
        parent,
        feature,
        landedFeature,
        landedMerge,
        head,
        "--",
        "crates",
        "davinci",
        "Cargo.toml",
        "Cargo.lock",
      ],
      { encoding: "utf8" },
    ),
  ) as {
    historicalParent: string;
    historicalFeature: string;
    landedFeature: string;
    landedMerge: string;
    head: string;
    changed: string[];
  };
  assert.equal(delta.historicalParent, parent);
  assert.equal(delta.historicalFeature, feature);
  assert.equal(delta.landedFeature, landedFeature);
  assert.equal(delta.landedMerge, landedMerge);
  assert.equal(delta.head, head);
  assert.deepEqual([...delta.changed].sort(), [...spec.changed].sort());
  const previous = new Map(
    paths.map((path) => [
      path,
      execFileSync("git", ["show", parent + ":" + path], { maxBuffer: 32 * 1024 * 1024 }),
    ]),
  );
  const start = current.indexOf(spec.marker),
    end = current.lastIndexOf("\n}");
  assert.ok(start >= 0 && end > start && !previous.get(component)!.includes(spec.marker));
  const tail = current.subarray(start, end);
  const receipt = {
    schema: "vize.asyncComponentPrecisionBefore.v1",
    productionSource: parent,
    observer: head,
    test: spec.test,
    mergedSquash: landedMerge,
    productionTree: git("rev-parse", parent + "^{tree}"),
    featureDelta: delta,
    newTestTailSha256: hash(tail),
    productionFiles: [...previous].map(([path, bytes]) => ({ path, sha256: hash(bytes) })),
    claim:
      "full pinned parent production with only new regression tests; not source SDK or public acceptance",
  };
  const prefix = join(proof, spec.id + "-parent");
  writeJson(prefix + ".json", receipt);
  const worktree = join(requiredEnv("RUNNER_TEMP"), "async-component-" + spec.id + "-parent");
  git("worktree", "add", "--detach", worktree, parent);
  try {
    const file = join(worktree, component),
      old = readFileSync(file),
      closing = old.lastIndexOf("\n}");
    assert.ok(closing >= 0);
    writeFileSync(
      file,
      Buffer.concat([old.subarray(0, closing), Buffer.from("\n"), tail, old.subarray(closing)]),
    );
    assert.equal(git("-C", worktree, "diff", "--name-only"), component);
    const result = spawnSync(
      "cargo",
      ["test", "--locked", "-p", "vize_croquis", spec.test, "--", "--exact", "--nocapture"],
      {
        cwd: worktree,
        env: { ...process.env, CARGO_TARGET_DIR: join(process.cwd(), "target") },
        maxBuffer: 256 * 1024 * 1024,
      },
    );
    if (result.error) throw result.error;
    writeFileSync(prefix + ".stdout", result.stdout);
    writeFileSync(prefix + ".stderr", result.stderr);
    writeJson(prefix + ".json", {
      ...receipt,
      status: result.status,
      stdoutSha256: hash(result.stdout),
      stderrSha256: hash(result.stderr),
    });
    assert.equal(result.status, 101);
    for (const marker of ["running 1 test", "test result: FAILED. 0 passed; 1 failed;"])
      assert.ok(result.stdout.includes(marker), marker);
    for (const marker of ["assertion", "failed", spec.failure])
      assert.ok(result.stderr.includes(marker), marker);
  } finally {
    git("worktree", "remove", "--force", worktree);
  }
  git("diff", "--exit-code");
}
