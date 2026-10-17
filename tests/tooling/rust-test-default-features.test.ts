import assert from "node:assert/strict";
import { test } from "node:test";
import { rustTestTargetProjection } from "../../tools/support/compat/github/run-rust-test-cohort.ts";
import { prDefaultFeatures } from "../../tools/support/compat/github/rust-test-default-features.ts";

const options = [
  "--cargo-profile",
  "ci",
  "--timings",
  "--cargo-message-format=json-render-diagnostics",
  "--profile",
  "pr",
  "-E",
  "package(=vize_canon)",
  "--no-tests=pass",
];
const target = (name: string, kind = "test", required: string[] = []) => ({
  name,
  kind: [kind],
  test: kind !== "example",
  "required-features": required,
});

function fixture() {
  const packages = [
    {
      id: "canon",
      name: "vize_canon",
      manifest_path: "/workspace/canon/Cargo.toml",
      dependencies: [],
      features: { default: ["native"], native: ["transitive"], transitive: [], legacy: [] },
      targets: [
        target("vize_canon", "lib"),
        target("native_case", "test", ["native"]),
        target("legacy_case", "test", ["legacy"]),
      ],
    },
    {
      id: "l1",
      name: "vize_l1",
      manifest_path: "/workspace/l1/Cargo.toml",
      dependencies: [],
      features: {},
      targets: [
        target("vize_l1", "lib"),
        target("compiler_case"),
        target("example_main", "example"),
      ],
    },
  ];
  const metadata = { workspace_root: "/workspace", workspace_members: ["canon", "l1"], packages };
  const full = {
    ...structuredClone(metadata),
    resolve: {
      nodes: [
        { id: "canon", features: ["default", "native", "transitive"] },
        { id: "l1", features: [] },
      ],
    },
  };
  const selected = packages.map((pkg) => pkg.name);
  const proof = () => prDefaultFeatures(metadata, selected, options, "pull_request", () => full);
  return { metadata, full, selected, proof };
}

test("only direct root defaults prove positive gates; metadata absence proves negative gates", () => {
  const { metadata, selected, proof } = fixture();
  const features = proof();
  assert.ok(features);
  assert.equal(features.requiredFeatures("vize_canon", ["native", "default"]), "active");
  assert.equal(features.requiredFeatures("vize_canon", ["legacy"]), "inactive");
  assert.equal(features.requiredFeatures("vize_canon", ["transitive"]), "unknown");
  assert.equal(features.requiredFeatures("vize_canon", ["transitive", "legacy"]), "inactive");
  for (const required of [["unrecognized"], ["dep/native"], ["dep:native"], ["native?"]])
    assert.equal(features.requiredFeatures("vize_canon", required), "unknown");
  assert.equal(features.requiredFeatures("outside", ["native"]), "unknown");
  assert.deepEqual(rustTestTargetProjection(metadata, selected, "typechecker", features), {
    args: ["--lib", "--test", "native_case"],
  });
  assert.deepEqual(rustTestTargetProjection(metadata, selected, "compiler", features), {
    args: ["--lib", "--test", "compiler_case"],
  });
  assert.deepEqual(rustTestTargetProjection(metadata, selected, "typechecker"), {
    args: [],
    reason: "feature-gated or disabled integration targets",
  });
});

test("subset, non-PR, different profiles, features and unknown options never read a resolution", () => {
  const { metadata, selected, full } = fixture();
  let reads = 0;
  const read = () => {
    reads++;
    return full;
  };
  assert.equal(
    prDefaultFeatures(metadata, ["vize_canon"], options, "pull_request", read),
    undefined,
  );
  for (const event of [undefined, "merge_group", "schedule", "workflow_dispatch"])
    assert.equal(prDefaultFeatures(metadata, selected, options, event, read), undefined);
  for (const extra of [
    ["--features", "legacy"],
    ["-Flegacy"],
    ["--all-features"],
    ["--no-default-features"],
    ["--package", "vize_canon"],
    ["--target", "other"],
    ["--config", "build.target='other'"],
    ["--profile", "full"],
    ["--future-option"],
  ])
    assert.equal(
      prDefaultFeatures(metadata, selected, [...options, ...extra], "pull_request", read),
      undefined,
    );
  for (const changed of [
    [],
    options.slice(0, -1),
    options.map((option) => (option === "pr" ? "full" : option)),
  ])
    assert.equal(prDefaultFeatures(metadata, selected, changed, "pull_request", read), undefined);
  assert.equal(reads, 0);
});

test("missing, inconsistent and ambiguous metadata or offline failures retain the default build", () => {
  const original = fixture();
  for (const value of [undefined, {}, { ...original.full, resolve: null }])
    assert.equal(
      prDefaultFeatures(original.metadata, original.selected, options, "pull_request", () => value),
      undefined,
    );
  assert.equal(
    prDefaultFeatures(original.metadata, original.selected, options, "pull_request", () => {
      throw new Error("offline cache absent");
    }),
    undefined,
  );
  for (const corrupt of [
    (full: typeof original.full) => {
      full.workspace_members.pop();
    },
    (full: typeof original.full) => {
      full.workspace_root = "/different";
    },
    (full: typeof original.full) => {
      full.packages.push(full.packages[0]);
    },
    (full: typeof original.full) => {
      full.packages[0].features.default = [];
    },
    (full: typeof original.full) => {
      full.resolve.nodes.pop();
    },
    (full: typeof original.full) => {
      full.resolve.nodes.push(full.resolve.nodes[0]);
    },
    (full: typeof original.full) => {
      full.resolve.nodes[0].features.push("native");
    },
    (full: typeof original.full) => {
      full.resolve.nodes[0].features = ["default"];
    },
  ]) {
    const { metadata, selected, full } = fixture();
    corrupt(full);
    assert.equal(
      prDefaultFeatures(metadata, selected, options, "pull_request", () => full),
      undefined,
    );
  }
});

test("resolved nondefault gates remain ambiguous and same-name activation conflicts fall back", () => {
  const { metadata, selected, proof } = fixture();
  const targets = metadata.packages[0].targets;
  targets.push(target("transitive_case", "test", ["transitive"]));
  assert.deepEqual(rustTestTargetProjection(metadata, selected, "typechecker", proof()).args, []);
  targets.pop();
  metadata.packages[1].targets.push(target("legacy_case"));
  for (const cohort of ["compiler", "typechecker"])
    assert.deepEqual(rustTestTargetProjection(metadata, selected, cohort, proof()).args, []);
  metadata.packages[1].targets.pop();
  targets[1].test = false;
  metadata.packages[1].targets.push(target("native_case"));
  assert.deepEqual(rustTestTargetProjection(metadata, selected, "compiler", proof()).args, []);
});

test("only the supported PR recipe delegates normal example compilation to the required job", () => {
  const { metadata, selected, proof } = fixture();
  metadata.packages[0].targets = [target("vize_canon", "lib"), target("native_case")];
  assert.equal(
    rustTestTargetProjection(metadata, selected, "compiler").reason,
    "retain default example compilation",
  );
  assert.deepEqual(rustTestTargetProjection(metadata, selected, "compiler", proof()), {
    args: ["--lib", "--test", "compiler_case"],
  });
  metadata.packages[1].targets[2].test = true;
  assert.equal(
    rustTestTargetProjection(metadata, selected, "compiler", proof()).reason,
    "nonstandard default test targets",
  );
});
